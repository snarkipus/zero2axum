use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use axum_macros::debug_handler;
use base64::Engine;
use color_eyre::eyre::Context;
use hyper::{HeaderMap, StatusCode};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use std::sync::Arc;
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};
use sha3::Digest;

use crate::{
    db::Database, domain::SubscriberEmail, email_client::EmailClient, error::PublishError,
    startup::AppState,
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
    let user_id = validate_credentials(credentials, &database.client).await?;
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

#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(name = "Validating credentials", skip(headers))]
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

#[derive(Deserialize)]
struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

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

#[tracing::instrument(name = "Validating credentials", skip(credentials, conn))]
async fn validate_credentials(
    credentials: Credentials,
    conn: &Surreal<Client>,
) -> Result<Thing, PublishError> {
    let password_hash = sha3::Sha3_256::digest(
        credentials
            .password
            .expose_secret()
            .as_bytes(),
    );

    let password_hash = format!("{:x}", password_hash);
    
    let sql = "SELECT id FROM users WHERE username = $username and password_hash = $password_hash LIMIT 1";

    let mut res = conn
        .query(sql)
        .bind(("username", credentials.username))
        .bind(("password_hash", password_hash))
        .await?
        .check()?;
    dbg!(&res);
    let user_id: Option<Thing> = res.take((0, "id")).context("Invalid credentials")?;

    if let Some(user_id) = user_id {
        Ok(user_id)
    } else {
        Err(PublishError::AuthError(color_eyre::eyre::eyre!(
            "Invalid credentials"
        )))
    }
}
