use actix_web::{
    http::{header::LOCATION, StatusCode},
    web, HttpResponse, ResponseError,
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

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("Authentication Failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for LoginError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        // urlencode the query param
        let encoded_error = urlencoding::Encoded::new(self.to_string());
        HttpResponse::SeeOther()
            .insert_header((LOCATION, format!("/login?error={}", encoded_error)))
            .finish()
    }
    fn status_code(&self) -> StatusCode {
        match self {
            Self::AuthError(_) => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn login(
    form: web::Form<LoginData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, LoginError> {
    //validate login data
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    let _user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(e) => LoginError::AuthError(e),
            AuthError::UnexpectedError(e) => LoginError::UnexpectedError(e),
        })?;

    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish())
}
