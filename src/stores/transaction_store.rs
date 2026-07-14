use crate::models::{ApprovalRecord, EntityReference, EntityType, Transaction, TransactionStatus};
use crate::types::{Currency, TransactionType, UserRole};
use leptos::prelude::*;
use uuid::Uuid;

/// Dedicated store for transaction state and lifecycle methods.
/// Extracted from AppStore so transaction changes do not invalidate
/// consumers of unrelated domain state.
#[derive(Clone, Debug)]
pub struct TransactionStore {
    pub transactions: Vec<Transaction>,
}

impl Default for TransactionStore {
    fn default() -> Self {
        Self {
            transactions: Vec::new(),
        }
    }
}

impl TransactionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.push(tx);
    }

    /// Returns a mutable reference to the transaction with the given id, if present.
    pub fn transaction_by_id_mut(&mut self, id: Uuid) -> Option<&mut Transaction> {
        self.transactions.iter_mut().find(|t| t.id == id)
    }

    /// Returns a reference to the transaction with the given id, if present.
    pub fn transaction_by_id(&self, id: Uuid) -> Option<&Transaction> {
        self.transactions.iter().find(|t| t.id == id)
    }

    /// Record an approval action for a transaction.
    pub fn record_approval(
        &mut self,
        transaction_id: Uuid,
        record: ApprovalRecord,
    ) -> Result<(), String> {
        let tx = self
            .transaction_by_id_mut(transaction_id)
            .ok_or("Transaction not found")?;
        tx.record_approval(record);
        Ok(())
    }

    // Developer/test helpers

    /// Create a pending dev/test transaction and return it so the caller can
    /// emit the same notification as before the store extraction.
    pub fn dev_test_add_transaction(
        &mut self,
        current_user_id: Uuid,
        current_user_name: String,
        amount: f64,
        desc: &str,
    ) -> Transaction {
        let from_e = EntityReference {
            entity_type: EntityType::External,
            entity_id: Uuid::new_v4(),
            name: "Bot Corp".into(),
        };
        let to_e = EntityReference {
            entity_type: EntityType::User,
            entity_id: current_user_id,
            name: current_user_name,
        };
        let mut tx = Transaction::new(
            TransactionType::Transfer,
            amount,
            Currency::USD,
            from_e,
            to_e,
            current_user_id,
        );
        tx.description = Some(desc.into());
        tx.status = TransactionStatus::Pending;
        self.transactions.push(tx.clone());
        tx
    }

    /// Approve the most recent transaction and return it so the caller can
    /// emit the same notification as before the store extraction.
    pub fn dev_test_approve_last_tx(&mut self) -> Option<&Transaction> {
        if let Some(tx) = self.transactions.last_mut() {
            tx.approve();
        }
        self.transactions.last()
    }

    /// Execute the most recent transaction if it is approved and return it so
    /// the caller can emit the same notification as before the store extraction.
    pub fn dev_test_execute_last_tx(&mut self) -> Option<&Transaction> {
        if let Some(tx) = self.transactions.last_mut() {
            if tx.status == TransactionStatus::Approved {
                tx.execute();
            }
        }
        self.transactions
            .last()
            .filter(|tx| tx.status == TransactionStatus::Executed)
    }
}

/// Map a user role to the coarse transaction permissions it should have.
/// The app currently uses `UserRole` on `UserProfile` for most checks;
/// this helper maps those roles to the granular `Perm` semantics for the
/// transaction approval workflow.
pub fn role_can_create_transactions(role: &UserRole) -> bool {
    matches!(
        role,
        UserRole::Owner
            | UserRole::Director
            | UserRole::SeniorManager
            | UserRole::Manager
            | UserRole::Worker
            | UserRole::Contractor
    )
}

pub fn role_can_submit_transactions(role: &UserRole) -> bool {
    matches!(
        role,
        UserRole::Owner
            | UserRole::Director
            | UserRole::SeniorManager
            | UserRole::Manager
            | UserRole::Worker
    )
}

