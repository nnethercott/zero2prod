use actix_web::{http::header::ContentType, web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Params {
    error: Option<String>,
}

pub async fn login_form(query: web::Query<Params>) -> HttpResponse {
    let error_html = match query.0.error {
        Some(error) => format!(
            "<p><i>{}</i></p>",
            htmlescape::encode_minimal(&error)
        ),
        None => "".into(),
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title></title>
    <link href="css/style.css" rel="stylesheet">
  </head>
  <body>
  {error_html}
  <form action="/login" method = "post">
     <label for="">
        Username
        <input type="text" name="username" value="enter username">
      </label>
     <label for="">
        password
        <input type="password" name="password" value="enter password">
      </label>

      <button type="submit">Login</button>
   </form> 
  </body>
</html>"#
        ))
}
