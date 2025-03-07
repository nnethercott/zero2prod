use actix_web::{
    error::InternalError,
    http::header::LOCATION,
    web, HttpResponse,
};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sha2::Sha256;
use sqlx::PgPool;

use crate::{authentication::{validate_credentials, AuthError, Credentials}, HmacSecret};

#[derive(Deserialize)]
pub struct LoginData {
    username: String, // match the arg names in the form !
    password: Secret<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("Authentication Failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub async fn login(
    form: web::Form<LoginData>,
    pool: web::Data<PgPool>,
    secret: web::Data<HmacSecret>,
) -> Result<HttpResponse, InternalError<LoginError>> {
    //validate login data
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    match validate_credentials(credentials, &pool).await {
        Ok(_) => Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/"))
            .finish()),
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(e) => LoginError::AuthError(e),
                AuthError::UnexpectedError(e) => LoginError::UnexpectedError(e),
            };

            let query_string = format!("error={}", urlencoding::Encoded::new(e.to_string()));

            let hmac_tag = {
                let mut mac =
                    Hmac::<Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
                mac.update(query_string.as_bytes());
                mac.finalize().into_bytes()
            };

            let response = HttpResponse::SeeOther()
                .insert_header((LOCATION, format!("/login?{query_string}&tag={hmac_tag:x}")))
                .finish();

            Err(InternalError::from_response(e, response))
        }
    }
}
