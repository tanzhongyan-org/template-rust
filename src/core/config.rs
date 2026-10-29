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
    /// Product JWKS endpoint (Better Auth `jwt` plugin). When set, bearer
    /// tokens are validated and `None` means bearer tokens are refused.
    pub product_jwks_url: Option<String>,
    /// Expected token issuer. Checked only when set.
    pub product_jwt_issuer: Option<String>,
    /// Expected token audience. Checked only when set.
    pub product_jwt_audience: Option<String>,
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
            product_jwks_url: None,
            product_jwt_issuer: None,
            product_jwt_audience: None,
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

        let app_env = get("APP_ENV").unwrap_or(defaults.app_env);
        if app_env == "production" && trust_identity_header {
            return Err(AppError::Config(
                "TRUST_IDENTITY_HEADER must be false when APP_ENV=production; use product JWTs instead"
                    .to_string(),
            ));
        }

        Ok(Self {
            host: get("HOST").unwrap_or(defaults.host),
            port,
            app_name: get("APP_NAME").unwrap_or(defaults.app_name),
            app_env,
            log_level: get("LOG_LEVEL")
                .or_else(|| get("RUST_LOG"))
                .unwrap_or(defaults.log_level),
            database_url: get("DATABASE_URL").unwrap_or(defaults.database_url),
            trust_identity_header,
            cors_allowed_origins: parse_origins(get("CORS_ALLOWED_ORIGINS").as_deref())?,
            product_jwks_url: parse_optional_url(
                "PRODUCT_JWKS_URL",
                get("PRODUCT_JWKS_URL").as_deref(),
            )?,
            product_jwt_issuer: optional_value(get("PRODUCT_JWT_ISSUER").as_deref()),
            product_jwt_audience: optional_value(get("PRODUCT_JWT_AUDIENCE").as_deref()),
        })
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Reads an optional setting: missing or blank means `None`.
fn optional_value(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// Reads an optional URL setting, failing boot on a malformed value.
fn parse_optional_url(key: &str, raw: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(url) = optional_value(raw) else {
        return Ok(None);
    };
    reqwest::Url::parse(&url)
        .map_err(|_| AppError::Config(format!("Invalid {key} value: {url}")))?;
    Ok(Some(url))
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
        assert!(load(&[("PRODUCT_JWKS_URL", "not a url")]).is_err());
    }

    #[test]
    fn reads_jwt_settings() {
        let config = load(&[
            ("PRODUCT_JWKS_URL", "https://app.example.com/api/auth/jwks"),
            ("PRODUCT_JWT_ISSUER", "https://app.example.com"),
            ("PRODUCT_JWT_AUDIENCE", "my-service"),
        ])
        .unwrap();
        assert_eq!(
            config.product_jwks_url.as_deref(),
            Some("https://app.example.com/api/auth/jwks")
        );
        assert_eq!(
            config.product_jwt_issuer.as_deref(),
            Some("https://app.example.com")
        );
        assert_eq!(config.product_jwt_audience.as_deref(), Some("my-service"));

        let blank = load(&[
            ("PRODUCT_JWKS_URL", "  "),
            ("PRODUCT_JWT_ISSUER", ""),
            ("PRODUCT_JWT_AUDIENCE", " "),
        ])
        .unwrap();
        assert_eq!(blank.product_jwks_url, None);
        assert_eq!(blank.product_jwt_issuer, None);
        assert_eq!(blank.product_jwt_audience, None);
    }

    #[test]
    fn dev_identity_is_refused_in_production() {
        let err = load(&[("APP_ENV", "production"), ("TRUST_IDENTITY_HEADER", "true")])
            .expect_err("production must not trust the identity header");
        assert!(matches!(err, AppError::Config(_)));
        assert!(load(&[("APP_ENV", "production")]).is_ok());
        assert!(
            load(&[
                ("APP_ENV", "development"),
                ("TRUST_IDENTITY_HEADER", "true"),
            ])
            .is_ok()
        );
    }
}
