use actix_web::{error::ErrorInternalServerError, http::header::{ContentType, LOCATION}, web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;
use std::fmt::{Debug, Display};
use uuid::Uuid;

use crate::session_state::TypedSession;

fn e500<E>(e: E) -> actix_web::Error
where
    E: Debug + Display + 'static,
{
    ErrorInternalServerError(e)
}

pub async fn admin_dashboard(
    session: TypedSession,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let username = {
        if let Some(user_id) = session.get_user_id().map_err(e500)? {
            get_username(db_pool.as_ref(), user_id)
                .await
                .map_err(e500)?
        } else {
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/login"))
                .finish());
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
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title></title>
    <link href="css/style.css" rel="stylesheet">
  </head>
  <body>
    <p>Welcome {username} !</p> 
  </body>
</html>
"#
        )))
}

async fn get_username(db_pool: &PgPool, user_id: Uuid) -> Result<String, anyhow::Error> {
    let row = sqlx::query!("select name from users where user_id=$1", user_id)
        .fetch_one(db_pool)
        .await
        .context("failed to retrieve username")?;

    Ok(row.name)
}
