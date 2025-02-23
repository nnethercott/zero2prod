use crate::helpers::spawn_app;
use linkify::{LinkFinder, LinkKind};
use reqwest::{self, Url};
use tokio;

#[tokio::test]
async fn confirmations_without_token_throws_400() {
    // arrange
    let app = spawn_app().await;

    let response = reqwest::get(&format!("{}/subscribe/confirm", &app.address))
        .await
        .expect("failed to send");

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn link_returned_by_subscribe_returns_200_if_called() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _ = app.post_subscriptions(body).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let links = app.get_confirmation_links(&email_request);
    let mut confirmation_link = links.text;

    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    //call this
    let response = reqwest::get(confirmation_link)
        .await
        .expect("failed to send");

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_link_confirms_new_user() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Act
    app.post_subscriptions(body).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let links = app.get_confirmation_links(email_request);

    reqwest::get(links.text)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("failed to retrieve from db");

    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.status, "confirmed");
}
