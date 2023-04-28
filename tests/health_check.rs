use rstest::*;
use std::net::TcpListener;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};
use tracing::info;
use zero2axum::{configuration::get_configuration, routes::FormData};

// region: -- spawn_app
#[allow(clippy::let_underscore_future)]
async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let s = zero2axum::startup::run(listener).await.unwrap_or_else(|e| {
        panic!("Failed to start server: {}", e);
    });
    info!("Server listening on http://127.0.0.1:{port}");
    let _ = tokio::spawn(s);
    format!("http://127.0.0.1:{port}")
}
// endregion: -- spawn_app

// region: -- GET: 200 OK
#[tokio::test]
async fn health_check_works() {
    // Arrange
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_string = format!(
        "{}:{}",
        configuration.database.host, configuration.database.port
    );
    let _db = Surreal::new::<Ws>(connection_string)
        .await
        .expect("Failed to connect to SurrealDB.");
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
// endregion: -- GET: 200 OK

// region: -- POST Form: 200 OK
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_string = format!(
        "{}:{}",
        configuration.database.host, configuration.database.port
    );
    let db = Surreal::new::<Ws>(connection_string)
        .await
        .expect("Failed to connect to SurrealDB.");

    db.signin(Root {
        username: "surreal",
        password: "password",
    })
    .await
    .expect("Failed to signin.");

    db.use_ns("default")
        .use_db("newsletter")
        .await
        .expect("Failed to use db.");

    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscribe", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved: FormData = db
        .select(("email", "name"))
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}
// endregion: -- POST Form: 200 OK

// region: -- POST Form: 422 Unprocessable Entity
#[rstest]
#[case("", "missing both name and email")]
#[case("name=le%20guin", "missing the email")]
#[case("email=ursula_le_guin%40gmail.com", "missing the name")]
#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing(
    #[case] invalid_body: &str,
    #[case] error_message: &str,
) {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .post(&format!("{}/subscribe", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body.to_string())
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(
        422,
        response.status().as_u16(),
        "The API did not fail with 422 Bad Request when the payload was {}.",
        error_message
    );
}
// endregion: -- POST Form: 422 Unprocessable Entity
