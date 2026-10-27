use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub app_name: String,
    pub app_env: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        let app_name = env::var("APP_NAME").unwrap_or_else(|_| "template-rust".to_string());
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);

        Self {
            app_name,
            app_env,
            port,
        }
    }
}
