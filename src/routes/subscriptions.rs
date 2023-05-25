use crate::configuration::Settings;
use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;
use crate::startup::AppState;
use crate::{db, error::Result};
use axum::extract::State;
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
#[debug_handler(state = AppState)]
#[tracing::instrument(
    name = "Adding a new subscriber.",
    skip(data, configuration, email_client),
    fields(
        request_id = %uuid::Uuid::new_v4(),
        subscriber_email = %data.email,
        subscriber_name = %data.name,
        db_name = %configuration.database.database_name
    )
)]
pub async fn handler_subscribe(
    State(configuration): State<Settings>,
    State(email_client): State<EmailClient>,
    Form(data): Form<FormData>,
) -> Result<impl IntoResponse> {
    let new_subscriber: NewSubscriber = match Form(data).0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return Ok(StatusCode::BAD_REQUEST),
    };

    let db = match db::create_db_client(configuration.clone()).await {
        Ok(db) => db,
        Err(_) => return Ok(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let results = insert_subscriber(db, new_subscriber.clone()).await;

    if results.unwrap().check().is_err() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if send_confirmation_email(&email_client, new_subscriber)
        .await
        .is_err()
    {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Ok(StatusCode::OK)
}
// endregion: -- Subscribe Handler

// region: -- Send Confirmation Email
#[tracing::instrument(
    name = "Sending confirmation email.",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
) -> std::result::Result<(), reqwest::Error> {
    let confirmation_link = "https://there-is-no-such-domain.com/subscriptions/confirm";
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
}
// endregion: -- Send Confirmation Email

// region: -- SurrealDB Store
#[tracing::instrument(
    name = "Saving new subscriber details to SurrealDB",
    skip(new_subscriber, db)
)]
pub async fn insert_subscriber(
    db: Surreal<Client>,
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
