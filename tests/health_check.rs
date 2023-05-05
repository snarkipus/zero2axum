use once_cell::sync::Lazy;
use rstest::*;
use secrecy::ExposeSecret;
use std::net::TcpListener;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};
use surrealdb_migrations::{SurrealdbConfiguration, SurrealdbMigrations};
use tracing::info;
use uuid::Uuid;
use zero2axum::{
    configuration::{get_configuration, Settings},
    routes::FormData,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub configuration: Settings,
}

// region: -- spawn_app
#[allow(clippy::let_underscore_future)]
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let s = zero2axum::startup::run(listener, configuration.clone())
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to start server: {}", e);
        });
    info!("Server listening on http://127.0.0.1:{port}");
    let _ = tokio::spawn(s);
    TestApp {
        address,
        configuration,
    }
}
// endregion: -- spawn_app

// region: -- GET: 200 OK
#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &app.address))
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
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let db = create_db(app.configuration.clone()).await;

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscribe", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let sql = "SELECT email, name FROM subscriptions";

    let mut res = db
        .query(sql)
        .await
        .expect("Failed to fetch saved subscription.");

    let saved: Option<FormData> = res.take(0).unwrap();
    match saved {
        Some(s) => {
            assert_eq!(s.email, "ursula_le_guin@gmail.com");
            assert_eq!(s.name, "le guin");
        }
        None => panic!("No subscription found."),
    }
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
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .post(&format!("{}/subscribe", &app.address))
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

// region: -- SurrealDB: Initialize & Migration
async fn create_db(configuration: Settings) -> Surreal<Client> {
    let connection_string = format!(
        "{}:{}",
        configuration.database.host, configuration.database.port
    );

    let db = Surreal::new::<Ws>(connection_string.clone())
        .await
        .expect("Failed to connect to SurrealDB.");

    db.signin(Root {
        username: &configuration.database.username,
        password: configuration.database.password.expose_secret(),
    })
    .await
    .expect("Failed to signin.");

    db.use_ns("default")
        .use_db(&configuration.database.database_name)
        .await
        .expect("Failed to use database.");

    let mut db_configuration = SurrealdbConfiguration::default();
    db_configuration.url = Some(connection_string.clone());
    db_configuration.ns = Some("default".to_string());
    db_configuration.db = Some(configuration.database.database_name.clone());
    db_configuration.username = Some(configuration.database.username.clone());
    db_configuration.password = Some(configuration.database.password.expose_secret().clone());

    SurrealdbMigrations::new(db_configuration)
        .up()
        .await
        .expect("Failed to run migrations.");

    db
}
// endregion: --- SurrealDB: Initialize & Migration

