use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_macros::debug_handler;
use surrealdb::sql::Thing;

use crate::error::Result;
use crate::{db::Database, routes::Subscription, startup::AppState};

#[allow(dead_code)]
#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

// region: -- Confirm Subscriber (HTTP Handler)
#[debug_handler(state = AppState)]
#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, database))]
pub async fn handler_confirm(
    State(database): State<Database>,
    Query(parameters): Query<Parameters>,
) -> Result<impl IntoResponse> {
    let id = match get_subscriber_id_from_token(&parameters.subscription_token, &database).await {
        Ok(id) => id,
        Err(_) => return Ok(StatusCode::INTERNAL_SERVER_ERROR),
    };
    match id {
        Some(subscriber_id) => {
            if confirm_subscriber(&subscriber_id, &database).await.is_err() {
                return Ok(StatusCode::INTERNAL_SERVER_ERROR);
            }

            Ok(StatusCode::OK)
        }
        None => Ok(StatusCode::UNAUTHORIZED),
    }
}
// endregion: -- Confirm Subscriber (HTTP Handler)

// region: -- Confirm Subscriber (SurrealDB Update)
#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, database))]
pub async fn confirm_subscriber(
    subscriber_id: &Thing,
    database: &Database,
) -> std::result::Result<(), surrealdb::Error> {
    let db = database.get_connection();

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
    skip(subscription_token, database)
)]
pub async fn get_subscriber_id_from_token(
    subscription_token: &str,
    database: &Database,
) -> std::result::Result<Option<Thing>, surrealdb::Error> {
    let db = database.get_connection();

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
