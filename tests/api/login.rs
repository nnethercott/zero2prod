use crate::helpers::{assert_is_redirect_to, spawn_app};
use serde_json::json;

#[tokio::test]
async fn error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;

    let body = json!({
        "username": "nate",
        "password": "password"
    });

    let response = app.post_login(&body).await;
    // ensure redirect
    assert_is_redirect_to(&response, "/login");

    // ensure error html returned after failed login
    let mut login_html = app.get_login_html().await;
    assert!(login_html.contains(r#"<p><i>Authentication failed</i></p>"#));

    // ensure page reload does NOT contain cookie
    login_html = app.get_login_html().await;
    assert!(!login_html.contains(r#"<p><i>Authentication failed</i></p>"#));
}

#[tokio::test]
async fn redirect_to_admin_dashboard_on_login_success() {
    let app = spawn_app().await;
    let body = json!({
        "username": &app.user.username,
        "password": &app.user.password,
    });
    let response = app.post_login(&body).await;

    assert_is_redirect_to(&response, "/admin/dashboard");

    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {} !", app.user.username)));
}
