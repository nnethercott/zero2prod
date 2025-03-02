use actix_web::{
    http::header::LOCATION,
    web, HttpResponse,
};
use secrecy::Secret;
use serde::Deserialize;
use sqlx::PgPool;

use crate::authentication::{validate_credentials, AuthError, Credentials};

#[derive(Deserialize)]
pub struct LoginData {
    username: String,
    password: Secret<String>,
}

pub async fn login(
    form: web::Form<LoginData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AuthError> {
    //validate login data
    let credentials = Credentials{
        username: form.0.username,
        password: form.0.password
    };
    let _user_id = validate_credentials(credentials, &pool).await?;

    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish())
}
