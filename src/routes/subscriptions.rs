use crate::{
    db::{Database, Transaction},
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    error::{StoreTokenError, SubscribeError},
    startup::{AppState, ApplicationBaseUrl},
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Form,
};
use axum_macros::debug_handler;
use color_eyre::eyre::Context;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::{
    engine::remote::ws::Client,
    sql::{self, Thing},
    Surreal,
};

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

#[tracing::instrument(name = "Generating Subscription Token")]
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
    skip(data, email_client, base_url, database),
    fields(
        subscriber_email = %data.email,
        subscriber_name = %data.name,
    )
)]
pub async fn handler_subscribe(
    State(email_client): State<Arc<EmailClient>>,
    State(base_url): State<ApplicationBaseUrl>,
    State(database): State<Database>,
    Form(data): Form<FormData>,
) -> Result<Response, SubscribeError> {
    let new_subscriber: NewSubscriber = Form(data)
        .0
        .try_into()
        .map_err(SubscribeError::ValidationError)?;

    let transaction = Transaction::begin(&database.client)
        .await
        .context("Failed to begin SurrealDB Transaction")?;

    let conn = transaction.conn;

    let subscriber_id = insert_subscriber(&new_subscriber, conn)
        .await
        .context("Failed to insert new seubscriber in the database.")?;

    let subscription_token = generate_subscription_token();

    store_token(&subscriber_id, &subscription_token, conn)
        .await
        .context("Failed to store subscription token in the database")?;

    transaction
        .commit()
        .await
        .context("Failed to commit transaction to store a new subscriber.")?;

    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await
    .context("Failed to send a confirmation email to the new subscriber.")?;

    Ok(StatusCode::OK.into_response())
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
    skip(new_subscriber, client)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    client: &Surreal<Client>,
) -> Result<Thing, surrealdb::Error> {
    let subscriber_uuid = sql::Uuid::new_v4().to_raw();
    let subscriber_id = Thing::from(("subscriptions".into(), subscriber_uuid));

    let query = format!(
        "CREATE {} CONTENT {{ email: '{}', name: '{}', subscribed_at: time::now(), status: '{}' }}",
        &subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        "pending_confirmation"
    );

    match client.query(query).await?.check() {
        Ok(_) => Ok(subscriber_id),
        Err(e) => Err(e),
    }
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
    skip(subscriber_id, subscription_token, client)
)]
pub async fn store_token(
    subscriber_id: &Thing,
    subscription_token: &str,
    client: &Surreal<Client>,
) -> Result<(), StoreTokenError> {
    let subtoken_uuid = sql::Uuid::new_v4().to_raw();
    let subtoken_id = Thing::from(("subscription_tokens".into(), subtoken_uuid));

    let query = format!(
        "CREATE {} CONTENT {{ subscription_token: '{}' }}",
        subtoken_id, &subscription_token
    );

    client.query(&query).await?.check()?;

    // Associate the subscription token with the subscriber
    let query = format!(
        "RELATE {}->subscribes->{} SET id = {}",
        subtoken_id,
        subscriber_id,
        Thing::from(("subscribes".into(), sql::Uuid::new_v4().to_string()))
    );

    client.query(&query).await?.check()?;

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
) -> Result<(), reqwest::Error> {
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
        .send_email(&new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
}
// endregion: -- Send Confirmation Email
