use axum::{extract::State, response::Response, Form};
use axum_macros::debug_handler;
use hyper::{Body, StatusCode};
use secrecy::{ExposeSecret, Secret};
use tower_cookies::{Cookie, Cookies, Key};

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
#[tracing::instrument(name = "Login", skip(form, cookies, database), fields(
    username = tracing::field::Empty,
    user_id = tracing::field::Empty,
))]
pub async fn login(
    State(database): State<Database>,
    State(secret): State<HmacSecret>,
    cookies: Cookies,
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
                let err = LoginError::AuthError(e.into());
                let key = Key::from(secret.0.expose_secret().as_bytes());
                let private_cookies = cookies.private(&key);
                private_cookies.add(Cookie::new("_flash", err.to_string()));

                tracing::warn!(err = ?err, "Invalid credentials");
                Ok(Response::builder()
                    .status(StatusCode::SEE_OTHER)
                    .header("Location", "/login")
                    .body(axum::body::boxed(Body::empty()))
                    .map_err(|e| LoginError::UnexpectedError(e.into()))?)
            }
        },
    }
}
