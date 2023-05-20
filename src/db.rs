use secrecy::ExposeSecret;
use surrealdb::{
    engine::remote::ws::{Client, Wss},
    opt::auth::Root,
    Surreal,
};

use crate::configuration::Settings;

// region: -- SurrealDB: Initialize
#[tracing::instrument(
  name = "Creating new SurrealDB",
  skip(configuration),
  fields(
      db = %configuration.database.database_name
  )
)]
pub async fn create_db(configuration: Settings) -> Surreal<Client> {
    let connection_string = format!(
        "{}:{}",
        configuration.database.host, configuration.database.port
    );

    let db = Surreal::new::<Wss>(connection_string)
        .await
        .expect("Failed to connect to SurrealDB.");

    db.signin(Root {
        username: &configuration.database.username,
        password: configuration.database.password.expose_secret(),
    })
    .await
    .expect("Failed to signin.");

    db.use_ns("default")
        .use_db(&configuration.database.database_name)
        .await
        .expect("Failed to use database.");

    db
}
// endregion: --- SurrealDB: Initialize
