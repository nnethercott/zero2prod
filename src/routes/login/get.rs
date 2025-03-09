use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_html = String::new();
    for m in flash_messages.iter() {
        writeln!(error_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    let body = format!(
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
    );

    // on every response we unset the error cookie
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body)
}

// #[derive(Deserialize)]
// pub struct QueryParams {
//     error: String,
//     tag: String,
// }
//
// impl QueryParams {
//     #[deprecated(note = "exists for hmac verification, since removed but good ref")]
//     fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
//         let tag = hex::decode(self.tag)?;
//
//         let query_string = format!(
//             "error={}",
//             urlencoding::Encoded(&self.error), // implicit decoded when parsed into String ?
//         );
//
//         let mut hmac = Hmac::<Sha256>::new_from_slice(secret.0.expose_secret().as_bytes())?;
//         hmac.update(query_string.as_bytes());
//         hmac.verify_slice(&tag).context("HMAC failed")?;
//
//         Ok(self.error)
//     }
// }
