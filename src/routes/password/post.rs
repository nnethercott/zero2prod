use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

use crate::{session_state::TypedSession, utils::{e500, see_other}};

#[derive(Deserialize)]
pub struct FormData {
    old_password: Secret<String>,
    new_password: Secret<String>,
    confirm_new_password: Secret<String>,
}


pub async fn change_password(
    session: TypedSession,
    form: web::Form<FormData>,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none(){
        return Ok(see_other("/login"));
    }
    if form.new_password.expose_secret()!=form.confirm_new_password.expose_secret(){
        FlashMessage::error("<p><i>You entered two different passwords</i></p>").send();
        return Ok(see_other("/admin/password"));
    }

    todo!()
}
