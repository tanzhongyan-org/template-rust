use axum::http::HeaderValue;

use crate::core::error::AppError;

const DEFAULT_DATABASE_URL: &str = "postgres://postgres:password@localhost:5432/template_rust";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub app_name: String,
    pub app_env: String,
    pub log_level: String,
    pub database_url: String,
    /// Honour the `x-user-id` header as the caller identity. Off unless
    /// `TRUST_IDENTITY_HEADER=true`.
    pub trust_identity_header: bool,
    /// Browser origins allowed by CORS. Empty means CORS is disabled.
    pub cors_allowed_origins: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8000,
            app_name: "template-rust".to_string(),
            app_env: "development".to_string(),
            log_level: "info".to_string(),
            database_url: DEFAULT_DATABASE_URL.to_string(),
            trust_identity_header: false,
            cors_allowed_origins: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// Builds the configuration from any key lookup, so tests need no process environment.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, AppError> {
        let defaults = Self::default();
        let port = match get("PORT") {
            Some(val) => val
                .parse::<u16>()
                .map_err(|_| AppError::Config(format!("Invalid PORT value: {val}")))?,
            None => defaults.port,
        };
        let trust_identity_header = match get("TRUST_IDENTITY_HEADER").as_deref() {
            None | Some("false") => false,
            Some("true") => true,
            Some(other) => {
                return Err(AppError::Config(format!(
                    "Invalid TRUST_IDENTITY_HEADER value: {other} (use true or false)"
                )));
            }
        };

        Ok(Self {
            host: get("HOST").unwrap_or(defaults.host),
            port,
            app_name: get("APP_NAME").unwrap_or(defaults.app_name),
            app_env: get("APP_ENV").unwrap_or(defaults.app_env),
            log_level: get("LOG_LEVEL")
                .or_else(|| get("RUST_LOG"))
                .unwrap_or(defaults.log_level),
            database_url: get("DATABASE_URL").unwrap_or(defaults.database_url),
            trust_identity_header,
            cors_allowed_origins: parse_origins(get("CORS_ALLOWED_ORIGINS").as_deref())?,
        })
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn parse_origins(raw: Option<&str>) -> Result<Vec<String>, AppError> {
    let mut origins = Vec::new();
    for origin in raw.unwrap_or("").split(',').map(str::trim) {
        if origin.is_empty() {
            continue;
        }
        if origin == "*" || HeaderValue::from_str(origin).is_err() {
            return Err(AppError::Config(format!(
                "Invalid CORS_ALLOWED_ORIGINS entry: {origin} (list full origins such as https://app.example.com)"
            )));
        }
        origins.push(origin.to_string());
    }
    Ok(origins)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn load(pairs: &[(&str, &str)]) -> Result<AppConfig, AppError> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        AppConfig::from_lookup(|key| map.get(key).cloned())
    }

    #[test]
    fn defaults_are_fail_closed() {
        let config = load(&[]).unwrap();
        assert_eq!(config, AppConfig::default());
        assert!(!config.trust_identity_header);
        assert!(config.cors_allowed_origins.is_empty());
        assert_eq!(config.server_addr(), "0.0.0.0:8000");
    }

    #[test]
    fn reads_identity_and_cors_settings() {
        let config = load(&[
            ("TRUST_IDENTITY_HEADER", "true"),
            (
                "CORS_ALLOWED_ORIGINS",
                "https://a.example.com, https://b.example.com",
            ),
        ])
        .unwrap();
        assert!(config.trust_identity_header);
        assert_eq!(
            config.cors_allowed_origins,
            ["https://a.example.com", "https://b.example.com"]
        );
    }

    #[test]
    fn rejects_bad_values() {
        assert!(load(&[("PORT", "abc")]).is_err());
        assert!(load(&[("TRUST_IDENTITY_HEADER", "yes")]).is_err());
        assert!(load(&[("CORS_ALLOWED_ORIGINS", "*")]).is_err());
    }
}
