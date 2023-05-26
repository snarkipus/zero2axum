use crate::configuration::Settings;
use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;
use crate::startup::{AppState, ApplicationBaseUrl};
use crate::{db, error::Result};
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Form};
use axum_macros::debug_handler;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Subscription {
    pub id: Option<uuid::Uuid>,
    pub email: String,
    pub name: String,
    pub subscribed_at: String,
    pub status: String,
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

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

// region: -- Subscribe Handler
#[debug_handler(state = AppState)]
#[tracing::instrument(
    name = "Adding a new subscriber.",
    skip(data, configuration, email_client, base_url),
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
    State(base_url): State<ApplicationBaseUrl>,
    Form(data): Form<FormData>,
) -> Result<impl IntoResponse> {
    let new_subscriber: NewSubscriber = match Form(data).0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return Ok(StatusCode::BAD_REQUEST),
    };

    let subscriber_id = match insert_subscriber(&configuration, new_subscriber.clone()).await {
        Ok(id) => id,
        Err(_) => return Ok(StatusCode::INTERNAL_SERVER_ERROR),
    };
    let subscription_token = generate_subscription_token();
    if store_token(&configuration, &subscriber_id, &subscription_token).await.is_err() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if send_confirmation_email(&email_client, new_subscriber, &base_url.0, &subscription_token)
        .await
        .is_err()
    {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Ok(StatusCode::OK)
}
// endregion: -- Subscribe Handler

// region: -- Store Token (SurrealDB Store)
#[tracing::instrument(
    name = "Saving subscription token to SurrealDB",
    skip(configuration, subscription_token)
)]
pub async fn store_token(
    configuration: &Settings,
    subscriber_id: &surrealdb::sql::Uuid,
    subscription_token: &str,
) -> std::result::Result<(), surrealdb::Error> {
    let db = db::create_db_client(configuration.clone()).await?;

    let sql = "INSERT INTO subscription_tokens (subscriber_id, subscription_token) VALUES ($subscriber_id, $subscription_token)";
    let _ = db
        .query(sql)
        .bind(("subscriber_id", subscriber_id.as_ref()))
        .bind(("subscription_token", subscription_token))
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(())
}
// endregion: -- Store Token (SurrealDB Store)

// region: -- Send Confirmation Email
#[tracing::instrument(
    name = "Sending confirmation email.",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> std::result::Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscribe/confirm?subscription_token={}",
        base_url, subscription_token
    );
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

// region: -- Insert Subscriber (SurrealDB Store)
#[tracing::instrument(
    name = "Saving new subscriber details to SurrealDB",
    skip(new_subscriber, configuration)
)]
pub async fn insert_subscriber(
    configuration: &Settings,
    new_subscriber: NewSubscriber,
) -> std::result::Result<surrealdb::sql::Uuid, surrealdb::Error> {
    let db = db::create_db_client(configuration.clone()).await?;

    let sql = "INSERT INTO subscriptions (id, email, name, subscribed_at, status) VALUES ($id, $email, $name, $subscribed_at, $status)";
    let subscriber_id = surrealdb::sql::Uuid::new_v4();
    let _response = db
        .query(sql)
        .bind(("id", subscriber_id.as_ref()))
        .bind(("email", new_subscriber.email.as_ref()))
        .bind(("name", new_subscriber.name.as_ref()))
        .bind(("subscribed_at", chrono::Utc::now().to_rfc3339()))
        .bind(("status", "pending_confirmation"))
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {e}");
            e
        })?;
        // dbg!(&_response);
    Ok(subscriber_id)
}
// endregion: -- Insert Subscriber (SurrealDB Store)
