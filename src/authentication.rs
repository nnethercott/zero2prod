use actix_web::{http::StatusCode, ResponseError};
use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum AuthError{
    #[error("Authentication Failed")]
    InvalidCredentials(#[source] anyhow::Error), 
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for AuthError{
    fn status_code(&self) -> StatusCode {
        match self{
            Self::InvalidCredentials(_) => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

pub async fn validate_credentials(
    credentials: Credentials,
    db_pool: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    // default values to simulate work if user does not exist
    let mut user_id = None;
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(&credentials.username, db_pool).await?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    // compute hash in blocking thread
    let _ = tokio::task::spawn_blocking(move || {
        verify_password_hash(credentials.password, expected_password_hash)
    })
    .await
    .context("Failed to spawn thread")
    .map_err(AuthError::UnexpectedError)??;

    user_id
        .ok_or_else(|| anyhow::anyhow!("Incorrect username"))
        .map_err(AuthError::InvalidCredentials)
}

pub async fn get_stored_credentials(
    username: &str,
    db_pool: &PgPool,
) -> Result<Option<(Uuid, Secret<String>)>, AuthError> {
    let row = sqlx::query!(
        r#"select user_id, password_hash from users where name=$1"#,
        username
    )
    .fetch_optional(db_pool)
    .await
    .context("Failed to execute query")
    .map_err(AuthError::UnexpectedError)?
    .map(|row| (row.user_id, Secret::new(row.password_hash)));

    Ok(row)
}

pub fn verify_password_hash(
    password: Secret<String>,
    expected_hash: Secret<String>,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(&expected_hash.expose_secret())
        .context("Failed to parse password into PHC format")
        .map_err(AuthError::UnexpectedError)?;

    Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &expected_password_hash)
        .context("Invalid password")
        .map_err(AuthError::InvalidCredentials)
}

