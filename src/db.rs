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
pub async fn create_db(
    configuration: Settings,
) -> std::result::Result<Surreal<Client>, surrealdb::Error> {
    let connection_string = format!(
        "{}:{}",
        configuration.database.host, configuration.database.port
    );

    let db = match configuration.database.require_ssl {
        true => Surreal::new::<Wss>(connection_string).await?,
        false => Surreal::new::<Ws>(connection_string).await?,
    };

    db.signin(Root {
        username: &configuration.database.username,
        password: configuration.database.password.expose_secret(),
    })
    .await?;

    db.use_ns("default")
        .use_db(&configuration.database.database_name)
        .await?;

    Ok(db)
}
// endregion: --- SurrealDB: Initialize
