use crate::types::{Currency, TransactionType, UserRole};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Transaction record for financial tracking
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub transaction_type: TransactionType,
    pub amount: f64,
    pub currency: Currency,
    pub description: Option<String>,
    pub from_entity: EntityReference,
    pub to_entity: EntityReference,
    pub related_portfolio_id: Option<Uuid>,
    pub related_asset_group_id: Option<Uuid>,
    pub related_asset_id: Option<Uuid>,
    pub executed_by: Uuid,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    #[serde(default)]
    pub submitted_by: Option<Uuid>,
    #[serde(default)]
    pub submitted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub approved_by: Option<Uuid>,
    #[serde(default)]
    pub approved_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub rejected_by: Option<Uuid>,
    #[serde(default)]
    pub rejected_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub rejection_reason: Option<String>,
    #[serde(default)]
    pub approval_history: Vec<ApprovalRecord>,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Draft,
    Pending,
    Approved,
    Rejected,
    Executed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalAction {
    Submit,
    Approve,
    Reject,
    Withdraw,
    Execute,
    Lock,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub id: Uuid,
    pub actor_id: Uuid,
    pub actor_name: String,
    pub actor_role: String,
    pub action: ApprovalAction,
    pub timestamp: DateTime<Utc>,
    pub comment: Option<String>,
}

impl ApprovalRecord {
    pub fn new(
        actor_id: Uuid,
        actor_name: String,
        actor_role: String,
        action: ApprovalAction,
        comment: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            actor_id,
            actor_name,
            actor_role,
            action,
            timestamp: Utc::now(),
            comment,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EntityReference {
    pub entity_type: EntityType,
    pub entity_id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    User,
    Organization,
    Portfolio,
    AssetGroup,
    Asset,
    External,
    System,
}

impl Transaction {
    pub fn new(
        transaction_type: TransactionType,
        amount: f64,
        currency: Currency,
        from_entity: EntityReference,
        to_entity: EntityReference,
        executed_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            transaction_type,
            amount,
            currency,
            description: None,
            from_entity,
            to_entity,
            related_portfolio_id: None,
            related_asset_group_id: None,
            related_asset_id: None,
            executed_by,
            status: TransactionStatus::Draft,
            created_at: Utc::now(),
            executed_at: None,
            metadata: serde_json::json!({}),
            submitted_by: None,
            submitted_at: None,
            approved_by: None,
            approved_at: None,
            rejected_by: None,
            rejected_at: None,
            rejection_reason: None,
            approval_history: Vec::new(),
            locked: false,
        }
    }

    pub fn record_approval(&mut self, record: ApprovalRecord) {
        match record.action {
            ApprovalAction::Submit => {
                self.status = TransactionStatus::Pending;
                self.submitted_by = Some(record.actor_id);
                self.submitted_at = Some(record.timestamp);
            }
            ApprovalAction::Approve => {
                self.status = TransactionStatus::Approved;
                self.approved_by = Some(record.actor_id);
                self.approved_at = Some(record.timestamp);
            }
            ApprovalAction::Reject => {
                self.status = TransactionStatus::Rejected;
                self.rejected_by = Some(record.actor_id);
                self.rejected_at = Some(record.timestamp);
                self.rejection_reason = record.comment.clone();
            }
            ApprovalAction::Withdraw => {
                self.status = TransactionStatus::Draft;
                self.submitted_by = None;
                self.submitted_at = None;
            }
            ApprovalAction::Execute => {
                self.status = TransactionStatus::Executed;
                self.executed_at = Some(record.timestamp);
            }
            ApprovalAction::Lock => {
                self.locked = true;
            }
        }
        self.approval_history.push(record);
    }

    pub fn approve(&mut self) {
        self.status = TransactionStatus::Approved;
    }

    pub fn reject(&mut self, reason: Option<String>) {
        self.status = TransactionStatus::Rejected;
        if let Some(r) = reason {
            self.rejection_reason = Some(r);
            self.metadata["rejection_reason"] = serde_json::json!(self.rejection_reason.clone());
        }
    }

    pub fn execute(&mut self) {
        self.status = TransactionStatus::Executed;
        self.executed_at = Some(Utc::now());
    }

    /// Returns true if the given actor can edit this transaction.
    /// Locked transactions are not editable; only drafts are editable,
    /// and own-drafts can only be edited by the creator unless the actor has broader permission.
    pub fn can_edit(&self, actor_id: Uuid, actor_role: &UserRole, _can_edit_any: bool) -> bool {
        if self.locked || !matches!(self.status, TransactionStatus::Draft) {
            return false;
        }
        if _can_edit_any {
            return true;
        }
        self.executed_by == actor_id || matches!(actor_role, UserRole::Owner | UserRole::Director)
    }
}

// Transaction summary for reports
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub total_transactions: u64,
    pub total_inflow: f64,
    pub total_outflow: f64,
    pub net_flow: f64,
    pub by_type: Vec<(TransactionType, f64, u64)>,
    pub by_currency: Vec<(Currency, f64)>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

impl TransactionSummary {
    pub fn new(period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Self {
        Self {
            total_transactions: 0,
            total_inflow: 0.0,
            total_outflow: 0.0,
            net_flow: 0.0,
            by_type: Vec::new(),
            by_currency: Vec::new(),
            period_start,
            period_end,
        }
    }
}

// Quick sale request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuickSaleRequest {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub asset_group_id: Option<Uuid>,
    pub portfolio_id: Option<Uuid>,
    pub requested_price: f64,
    pub urgency: SaleUrgency,
    pub notes: Option<String>,
    pub requested_by: Uuid,
    pub status: QuickSaleStatus,
    pub created_at: DateTime<Utc>,
    pub notifications_sent: Vec<(NotificationChannel, DateTime<Utc>)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaleUrgency {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuickSaleStatus {
    Draft,
    Submitted,
    UnderReview,
    Approved,
    Listed,
    Sold,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Push,
    Sms,
    InApp,
}

impl QuickSaleRequest {
    pub fn new(requested_by: Uuid, requested_price: f64, urgency: SaleUrgency) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id: None,
            asset_group_id: None,
            portfolio_id: None,
            requested_price,
            urgency,
            notes: None,
            requested_by,
            status: QuickSaleStatus::Draft,
            created_at: Utc::now(),
            notifications_sent: Vec::new(),
        }
    }

    pub fn submit(&mut self) {
        self.status = QuickSaleStatus::Submitted;
    }

    pub fn record_notification(&mut self, channel: NotificationChannel) {
        self.notifications_sent.push((channel, Utc::now()));
    }
}
