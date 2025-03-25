use actix_web::{http::header::ContentType,HttpResponse};
use uuid::Uuid;

pub async fn create_newsletter<'a>() -> Result<HttpResponse, actix_web::Error> {
    let key = Uuid::new_v4().to_string();

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(r#"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Publish Newsletter</title>
    <link href="css/style.css" rel="stylesheet">
  </head>
  <body>
    <form action="/admin/newsletters" method="post">
     <label for="">
        title
        <input name="title" type="text" value="">
      </label>
     <label for="">
        text content
        <input name="content.text" type="text" value="">
      </label>
     <label for="">
        html content
        <input name="content.html" type="text" value="">
      </label>

      <!-- this input is hidden! -->
      <input hidden type="text" name="idempotency_key" value="{key}">

      <button type="submit">send!</button>
     </form> 
  </body>
</html>
        "#)))
}
