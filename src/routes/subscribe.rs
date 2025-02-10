use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
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
pub async fn subscribe<'a>(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let subscriber = match form.0.try_into(){
        Ok(sub) => sub,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&subscriber, pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().finish(),
    }
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
        r"insert into subscriptions values($1, $2, $3, $4)",
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
