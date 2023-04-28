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
