// src/config.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_address: String,
    pub log_path: String,
    pub target_path: String,
    pub note_path: String,
    pub key_path: String,
    pub extension: String,
    pub retries: u64,
    pub timeout_seconds: u64,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            .add_source(config::File::with_name(path));

        builder.build()?.try_deserialize()
    }
}