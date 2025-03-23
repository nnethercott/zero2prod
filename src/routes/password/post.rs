use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    authentication::{self, middleware::UserId, validate_credentials, AuthError, Credentials},
    routes::get_username,
    utils::{e500, see_other},
};

#[derive(Deserialize)]
pub struct FormData {
    old_password: Secret<String>,
    new_password: Secret<String>,
    confirm_new_password: Secret<String>,
}

pub async fn change_password(
    form: web::Form<FormData>,
    db_pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    if form.new_password.expose_secret() != form.confirm_new_password.expose_secret() {
        FlashMessage::error("<p><i>You entered two different passwords</i></p>").send();
        return Ok(see_other("/admin/password"));
    }

    let username = get_username(&db_pool, *user_id).await.map_err(e500)?;
    let credentials = Credentials {
        username,
        password: form.0.old_password,
    };

    // if this is an error do something
    if let Err(e) = validate_credentials(credentials, &db_pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("<p><i>Current password is incorrect</i></p>").send();
                return Ok(see_other("/admin/password"));
            }
            AuthError::UnexpectedError(_) => Err(e500(e).into()),
        };
    }

    // update new password hash
    authentication::change_password(*user_id, form.0.new_password, &db_pool)
        .await
        .map_err(e500)?;

    FlashMessage::info("<p><i>Password changed successfully</i></p>").send();
    Ok(see_other("/admin/password"))
}
