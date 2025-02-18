use actix_web::{
    web::{self, Query},
    HttpResponse,
};
use serde::Deserialize;
use sqlx::{query, PgPool};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Parameters {
    token: String,
}

#[tracing::instrument(name = "confirming a new subscriber", skip(pool, parameters))]
pub async fn confirm(parameters: Query<Parameters>, pool: web::Data<PgPool>) -> HttpResponse {
    let subscriber_id = match get_subscriber_id_from_token(&parameters.token, &pool).await {
        Ok(uid) => uid,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    match subscriber_id {
        Some(id) => {
            if confirm_subscriber(id, &pool).await.is_err() {
                HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
        None => HttpResponse::Unauthorized().finish()
    }
}

#[tracing::instrument(name = "retrieve subscriber_id from token", skip(pool, token))]
async fn get_subscriber_id_from_token(
    token: &str,
    pool: &PgPool,
) -> Result<Option<Uuid>, sqlx::Error> {
    let record = query!(
        r"select subscriber_id from subscription_tokens where subscription_token = $1",
        token,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(record.map(|r| r.subscriber_id))
}

#[tracing::instrument(name = "update subscriber status to confirmed", skip(pool, uid))]
async fn confirm_subscriber(uid: Uuid, pool: &PgPool) -> Result<(), sqlx::Error> {
    query!(
        r"update subscriptions set status='confirmed' where id = $1",
        uid,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
