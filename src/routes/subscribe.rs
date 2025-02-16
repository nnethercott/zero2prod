use crate::{domain::{NewSubscriber, SubscriberEmail, SubscriberName}, email_client::{self, EmailClient}};
use actix_web::{web, HttpResponse};
use chrono::Local as Utc;
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
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subscribe<'a>(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let subscriber = match form.0.try_into(){
        Ok(sub) => sub,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    // perform db insert
    if insert_subscriber(&subscriber, pool.get_ref()).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    if email_client.send_email(
        subscriber.email,
        "welcome!",
        "welcome to the newsletter!",
        "welcome to the newsletter!",
    ).await.is_err(){
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(sub, pool)
)]
pub async fn insert_subscriber(sub: &NewSubscriber, pool: &PgPool) -> Result<(), sqlx::Error> {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        subscriber_email = %sub.email.as_ref(),
        subscriber_name = %sub.name.as_ref(),
    );

    query!(
        r"insert into subscriptions values($1, $2, $3, $4, 'confirmed')",
        Uuid::new_v4(),
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
    Ok(())
}
