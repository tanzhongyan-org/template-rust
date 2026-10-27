use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthStatus {
    pub status: String,
    pub app_name: String,
    pub environment: String,
    pub timestamp: DateTime<Utc>,
}

pub fn get_health(app_name: &str, environment: &str) -> HealthStatus {
    HealthStatus {
        status: "healthy".to_string(),
        app_name: app_name.to_string(),
        environment: environment.to_string(),
        timestamp: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_health() {
        let app = "template-rust";
        let env = "test";
        let status = get_health(app, env);

        assert_eq!(status.status, "healthy");
        assert_eq!(status.app_name, app);
        assert_eq!(status.environment, env);
    }
}
