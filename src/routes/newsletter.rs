use crate::email_client::EmailClient;
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use serde::Deserialize;
use sqlx::{query_as, PgPool};

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
    email: String,
}

#[derive(thiserror::Error, Debug)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for PublishError {}

pub async fn publish_newsletter<'a>(
    pool: web::Data<PgPool>,
    _body: web::Json<BodyData>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, PublishError> {
    let subscribers = get_confirmed_subscribers(pool.as_ref())
        .await
        .context("failed to retrieve confirmed subs")?;

    send_email_to_subs(email_client.as_ref(), _body.into_inner(), subscribers)
        .await
        .context("failed to send email to subscribers")?;

    Ok(HttpResponse::Ok().finish())
}

async fn send_email_to_subs(
    email_client: &EmailClient,
    body: BodyData,
    emails: Vec<ConfirmedSubscriber>,
) -> Result<(), anyhow::Error> {
    for email in emails {
        //post to postmark ...
    }
    Ok(())
}

async fn get_confirmed_subscribers(pool: &PgPool) -> Result<Vec<ConfirmedSubscriber>, sqlx::Error> {
    let rows = query_as!(
        ConfirmedSubscriber,
        "select email from subscriptions where status='confirmed'"
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
