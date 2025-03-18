use actix_web::{error::ErrorInternalServerError, http::header::LOCATION, HttpResponse};
use std::fmt::{Debug, Display};

pub fn e500<E>(e: E) -> actix_web::Error
where
    E: Debug + Display + 'static,
{
    ErrorInternalServerError(e)
}

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}
