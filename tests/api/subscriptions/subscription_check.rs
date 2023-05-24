use rstest::rstest;
use zero2axum::{db, routes::FormData};

use crate::helpers::spawn_app;
use crate::subscriptions::helpers::migrate_db;

// region: -- POST Form: 200 OK
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let db = match db::create_db(app.configuration.clone()).await {
        Ok(db) => db,
        Err(e) => panic!("Failed to create database: {}", e),
    };
    migrate_db(app.configuration.clone())
        .await
        .expect("Failed to migrate database.");
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Act
    let response = app.post_subscriptions(body.into()).await;

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

    // Act
    let response = app.post_subscriptions(invalid_body.into()).await;

    // Assert
    assert_eq!(
        422,
        response.status().as_u16(),
        "The API did not fail with 422 Bad Request when the payload was {}.",
        error_message
    );
}
// endregion: -- POST Form: 422 Unprocessable Entity

// region: -- POST Form: 400 w/fields present but empty
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
// endregion: -- POST Form: 200 w/fields present but empty
