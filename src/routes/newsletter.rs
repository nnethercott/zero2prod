use crate::{domain::SubscriberEmail, email_client::EmailClient};
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use serde::Deserialize;
use sqlx::{query, PgPool};

// type-driven design !
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
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for PublishError {}

#[tracing::instrument(name = "publish newsletter", skip(pool, body, email_client))]
pub async fn publish_newsletter<'a>(
    pool: web::Data<PgPool>,
    body: web::Json<BodyData>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, PublishError> {
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
    let rows = query!(
        "select email from subscriptions where status='confirmed'"
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| match SubscriberEmail::parse(row.email) {
        Ok(sub) => Ok(ConfirmedSubscriber { email: sub }),
        Err(error) => Err(anyhow::anyhow!(error)),
    })
    .collect();

    Ok(rows)
}
