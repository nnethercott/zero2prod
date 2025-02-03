use crate::domain::{NewSubscriber, SubscriberName};
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

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subscribe<'a>(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let name = match SubscriberName::parse(form.0.name){
        Ok(name) => name,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let subscriber = NewSubscriber {
        email: form.0.email,
        name,
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
        subscriber_email = %sub.email,
        subscriber_name = %sub.name.as_ref(),
    );

    query!(
        r"insert into subscriptions values($1, $2, $3, $4)",
        Uuid::new_v4(),
        &sub.email,
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
