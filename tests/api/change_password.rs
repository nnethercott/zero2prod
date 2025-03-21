use serde_json::json;
use uuid::Uuid;

use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn must_be_logged_in_to_see_password_form() {
    let app = spawn_app().await;
    let response = app.get_change_password().await;
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn must_be_logged_in_to_change_password() {
    let app = spawn_app().await;
    let body = json!({
        "old_password": "df634959-bc1b-4e2a-a007-93c9566d0db6",
        "new_password": "c78ccca6-2aad-44cd-938a-637bd7d590be",
        "confirm_new_password": "c78ccca6-2aad-44cd-938a-637bd7d590be",
    });
    let response = app.post_change_password(&body).await;
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn passwords_must_match() {
    let app = spawn_app().await;

    //login
    app.post_login(&json!({
        "username": app.user.username,
        "password": app.user.password
    }))
    .await;

    // new and confirm new don't match
    let body = json!({
        "old_password": app.user.password,
        "new_password": Uuid::new_v4().to_string(),
        "confirm_new_password": Uuid::new_v4().to_string(),
    });

    let response = app.post_change_password(&body).await;
    assert_is_redirect_to(&response, "/admin/password");
    
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("You entered two different passwords"))
}

#[tokio::test]
async fn current_password_must_be_valid() {
    let app = spawn_app().await;

    //login
    app.post_login(&json!({
        "username": app.user.username,
        "password": app.user.password
    }))
    .await;

    // make sure old password is bad
    let new_password = Uuid::new_v4().to_string();
    let body = json!({
        "old_password": Uuid::new_v4().to_string(),
        "new_password": new_password,
        "confirm_new_password": new_password,
    });

    let response = app.post_change_password(&body).await;
    assert_is_redirect_to(&response, "/admin/password");
    
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("Current password is incorrect"))
}

#[tokio::test]
async fn logout_clears_session_state() {
    let app = spawn_app().await;

    //Part 1. login
    app.post_login(&json!({
        "username": app.user.username,
        "password": app.user.password
    }))
    .await;

    //Part 2. change password
    let new_password = Uuid::new_v4().to_string();
    let body = json!({
        "old_password": app.user.password,
        "new_password": new_password,
        "confirm_new_password": new_password,
    });
    let response = app.post_change_password(&body).await;
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("Password changed successfully"));

    // Part 3. logout
    let response = app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    // Part 4. Verify logout html 
    let html_page = app.get_login_html().await;
    assert!(html_page.contains("You have successfully logged out"));

    // Part 5. login with new password
    let response = app.post_login(&json!({
        "username": app.user.username,
        "password": new_password,
    }))
    .await;
    assert_is_redirect_to(&response, "/admin/dashboard");
}
