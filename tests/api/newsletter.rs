use serde_json::json;
use uuid::Uuid;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{spawn_app, ConfirmationLinks, TestApp};

#[tokio::test]
async fn fails_without_body_in_post() {
    let app = spawn_app().await;

    let test_cases = vec![
        (
            json!({
                "title": "nate",
            }),
            "missing content",
        ),
        (
            json!({
                "content":{
                "text": "nate",
                "html": "nate"
            }
            }),
            "missing title",
        ),
        (json!({}), "missing title and content"),
    ];

    for (body, message) in test_cases {
        let response = app.post_newsletters(&body).await;
        assert_eq!(response.status().as_u16(), 400, "{}", message);
    }
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_user(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });
    let response = app.post_newsletters(&newsletter_request_body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

async fn create_unconfirmed_user(app: &TestApp) -> ConfirmationLinks {
    let body = "name=nate&email=nnethercott99@gmail.com";

    // postmark endpoint called to send confirmation email
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    let response = app
        .post_subscriptions(body)
        .await
        .error_for_status()
        .unwrap();

    // retrieve confirmation link so we can register user later
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    app.get_confirmation_links(&email_request)
}

async fn create_confirmed_user(app: &TestApp) {
    // note: link contains formatted endpoint already
    let links = create_unconfirmed_user(app).await;

    reqwest::Client::new()
        .get(links.html)
        .query(&[("token", links.text.as_str())])
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn requests_missing_authentication_are_rejected(){
    // mock post to publish with no header
    // inspect response to make sure we get a 401 Unauthorized
    let app = spawn_app().await;

    let body = json!({
        "title": "not allowed",
        "content": {
            "text": "some text",
            "html": "some text",
    }
    });

    let response = reqwest::Client::new()
            .post(&format!("{}/newsletters", &app.address))
            .json(&body)
            .send()
            .await
            .expect("failed to post to /newsletters")
;
    assert_eq!(response.status().as_u16(), 401);

    assert_eq!(r#"Basic realm="publish""#, response.headers()["WWW-Authenticate"]);
}

async fn non_existing_user_is_rejected(){
    let app = spawn_app().await;

    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    let response = reqwest::Client::new()
            .post(&format!("{}/newsletters", &app.address))
            .basic_auth(username, Some(password))
            .send()
            .await
            .expect("failed to post to /newsletters")
;
    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(r#"Basic realm="publish""#, response.headers()["WWW-Authenticate"]);
}
