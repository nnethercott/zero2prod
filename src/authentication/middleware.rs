use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    FromRequest
};

use crate::session_state::TypedSession;

pub async fn reject_anonymous_userse(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let session = {
        let (request, mut payload) = req.parts_mut();
        TypedSession::from_request(&request, &mut payload).await
    }?;
    todo!();
}
