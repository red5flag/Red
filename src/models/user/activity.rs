use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserActivity {
    pub action: String,
    pub target_type: String,
    pub target_name: String,
    pub timestamp: DateTime<Utc>,
    pub reason: Option<String>,
}
