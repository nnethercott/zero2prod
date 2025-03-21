use actix_web::{http::StatusCode, ResponseError};
use anyhow::Context;
use argon2::{
    password_hash::SaltString, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rand::rngs::OsRng;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Authentication Failed")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCredentials(_) => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

/// validation fn simulating same level of work if no user found
pub async fn validate_credentials(
    credentials: Credentials,
    db_pool: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    // random hash as placeholder
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );
    let mut user_id = None;

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

fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>, anyhow::Error> {
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    );
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2
        .hash_password(password.expose_secret().as_bytes(), &salt)
        .context("failed to hash")?
        .to_string();

    Ok(Secret::new(hash))
}

pub async fn change_password(
    user_id: Uuid,
    password: Secret<String>,
    db_pool: &PgPool,
) -> Result<(), anyhow::Error> {
    let password_hash = tokio::task::spawn_blocking(move || compute_password_hash(password))
        .await?
        .context("Failed to spawn thread")
        .map_err(AuthError::UnexpectedError)?;

    // update db
    sqlx::query!(
        "update users set password_hash=$1 where user_id=$2",
        password_hash.expose_secret(),
        user_id
    )
    .execute(db_pool)
    .await
    .context("failed to update user's password")?;

    Ok(())
}
