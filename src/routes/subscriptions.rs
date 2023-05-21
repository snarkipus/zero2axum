use crate::domain::{NewSubscriber, SubscriberName};
use crate::{
    configuration::Settings,
    db,
    error::{Error, Result},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};
use tracing::error;

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

// region: -- Subscribe Handler
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
    let new_subscriber = NewSubscriber {
        email: data.email,
        name: SubscriberName::parse(data.name).expect("Name validation failed."),
    };

    let db = db::create_db(configuration).await;
    let results = insert_subscriber(&db, new_subscriber).await;

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
#[tracing::instrument(
    name = "Saving new subscriber details to SurrealDB",
    skip(new_subscriber, db)
)]
pub async fn insert_subscriber(
    db: &Surreal<Client>,
    new_subscriber: NewSubscriber,
) -> std::result::Result<surrealdb::Response, surrealdb::Error> {
    let sql = "INSERT INTO subscriptions (email, name, subscribed_at) VALUES ($email, $name, $subscribed_at)";

    db.query(sql)
        .bind(("email", new_subscriber.email))
        .bind(("name", new_subscriber.name.as_ref()))
        .bind(("subscribed_at", chrono::Utc::now().to_rfc3339()))
        .await
}
// endregion: -- SurrealDB Store
