use crate::helpers::spawn_app;

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
