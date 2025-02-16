use tokio;
use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

// check user subscribe
#[tokio::test]
async fn test_subscribe_returns_200_for_valid_data(){
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

