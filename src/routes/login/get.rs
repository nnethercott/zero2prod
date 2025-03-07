use actix_web::{http::header::ContentType, web, HttpResponse};
use anyhow::Context;
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;
use serde::Deserialize;
use sha2::Sha256;

use crate::HmacSecret;

#[derive(Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;

        let query_string = format!(
            "error={}",
            urlencoding::Encoded(&self.error), // implicit decoded when parsed into String ?
        );

        let mut hmac = Hmac::<Sha256>::new_from_slice(secret.0.expose_secret().as_bytes())?;
        hmac.update(query_string.as_bytes());
        hmac.verify_slice(&tag).context("HMAC failed")?;

        Ok(self.error)
    }
}

pub async fn login_form(
    query: Option<web::Query<QueryParams>>,
    secret: web::Data<HmacSecret>,
) -> HttpResponse {
    let error_html = match query {
        Some(q) => {
            let error = match q.0.verify(&secret) {
                Ok(e) => format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&e)),
                Err(_) => {
                    // do some tracing here to notify bad mac attempt
                    "".into()
                }
            };
            error
        }
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
    <title>Nate</title>
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
