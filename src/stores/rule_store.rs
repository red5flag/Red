use crate::models::{Rule, RuleHistoryEntry};
use leptos::prelude::*;
use uuid::Uuid;

/// Dedicated store for rule engine state: rules and their audit history.
/// Extracted from AppStore so rule lifecycle changes do not invalidate
/// consumers of unrelated domain state.
#[derive(Clone, Debug)]
pub struct RuleStore {
    pub rules: Vec<Rule>,
    pub rule_history: Vec<RuleHistoryEntry>,
}

impl Default for RuleStore {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            rule_history: Vec::new(),
        }
    }
}

impl RuleStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule(&mut self, rule: Rule, current_user_name: String) {
        let entry = RuleHistoryEntry::new(
            rule.id,
            rule.name.clone(),
            "Created".to_string(),
            rule.created_by,
            current_user_name,
            format!("Rule '{}' was created", rule.name),
        );
        self.rule_history.push(entry);
        self.rules.push(rule);
    }

    pub fn update_rule(&mut self, rule: Rule, updated_by: Uuid, current_user_name: String) {
        if let Some(existing) = self.rules.iter_mut().find(|r| r.id == rule.id) {
            let entry = RuleHistoryEntry::new(
                rule.id,
                rule.name.clone(),
                "Updated".to_string(),
                updated_by,
                current_user_name,
                format!("Rule '{}' was updated", rule.name),
            );
            self.rule_history.push(entry);
            *existing = rule;
            existing.updated_by = updated_by;
            existing.updated_at = chrono::Utc::now();
        }
    }

    pub fn delete_rule(&mut self, rule_id: Uuid, deleted_by: Uuid, current_user_name: String) {
        if let Some(rule) = self.rules.iter().find(|r| r.id == rule_id) {
            let entry = RuleHistoryEntry::new(
                rule_id,
                rule.name.clone(),
                "Deleted".to_string(),
                deleted_by,
                current_user_name,
                format!("Rule '{}' was deleted", rule.name),
            );
            self.rule_history.push(entry);
        }
        self.rules.retain(|r| r.id != rule_id);
    }

    pub fn toggle_rule(&mut self, rule_id: Uuid, toggled_by: Uuid, current_user_name: String) {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = !rule.enabled;
            let action = if rule.enabled { "Enabled" } else { "Disabled" };
            let entry = RuleHistoryEntry::new(
                rule_id,
                rule.name.clone(),
                action.to_string(),
                toggled_by,
                current_user_name,
                format!("Rule '{}' was {}", rule.name, action.to_lowercase()),
            );
            self.rule_history.push(entry);
            rule.updated_by = toggled_by;
            rule.updated_at = chrono::Utc::now();
        }
    }

    pub fn duplicate_rule(
        &mut self,
        rule_id: Uuid,
        duplicated_by: Uuid,
        current_user_name: String,
    ) -> Option<Rule> {
        let rule = self.rules.iter().find(|r| r.id == rule_id)?.clone();
        let mut new_rule = rule.clone();
        new_rule.id = Uuid::new_v4();
        new_rule.name = format!("{} (Copy)", rule.name);
        new_rule.enabled = false;
        new_rule.created_by = duplicated_by;
        new_rule.created_at = chrono::Utc::now();
        new_rule.updated_by = duplicated_by;
        new_rule.updated_at = chrono::Utc::now();
        let entry = RuleHistoryEntry::new(
            new_rule.id,
            new_rule.name.clone(),
            "Duplicated".to_string(),
            duplicated_by,
            current_user_name,
            format!(
                "Rule '{}' was duplicated from '{}'",
                new_rule.name, rule.name
            ),
        );
        self.rule_history.push(entry);
        self.rules.push(new_rule.clone());
        Some(new_rule)
    }

    pub fn rules_for_org(&self, org_id: Uuid) -> Vec<&Rule> {
        self.rules
            .iter()
            .filter(|r| r.organization_id == org_id)
            .collect()
    }

    pub fn rule_history_for_rule(&self, rule_id: Uuid) -> Vec<&RuleHistoryEntry> {
        self.rule_history
            .iter()
            .filter(|h| h.rule_id == rule_id)
            .collect()
    }
}

pub fn create_rule_store() -> RwSignal<RuleStore> {
    RwSignal::new(RuleStore::new())
}

pub fn provide_rule_store() -> RwSignal<RuleStore> {
    let store = create_rule_store();
    provide_context(store);
    store
}

pub fn use_rule_store() -> RwSignal<RuleStore> {
    expect_context::<RwSignal<RuleStore>>()
}
