use actix_web::{http::StatusCode, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use super::IdempotencyKey;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

pub async fn get_saved_response(
    pool: &PgPool,
    key: &str,
    user_id: Uuid,
) -> Result<Option<HttpResponse>, anyhow::Error> {
    let saved_response = sqlx::query!(
        r#"
    select 
        response_status_code, 
        response_headers as "response_headers: Vec<HeaderPairRecord>", 
        response_body
    from idempotency
    where 
        user_id = $1 and 
        idempotency_key = $2
    "#,
        user_id,
        key
    )
    .fetch_optional(pool)
    .await?;

    if let Some(r) = saved_response {
        let status_code = StatusCode::from_u16(r.response_status_code.try_into()?)?;
        let mut response = HttpResponse::build(status_code);

        for HeaderPairRecord { name, value } in r.response_headers {
            response.append_header((name, value));
        }
        Ok(Some(response.body(r.response_body)))
    } else {
        Ok(None)
    }
}
// idempotency_key TEXT NOT NULL,
// response_status_code SMALLINT NOT NULL,
// response_headers header_pair[] NOT NULL,
// response_body BYTEA NOT NULL,
// created_at timestamptz NOT NULL,

pub async fn save_response(
    pool: &PgPool,
    key: &str,
    user_id: Uuid,
    response: reqwest::Response,
)->Result<(), anyhow::Error>{
    todo!()
}
