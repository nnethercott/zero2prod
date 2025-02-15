use tokio;
use reqwest::Client;

use crate::helpers::spawn_app;

// check user subscribe
#[tokio::test]
async fn test_subscribe_returns_200_for_valid_data() {
    let test_app = spawn_app().await;
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
    let test_app = spawn_app().await;
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

#[tokio::test]
async fn test_subscribe_fail_with_bad_params() {
    let test_app = spawn_app().await;
    let client = Client::new();

    let test_cases = vec![
        ("name=&email=ursula-le-guin%40gmail.com", "missing the name"),
        ("name=le%20guin&email=", "missing the email"),
        ("name=nate&email=super-bad-email", "poorly formatted email"),
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
