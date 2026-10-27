use std::env;

use crate::core::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub app_name: String,
    pub app_env: String,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8000,
            app_name: "template-rust".to_string(),
            app_env: "development".to_string(),
            log_level: "info".to_string(),
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = match env::var("PORT") {
            Ok(val) => val
                .parse::<u16>()
                .map_err(|_| AppError::Config(format!("Invalid PORT value: {val}")))?,
            Err(_) => 8000,
        };
        let app_name = env::var("APP_NAME").unwrap_or_else(|_| "template-rust".to_string());
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let log_level = env::var("LOG_LEVEL")
            .or_else(|_| env::var("RUST_LOG"))
            .unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            host,
            port,
            app_name,
            app_env,
            log_level,
        })
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

pub type Config = AppConfig;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8000);
        assert_eq!(config.app_name, "template-rust");
        assert_eq!(config.app_env, "development");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.server_addr(), "0.0.0.0:8000");
    }

    #[test]
    fn test_app_config_from_env_defaults() {
        let config = AppConfig::from_env().expect("Config should load with defaults");
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8000);
    }
}
