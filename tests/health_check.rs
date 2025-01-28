use actix_web::http::StatusCode;
use reqwest::{Client, Response};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    run,
    telemetry::{get_subscriber, init_subscriber},
};

use once_cell::sync::Lazy;

struct TestApp {
    db_pool: PgPool,
    address: String,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    // telemetry
    let default_filter_level = "into".into();
    let subscriber_name = "tests".into();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("could not connect!");

    let query = format!(r#"create database "{}";"#, config.database_name.as_str());
    connection
        .execute(query.as_str())
        .await
        .expect("failed to create database");

    // run migrations on database
    let db_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("no pool");

    // creates tables
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("failed to make migrations");

    db_pool
}

async fn spawn_server() -> TestApp {
    // telemetry
    Lazy::force(&TRACING);

    // bind random port
    let listener = TcpListener::bind("127.0.0.1:0").expect("address in use");
    let port = listener.local_addr().unwrap().port();
    let mut address = format!("http://127.0.0.1:{}", port);

    let mut settings = get_configuration("configuration.yaml").unwrap();
    settings.database.database_name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&settings.database).await;
    let server = run(listener, db_pool.clone()).expect("failed to bind address");
    //spawn server
    let _ = tokio::spawn(server);

    TestApp { db_pool, address }
}

#[tokio::test]
async fn test_health() {
    //spawn the server!!
    let test_app = spawn_server().await;
    let client = Client::new();
    let settings = get_configuration("configuration.yaml").unwrap();

    let response = client
        .get(format!("{}/health_check", test_app.address))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    // handle.join();

    //check we have db connection
    let connection = test_app.db_pool.acquire().await;
    assert!(connection.is_ok());
}

// check user subscribe
#[tokio::test]
async fn test_subscribe_returns_200_for_valid_data() {
    let test_app = spawn_server().await;
    let client = Client::new();

    let body = "name=le%20guin&email=ursula-le-guin%40gmail.com";
    let response = client
        .post(format!("{}/subscribe", test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    let saved = sqlx::query!("SELECT name, email from subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch subscription.");

    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula-le-guin@gmail.com");
}

#[tokio::test]
async fn test_subscribe_returns_400_for_invalid_data() {
    let test_app = spawn_server().await;
    let client = Client::new();

    let test_cases = vec![
        ("email=ursula-le-guin%40gmail.com", "missing the name"),
        ("name=le%20guin", "missing the email"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscribe", test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("failed to execute request");

        assert_eq!(response.status().as_u16(), 400, "{}", error_message);
    }
}
