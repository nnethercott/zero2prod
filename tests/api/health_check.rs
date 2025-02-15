use reqwest::Client;
use crate::helpers::spawn_app;


#[tokio::test]
async fn test_health() {
    //spawn the server!!
    let test_app = spawn_app().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/health_check", test_app.address))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    //check we have db connection
    let connection = test_app.db_pool.acquire().await;
    assert!(connection.is_ok());
}

