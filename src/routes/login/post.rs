use axum::{extract::State, response::Response, Form};
use axum_macros::debug_handler;
use hyper::{StatusCode, Body};
use secrecy::Secret;

use crate::{
    authentication::{validate_credentials, Credentials},
    db::Database,
    error::{AuthError, LoginError},
    startup::AppState,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[debug_handler(state = AppState)]
#[tracing::instrument(name = "Login", skip(form, database), fields(
    username = tracing::field::Empty,
    user_id = tracing::field::Empty,
))]
pub async fn login(
    database: State<Database>,
    form: Form<FormData>,
) -> Result<Response, LoginError> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &database.client)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
        })?;

    tracing::Span::current().record("user_id", &tracing::field::display(&user_id.id));
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/")
        .body(axum::body::boxed(Body::empty()))
        .map_err(|e| LoginError::UnexpectedError(e.into()))
}
