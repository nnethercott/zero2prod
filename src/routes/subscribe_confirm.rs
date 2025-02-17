use actix_web::{web::Query, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Parameters{
    token: String,
}

#[tracing::instrument(
    name="confirm pending subscribe"
    skip(_parameters),
)]
pub async fn subscribe_confirm(_parameters: Query<Parameters>) -> HttpResponse {
    // parse query for token
    // check db to ensure token exists
    // db inner join query ?
    HttpResponse::Ok().finish()
}
