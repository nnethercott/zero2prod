use crate::{session_state::TypedSession, utils::see_other};
use actix_web::HttpResponse;
use actix_web_flash_messages::FlashMessage;

pub async fn logout(session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    session.purge();
    FlashMessage::info("You have successfully logged out").send();
    return Ok(see_other("/login"));
}
