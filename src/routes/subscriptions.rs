use crate::{
    configuration::Settings,
    db::{Database, Transaction},
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    error::Result,
    startup::{AppState, ApplicationBaseUrl},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use axum_macros::debug_handler;
use color_eyre::eyre::Context;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::{sql::{self, Thing}, engine::remote::ws::Client, Surreal};
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

    let transaction = Transaction::begin(&database.client).await?;
    let conn = transaction.conn;
    let new_subscriber: NewSubscriber = match Form(data).0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return Ok(StatusCode::BAD_REQUEST),
    };

    let subscriber_id = match insert_subscriber(new_subscriber.clone(), conn).await {
        Ok(id) => id,
        Err(_) => return Ok(StatusCode::INTERNAL_SERVER_ERROR),
    };
    tracing::info!("Subscriber ID: {:?}", subscriber_id);

    let subscription_token = generate_subscription_token();

    if store_token(&subscriber_id, &subscription_token, conn)
        .await
        .is_err()
    {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let tx_rs = transaction.commit().await;

    let tx_res = match tx_rs {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("Failed to execute transaction: {:?}", e);
            return Ok(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(e) = tx_res.check() {
        tracing::error!("Failed to execute transaction: {:?}", e);
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

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
    skip(new_subscriber, client)
)]
pub async fn insert_subscriber(
    new_subscriber: NewSubscriber,
    client: &Surreal<Client>,
) -> color_eyre::Result<Thing> {
    let subscriber_uuid = sql::Uuid::new_v4().to_raw();
    let subscriber_id = Thing::from(("subscriptions".into(), subscriber_uuid));

    let query = format!(
        "CREATE {} CONTENT {{ email: '{}', name: '{}', subscribed_at: time::now(), status: '{}' }}",
        &subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        "pending_confirmation"
    );

    client.query(query).await.context("Failed to insert subscriber");
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
    skip(subscriber_id, subscription_token, client)
)]
pub async fn store_token(
    subscriber_id: &Thing,
    subscription_token: &str,
    client: &Surreal<Client>,
) -> color_eyre::Result<()> {
    let subtoken_uuid = sql::Uuid::new_v4().to_raw();
    let subtoken_id = Thing::from(("subscription_tokens".into(), subtoken_uuid));

    let query = format!(
        "CREATE {} CONTENT {{ subscription_token: '{}' }}",
        subtoken_id, &subscription_token
    );

    client.query(&query).await.context("Failed to add sub token query")?;
    
    // Associate the subscription token with the subscriber
    let query = format!(
        "RELATE {}->subscribes->{} SET id = {}",
        subtoken_id,
        subscriber_id,
        Thing::from(("subscribes".into(), sql::Uuid::new_v4().to_string()))
    );


    client.query(&query).await.context("Failed to add relate query")?;

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
