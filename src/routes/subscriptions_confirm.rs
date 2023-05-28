use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_macros::debug_handler;
use surrealdb::sql::Thing;

use crate::routes::Subscription;
use crate::{configuration::Settings, db, error::Result};

#[allow(dead_code)]
#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

// region: -- Confirm Subscriber (HTTP Handler)
#[debug_handler]
#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, configuration))]
pub async fn handler_confirm(
    State(configuration): State<Settings>,
    Query(parameters): Query<Parameters>,
) -> Result<impl IntoResponse> {
    let id =
        match get_subscriber_id_from_token(&configuration, &parameters.subscription_token).await {
            Ok(id) => id,
            Err(_) => return Ok(StatusCode::INTERNAL_SERVER_ERROR),
        };
    match id {
        Some(subscriber_id) => {
            if confirm_subscriber(&configuration, &subscriber_id)
                .await
                .is_err()
            {
                return Ok(StatusCode::INTERNAL_SERVER_ERROR);
            }

            Ok(StatusCode::OK)
        }
        None => Ok(StatusCode::UNAUTHORIZED),
    }
}
// endregion: -- Confirm Subscriber (HTTP Handler)

// region: -- Confirm Subscriber (SurrealDB Update)
#[tracing::instrument(
    name = "Mark subscriber as confirmed",
    skip(configuration, subscriber_id)
)]
pub async fn confirm_subscriber(
    configuration: &Settings,
    subscriber_id: &Thing,
) -> std::result::Result<(), surrealdb::Error> {
    let db = db::create_db_client(configuration.clone()).await?;

    let sql = "UPDATE subscriptions SET status = 'confirmed' WHERE id = $subscriber_id";

    let _ = db
        .query(sql)
        .bind(("subscriber_id", subscriber_id))
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(())
}
// endregion: -- Confirm Subscriber (SurrealDB Update)

// region: -- Get Subscriber ID from Token (SurrealDB Retrieve)
#[tracing::instrument(
    name = "Retrieve a subscriber ID from a subscription token",
    skip(configuration, subscription_token)
)]
pub async fn get_subscriber_id_from_token(
    configuration: &Settings,
    subscription_token: &str,
) -> std::result::Result<Option<Thing>, surrealdb::Error> {
    let db = db::create_db_client(configuration.clone()).await?;

    let sql = "
        let $token_id = SELECT id FROM subscription_tokens WHERE subscription_token = $subscription_token;
        SELECT *, $token_id->subscribes->id from subscriptions;
    ";

    let mut res = db
        .query(sql)
        .bind(("subscription_token", subscription_token))
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    let subscriber_id: Thing = res
        .take::<Vec<Subscription>>(1)
        .map(|mut v: Vec<Subscription>| v.pop().unwrap())
        .map(|s: Subscription| s.id.unwrap())
        .unwrap();

    Ok(Some(subscriber_id))
}
// endregion: -- Get Subscriber ID from Token (SurrealDB Retrieve)
