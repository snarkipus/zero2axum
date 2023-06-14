use axum::response::{IntoResponse, Response};
use hyper::StatusCode;

// region: -- SubscribeError
#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] color_eyre::eyre::Error),
}

impl From<surrealdb::Error> for SubscribeError {
    fn from(error: surrealdb::Error) -> Self {
        Self::UnexpectedError(error.into())
    }
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for SubscribeError {
    fn into_response(self) -> Response {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST.into_response(),
            SubscribeError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
// endregion: SubscribeError

// region: -- ConfrimationError
#[derive(thiserror::Error)]
pub enum ConfirmationError {
    #[error("There is no subscriber associated with the provided token.")]
    UnknownToken,
    #[error(transparent)]
    UnexpectedError(#[from] color_eyre::eyre::Error),
}

impl std::fmt::Debug for ConfirmationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<surrealdb::Error> for ConfirmationError {
    fn from(error: surrealdb::Error) -> Self {
        Self::UnexpectedError(error.into())
    }
}

impl IntoResponse for ConfirmationError {
    fn into_response(self) -> Response {
        match self {
            ConfirmationError::UnknownToken => StatusCode::UNAUTHORIZED.into_response(),
            ConfirmationError::UnexpectedError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
// endregion: ConfirmationError

// region: -- TransactionError
pub struct TransactionError(surrealdb::Error);

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database failure was encountered while trying perform a transaction."
        )
    }
}

impl IntoResponse for TransactionError {
    fn into_response(self) -> Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl From<surrealdb::Error> for TransactionError {
    fn from(error: surrealdb::Error) -> Self {
        Self(error)
    }
}

impl std::error::Error for TransactionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Debug for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
// endregion: TransactionError

// region: -- StoreTokenError
pub struct StoreTokenError(surrealdb::Error);

impl From<surrealdb::Error> for StoreTokenError {
    fn from(error: surrealdb::Error) -> Self {
        Self(error)
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database failure was encountered while trying to store a subscription token."
        )
    }
}
// endregion: StoreTokenError

// region: -- Publish Error
#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication failed.")]
    AuthError(#[source] color_eyre::eyre::Error),
    #[error(transparent)]
    UnexpectedError(#[from] color_eyre::eyre::Error),
}

impl From<surrealdb::Error> for PublishError {
    fn from(error: surrealdb::Error) -> Self {
        Self::UnexpectedError(error.into())
    }
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for PublishError {
    fn into_response(self) -> Response {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            PublishError::AuthError(_) => {
                let response = Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header("WWW-Authenticate", r#"Basic realm="publish""#)
                    .body(hyper::Body::empty())
                    .unwrap();
                response.into_response()
            }
        }
    }
}
// endregion: Publish Error

// region: -- Error Chaining (clever)
pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
// endregion: Error Chaining (clever)
