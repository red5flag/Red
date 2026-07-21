use crate::types::{short_uuid_suffix, Currency};
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Draft,
    Pending,
    Confirmed,
    Cancelled,
    Completed,
}

impl Default for OrderStatus {
    fn default() -> Self {
        OrderStatus::Draft
    }
}

/// A purchase or sales order linked to assets, portfolios, or external parties.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    #[serde(default)]
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub country_code: String,
    #[serde(default)]
    pub status: OrderStatus,
    pub total: f64,
    pub currency: Currency,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn new(name: String, country_code: String, currency: Currency) -> Self {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let code = format!(
            "ORD-{}-{}-{}",
            country_code.to_uppercase(),
            now.year(),
            short_uuid_suffix(id, 6)
        );
        Self {
            id,
            code,
            name,
            description: None,
            country_code: country_code.to_uppercase(),
            status: OrderStatus::Draft,
            total: 0.0,
            currency,
            created_at: now,
            updated_at: now,
        }
    }
}
