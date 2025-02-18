use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::{self, EmailClient},
    ApplicationBaseUrl,
};
use actix_web::{web, HttpResponse};
use chrono::Local as Utc;
use rand::{distributions::Alphanumeric, prelude::Distribution, rngs::StdRng, Rng};
use serde::Deserialize;
use sqlx::{query, PgPool};
use tracing::{self, Instrument};
use uuid::Uuid;

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
) -> HttpResponse {
    let subscriber = match form.0.try_into() {
        Ok(sub) => sub,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    // perform db insert
    let uid = match insert_subscriber(&subscriber, pool.get_ref()).await{
        Ok(uid) => uid,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let token = generate_random_token();
    if store_token(&token, uid, pool.get_ref()).await.is_err(){
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(&email_client, subscriber, &base_url.0, &token)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
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
) -> Result<(), String> {
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

    email_client
        .send_email(sub.email, "welcome!", &html_body, &plain_body)
        .await;

    Ok(())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(sub, pool)
)]
pub async fn insert_subscriber(sub: &NewSubscriber, pool: &PgPool) -> Result<Uuid, sqlx::Error> {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        subscriber_email = %sub.email.as_ref(),
        subscriber_name = %sub.name.as_ref(),
    );

    let uid = Uuid::new_v4();

    query!(
        r"insert into subscriptions values($1, $2, $3, $4, 'pending_confirmation')",
        uid,
        &sub.email.as_ref(),
        sub.name.as_ref(),
        Utc::now(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(uid)
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(subscriber_token, subscriber_id, pool)
)]
pub async fn store_token(subscriber_token: &str, subscriber_id: Uuid, pool: &PgPool) -> Result<(), sqlx::Error> {
    let request_id = Uuid::new_v4();

    query!(
        r"insert into subscription_tokens values($1, $2)",
        subscriber_token,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
