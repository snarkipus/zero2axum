use once_cell::sync::Lazy;
use surrealdb::{sql::Thing, Surreal, engine::remote::ws::Client};
use uuid::Uuid;
use wiremock::MockServer;
use zero2axum::{
    configuration::{get_configuration, Settings},
    db::Database,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber}
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
    color_eyre::install().expect("Failed to install `color_eyre`");
});
// endregion: -- conditional tracing for tests

// region: -- spawn_app
#[derive(Clone, Debug)]
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestApp {
    pub configuration: Settings,
    pub email_server: MockServer,
    pub database: Database,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!(
                "http://{}:{}/subscribe",
                &self.configuration.application.host, &self.configuration.application.port
            ))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.to_string())
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link
                .set_port(Some(self.configuration.application.port))
                .unwrap();
            confirmation_link
        };

        let html = get_link(body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        let (username, password) = self.test_user().await;
        dbg!(username.clone());
        dbg!(password.clone());
        reqwest::Client::new()
            .post(&format!(
                "http://{}:{}/newsletters",
                &self.configuration.application.host, &self.configuration.application.port
            ))
            // .basic_auth(Uuid::new_v4().to_string(), Some(Uuid::new_v4().to_string()))
            .basic_auth(username, Some(password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn test_user(&self) -> (String, String) {
        let sql = "SELECT username, password from users LIMIT 1";

        let mut res = self.database.client.query(sql).await.unwrap().check().expect("Failed to get test user.");
        let username: Option<String> = res.take((0, "username")).unwrap();
        let password: Option<String> = res.take((0, "password")).unwrap();

        (username.unwrap(), password.unwrap())
    }
}

#[allow(clippy::let_underscore_future)]
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let mut configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    let database = Database::new(&configuration)
        .await
        .expect("Failed to create to database.");

    database
        .migrate(&configuration)
        .await
        .expect("Failed to migrate database.");

    let application = Application::build(configuration.clone(), database.clone())
        .await
        .expect("Failed to build application.");

    configuration.application.port = application.port();

    let _ = tokio::spawn(application.run_until_stopped());

    let test_app = TestApp {
        configuration,
        email_server,
        database,
    };

    add_test_user(&test_app.database.client).await;
    test_app
}
// endregion: -- spawn_app

async fn add_test_user(conn: &Surreal<Client>) {
    let sql = format!("INSERT INTO users (id, username, password) VALUES ({}, '{}', '{}')",
        Thing::from(("users".to_string(), Uuid::new_v4().to_string())),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );

    conn.query(sql).await.unwrap().check().expect("Failed to add test user.");
}