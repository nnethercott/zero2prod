use actix_web::{http::header::ContentType,HttpResponse};

pub async fn create_newsletter<'a>() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("get.html")))
}
