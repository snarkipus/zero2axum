use axum::{extract::State, response::Response, Form};
use axum_macros::debug_handler;
use hmac::{Hmac, Mac};
use hyper::{Body, StatusCode};
use secrecy::{ExposeSecret, Secret};

use crate::{
    authentication::{validate_credentials, Credentials},
    db::Database,
    error::{AuthError, LoginError},
    startup::{AppState, HmacSecret},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[debug_handler(state = AppState)]
#[tracing::instrument(name = "Login", skip(form, database, secret), fields(
    username = tracing::field::Empty,
    user_id = tracing::field::Empty,
))]
pub async fn login(
    State(database): State<Database>,
    State(secret): State<HmacSecret>,
    Form(form): Form<FormData>,
) -> Result<Response, LoginError> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    match validate_credentials(credentials, &database.client).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id.id));
            Ok(Response::builder()
                .status(StatusCode::SEE_OTHER)
                .header("Location", "/")
                .body(axum::body::boxed(Body::empty()))
                .map_err(|e| LoginError::UnexpectedError(e.into()))?)
        }
        Err(e) => match e {
            AuthError::UnexpectedError(_) => Err(LoginError::UnexpectedError(e.into())),
            AuthError::InvalidCredentials(_) => {
                let query_string = format!("error={}", urlencoding::Encoded::new(e.to_string()));

                let hmac_tag = {
                    let mut mac =
                        Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes())
                            .unwrap();
                    mac.update(query_string.as_bytes());
                    mac.finalize().into_bytes()
                };

                tracing::warn!("Login failed: {}", e);

                Ok(Response::builder()
                    .status(StatusCode::SEE_OTHER)
                    .header(
                        "Location",
                        format!("/login?{query_string}&tag={hmac_tag:x}"),
                    )
                    .body(axum::body::boxed(Body::empty()))
                    .map_err(|e| LoginError::UnexpectedError(e.into()))?)
            }
        },
    }
}
