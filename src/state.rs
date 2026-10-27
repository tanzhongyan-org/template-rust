use crate::core::config::AppConfig;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: sqlx::PgPool,
}

impl AppState {
    pub fn new(config: AppConfig, db: sqlx::PgPool) -> Self {
        Self {
            config: Arc::new(config),
            db,
        }
    }
}
