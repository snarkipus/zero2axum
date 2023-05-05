use crate::{
    configuration::Settings,
    error::{Error, Result},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
    Surreal,
};
use tracing::error;

// region: -- Subscribe Handler
#[derive(Deserialize, Debug)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Subscription {
    id: Thing,
    email: String,
    name: String,
    subscribed_at: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber.",
    skip(data, configuration),
    fields(
        email = %data.email,
        name = %data.name,
        db = %configuration.database.database_name
    )
)]
pub async fn handler_subscribe(
    State(configuration): State<Settings>,
    Form(data): Form<FormData>,
) -> Result<impl IntoResponse> {
    let db = create_db(configuration).await;
    let results = insert_subscriber(&db, data).await;

    match results.unwrap().check() {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            error!("Failed to execute query: {e}");
            Err(Error::SurrealDBError)
        }
    }
}
// endregion: -- Subscribe Handler

// region: -- SurrealDB Store
#[tracing::instrument(name = "Saving new subscriber details to SurrealDB", skip(data, db))]
pub async fn insert_subscriber(
    db: &Surreal<Client>,
    data: FormData,
) -> std::result::Result<surrealdb::Response, surrealdb::Error> {
    let sql = "INSERT INTO subscriptions (email, name, subscribed_at) VALUES ($email, $name, $subscribed_at)";

    db.query(sql)
        .bind(("email", data.email))
        .bind(("name", data.name))
        .bind(("subscribed_at", chrono::Utc::now().to_rfc3339()))
        .await
}
// endregion: -- SurrealDB Store

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

    let db = Surreal::new::<Ws>(connection_string)
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
