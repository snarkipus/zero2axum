use anyhow::Result;
use secrecy::ExposeSecret;
use surrealdb::{
    engine::remote::ws::{Client, Ws, Wss},
    opt::auth::Root,
    Surreal,
};
use surrealdb_migrations::SurrealdbMigrations;

use crate::configuration::Settings;

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
    pub async fn migrate(&self, configuration: &Settings) -> Result<()> {
        SurrealdbMigrations::new(configuration.database.with_db())
            .up()
            .await?;
        Ok(())
    }
    // endregion: --- SurrealDB Migration

    // region:: -- Get Connection
    pub fn get_connection(&self) -> Surreal<Client> {
        self.client.clone()
    }
    // region:: -- Get Connection
}
