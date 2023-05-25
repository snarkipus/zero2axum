use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::startup::AppState;
use crate::{db, error::Result};
use axum::Extension;
use axum::{http::StatusCode, response::IntoResponse, Form};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};

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

pub fn parse_subscriber(Form(data): Form<FormData>) -> std::result::Result<NewSubscriber, String> {
    let name = SubscriberName::parse(data.name)?;
    let email = SubscriberEmail::parse(data.email)?;
    Ok(NewSubscriber { email, name })
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> std::result::Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}

// region: -- Subscribe Handler
#[debug_handler]
#[tracing::instrument(
    name = "Adding a new subscriber.",
    skip(data, state),
    fields(
        request_id = %uuid::Uuid::new_v4(),
        subscriber_email = %data.email,
        subscriber_name = %data.name,
        db_name = %state.configuration.database.database_name
    )
)]
pub async fn handler_subscribe(
    Extension(state): Extension<AppState>,
    Form(data): Form<FormData>,
) -> Result<impl IntoResponse> {
    let new_subscriber: NewSubscriber = match Form(data).0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return Ok(StatusCode::BAD_REQUEST),
    };

    let db = match db::create_db_client(state.configuration.clone()).await {
        Ok(db) => db,
        Err(_) => return Ok(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let results = insert_subscriber(&db, new_subscriber.clone()).await;

    if results.unwrap().check().is_err() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let email_client = state.email_client;

    if email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            "Welcome to our newsletter!",
            "Welcome to our newsletter!",
        )
        .await
        .is_err()
    {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Ok(StatusCode::OK)
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
    let sql = "INSERT INTO subscriptions (email, name, subscribed_at, status) VALUES ($email, $name, $subscribed_at, $status)";

    let response = db
        .query(sql)
        .bind(("email", new_subscriber.email.as_ref()))
        .bind(("name", new_subscriber.name.as_ref()))
        .bind(("subscribed_at", chrono::Utc::now().to_rfc3339()))
        .bind(("status", "confirmed"))
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {e}");
            e
        })?;
    Ok(response)
}
// endregion: -- SurrealDB Store
