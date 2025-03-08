use actix_web::{
    cookie::Cookie,
    http::header::ContentType,
    HttpRequest, HttpResponse,
};

pub async fn login_form(request: HttpRequest) -> HttpResponse {
    let error_html = match request.cookie("_flash") {
        Some(c) => format!("<p><i>{}</i></p>", c.value()),
        None => "".into(),
    };

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
    let mut response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body);

    response
        .add_removal_cookie(&Cookie::new("_flash", ""))
        .unwrap();
    response
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
