use axum::{
    extract::{Query, State},
    response::Response,
};
use axum_macros::debug_handler;
use hmac::{Hmac, Mac};
use hyper::Body;
use secrecy::ExposeSecret;

use crate::startup::HmacSecret;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> color_eyre::Result<String> {
        let tag = hex::decode(self.tag)?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));

        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes())
            .map_err(|e| color_eyre::Report::msg(e.to_string()))?;

        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)
            .map_err(|e| color_eyre::Report::msg(e.to_string()))?;

        Ok(self.error)
    }
}

#[debug_handler]
pub async fn login_form(
    query: Option<Query<QueryParams>>,
    State(secret): State<HmacSecret>,
) -> Response {
    let error_html = match query {
        None => "".into(),
        Some(query) => match query.0.verify(&secret) {
            Ok(error) => format!(
                "<p class=\"error\"><i>{}</i></p>",
                htmlescape::encode_minimal(&error)
            ),
            Err(e) => {
                tracing::warn!(
                    error.message = %e,
                    error.cause_chain = ?e,
                    "Failed to verify query parameters using the HMAC tag"
                );
                "".into()
            }
        },
    };

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
