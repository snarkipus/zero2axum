use argon2::{Argon2, PasswordHash, PasswordVerifier};
use color_eyre::eyre::Context;
use secrecy::{ExposeSecret, Secret};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};

use crate::{
    error::{AuthError, PublishError},
    telemetry::spawn_block_with_tracing,
};

#[derive(serde::Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

// region: -- Validate Credentials
#[tracing::instrument(name = "Validating credentials", skip(credentials, conn))]
pub async fn validate_credentials(
    credentials: Credentials,
    conn: &Surreal<Client>,
) -> Result<Thing, AuthError> {
    let mut user_id = None;
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(&credentials.username, conn).await?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    spawn_block_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task")??;

    user_id
        .ok_or_else(|| color_eyre::eyre::eyre!("Unknown username."))
        .map_err(AuthError::InvalidCredentials)
}
// endregion: -- Validate Credentials

// region: -- Verify Password Hash
#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(AuthError::InvalidCredentials)
}
// endregion: -- Verify Password Hash

// region: -- Get Stored Credentials
#[derive(Debug, serde::Deserialize)]
struct StoredCredentials {
    id: Thing,
    password_hash: String,
}

#[tracing::instrument(name = "Get stored credentials", skip(username, conn))]
async fn get_stored_credentials(
    username: &str,
    conn: &Surreal<Client>,
) -> color_eyre::Result<Option<(Thing, Secret<String>)>> {
    let sql = "SELECT id, password_hash FROM users WHERE username = $username";

    let mut res = conn
        .query(sql)
        .bind(("username", username))
        .await
        .context("Failed to perform a query to retrieve stored credentials")?
        .check()
        .map_err(|e| PublishError::UnexpectedError(color_eyre::eyre::eyre!(e)))?;

    let creds: Option<StoredCredentials> = res
        .take(0)
        .map_err(|e| PublishError::UnexpectedError(color_eyre::eyre::eyre!(e)))?;

    Ok(creds.map(|c| (c.id, Secret::new(c.password_hash))))
}
// endregion: -- Get Stored Credentials
