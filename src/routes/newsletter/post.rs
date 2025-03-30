use crate::{
    authentication::{middleware::UserId, Credentials},
    domain::SubscriberEmail,
    idempotency::{save_response, try_processing, IdempotencyKey, NextAction},
    utils::{e400, e500, see_other},
};
use actix_web::{
    http::{
        header::{self, HeaderMap, HeaderValue},
        StatusCode,
    },
    web::{self},
    HttpRequest, HttpResponse, ResponseError,
};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use base64::Engine;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use sqlx::{query, Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

/// type-driven design !
/// example body:
/// {
///     title: "bleh",
///     content: {
///         "text": "some stuff",
///         "html": "some stuff",
///     }
/// }
#[derive(Serialize, Deserialize)]
pub struct BodyData {
    pub title: String,
    #[serde(flatten)]
    pub content: Content,
    idempotency_key: String,
}
#[derive(Serialize, Deserialize)]
pub struct Content {
    pub text: String,
    pub html: String,
}

impl BodyData {
    pub fn new(title: String, content: Content) -> Self {
        Self {
            title,
            content,
            idempotency_key: Uuid::new_v4().to_string(),
        }
    }
}

#[allow(dead_code)]
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

#[tracing::instrument(name = "publish newsletter", skip(pool, form))]
pub async fn publish_newsletter<'a>(
    pool: web::Data<PgPool>,
    form: web::Form<BodyData>,
    // email_client: web::Data<EmailClient>,
    request: HttpRequest,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    // idempotency check
    let BodyData {
        title,
        content,
        idempotency_key,
    } = form.0;

    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;
    let user_id = user_id.into_inner();

    let success_message = || FlashMessage::info("Successfully sent out newsletter");

    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(cached_response) => {
            success_message().send();
            return Ok(cached_response);
        }
    };

    // init send task
    let issue_id = insert_newsletter_issue(&mut transaction, &title, &content.text, &content.html)
        .await
        .context("failed to store newsletter issue details")
        .map_err(e500)?;

    enqueue_delivery_tasks(&mut transaction, issue_id)
        .await
        .context("failed to enqueue delivery task")
        .map_err(e500)?;

    // let subscribers = get_confirmed_subscribers(pool.as_ref())
    //     .await
    //     .context("failed to retrieve confirmed subs")
    //     .map_err(e500)?;
    //
    // for subscriber in subscribers {
    //     match subscriber {
    //         Ok(sub) => {
    //             let _ = email_client
    //                 .send_email(&sub.email, &title, &content.text, &content.html)
    //                 .await
    //                 .with_context(|| format!("failed to send email to {:?}", sub.email))
    //                 .map_err(e500)?;
    //         }
    //         Err(_) => {
    //             tracing::warn!("invalid email retrieved from db");
    //         }
    //     }
    // }

    let response = see_other("/admin/dashboard");
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;

    success_message().send();
    Ok(response)
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

#[allow(dead_code)]
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

pub async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> Result<Uuid, sqlx::Error> {
    let issue_id = Uuid::new_v4();

    let query = sqlx::query!(
        r#"
        insert into newsletter_issues(
            issue_id, 
            title, 
            text_content,
            html_content,
            published_at
        )
        values($1, $2, $3, $4, now())
    "#,
        issue_id,
        title,
        text_content,
        html_content
    );

    transaction.execute(query).await?;
    Ok(issue_id)
}

pub async fn enqueue_delivery_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO issue_delivery_queue(
            issue_id, 
            email
        )
        SELECT $1, email 
        FROM subscriptions 
        WHERE status='confirmed'
    "#,
        issue_id
    );

    transaction.execute(query).await?;
    Ok(())
}
