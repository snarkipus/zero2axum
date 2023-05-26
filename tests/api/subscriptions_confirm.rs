use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use zero2axum::db;

// region: -- Confirmations w/o token rejected with 400 Bad Request
#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = reqwest::get(&format!(
        "http://{}:{}/subscribe/confirm",
        app.configuration.application.host, app.configuration.application.port
    ))
    .await
    .unwrap();

    // Assert
    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not return a 400 Bad Request when the token was missing."
    );
}
// endregion: -- Confirmations w/o token rejected with 400 Bad Request

// region: -- Link Returned by subscribe returns a 200 Ok
#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    // Act
    let response = reqwest::get(confirmation_links.html).await.unwrap();

    // Assert
    assert_eq!(200, response.status().as_u16());
}
// endregion: -- Link Returned by subscribe returns a 200 Ok

// region: -- Clicking on the Confirmation Link Confirms a Subscriber
#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    // Act
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Assert
    let db = match db::create_db_client(app.configuration.clone()).await {
        Ok(db) => db,
        Err(e) => panic!("Failed to create database: {}", e),
    };

    let sql = "SELECT email, name, status FROM subscriptions";
    let mut res = db
        .query(sql)
        .await
        .expect("Failed to fetch saved subscription.");

    #[derive(serde::Deserialize)]
    struct TestQuery {
        email: String,
        name: String,
        status: String,
    }

    let saved: Option<TestQuery> = res.take(0).unwrap();
    match saved {
        Some(s) => {
            assert_eq!(s.email, "ursula_le_guin@gmail.com");
            assert_eq!(s.name, "le guin");
            assert_eq!(s.status, "confirmed");
        }
        None => panic!("No subscription found."),
    }
}

// endregion: -- Clicking on the Confirmation Link Confirms a Subscriber
