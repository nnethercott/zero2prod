use actix_web::{web, HttpResponse};
use chrono::Local as Utc;
use serde::Deserialize;
use sqlx::{query, PgPool};
use tracing;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe<'a>(
    form: web::Form<FormData>,
    connection: web::Data<PgPool>,
) -> HttpResponse {
    let request_id = Uuid::new_v4();
    tracing::info!(
        "request_id {} - Adding new subscriber: '{}' '{}'",
        request_id,
        form.name,
        form.email
    );
    match query!(
        r"insert into subscriptions values($1, $2, $3, $4)",
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(connection.get_ref())
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} - successfully registered new user",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("request_id {} - failed to insert entry {:?}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
