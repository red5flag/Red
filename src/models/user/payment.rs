use crate::types::{Currency, PaymentInterval, PaymentMethod};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaymentSettings {
    pub payment_method: PaymentMethod,
    pub account_details: String,
    pub payment_interval: PaymentInterval,
    pub currency: Currency,
    pub automatic_payout: bool,
    pub payout_threshold: Option<f64>,
}

impl Default for PaymentSettings {
    fn default() -> Self {
        Self {
            payment_method: PaymentMethod::BankTransfer,
            account_details: String::new(),
            payment_interval: PaymentInterval::Monthly,
            currency: Currency::USD,
            automatic_payout: true,
            payout_threshold: None,
        }
    }
}

// Payment/Transaction record
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub amount: f64,
    pub currency: Currency,
    pub payment_method: PaymentMethod,
    pub description: Option<String>,
    pub related_asset_id: Option<Uuid>,
    pub related_portfolio_id: Option<Uuid>,
    pub status: PaymentStatus,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub executed_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub is_recurring: bool,
    pub recurrence_rule: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Scheduled,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl Payment {
    pub fn new(
        from_user_id: Uuid,
        to_user_id: Uuid,
        amount: f64,
        currency: Currency,
        payment_method: PaymentMethod,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_user_id,
            to_user_id,
            amount,
            currency,
            payment_method,
            description: None,
            related_asset_id: None,
            related_portfolio_id: None,
            status: PaymentStatus::Pending,
            scheduled_date: None,
            executed_date: None,
            created_at: Utc::now(),
            is_recurring: false,
            recurrence_rule: None,
        }
    }

    pub fn schedule(&mut self, date: DateTime<Utc>) {
        self.scheduled_date = Some(date);
        self.status = PaymentStatus::Scheduled;
    }

    pub fn mark_completed(&mut self) {
        self.status = PaymentStatus::Completed;
        self.executed_date = Some(Utc::now());
    }
}
