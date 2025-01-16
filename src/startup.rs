use serde::Deserialize;
use sqlx::PgPool;
use std::{fmt::format, net::TcpListener};

use actix_web::{
    dev::Server, get, http::StatusCode, post, web, App, HttpRequest, HttpResponse, HttpServer,
    Responder, Route,
};

use crate::routes::*;

// #[get("/")]
// returning an impl is sick
async fn hello(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("world");
    format!("hello, {}", name)
}

// an example of a custom responder and handler
struct Nate;
impl Responder for Nate {
    type Body = actix_web::body::BoxBody; //not sure what this does

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        let msg = "nate's custom responder";
        HttpResponse::Ok().body(msg)
    }
}
async fn nate() -> impl Responder {
    Nate
}

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    println!("{:?}", listener.local_addr());

    // an Arc<PgPool> we need to be Clone
    let connection = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        //builder pattern
        App::new()
            .app_data(connection.clone())
            .route("/health_check", web::get().to(check_health))
            .route("/nate", web::get().to(nate))
            .route("/subscribe", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
