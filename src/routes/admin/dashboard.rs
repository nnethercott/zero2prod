use crate::utils::{e500, see_other};
use actix_web::{http::header::ContentType, web, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;
use std::fmt::Write;

use crate::session_state::TypedSession;

pub async fn admin_dashboard(
    session: TypedSession,
    db_pool: web::Data<PgPool>,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = "".to_string();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    let username = {
        if let Some(user_id) = session.get_user_id().map_err(e500)? {
            get_username(db_pool.as_ref(), user_id)
                .await
                .map_err(e500)?
        } else {
            return Ok(see_other("/login"));
        }
    };
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <meta name="Admin dashboard" content="width=device-width, initial-scale=1">
    <title></title>
    <link href="css/style.css" rel="stylesheet">
  </head>
  <body>
    {msg_html}
    <p>Welcome {username} !</p> 
    <p>available actions:</p>
    <ol>
      <li><a href="/admin/password">change password</a></li>
      <li><a href="/admin/newsletters">create newsletter</a></li>
      <li><form name="logoutForm" action="/admin/logout" method="post">
       <input type="submit" value="Logout"> 
      </form></li>
    </ol>
  </body>
</html>
        "#
        )))
}

pub async fn get_username(db_pool: &PgPool, user_id: Uuid) -> Result<String, anyhow::Error> {
    let row = sqlx::query!("select name from users where user_id=$1", user_id)
        .fetch_one(db_pool)
        .await
        .context("failed to retrieve username")?;

    Ok(row.name)
}
