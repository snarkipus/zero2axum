use axum::{extract::State, response::IntoResponse, Form};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};
use tracing::info;

// region: -- Subscribe Handler
#[derive(Deserialize, Debug)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Subscription {
    id: Thing,
    email: String,
    name: String,
    subscribed_at: String,
}

pub async fn handler_subscribe(
    State(db): State<Surreal<Client>>,
    Form(data): Form<FormData>,
) -> impl IntoResponse {
    info!("{:<8} - handler_subscribe - {data:?}", "HANDLER");

    let sql = "INSERT INTO subscriptions (email, name, subscribed_at) VALUES ($email, $name, $subscribed_at)";

    let _results = db
        .query(sql)
        .bind(("email", data.email))
        .bind(("name", data.name))
        .bind(("subscribed_at", chrono::Utc::now().to_rfc3339()))
        .await
        .expect("Failed to execute query.");

    StatusCode::OK
}
// endregion: -- Subscribe Handler
