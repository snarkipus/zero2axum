use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use axum_macros::debug_handler;
use color_eyre::eyre::Context;
use hyper::StatusCode;
use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{db::Database, domain::SubscriberEmail, email_client::EmailClient, startup::AppState};

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
pub async fn publish_newsletter(
    State(database): State<Database>,
    State(email_client): State<Arc<EmailClient>>,
    body: Json<BodyData>,
) -> Result<Response, PublishError> {
    let subscribers = get_confirmed_subscribers(database.client).await?;
    for subscriber in subscribers {
        email_client
            .send_email(
                subscriber.email.clone(),
                &body.title,
                &body.content.html,
                &body.content.text,
            )
            .await
            .wrap_err_with(|| format!("Failed to send newsletter to {}", subscriber.email))?;
    }

    Ok(StatusCode::OK.into_response())
}

#[derive(Deserialize)]
struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Getting Confirmed Subscribers", skip(conn))]
async fn get_confirmed_subscribers(
    conn: Surreal<Client>,
) -> color_eyre::Result<Vec<ConfirmedSubscriber>> {
    #[derive(Deserialize)]
    struct Subscriber {
        email: String,
    }

    let sql = "SELECT email FROM subscriptions WHERE status = 'confirmed'";

    let mut res = conn.query(sql).await?.check()?;
    let subscribers: Vec<Subscriber> = res.take(0)?;

    let confirmed_subscribers = subscribers
        .into_iter()
        .filter_map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Some(ConfirmedSubscriber { email }),
            Err(e) => {
                tracing::warn!(
                    "A confirmed subscriber is using an invalid email address.\n{}",
                    e
                );
                None
            }
        })
        .collect();

    Ok(confirmed_subscribers)
}

// region: -- Error
#[derive(strum_macros::AsRefStr, thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] color_eyre::Report),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::error::error_chain_fmt(self, f)
    }
}

impl IntoResponse for PublishError {
    fn into_response(self) -> Response {
        let mut response = match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        response.extensions_mut().insert(self);

        response
    }
}
// endregion: Error
