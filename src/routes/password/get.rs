use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn change_password_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = "".to_string();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    let page_html = format!(r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <title></title>
            <link href="css/style.css" rel="stylesheet">
        </head>
        <body>
        {msg_html}
        <form action="/admin/password" method="post">
            <label for="">
                Current password
                <input type="password" name="old_password" value="enter password">
            </label>
            <label for="">
                New password
                <input type="password" name="new_password" placeholder="enter new password">
            </label>
            <label for="">
                Confirm new password
                <input type="password" name="confirm_new_password" value="confirm new password">
            </label>

            <button type="submit">confirm</button>
        </form>
            <p><a href="/admin/dashboard">&lt;- Back</a></p>
        </body>
        </html>
    "#);

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page_html))
}
