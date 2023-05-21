use once_cell::sync::Lazy;
use rstest::*;
use std::net::TcpListener;
use surrealdb_migrations::SurrealdbMigrations;
use tracing::info;
use uuid::Uuid;
use zero2axum::{
    configuration::{get_configuration, Settings},
    db,
    routes::FormData,
    telemetry::{get_subscriber, init_subscriber},
};

// region: -- conditional tracing for tests
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
// endregion: -- conditional tracing for tests

pub struct TestApp {
    pub configuration: Settings,
}

// region: -- spawn_app
#[allow(clippy::let_underscore_future)]
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    configuration.application.port = listener.local_addr().unwrap().port();

    let s = zero2axum::startup::run(listener, configuration.clone())
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to start server: {}", e);
        });
    info!(
        "Server listening on http://{}:{}",
        configuration.application.host, configuration.application.port
    );
    let _ = tokio::spawn(s);
    TestApp { configuration }
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
        .get(&format!(
            "http://{}:{}/health_check",
            &app.configuration.application.host, &app.configuration.application.port
        ))
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
    let db = db::create_db(app.configuration.clone()).await;
    migrate_db(app.configuration.clone())
        .await
        .expect("Failed to migrate database.");

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!(
            "http://{}:{}/subscribe",
            &app.configuration.application.host, &app.configuration.application.port
        ))
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
        .post(&format!(
            "http://{}:{}/subscribe",
            &app.configuration.application.host, &app.configuration.application.port
        ))
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

// region: -- POST Form: 200 w/fields present but empty
#[rstest]
#[case("name=&email=ursula_le_guin%40gmail.com", "empty name")]
#[case("name=Ursula&email=", "empty email")]
#[case("name=Ursula&email=definitely-not-an-email", "invalid email")]
#[tokio::test]
async fn subscribe_returns_a_200_when_fields_are_present_but_empty(
    #[case] invalid_body: &str,
    #[case] error_message: &str,
) {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .post(&format!(
            "http://{}:{}/subscribe",
            &app.configuration.application.host, &app.configuration.application.port
        ))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body.to_string())
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not return a 400 Bad Request when the payload was {}.",
        error_message
    );
}
// region: -- SurrealDB: Initialize & Migration
async fn migrate_db(configuration: Settings) -> Result<(), surrealdb::Error> {
    let db_configuration = configuration.database.with_db();

    SurrealdbMigrations::new(db_configuration)
        .up()
        .await
        .expect("Failed to run migrations.");

    Ok(())
}
// endregion: --- SurrealDB: Initialize & Migration
