use secrecy::ExposeSecret;
use surrealdb::{
    engine::remote::ws::{Client, Ws, Wss},
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

    let db = match std::env::var("APP_ENVIRONMENT") {
        Ok(_) => Surreal::new::<Wss>(connection_string).await.unwrap(),
        Err(_) => Surreal::new::<Ws>(connection_string).await.unwrap(),
    };

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
