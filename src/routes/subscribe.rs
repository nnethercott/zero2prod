use actix_web::{web, HttpResponse};
use chrono::Local as Utc;
use serde::Deserialize;
use sqlx::{query, PgPool};
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
    let a = connection.get_ref();

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
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("failed to insert entry {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
