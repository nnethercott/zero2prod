use crate::helpers::{assert_is_redirect_to, spawn_app};


#[tokio::test]
async fn must_be_logged_in_to_access_dashboard(){
    let app = spawn_app().await;

    let response = app.get_admin_dashboard().await;
    assert_is_redirect_to(&response, "/login");
}
