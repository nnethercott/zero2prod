use crate::{domain::SubscriberEmail, email_client::EmailClient};
use actix_web::{
    http::{
        header::{self, HeaderMap, HeaderValue},
        StatusCode,
    },
    web::{self},
    HttpRequest, HttpResponse, ResponseError,
};
use anyhow::Context;
use base64::Engine;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sha3::Digest;
use sqlx::{query, PgPool};

/// type-driven design !
/// example body:
/// {
///     title: "bleh",
///     content: {
///         "text": "some stuff",
///         "html": "some stuff",
///     }
/// }
#[derive(Deserialize)]
pub struct BodyData {
    pub title: String,
    pub content: Content,
}
#[derive(Deserialize)]
pub struct Content {
    pub text: String,
    pub html: String,
}

pub struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error, Debug)]
pub enum PublishError {
    #[error("Authentication failed")]
    AuthErr(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            Self::AuthErr(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                response.headers_mut().insert(
                    header::WWW_AUTHENTICATE,
                    HeaderValue::from_str(r#"Basic realm="publish""#).unwrap(),
                );
                response
            }
            _ => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

#[tracing::instrument(name = "publish newsletter", skip(pool, body, email_client))]
pub async fn publish_newsletter<'a>(
    pool: web::Data<PgPool>,
    body: web::Json<BodyData>,
    email_client: web::Data<EmailClient>,
    request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let credentials =
        basic_authentication(&request.headers()).map_err(|e| PublishError::AuthErr(e))?;
    let user_id = validate_credentials(&credentials, &pool).await?;

    let subscribers = get_confirmed_subscribers(pool.as_ref())
        .await
        .context("failed to retrieve confirmed subs")?;

    for subscriber in subscribers {
        match subscriber {
            Ok(sub) => email_client
                .send_email(
                    &sub.email,
                    &body.title,
                    &body.content.text,
                    &body.content.html,
                )
                .await
                .with_context(|| format!("failed to send email to {:?}", sub.email))?,
            Err(_) => {
                tracing::warn!("invalid email retrieved from db");
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "get all subscribers with `confirmed` status", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = query!("select email from subscriptions where status='confirmed'")
        .fetch_all(pool)
        .await?
        .into_iter()
        // schema may have changed invalidating
        // old entries
        .map(|row| match SubscriberEmail::parse(row.email) {
            Ok(sub) => Ok(ConfirmedSubscriber { email: sub }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();

    Ok(rows)
}

struct Credentials {
    username: String,
    password: Secret<String>,
}

async fn validate_credentials(
    credentials: &Credentials,
    db_pool: &PgPool,
) -> Result<uuid::Uuid, PublishError> {
    let password_hash = sha3::Sha3_256::digest(credentials.password.expose_secret().as_bytes());
    let password_hash = format!("{:x}", password_hash);

    let user_id = sqlx::query!(
        r#"select user_id from users where name=$1 and password_hash=$2"#,
        &credentials.username,
        password_hash,
    )
    .fetch_optional(db_pool)
    .await
    .context("Failed to run query")
    .map_err(|e| PublishError::UnexpectedError(e))?;

    user_id
        .map(|row| row.user_id)
        .ok_or_else(|| anyhow::anyhow!("No user found with provided credentials"))
        .map_err(PublishError::AuthErr)
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    // The header value, if present, must be a valid UTF8 string
    let header_value = headers
        .get("Authorization")
        .context("The Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.")?;
    let base64encoded_credentials = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    let decoded_credentials = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_credentials)
        .context("Failed to base64-decode 'Basic' credentials.")?;
    let decoded_credentials = String::from_utf8(decoded_credentials)
        .context("The decoded credential string is valid UTF8.")?;

    // splitn returns at most 2 elements !
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}
