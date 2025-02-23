use quickcheck::Testable;
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
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

#[ignore]
#[tokio::test]
async fn emails_are_not_sent_to_unconfirmed_subs() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_user(&app).await;

    // postmark endpoint called to schedule newsletter send
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let body = json!({
        "title": "Some super cool title",
        "content": {
            "text": "newsletter body",
            "html": "<p>newsletter body as html</p>"
        }
    });

    // act
    let response = app.post_newsletters(&body).await;
    assert_eq!(response.status().as_u16(), 200);
}

#[ignore]
#[tokio::test]
async fn all_subscribed_users_receive_email() {
    todo!();
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
