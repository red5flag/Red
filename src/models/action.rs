use crate::types::{ActionType, ChangeSeverity, ViewportContext};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Action record for complete undo/redo system
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    pub id: Uuid,
    pub action_type: ActionType,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub description: String,
    pub user_id: Uuid,
    pub user_name: String,
    pub user_role: String,
    pub organization_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub previous_state: Option<serde_json::Value>,
    pub new_state: Option<serde_json::Value>,
    pub tab_context: Option<String>,
    pub navigated_from: Option<String>,
    pub navigated_to: Option<String>,
    pub metadata: serde_json::Value,
    pub reason: Option<String>,
    #[serde(default)]
    pub change_severity: ChangeSeverity,
    #[serde(default)]
    pub viewport_context: Option<ViewportContext>,
    #[serde(default)]
    pub notarised: bool,
    #[serde(default)]
    pub notarisation_hash: Option<String>,
}

impl Action {
    pub fn new(
        action_type: ActionType,
        entity_type: String,
        entity_id: Uuid,
        description: String,
        user_id: Uuid,
    ) -> Self {
        let severity = ChangeSeverity::from_action_type(&action_type);
        Self {
            id: Uuid::new_v4(),
            action_type,
            entity_type,
            entity_id,
            description,
            user_id,
            user_name: String::new(),
            user_role: String::new(),
            organization_id: None,
            timestamp: Utc::now(),
            previous_state: None,
            new_state: None,
            tab_context: None,
            navigated_from: None,
            navigated_to: None,
            metadata: serde_json::json!({}),
            reason: None,
            change_severity: severity,
            viewport_context: None,
            notarised: false,
            notarisation_hash: None,
        }
    }

    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    pub fn with_user(mut self, name: String, role: String, org_id: Option<Uuid>) -> Self {
        self.user_name = name;
        self.user_role = role;
        self.organization_id = org_id;
        self
    }

    pub fn with_state(mut self, previous: serde_json::Value, new: serde_json::Value) -> Self {
        self.previous_state = Some(previous);
        self.new_state = Some(new);
        self
    }

    pub fn with_navigation(mut self, from: String, to: String) -> Self {
        self.navigated_from = Some(from);
        self.navigated_to = Some(to);
        self.action_type = ActionType::Navigate;
        self.change_severity = ChangeSeverity::from_action_type(&self.action_type);
        self
    }

    pub fn with_tab_context(mut self, tab: String) -> Self {
        self.tab_context = Some(tab);
        self
    }

    pub fn with_viewport_context(mut self, viewport_context: ViewportContext) -> Self {
        self.viewport_context = Some(viewport_context);
        self
    }

    /// Returns true if this action can be undone by a user.
    /// Auth and undo/redo meta-actions are never undoable.
    pub fn is_undoable(&self) -> bool {
        !matches!(
            self.action_type,
            ActionType::Login | ActionType::Logout | ActionType::Undo | ActionType::Redo
        )
    }

    /// Mark this action as notarised.
    /// The actual cryptographic hash is deferred until a SHA-256 crate is added.
    pub fn notarise(&mut self, _previous_hash: Option<&str>) {
        self.notarised = true;
        // Hash computation deferred: no SHA-256 crate is currently in the dependency tree.
        self.notarisation_hash = None;
    }

    pub fn action_type_badge(&self) -> &'static str {
        match self.action_type {
            ActionType::Create => "Create",
            ActionType::Update => "Update",
            ActionType::Delete => "Delete",
            ActionType::View => "View",
            ActionType::Navigate => "Navigate",
            ActionType::Setting => "Setting",
            ActionType::Payment => "Payment",
            ActionType::Notification => "Notification",
            ActionType::Search => "Search",
            ActionType::Undo => "Undo",
            ActionType::Redo => "Redo",
            ActionType::Login => "Login",
            ActionType::Logout => "Logout",
        }
    }
}

// Action history entry with user info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionHistoryEntry {
    pub action: Action,
    pub user_name: String,
    pub user_role: String,
    pub organization_id: Option<Uuid>,
}

// Filter for history queries
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActionFilter {
    pub action_types: Vec<ActionType>,
    pub entity_types: Vec<String>,
    pub user_ids: Vec<Uuid>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub tab_context: Option<String>,
}
