use color_eyre::{eyre::Context, Result};
use secrecy::ExposeSecret;
use surrealdb::{
    engine::remote::ws::{Client, Ws, Wss},
    opt::{auth::Root, IntoQuery},
    sql,
    sql::Statement,
    Surreal,
};
use surrealdb_migrations::SurrealdbMigrations;

use crate::configuration::Settings;

#[derive(Clone, Debug)]
pub struct Database {
    pub client: Surreal<Client>,
    pub manager: QueryManager
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
        Ok(Self { client, manager: QueryManager::new() })
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

// region: -- Query Manager
#[derive(Clone, Debug, Default)]
pub struct QueryManager {
    pub queries: Vec<String>,
}

impl QueryManager {
    pub fn new() -> QueryManager {
        QueryManager {
            queries: Vec::new(),
        }
    }

    #[tracing::instrument(
        name = "Adding query to QueryManager",
        skip(self, query),
        fields(
            query = %query
        )
    )]
    pub fn add_query(&mut self, query: &str) -> Result<()> {
        let query = sql::parse(query).context("Failed to parse query")?;
        self.queries.push(query.to_string());
        Ok(())
    }

    pub fn generate_transaction(&self) -> Transaction {
        let mut transaction = String::from("BEGIN TRANSACTION;\n");
        for query in &self.queries {
            transaction.push_str(query);
            transaction.push_str(";\n");
        }
        transaction.push_str("COMMIT TRANSACTION;");
        Transaction(transaction)
    }

    #[tracing::instrument(name = "Executing QueryManager", skip(self, db))]
    pub async fn execute(&mut self, db: &Surreal<Client>) -> Result<()> {
        let transaction = self.generate_transaction();
        tracing::debug!(transaction = %transaction.0);
        match db.query(transaction).await {
            Ok(_) => {
                self.queries.clear();
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
}

pub struct Transaction(pub String);

impl AsRef<str> for Transaction {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl IntoQuery for Transaction {
    fn into_query(self) -> Result<Vec<Statement>, surrealdb::Error> {
        sql::parse(self.as_ref())?.into_query()
    }
}
