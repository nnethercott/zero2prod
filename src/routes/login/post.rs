use actix_web::{
    error::InternalError, http::header::LOCATION, web, HttpResponse,
};
use actix_web_flash_messages::FlashMessage;
use secrecy::Secret;
use serde::Deserialize;
use sqlx::PgPool;

use crate::{authentication::{validate_credentials, AuthError, Credentials}, session_state::TypedSession};

#[derive(Deserialize)]
pub struct LoginData {
    username: String, // match the arg names in the form !
    password: Secret<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub fn login_redirect(error: LoginError) -> InternalError<LoginError> {
    FlashMessage::error(error.to_string()).send();

    let response = HttpResponse::SeeOther()
        .insert_header((LOCATION, "/login"))
        .finish();
    InternalError::from_response(error, response)
}

pub async fn login(
    form: web::Form<LoginData>,
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, InternalError<LoginError>> {
    //validate login data
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            //update session for user
            session.renew(); // NOTE: rotate session keys on login
            session.insert_user_id(user_id)
                .map_err(|e| login_redirect(LoginError::UnexpectedError(e.into())))?;

            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/admin/dashboard"))
                .finish())
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(e) => LoginError::AuthError(e),
                AuthError::UnexpectedError(e) => LoginError::UnexpectedError(e),
            };

            // old attempt at hmac to verify redirect authenticity
            // let query_string = format!("error={}", urlencoding::Encoded::new(e.to_string()));
            //
            // let hmac_tag = {
            //     let mut mac =
            //         Hmac::<Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
            //     mac.update(query_string.as_bytes());
            //     mac.finalize().into_bytes()
            // };

            Err(login_redirect(e))
        }
    }
}
