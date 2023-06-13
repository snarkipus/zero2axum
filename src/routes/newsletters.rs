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
    skip(database, email_client, body),
)]
pub async fn publish_newsletter(
    State(database): State<Database>,
    State(email_client): State<Arc<EmailClient>>,
    body: Json<BodyData>,
) -> Result<Response, PublishError> {
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
