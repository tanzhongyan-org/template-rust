use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

impl HealthStatus {
    pub fn new() -> Self {
        Self {
            status: "ok".to_string(),
            timestamp: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status() {
        let status = HealthStatus::new();
        assert_eq!(status.status, "ok");
        assert_eq!(status.version, env!("CARGO_PKG_VERSION"));
    }
}
