use crate::configuration::Settings;
use crate::db::Database;
use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;
use crate::error::Result;
use crate::startup::{AppState, ApplicationBaseUrl};
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Form};
use axum_macros::debug_handler;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Thing;
use tracing::info;

#[derive(Deserialize, Debug)]
pub struct FormData {
    pub email: String,
    pub name: String,
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
    skip(data, configuration, email_client, base_url, database),
    fields(
        request_id = %uuid::Uuid::new_v4(),
        subscriber_email = %data.email,
        subscriber_name = %data.name,
        db_name = %configuration.database.database_name
    )
)]
pub async fn handler_subscribe(
    State(configuration): State<Settings>,
    State(email_client): State<Arc<EmailClient>>,
    State(base_url): State<ApplicationBaseUrl>,
    State(database): State<Database>,
    Form(data): Form<FormData>,
) -> Result<impl IntoResponse> {
    info!("{:<8} - handler_subscribe", "HANDLER");
    let new_subscriber: NewSubscriber = match Form(data).0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return Ok(StatusCode::BAD_REQUEST),
    };

    let subscriber_id = match insert_subscriber(new_subscriber.clone(), &database).await {
        Ok(id) => id,
        Err(_) => return Ok(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let subscription_token = generate_subscription_token();
    if store_token(&subscriber_id, &subscription_token, &database)
        .await
        .is_err()
    {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    };

    match send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Failed to send confirmation email: {:?}", e);
            return Ok(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(StatusCode::OK)
}
// endregion: -- Subscribe Handler

// region: -- Insert Subscriber (SurrealDB Store)
#[derive(Deserialize, Serialize, Debug)]
pub struct Subscription {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub email: String,
    pub name: String,
    pub subscribed_at: String,
    pub status: String,
}

#[tracing::instrument(
    name = "Saving new subscriber details to SurrealDB",
    skip(new_subscriber, database)
)]
pub async fn insert_subscriber(
    new_subscriber: NewSubscriber,
    database: &Database,
) -> std::result::Result<Thing, surrealdb::Error> {
    let db = database.get_connection();

    let sql = "CREATE subscriptions:uuid() CONTENT { email: $email, name: $name, subscribed_at: $subscribed_at, status: $status }";

    let mut response = db
        .query(sql)
        .bind(("email", new_subscriber.email.as_ref()))
        .bind(("name", new_subscriber.name.as_ref()))
        .bind(("subscribed_at", chrono::Utc::now().to_rfc3339()))
        .bind(("status", "pending_confirmation"))
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {e}");
            e
        })?;

    let subscriber_id: Thing = response
        .take(0)
        .map(|s: Option<Subscription>| s.unwrap())
        .map(|s: Subscription| s.id.unwrap())?;

    Ok(subscriber_id)
}
// endregion: -- Insert Subscriber (SurrealDB Store)

// region: -- Store Token (SurrealDB Store)
#[derive(Deserialize, Serialize, Debug)]
pub struct SubscriptionToken {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub subcription_token: String,
}

#[tracing::instrument(
    name = "Saving subscription token to SurrealDB",
    skip(subscriber_id, subscription_token, database)
)]
pub async fn store_token(
    subscriber_id: &Thing,
    subscription_token: &str,
    database: &Database,
) -> std::result::Result<(), surrealdb::Error> {
    let db = database.get_connection();

    // Create the subscription token record
    let sql =
        "CREATE subscription_tokens:uuid() CONTENT { subscription_token: $subscription_token }";
    let _res = db
        .query(sql)
        .bind(("subscription_token", subscription_token))
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    // Associate the subscription token with the subscriber
    let sql = "
        LET $newtoken = SELECT id FROM subscription_tokens WHERE subscription_token = $subscription_token;
        RELATE $newtoken->subscribes->$subscriber_id SET id = subscribes:uuid();    
    ";
    let _res = db
        .query(sql)
        .bind(("subscriber_id", subscriber_id))
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
