use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use chrono::Local as Utc;
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;
use sqlx::{query, Executor, PgPool, Postgres, Row, Transaction};
use std::fmt::Debug;
use thiserror;
use tracing;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    ApplicationBaseUrl,
};

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(NewSubscriber { name, email })
    }
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut cur = e.source(); // we implemented this below !

    while let Some(cause) = cur {
        writeln!(f, "caused by:\n\t{}", cause)?;
        cur = cause.source();
    }
    Ok(())
}

// gonna get replaced by macros
impl Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(&self, f)
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subscribe<'a>(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    let subscriber = form.0.try_into().map_err(SubscribeError::ValidationError)?;

    // NOTE:make subscribe and tokens table update atomic
    let mut transaction = pool
        .begin()
        .await
        .context("failed to establish connection to postgres")?;

    // perform db insert
    let uid = insert_subscriber(&subscriber, &mut transaction)
        .await
        .context("failed to insert subscriber")?;

    let token = generate_random_token();
    store_token(&token, uid, &mut transaction)
        .await
        .context("failed to persist subscription token")?;

    // make sure to commit transaction !
    transaction
        .commit()
        .await
        .context("failed to commit postgres transaction")?;

    send_confirmation_email(&email_client, subscriber, &base_url.0, &token)
        .await
        .context("failed to send confirmation email")?;

    Ok(HttpResponse::Ok().finish())
}

fn generate_random_token() -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .map(char::from)
        .take(32)
        .collect::<String>()
}

#[tracing::instrument(
    name = "send confirmation email to new subscriber",
    skip(email_client, sub, base_url, token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    sub: NewSubscriber,
    base_url: &str,
    token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!("{}/subscribe/confirm?token={}", base_url, token);

    let plain_body = format!(
        "welcome to the newsletter!\nClick {} to confirm",
        confirmation_link
    );
    let html_body = format!(
        "welcome to the newsletter!<br />\
            Click <a href=\"{}\"here</a> to confirm",
        confirmation_link
    );

    let response = email_client
        .send_email(&sub.email, "welcome!", &html_body, &plain_body)
        .await?;

    Ok(())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(sub, transaction)
)]
pub async fn insert_subscriber(
    sub: &NewSubscriber,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let request_id = Uuid::new_v4();
    let _ = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        subscriber_email = %sub.email.as_ref(),
        subscriber_name = %sub.name.as_ref(),
    );

    let uid = Uuid::new_v4();

    // check if sub exists already -- return uid if so
    let row = transaction
        .fetch_optional(query!(
            r"select id from subscriptions where name = $1",
            sub.name.as_ref()
        ))
        .await?;

    if row.is_some() {
        return Ok(row.unwrap().get("id"));
    }

    let query = query!(
        r"insert into subscriptions values($1, $2, $3, $4, 'pending_confirmation')",
        uid,
        &sub.email.as_ref(),
        sub.name.as_ref(),
        Utc::now(),
    );
    transaction.execute(query).await?;
    Ok(uid)
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(subscriber_token, subscriber_id, transaction)
)]
pub async fn store_token(
    subscriber_token: &str,
    subscriber_id: Uuid,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    let query = query!(
        r"insert into subscription_tokens values($1, $2)",
        subscriber_token,
        subscriber_id,
    );
    transaction.execute(query).await?;
    Ok(())
}
