use color_eyre::{eyre::Context, Result};
use futures_core::future::BoxFuture;
use secrecy::ExposeSecret;
use surrealdb::{
    engine::remote::ws::{Client, Ws, Wss},
    opt::auth::Root,
    Response, Surreal,
};
use surrealdb_migrations::SurrealdbMigrations;

use crate::{configuration::Settings, error::Error};

// region: -- Database
#[derive(Clone, Debug)]
pub struct Database {
    pub client: Surreal<Client>,
}

impl Database {
    // region: -- SurrealDB Initialization
    #[tracing::instrument(
        name = "Creating new SurrealDB Client",
        skip(configuration),
        fields(
            db = %configuration.database.database_name
        )
      )]
    pub async fn new(configuration: &Settings) -> Result<Self> {
        let connection_string = format!(
            "{}:{}",
            configuration.database.host, configuration.database.port
        );

        let client = match configuration.database.require_ssl {
            true => Surreal::new::<Wss>(connection_string)
                .await
                .context("Failed to make Wss connection")?,
            false => Surreal::new::<Ws>(connection_string)
                .await
                .context("Failed to make Ws connection")?,
        };

        client
            .signin(Root {
                username: &configuration.database.username,
                password: configuration.database.password.expose_secret(),
            })
            .await
            .context("Failed to Sign-In")?;

        client
            .use_ns("default")
            .use_db(&configuration.database.database_name)
            .await
            .context("Failed to get namespace & database")?;
        Ok(Self { client })
    }
    // endregion: --- SurrealDB Initialization

    // region: -- SurrealDB Migration
    #[tracing::instrument(
        name = "Performing SurrealDB Migrations",
        skip(configuration),
        fields(
            db = %configuration.database.database_name
        )
      )]
    pub async fn migrate(&self, configuration: &Settings) -> Result<()> {
        SurrealdbMigrations::new(configuration.database.with_db())
            .up()
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!("Failed to run database migrations: {e}"))
            })?;
        Ok(())
    }
    // endregion: --- SurrealDB Migration

    // region:: -- Get Connection
    pub fn get_connection(&self) -> Surreal<Client> {
        self.client.clone()
    }
    // endregion:: -- Get Connection
}
// endregion: --- Database

// region: -- Transaction
pub struct Transaction<'c> {
    pub conn: &'c Surreal<Client>,
    pub open: bool,
}

impl<'c> Transaction<'c> {
    pub fn begin(conn: &'c Surreal<Client>) -> BoxFuture<'c, Result<Self, Error>> {
        Box::pin(async move {
            let sql = "BEGIN TRANSACTION;".to_string();
            let response = conn.query(sql).await?;
            response.check()?;

            Ok(Self { conn, open: true })
        })
    }

    pub async fn commit(mut self) -> std::result::Result<Response, Error> {
        let sql = "COMMIT TRANSACTION;";
        let response = self.conn.query(sql).await?.check()?;
        self.open = false;
        Ok(response)
    }

    pub async fn rollback(mut self) -> BoxFuture<'c, Result<(), Error>> {
        Box::pin(async move {
            let sql = "CANCEL TRANSACTION;";
            let response = self.conn.query(sql).await?;
            response.check()?;
            self.open = false;
            Ok(())
        })
    }
}

impl<'c> Drop for Transaction<'c> {
    fn drop(&mut self) {
        if self.open {
            let conn = self.conn.clone();
            tokio::spawn(async move {
                let sql = "CANCEL TRANSACTION;";
                let _ = conn.query(sql).await;
            });
        }
    }
}
// endregion: -- Transaction
