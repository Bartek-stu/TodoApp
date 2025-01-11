use config::Case;
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub cosmos: CosmosSettings,
    pub telemetry: TelemetrySettings,
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config_dir = std::env::current_dir().expect("Failed to determine current directory");

        config::Config::builder()
            .add_source(config::File::from(config_dir.join("configuration")).required(false))
            .add_source(
                config::Environment::with_prefix("APP")
                    .try_parsing(true)
                    .separator("__")
                    .keep_prefix(false),
            )
            .build()?
            .try_deserialize()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CosmosSettings {
    pub account: SecretString,
    pub primary_key: SecretString,
    pub database_name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TelemetrySettings {
    pub log_level: String,
    pub app_insights_connection_string: SecretString,
}
