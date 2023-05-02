#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

impl DatabaseSettings {
    // region: -- connection_string
    pub fn connection_string(&self) -> String {
        format!("http://{}:{}/", self.host, self.port)
    }
    // endregion: -- connection_string

    // region: -- connection_string_without_db
    pub fn connection_string_without_db(&self) -> String {
        format!("http://{}:{}/", self.host, self.port)
    }
    // endregion: -- connection_string_without_db
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialize our configuraiton reader
    let settings = config::Config::builder()
        // Add configuration values from a file named `configuration.yaml`
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;

    // Try to convert the configuration values it read into our Settings type
    settings.try_deserialize::<Settings>()
}
