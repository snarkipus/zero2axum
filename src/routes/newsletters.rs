use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use axum_macros::debug_handler;
use base64::Engine;
use color_eyre::eyre::Context;
use hyper::{HeaderMap, StatusCode};
use secrecy:: Secret;
use serde::Deserialize;
use std::sync::Arc;
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{error::AuthError, authentication::Credentials};
#[allow(unused_imports)]
use crate::{
    db::Database, domain::SubscriberEmail, email_client::EmailClient, error::PublishError,
    startup::AppState, telemetry::spawn_block_with_tracing, authentication::validate_credentials,
};

#[derive(Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(Deserialize)]
pub struct Content {
    text: String,
    html: String,
}

// region: -- /newsletters handler
#[debug_handler(state = AppState)]
#[tracing::instrument(
    name = "Publishing a newsletter",
    skip(database, email_client, headers, body),
    fields(
        username = tracing::field::Empty,
        user_id = tracing::field::Empty,
    )

)]
pub async fn publish_newsletter(
    State(database): State<Database>,
    State(email_client): State<Arc<EmailClient>>,
    headers: HeaderMap,
    body: Json<BodyData>,
) -> Result<Response, PublishError> {
    let credentials = basic_authentication(&headers).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &database.client)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into()),
        })?;
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id.id));

    let subscribers = get_confirmed_subscribers(database.client).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    .wrap_err_with(|| {
                        color_eyre::eyre::eyre!("Failed to send newsletter to {}", subscriber.email)
                    })?;
            }
            Err(e) => {
                tracing::warn!(
                    e.cause_chain = ?e,
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid.",
                );
            }
        }
    }
    Ok(StatusCode::OK.into_response())
}
// endregion: -- /newsletters handler



// region: -- Basic Authentication
#[tracing::instrument(name = "Basic Authentication", skip(headers))]
fn basic_authentication(headers: &HeaderMap) -> color_eyre::Result<Credentials> {
    let header_value = headers
        .get("Authorization")
        .ok_or_else(|| color_eyre::eyre::eyre!("Missing authorization header"))?
        .to_str()
        .context("Authorization header was not valid UTF8")?;

    let base64encoded_credentials = header_value
        .strip_prefix("Basic ")
        .ok_or_else(|| color_eyre::eyre::eyre!("Authorization header did not start with Basic"))?;

    let decoded_credentials = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_credentials)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to base64-decode credentials: {}", e))?;

    let decoded_credentials = String::from_utf8(decoded_credentials)
        .map_err(|e| color_eyre::eyre::eyre!("Credentials were not valid UTF8: {}", e))?;

    let mut credentials = decoded_credentials.splitn(2, ':');

    let username = credentials
        .next()
        .ok_or_else(|| color_eyre::eyre::eyre!("A username must be provided in 'Basic' auth."))?
        .to_owned();

    let password = credentials
        .next()
        .ok_or_else(|| color_eyre::eyre::eyre!("A password must be provided in 'Basic' auth."))?
        .to_owned();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}
// endregion: -- Basic Authentication

#[derive(Deserialize)]
struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

// region: -- Get Confirmed Subscribers
#[tracing::instrument(name = "Getting Confirmed Subscribers", skip(conn))]
async fn get_confirmed_subscribers(
    conn: Surreal<Client>,
) -> color_eyre::Result<Vec<color_eyre::Result<ConfirmedSubscriber>>> {
    #[derive(Deserialize)]
    struct Subscriber {
        email: String,
    }

    let sql = "SELECT email FROM subscriptions WHERE status = 'confirmed'";

    let mut res = conn.query(sql).await?.check()?;
    let subscribers: Vec<Subscriber> = res.take(0)?;

    let confirmed_subscribers = subscribers
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(e) => Err(color_eyre::eyre::eyre!(format!("Failed to parse {}", e))),
        })
        .collect();

    Ok(confirmed_subscribers)
}
// endregion: -- Get Confirmed Subscribers
