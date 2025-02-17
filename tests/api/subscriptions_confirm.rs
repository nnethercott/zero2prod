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

    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let link = get_link(&body["HtmlBody"].as_str().unwrap());
    let mut confirmation_link = Url::parse(&link).unwrap();

    // overwrite port with app value!
    confirmation_link.set_port(Some(app.port));

    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    //call this
    let response = reqwest::get(confirmation_link)
        .await
        .expect("failed to send");

    assert_eq!(response.status().as_u16(), 200);
}
