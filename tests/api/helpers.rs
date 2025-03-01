use actix_web::http::StatusCode;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use linkify::{LinkFinder, LinkKind};
use rand::thread_rng;
use reqwest::{get, Client, Response, Url};
use secrecy::Secret;
use sqlx::{postgres::PgPoolOptions, Connection, Executor, PgConnection, PgPool};
use std::{net::TcpListener, sync::LazyLock};
use tracing_subscriber::fmt::format;
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::{self, EmailClient},
    get_connection_pool,
    routes::BodyData,
    telemetry::{get_subscriber, init_subscriber},
    Application,
};

use once_cell::sync::Lazy;

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub text: reqwest::Url,
}

pub struct TestUser {
    user_id: Uuid,
    username: String,
    password: String,
}
impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }
    pub async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut thread_rng());
        let password_hash = Argon2::default()
            .hash_password(self.password.as_bytes(), &salt)
            .unwrap();

        sqlx::query!(
            r#"insert into users values($1, $2, $3)"#,
            &self.user_id,
            &self.username,
            password_hash.to_string(),
        )
        .execute(pool)
        .await
        .expect("failed to add user");
    }
}

pub struct TestApp {
    pub db_pool: PgPool,
    pub address: String,
    pub email_server: MockServer,
    pub port: u16,
    pub user: TestUser,
}

impl TestApp {
    pub async fn post_subscriptions<T: Into<String>>(&self, body: T) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscribe", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.into())
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_newsletters(&self, body: &serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.address))
            .basic_auth(&self.user.username, Some(&self.user.password))
            // equivalent to: .header("Authorization", "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==")
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str();
            let mut link = Url::parse(raw_link).expect("failed to parse url");

            // overwrite port with app value!
            link.set_port(Some(self.port));
            link
        };

        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let text = get_link(&body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, text }
    }
}

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: Secret::new("password".to_string()),
        ..config.clone() // cool syntax -- populate with rest?
    };
    let mut connection = PgConnection::connect_with(&maintenance_settings.connect_options())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.connect_options())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    // use this to mock the postmark service
    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.app.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");

    let port = application.port();
    let address = format!("http://localhost:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    let db_pool = get_connection_pool(&configuration.database);

    // add verified user
    let user = TestUser::generate();
    user.store(&db_pool).await;

    let test_app = TestApp {
        address,
        db_pool,
        email_server,
        port,
        user,
    };

    test_app
}
