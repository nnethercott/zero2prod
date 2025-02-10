use sqlx::PgPool;
use std::net::TcpListener;

use actix_web::middleware::Logger;
use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

use crate::email_client::EmailClient;
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

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let msg = "nate's custom responder";
        HttpResponse::Ok().body(msg)
    }
}
async fn nate() -> impl Responder {
    Nate
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    println!("{:?}", listener.local_addr());

    // an Arc<PgPool> we need to be Clone
    let connection = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);

    let server = HttpServer::new(move || {
        //builder pattern
        App::new()
            .wrap(Logger::default())
            .app_data(connection.clone())
            .app_data(email_client.clone())
            .route("/health_check", web::get().to(check_health))
            .route("/nate", web::get().to(nate))
            .route("/hello", web::get().to(hello))
            .route("/subscribe", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
