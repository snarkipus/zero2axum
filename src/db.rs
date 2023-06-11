use futures_core::future::BoxFuture;
use secrecy::ExposeSecret;
use surrealdb::{
    engine::remote::ws::{Client, Ws, Wss},
    opt::auth::Root,
    Response, Surreal,
};
use surrealdb_migrations::SurrealdbMigrations;

use crate::configuration::Settings;

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
    pub async fn new(configuration: &Settings) -> Result<Self, surrealdb::Error> {
        let connection_string = format!(
            "{}:{}",
            configuration.database.host, configuration.database.port
        );

        let client = match configuration.database.require_ssl {
            true => Surreal::new::<Wss>(connection_string).await?,
            false => Surreal::new::<Ws>(connection_string).await?,
        };

        client
            .signin(Root {
                username: &configuration.database.username,
                password: configuration.database.password.expose_secret(),
            })
            .await?;

        client
            .use_ns("default")
            .use_db(&configuration.database.database_name)
            .await?;
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
    pub async fn migrate(&self, configuration: &Settings) -> Result<(), anyhow::Error> {
        SurrealdbMigrations::new(configuration.database.with_db())
            .up()
            .await?;
        Ok(())
    }
    // endregion: --- SurrealDB Migration
}
// endregion: --- Database

// region: -- Transaction
pub struct Transaction<'c> {
    pub conn: &'c Surreal<Client>,
    pub open: bool,
}

impl<'c> Transaction<'c> {
    pub fn begin(conn: &'c Surreal<Client>) -> BoxFuture<'c, Result<Self, surrealdb::Error>> {
        Box::pin(async move {
            let sql = "BEGIN TRANSACTION;".to_string();
            conn.query(sql).await?.check()?;

            Ok(Self { conn, open: true })
        })
    }

    pub async fn commit(mut self) -> std::result::Result<Response, surrealdb::Error> {
        let sql = "COMMIT TRANSACTION;";
        let response = self.conn.query(sql).await?.check()?;
        self.open = false;
        Ok(response)
    }

    pub async fn rollback(mut self) -> BoxFuture<'c, Result<(), surrealdb::Error>> {
        Box::pin(async move {
            let sql = "CANCEL TRANSACTION;";
            self.conn.query(sql).await?.check()?;
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
