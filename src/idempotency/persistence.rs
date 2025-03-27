use actix_web::{body::to_bytes, http::StatusCode, HttpResponse};
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::IdempotencyKey;

// for deserializing from db
#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

pub async fn get_saved_response(
    pool: &PgPool,
    key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<Option<HttpResponse>, anyhow::Error> {
    let saved_response = sqlx::query!(
        r#"
    select 
        response_status_code as "response_status_code!", 
        response_headers as "response_headers!: Vec<HeaderPairRecord>", 
        response_body as "response_body!"
    from idempotency
    where 
        user_id = $1 and 
        idempotency_key = $2
    "#,
        user_id,
        key.as_ref()
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

pub async fn save_response(
    mut transaction: Transaction<'static, Postgres>,
    key: &IdempotencyKey,
    user_id: Uuid,
    response: HttpResponse,
) -> Result<HttpResponse, anyhow::Error> {
    let status_code = response.status().as_u16() as i16;
    let (response_head, body) = response.into_parts();
    let body = to_bytes(body).await.map_err(|e| anyhow::anyhow!("{}", e))?;

    let headers = {
        let mut h = Vec::with_capacity(response_head.headers().len());
        for (name, value) in response_head.headers() {
            let name = name.as_str().to_owned();
            let value = value.as_bytes().to_owned();
            h.push(HeaderPairRecord { name, value });
        }
        h
    };
    let query = sqlx::query_unchecked!(
        r#"
        update idempotency 
        set
            response_status_code = $3,
            response_headers = $4,
            response_body = $5
        where 
            user_id = $1 and 
            idempotency_key = $2
        "#,
        user_id,
        key.as_ref(),
        status_code,
        headers,
        body.as_ref(),
    );

    transaction.execute(query).await?;
    transaction.commit().await?;

    // apparently need to go from HttpResponse<Bytes> to HttpResponse<BoxBody>
    let new_response = response_head.set_body(body).map_into_boxed_body();
    Ok(new_response)
}

pub enum NextAction {
    StartProcessing(Transaction<'static, Postgres>),
    ReturnSavedResponse(HttpResponse),
}
pub async fn try_processing(
    pool: &PgPool,
    key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<NextAction, anyhow::Error> {
    let mut transaction = pool.begin().await?;

    let query = sqlx::query!(
        r#"
        INSERT INTO idempotency(
            user_id, 
            idempotency_key,
            created_at
        )
        VALUES ($1, $2, now()) 
        ON CONFLICT DO NOTHING
    "#,
        user_id,
        key.as_ref()
    );

    let n_inserted_rows = transaction.execute(query).await?.rows_affected();

    if n_inserted_rows > 0 {
        return Ok(NextAction::StartProcessing(transaction));
    } else {
        let response = get_saved_response(pool, key, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("we expected a row to exist"))?;
        return Ok(NextAction::ReturnSavedResponse(response));
    }
}
