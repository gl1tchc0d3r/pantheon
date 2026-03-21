use crate::session::SessionConfig;
use serde::Deserialize;
use std::env;
use std::fmt;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub provider: ProviderConfig,
    #[serde(default)]
    pub session: SessionConfig,
    #[serde(default)]
    pub identity: IdentityConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct IdentityConfig {
    #[serde(default = "default_soul_path")]
    pub soul_path: String,
    #[serde(default = "default_identity_path")]
    pub identity_path: String,
}

fn default_soul_path() -> String {
    ".ao/SOUL.md".to_string()
}

fn default_identity_path() -> String {
    ".ao/IDENTITY.md".to_string()
}

#[derive(Debug, Deserialize)]
pub struct ProviderConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    YamlError(serde_yaml::Error),
    MissingApiKey,
    NotFound(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::YamlError(e) => write!(f, "YAML error: {}", e),
            ConfigError::MissingApiKey => {
                write!(f, "Missing OPENROUTER_API_KEY environment variable")
            }
            ConfigError::NotFound(path) => write!(f, "Config file not found: {}", path),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> Self {
        ConfigError::YamlError(err)
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Err(ConfigError::NotFound(config_path.display().to_string()));
        }

        let contents = fs::read_to_string(&config_path)?;
        let mut config: Config = serde_yaml::from_str(&contents)?;

        let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| ConfigError::MissingApiKey)?;

        config.provider.api_key = api_key;

        Ok(config)
    }

    fn config_path() -> Result<PathBuf, ConfigError> {
        let cwd = env::current_dir()?;
        Ok(cwd.join(".ao").join("config.yaml"))
    }
}