pub fn role_can_approve_transactions(role: &UserRole) -> bool {
    matches!(
        role,
        UserRole::Owner | UserRole::Director | UserRole::SeniorManager | UserRole::Manager
    )
}

pub fn role_can_reject_transactions(role: &UserRole) -> bool {
    role_can_approve_transactions(role)
}

pub fn role_can_execute_transactions(role: &UserRole) -> bool {
    role_can_approve_transactions(role)
}

pub fn role_can_lock_transactions(role: &UserRole) -> bool {
    matches!(
        role,
        UserRole::Owner | UserRole::Director | UserRole::SeniorManager
    )
}

pub fn role_can_edit_any_draft(role: &UserRole) -> bool {
    matches!(
        role,
        UserRole::Owner | UserRole::Director | UserRole::SeniorManager | UserRole::Manager
    )
}

pub fn role_can_edit_own_draft(role: &UserRole) -> bool {
    matches!(
        role,
        UserRole::Owner
            | UserRole::Director
            | UserRole::SeniorManager
            | UserRole::Manager
            | UserRole::Worker
            | UserRole::Contractor
    )
}

pub fn role_can_withdraw_submitted(
    role: &UserRole,
    submitted_by: Option<Uuid>,
    actor_id: Uuid,
) -> bool {
    if role_can_approve_transactions(role) {
        return true;
    }
    submitted_by == Some(actor_id) && role_can_submit_transactions(role)
}

/// Check if an actor can approve a specific transaction (prevents self-approval).
pub fn can_approve_transaction(tx: &Transaction, actor_id: Uuid, role: &UserRole) -> bool {
    if !role_can_approve_transactions(role) {
        return false;
    }
    tx.submitted_by != Some(actor_id)
}

/// Check if an actor can reject a specific transaction (prevents self-rejection).
pub fn can_reject_transaction(tx: &Transaction, actor_id: Uuid, role: &UserRole) -> bool {
    if !role_can_reject_transactions(role) {
        return false;
    }
    tx.submitted_by != Some(actor_id)
}

/// Check if an actor can submit a transaction.
pub fn can_submit_transaction(tx: &Transaction, actor_id: Uuid, role: &UserRole) -> bool {
    if !role_can_submit_transactions(role) {
        return false;
    }
    matches!(tx.status, TransactionStatus::Draft)
        && (tx.executed_by == actor_id || role_can_edit_any_draft(role))
}

/// Check if an actor can withdraw a submitted transaction.
pub fn can_withdraw_transaction(tx: &Transaction, actor_id: Uuid, role: &UserRole) -> bool {
    if !matches!(tx.status, TransactionStatus::Pending) {
        return false;
    }
    role_can_withdraw_submitted(role, tx.submitted_by, actor_id)
}

/// Check if an actor can execute a transaction.
pub fn can_execute_transaction(tx: &Transaction, actor_id: Uuid, role: &UserRole) -> bool {
    if !role_can_execute_transactions(role) {
        return false;
    }
    matches!(tx.status, TransactionStatus::Approved) && tx.submitted_by != Some(actor_id)
}

/// Check if an actor can lock a transaction.
pub fn can_lock_transaction(tx: &Transaction, role: &UserRole) -> bool {
    if !role_can_lock_transactions(role) {
        return false;
    }
    matches!(
        tx.status,
        TransactionStatus::Executed | TransactionStatus::Approved
    ) && !tx.locked
}

pub fn create_transaction_store() -> RwSignal<TransactionStore> {
    RwSignal::new(TransactionStore::new())
}

pub fn provide_transaction_store() -> RwSignal<TransactionStore> {
    let store = create_transaction_store();
    provide_context(store);
    store
}

pub fn use_transaction_store() -> RwSignal<TransactionStore> {
    expect_context::<RwSignal<TransactionStore>>()
}
