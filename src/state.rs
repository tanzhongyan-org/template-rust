use crate::core::config::AppConfig;
use crate::core::jwt::JwtValidator;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: sqlx::PgPool,
    /// Product JWT validator. `None` when no JWKS URL is configured, in
    /// which case bearer tokens are refused.
    pub jwt: Option<Arc<JwtValidator>>,
}

impl AppState {
    pub fn new(config: AppConfig, db: sqlx::PgPool, jwt: Option<JwtValidator>) -> Self {
        Self {
            config: Arc::new(config),
            db,
            jwt: jwt.map(Arc::new),
        }
    }
}
