use axum::{extract::State, response::Response};
use axum_macros::debug_handler;

use hyper::Body;
use secrecy::ExposeSecret;
use tower_cookies::{Cookie, Cookies, Key};

use crate::startup::HmacSecret;

#[debug_handler]
pub async fn login_form(cookies: Cookies, State(secret): State<HmacSecret>) -> Response {
    let key = Key::from(secret.0.expose_secret().as_bytes());
    let private_cookies = cookies.private(&key);
    let error_html = match private_cookies.get("_flash") {
        Some(flash_cookie) => {
            format!(r#"<p class="error"><i>{}</i></p>"#, flash_cookie.value())
        }
        None => "".to_string(),
    };
    private_cookies.remove(Cookie::new("_flash", ""));

    Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
        // Github Copilot did super sexy shit here
        .body(axum::body::boxed(Body::from(format!(
            r#"<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="utf-8">
                <title>Login</title>
                <style>
                    body {{
                        font-family: sans-serif;
                        margin: 0;
                        padding: 0;
                    }}
                    .error {{
                        color: red;
                    }}
                    .container {{
                        display: flex;
                        flex-direction: column;
                        justify-content: center;
                        align-items: center;
                        height: 100vh;
                    }}
                    .form {{
                        display: flex;
                        flex-direction: column;
                        justify-content: center;
                        align-items: center;
                        width: 300px;
                        height: 300px;
                        border: 1px solid #ccc;
                        border-radius: 5px;
                    }}
                    .form input {{
                        margin-bottom: 10px;
                        padding: 5px;
                        border: 1px solid #ccc;
                        border-radius: 5px;
                    }}
                    .form input[type="submit"] {{
                        width: 100px;
                        background-color: #ccc;
                        border: 1px solid #ccc;
                        border-radius: 5px;
                    }}
                </style>
            </head>
            <body>
                <div class="container">
                    <form class="form" action="/login" method="post">
                        <h1>Login</h1>
                        {}
                        <input type="text" name="username" placeholder="Username" required>
                        <input type="password" name="password" placeholder="Password" required>
                        <input type="submit" value="Login">
                    </form>
                </div>
            </body>
            </html>
            "#,
            error_html
        ))))
        .unwrap()
}
