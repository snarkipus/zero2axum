use argon2::{Argon2, PasswordHash, PasswordVerifier};
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

#[allow(unused_imports)]
use crate::{
    db::Database, domain::SubscriberEmail, email_client::EmailClient, error::PublishError,
    startup::AppState, telemetry::spawn_block_with_tracing,
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
// endregion: -- /newsletters handler

#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    password: Secret<String>,
}

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

// region: -- Validate Credentials
#[tracing::instrument(name = "Validating credentials", skip(credentials, conn))]
async fn validate_credentials(
    credentials: Credentials,
    conn: &Surreal<Client>,
) -> Result<Thing, PublishError> {
    let mut user_id = None;
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(&credentials.username, conn)
            .await
            .map_err(PublishError::UnexpectedError)?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    spawn_block_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task")
    .map_err(PublishError::UnexpectedError)??;

    user_id.ok_or_else(|| PublishError::AuthError(color_eyre::eyre::eyre!("Unknown username.")))
}
// endregion: -- Validate Credentials

// region: -- Verify Password Hash
#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), PublishError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")
        .map_err(PublishError::UnexpectedError)?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(PublishError::AuthError)
}
// endregion: -- Verify Password Hash

// region: -- Get Stored Credentials
#[derive(Debug, Deserialize)]
struct StoredCredentials {
    id: Thing,
    password_hash: String,
}

#[tracing::instrument(name = "Get stored credentials", skip(username, conn))]
async fn get_stored_credentials(
    username: &str,
    conn: &Surreal<Client>,
) -> color_eyre::Result<Option<(Thing, Secret<String>)>> {
    let sql = "SELECT id, password_hash FROM users WHERE username = $username";

    let mut res = conn
        .query(sql)
        .bind(("username", username))
        .await
        .context("Failed to perform a query to retrieve stored credentials")?
        .check()
        .map_err(|e| PublishError::UnexpectedError(color_eyre::eyre::eyre!(e)))?;

    let creds: Option<StoredCredentials> = res
        .take(0)
        .map_err(|e| PublishError::UnexpectedError(color_eyre::eyre::eyre!(e)))?;

    Ok(creds.map(|c| (c.id, Secret::new(c.password_hash))))
}
