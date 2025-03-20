use crate::{session_state::TypedSession, utils::{e500, see_other}};
use actix_web::HttpResponse;
use actix_web_flash_messages::FlashMessage;

pub async fn logout(session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }else{
        session.purge();
        FlashMessage::info("You have successfully logged out").send();
        return Ok(see_other("/login"));
    }
}
