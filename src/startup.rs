use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;

use actix_web::middleware::Logger;
use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::*;

pub fn get_connection_pool(config: &DatabaseSettings) -> PgPool {
    // since its lazy fn doesn't need to be async
    PgPoolOptions::new().connect_lazy_with(config.connect_options())
}

pub struct Application {
    port: u16,
    server: Server,
}

pub struct HmacSecret(pub Secret<String>);

impl Application {
    pub async fn build(settings: Settings) -> Result<Self, std::io::Error> {
        let db_pool = get_connection_pool(&settings.database);

        let sender_email = settings.email_client.sender().expect("invalid email");
        let timeout = settings.email_client.timeout();
        let email_client = EmailClient::new(
            settings.email_client.base_url,
            sender_email,
            settings.email_client.auth_token,
            timeout,
        );

        let app_settings = settings.app;
        let address = format!("{}:{}", app_settings.host, app_settings.port);
        let listener = TcpListener::bind(address)?;

        let hmac_secret = app_settings.hmac_secret;
        let port = listener.local_addr().unwrap().port();
        let redis_uri = settings.redis_uri;

        let server = run(
            listener,
            db_pool,
            email_client,
            app_settings.base_url,
            hmac_secret,
            redis_uri,
        ).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct ApplicationBaseUrl(pub String);

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String, // set in env
    hmac_secret: Secret<String>,
    redis_uri: Secret<String>,
) -> Result<Server, std::io::Error> {
    println!("{:?}", listener.local_addr());

    // an Arc<PgPool> we need to be Clone
    let connection = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();
    let session_store = RedisSessionStore::new(redis_uri.expose_secret())
        .await
        .unwrap();

    let server = HttpServer::new(move || {
        //builder pattern
        App::new()
            .wrap(Logger::default())
            .wrap(message_framework.clone())
            .wrap(SessionMiddleware::new(session_store.clone(), secret_key.clone()))
            .app_data(connection.clone())
            .app_data(email_client.clone()) // wanna reuse same email client ?
            .app_data(base_url.clone())
            .app_data(secret_key.clone())
            .route("/health_check", web::get().to(check_health))
            .route("/nate", web::get().to(nate))
            .route("/subscribe", web::post().to(subscribe))
            .route("/subscribe/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .route("/admin/dashboard", web::get().to(admin_dashboard))
            .route("/admin/password", web::get().to(change_password_form))
            .route("/admin/password", web::post().to(change_password))
            .route("/admin/logout", web::post().to(logout))
    })
    .listen(listener)?
    .run();

    Ok(server)
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
