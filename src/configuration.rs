use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use surrealdb_migrations::SurrealdbConfiguration;

use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct ApplicationSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub base_url: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> SurrealdbConfiguration {
        let mut db_configuration = SurrealdbConfiguration::default();
        db_configuration.url = Some(format!("{}:{}", &self.host, &self.port));
        db_configuration.ns = Some("default".to_string());
        db_configuration.username = Some(self.username.clone());
        db_configuration.password = Some(self.password.expose_secret().to_string());
        db_configuration
    }

    pub fn with_db(&self) -> SurrealdbConfiguration {
        let mut db_configuration = self.without_db();
        db_configuration.db = Some(self.database_name.clone());
        db_configuration
    }

    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!("{}:{}", self.host, self.port))
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!("{}:{}", self.host, self.port))
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory.");
    let configuration_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    if environment == Environment::Production {
        let mut base_url = std::env::var("FLY_APP_NAME").expect("FLY_APP_NAME must be set");
        base_url.push_str(".fly.dev");
        std::env::set_var("APP_APPLICATION__BASE_URL", base_url);
    };

    if environment.as_str() == "production" {
        let mut base_url = std::env::var("FLY_APP_NAME").expect("FLY_APP_NAME must be set");
        base_url.push_str(".fly.dev");
    }

    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "Unsupported environment: {}. Use either 'local' or 'production'.",
                other
            )),
        }
    }
}
