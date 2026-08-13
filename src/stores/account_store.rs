use crate::pages::transactions::{Account, AccountType, Card};
use crate::types::Currency;
use leptos::prelude::*;
use uuid::Uuid;

/// Store for accounts and their cards, organized by organization.
/// When no organization exists (sole trader), accounts are linked to None.
#[derive(Clone, Debug)]
pub struct AccountStore {
    pub accounts: Vec<Account>,
}

impl Default for AccountStore {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
        }
    }
}

impl AccountStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_account(&mut self, account: Account) {
        self.accounts.push(account);
    }

    pub fn accounts_for_org(&self, org_id: Option<Uuid>) -> Vec<&Account> {
        self.accounts.iter().filter(|a| a.organization_id == org_id).collect()
    }

    pub fn accounts_for_org_owned(&self, org_id: Option<Uuid>) -> Vec<Account> {
        self.accounts.iter().filter(|a| a.organization_id == org_id).cloned().collect()
    }

    pub fn get_account(&self, id: Uuid) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    pub fn get_account_mut(&mut self, id: Uuid) -> Option<&mut Account> {
        self.accounts.iter_mut().find(|a| a.id == id)
    }

    pub fn add_card_to_account(&mut self, account_id: Uuid, mut card: Card) {
        if let Some(account) = self.get_account_mut(account_id) {
            card.account_id = account_id;
            account.cards.push(card);
        }
    }

    pub fn transactions_for_account(
        &self,
        account_id: Uuid,
        transactions: &[crate::models::Transaction],
    ) -> Vec<crate::models::Transaction> {
        transactions
            .iter()
            .filter(|t| t.account_id == Some(account_id))
            .cloned()
            .collect()
    }

    pub fn transactions_for_card(
        &self,
        card_id: Uuid,
        transactions: &[crate::models::Transaction],
    ) -> Vec<crate::models::Transaction> {
        transactions
            .iter()
            .filter(|t| t.card_id == Some(card_id))
            .cloned()
            .collect()
    }

    /// Transactions for an account that are not assigned to any specific card.
    pub fn account_level_transactions(
        &self,
        account_id: Uuid,
        transactions: &[crate::models::Transaction],
    ) -> Vec<crate::models::Transaction> {
        transactions
            .iter()
            .filter(|t| t.account_id == Some(account_id) && t.card_id.is_none())
            .cloned()
            .collect()
    }

    /// All transactions for an account (including its cards).
    pub fn all_transactions_for_account(
        &self,
        account_id: Uuid,
        transactions: &[crate::models::Transaction],
    ) -> Vec<crate::models::Transaction> {
        let card_ids: Vec<Uuid> = self
            .get_account(account_id)
            .map(|a| a.cards.iter().map(|c| c.id).collect())
            .unwrap_or_default();

        transactions
            .iter()
            .filter(|t| {
                t.account_id == Some(account_id)
                    || t.card_id.map(|cid| card_ids.contains(&cid)).unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Seed mock accounts for development.
    pub fn seed_mock_data(&mut self, org_id: Option<Uuid>) {
        if !self.accounts.is_empty() {
            return;
        }

        // Operating Account
        let mut operating = Account::new(
            org_id,
            "Operating Account".to_string(),
            AccountType::Bank,
            1_250_000.0,
            Currency::AUD,
        );
        operating.cards.push(Card::new(
            operating.id,
            "Business Debit".to_string(),
            "4821".to_string(),
            "Debit".to_string(),
        ));
        operating.cards.push(Card::new(
            operating.id,
            "Business Credit".to_string(),
            "9912".to_string(),
            "Credit".to_string(),
        ));
        self.add_account(operating);

        // Payroll Account
        let mut payroll = Account::new(
            org_id,
            "Payroll Account".to_string(),
            AccountType::Bank,
            85_000.0,
            Currency::AUD,
        );
        payroll.cards.push(Card::new(
            payroll.id,
            "Payroll Card".to_string(),
            "2244".to_string(),
            "Debit".to_string(),
        ));
        self.add_account(payroll);

        // Savings / Reserve
        let savings = Account::new(
            org_id,
            "Savings / Reserve".to_string(),
            AccountType::Savings,
            500_000.0,
            Currency::AUD,
        );
        self.add_account(savings);

        // External Accounts (crypto)
        let mut coinspot = Account::new(
            org_id,
            "Coinspot".to_string(),
            AccountType::Crypto,
            45_000.0,
            Currency::AUD,
        );
        coinspot.cards.push(Card::new(
            coinspot.id,
            "Coinspot AUD".to_string(),
            "SPOT".to_string(),
            "Crypto".to_string(),
        ));
        self.add_account(coinspot);

        let mut coinbase = Account::new(
            org_id,
            "Coinbase".to_string(),
            AccountType::Crypto,
            28_000.0,
            Currency::USD,
        );
        coinbase.cards.push(Card::new(
            coinbase.id,
            "Coinbase USD".to_string(),
            "BASE".to_string(),
            "Crypto".to_string(),
        ));
        self.add_account(coinbase);

        let mut cold_wallet = Account::new(
            org_id,
            "Cold Wallet".to_string(),
            AccountType::Crypto,
            3.5,
            Currency::Custom("BTC".to_string()),
        );
        cold_wallet.cards.push(Card::new(
            cold_wallet.id,
            "BTC Wallet".to_string(),
            "BTC".to_string(),
            "Crypto".to_string(),
        ));
        self.add_account(cold_wallet);
    }
}

pub fn use_account_store() -> RwSignal<AccountStore> {
    expect_context::<RwSignal<AccountStore>>()
}
