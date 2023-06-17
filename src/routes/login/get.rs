use axum::{response::Response, extract::Query};
use axum_macros::debug_handler;
use hyper::Body;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: Option<String>,
}

#[debug_handler]
pub async fn login_form(query: Query<QueryParams>) -> Response {
    let error = match query.0.error {
        None => "".into(),
        Some(error) => format!("<p class=\"error\"><i>{}</i></p>", error),
    };    
    Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
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
            error
        ))))
        .unwrap()
}
