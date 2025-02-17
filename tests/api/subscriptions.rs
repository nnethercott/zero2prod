use crate::helpers::spawn_app;
use linkify::{LinkFinder, LinkKind};
use tokio;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

// check user subscribe
#[tokio::test]
async fn test_subscribe_returns_200_for_valid_data() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    // Act
    let response = test_app.post_subscriptions(body).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_user() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Act
    test_app.post_subscriptions(body).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("failed to retrieve from db");

    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_sends_confirmation_email_with_a_link() {
    let test_app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let _ = test_app.post_subscriptions(body).await;
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];

    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    dbg!(&body);
    let html_link = get_link(&body["HtmlBody"].as_str().unwrap());
    let text_link = get_link(&body["TextBody"].as_str().unwrap());

    assert_eq!(html_link, text_link);
}

#[tokio::test]
async fn test_subscribe_returns_400_for_invalid_data() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("email=ursula-le-guin%40gmail.com", "missing the name"),
        ("name=le%20guin", "missing the email"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = test_app.post_subscriptions(invalid_body).await;
        assert_eq!(response.status().as_u16(), 400, "{}", error_message);
    }
}

#[tokio::test]
async fn test_subscribe_fail_with_bad_params() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=&email=ursula-le-guin%40gmail.com", "missing the name"),
        ("name=le%20guin&email=", "missing the email"),
        ("name=nate&email=super-bad-email", "poorly formatted email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = test_app.post_subscriptions(invalid_body).await;
        assert_eq!(response.status().as_u16(), 400, "{}", error_message);
    }
}
