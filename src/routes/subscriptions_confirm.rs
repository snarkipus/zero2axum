use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_macros::debug_handler;
use color_eyre::eyre::Context;
use surrealdb::sql::Thing;

#[allow(unused_imports)]
use crate::{db::Database, error::ConfirmationError, startup::AppState};

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
) -> Result<Response, ConfirmationError> {
    let id = get_subscriber_id_from_token(&parameters.subscription_token, &database)
        .await
        .context("Failed to retrieve the subscriber id associated with the provided token.")?
        .ok_or(ConfirmationError::UnknownToken)?;

    confirm_subscriber(&id, &database)
        .await
        .context("Failed to confirm the subscriber.")?;

    Ok(StatusCode::OK.into_response())
}
// endregion: -- Confirm Subscriber (HTTP Handler)

// region: -- Confirm Subscriber (SurrealDB Update)
#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, database))]
pub async fn confirm_subscriber(
    subscriber_id: &Thing,
    database: &Database,
) -> std::result::Result<(), surrealdb::Error> {
    let client = &database.client;

    let sql = "UPDATE subscriptions SET status = 'confirmed' WHERE id = $subscriber_id";

    client
        .query(sql)
        .bind(("subscriber_id", subscriber_id))
        .await?
        .check()?;

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
    let client = &database.client;

    let sql = "
        let $token_id = SELECT id FROM subscription_tokens WHERE subscription_token = $subscription_token;
        SELECT *, $token_id->subscribes->id from subscriptions;
    ";

    let mut res = client
        .query(sql)
        .bind(("subscription_token", subscription_token))
        .await?
        .check()?;

    let subscriber_id: Option<Thing> = res.take((1, "id"))?;

    Ok(subscriber_id)
}
// endregion: -- Get Subscriber ID from Token (SurrealDB Retrieve)
