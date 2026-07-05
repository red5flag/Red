use crate::models::{EntityReference, EntityType, Transaction, TransactionStatus};
use crate::types::{Currency, TransactionType};
use chrono::Utc;
use uuid::Uuid;

pub mod approval_panel;
pub mod page;
pub mod transaction_card;
pub mod transaction_detail;
pub mod transaction_filters;
pub mod transaction_form;
pub mod transaction_list;
pub mod transaction_summary;

pub use page::TransactionsPage;

/// Local card model for wallet display.
#[derive(Clone, Debug)]
pub struct Card {
    pub id: Uuid,
    pub label: String,
    pub last4: String,
}

/// Local wallet model for transaction page UI.
#[derive(Clone, Debug)]
pub struct Wallet {
    pub id: Uuid,
    pub name: String,
    pub wallet_type: String,
    pub balance: f64,
    pub currency: String,
    pub cards: Vec<Card>,
}

/// Local contact model for payees/payers.
#[derive(Clone, Debug)]
pub struct Contact {
    pub id: Uuid,
    pub name: String,
    pub account: String,
    pub currency: String,
}

/// Local payment record model.
#[derive(Clone, Debug)]
pub struct Payment {
    pub _id: Uuid,
    pub direction: String,
    pub from_wallet_id: Uuid,
    pub from_card_id: Option<Uuid>,
    pub to_contact_id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub is_routine: bool,
    #[allow(dead_code)]
    pub interval: String,
    pub is_auto: bool,
    pub created_at: chrono::DateTime<Utc>,
}

/// Local invoice model.
#[derive(Clone, Debug)]
pub struct Invoice {
    #[allow(dead_code)]
    pub id: Uuid,
    pub number: String,
    pub issue_date: chrono::DateTime<Utc>,
    pub due_date: chrono::DateTime<Utc>,
    pub amount: f64,
    pub currency: String,
    pub status: InvoiceStatus,
    pub from_name: String,
    pub to_name: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvoiceStatus {
    Draft,
    Sent,
    #[allow(dead_code)]
    Viewed,
    Paid,
    Overdue,
    #[allow(dead_code)]
    Cancelled,
}

impl InvoiceStatus {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            InvoiceStatus::Draft => "Draft",
            InvoiceStatus::Sent => "Sent",
            InvoiceStatus::Viewed => "Viewed",
            InvoiceStatus::Paid => "Paid",
            InvoiceStatus::Overdue => "Overdue",
            InvoiceStatus::Cancelled => "Cancelled",
        }
    }
}

pub(crate) fn invoice_status_class(status: &InvoiceStatus) -> &'static str {
    match status {
        InvoiceStatus::Draft => "tx-status-draft",
        InvoiceStatus::Sent => "tx-status-pending",
        InvoiceStatus::Viewed => "tx-status-pending",
        InvoiceStatus::Paid => "tx-status-success",
        InvoiceStatus::Overdue => "tx-status-danger",
        InvoiceStatus::Cancelled => "tx-status-muted",
    }
}

pub(crate) fn currency_symbol(currency: &str) -> &str {
    match currency {
        "USD" => "$",
        "AUD" => "A$",
        "EUR" => "€",
        "GBP" => "£",
        "BTC" => "₿",
        "ETH" => "Ξ",
        _ => "$",
    }
}

pub(crate) fn format_dollars(amount: f64, currency: &str) -> String {
    let whole = amount.trunc() as i64;
    let cents = ((amount.fract() * 100.0).round() as i64).abs();
    let s = whole.abs().to_string();
    let mut grouped = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    let grouped = grouped.chars().rev().collect::<String>();
    let sign = if amount < 0.0 { "-" } else { "" };
    format!(
        "{}{}{}.{:02} {}",
        sign,
        currency_symbol(currency),
        grouped,
        cents,
        currency
    )
}

pub(crate) fn format_date(d: &chrono::DateTime<Utc>) -> String {
    d.format("%d %b %Y").to_string()
}

pub(crate) fn create_mock_transaction(
    transaction_type: TransactionType,
    amount: f64,
    description: &str,
    from: &str,
    to: &str,
    status: TransactionStatus,
) -> Transaction {
    Transaction {
        id: Uuid::new_v4(),
        transaction_type,
        amount,
        currency: Currency::USD,
        description: Some(description.to_string()),
        from_entity: EntityReference {
            entity_type: EntityType::Organization,
            entity_id: Uuid::new_v4(),
            name: from.to_string(),
        },
        to_entity: EntityReference {
            entity_type: EntityType::External,
            entity_id: Uuid::new_v4(),
            name: to.to_string(),
        },
        related_portfolio_id: None,
        related_asset_group_id: None,
        related_asset_id: None,
        executed_by: Uuid::new_v4(),
        status,
        created_at: Utc::now(),
        executed_at: Some(Utc::now()),
        metadata: serde_json::json!({}),
    }
}
