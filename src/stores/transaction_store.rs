use crate::models::{EntityReference, EntityType, Transaction};
use crate::types::{Currency, TransactionType};
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
        tx.status = crate::models::TransactionStatus::Pending;
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
            if tx.status == crate::models::TransactionStatus::Approved {
                tx.execute();
            }
        }
        self.transactions
            .last()
            .filter(|tx| tx.status == crate::models::TransactionStatus::Executed)
    }
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
