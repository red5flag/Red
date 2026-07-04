use crate::models::Action;
use crate::types::{ActionType, TabType};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use std::collections::VecDeque;
use uuid::Uuid;

const MAX_HISTORY_SIZE: usize = 100;

// Helper to create an action with user info populated from AppStore
pub fn create_action(
    action_type: ActionType,
    entity_type: &str,
    description: &str,
    user_id: Uuid,
    user_name: &str,
    user_role: &str,
    org_id: Option<Uuid>,
    reason: Option<String>,
) -> Action {
    let mut action = Action::new(
        action_type,
        entity_type.to_string(),
        Uuid::new_v4(),
        description.to_string(),
        user_id,
    )
    .with_user(user_name.to_string(), user_role.to_string(), org_id);
    if let Some(r) = reason {
        action = action.with_reason(r);
    }
    action
}

// Undo/Redo store for complete action history
#[derive(Clone, Debug)]
pub struct UndoRedoStore {
    // History of actions - past actions that can be undone
    pub past: VecDeque<Action>,
    // Future actions that can be redone (after undo)
    pub future: VecDeque<Action>,
    // Current state reference
    pub current_action_id: Option<Uuid>,
}

impl Default for UndoRedoStore {
    fn default() -> Self {
        Self {
            past: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            future: VecDeque::new(),
            current_action_id: None,
        }
    }
}

impl UndoRedoStore {
    pub fn new() -> Self {
        Self::default()
    }

    // Record a new action, clearing the redo history
    pub fn record_action(&mut self, action: Action) {
        // Clear redo history when new action is performed
        self.future.clear();

        // Add to past history
        self.past.push_back(action);

        // Maintain max history size
        if self.past.len() > MAX_HISTORY_SIZE {
            self.past.pop_front();
        }
    }

    // Record a history entry (e.g. Undo/Redo meta-actions) without clearing the redo history
    pub fn record_history_action(&mut self, action: Action) {
        self.past.push_back(action);
        if self.past.len() > MAX_HISTORY_SIZE {
            self.past.pop_front();
        }
    }

    // Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    // Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    // Undo the last action
    pub fn undo(&mut self) -> Option<Action> {
        if let Some(action) = self.past.pop_back() {
            self.future.push_front(action.clone());
            Some(action)
        } else {
            None
        }
    }

    // Redo the next action
    pub fn redo(&mut self) -> Option<Action> {
        if let Some(action) = self.future.pop_front() {
            self.past.push_back(action.clone());
            Some(action)
        } else {
            None
        }
    }

    // Check if undo is available for a specific user
    pub fn can_undo_by_user(&self, user_id: Uuid) -> bool {
        self.past.iter().any(|a| a.user_id == user_id)
    }

    // Check if redo is available for a specific user
    pub fn can_redo_by_user(&self, user_id: Uuid) -> bool {
        self.future.iter().any(|a| a.user_id == user_id)
    }

    // Undo the most recent action performed by a specific user
    pub fn undo_by_user(&mut self, user_id: Uuid) -> Option<Action> {
        if let Some(pos) = self.past.iter().rposition(|a| a.user_id == user_id) {
            let action = self.past.remove(pos)?;
            self.future.push_front(action.clone());
            Some(action)
        } else {
            None
        }
    }

    // Redo the most recent undone action performed by a specific user
    pub fn redo_by_user(&mut self, user_id: Uuid) -> Option<Action> {
        if let Some(pos) = self.future.iter().position(|a| a.user_id == user_id) {
            let action = self.future.remove(pos)?;
            self.past.push_back(action.clone());
            Some(action)
        } else {
            None
        }
    }

    // Undo a specific action by its ID (used by dropdown / history selection)
    pub fn undo_action_by_id(&mut self, action_id: Uuid) -> Option<Action> {
        if let Some(pos) = self.past.iter().position(|a| a.id == action_id) {
            let action = self.past.remove(pos)?;
            self.future.push_front(action.clone());
            Some(action)
        } else {
            None
        }
    }

    // Redo a specific action by its ID (used by dropdown / history selection)
    pub fn redo_action_by_id(&mut self, action_id: Uuid) -> Option<Action> {
        if let Some(pos) = self.future.iter().position(|a| a.id == action_id) {
            let action = self.future.remove(pos)?;
            self.past.push_back(action.clone());
            Some(action)
        } else {
            None
        }
    }

    // Get actions that the given user can currently undo, newest first
    pub fn undoable_by_user(&self, user_id: Uuid) -> Vec<&Action> {
        self.past.iter().filter(|a| a.user_id == user_id).rev().collect()
    }

    // Get actions that the given user can currently redo, in redo order
    pub fn redoable_by_user(&self, user_id: Uuid) -> Vec<&Action> {
        self.future.iter().filter(|a| a.user_id == user_id).collect()
    }

    // Get the last action without removing it
    pub fn peek_last(&self) -> Option<&Action> {
        self.past.back()
    }

    // Get recent actions for history view
    pub fn get_recent_actions(&self, count: usize) -> Vec<Action> {
        self.past.iter().rev().take(count).cloned().collect()
    }

    // Clear all history
    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
        self.current_action_id = None;
    }

    // Get action by ID
    pub fn get_action(&self, id: Uuid) -> Option<&Action> {
        self.past.iter().find(|a| a.id == id)
    }

    // Filter actions by type
    pub fn filter_by_type(&self, action_type: ActionType) -> Vec<&Action> {
        self.past
            .iter()
            .filter(|a| a.action_type == action_type)
            .collect()
    }

    // Filter actions by entity
    pub fn filter_by_entity(&self, entity_type: &str, entity_id: Uuid) -> Vec<&Action> {
        self.past
            .iter()
            .filter(|a| a.entity_type == entity_type && a.entity_id == entity_id)
            .collect()
    }

    // Get count of actions
    pub fn len(&self) -> usize {
        self.past.len()
    }

    pub fn is_empty(&self) -> bool {
        self.past.is_empty()
    }

    /// Complex search across the full action history: who, what, where, when, why.
    /// Query text is matched against description, entity_type, user_name, user_role, tab_context, and reason.
    pub fn search_actions(&self, query: &HistoryQuery) -> Vec<&Action> {
        let q = query.text.trim().to_lowercase();
        self.past
            .iter()
            .filter(|a| {
                let mut keep = true;
                if !q.is_empty() {
                    keep = keep && (
                        a.description.to_lowercase().contains(&q) ||
                        a.entity_type.to_lowercase().contains(&q) ||
                        a.user_name.to_lowercase().contains(&q) ||
                        a.user_role.to_lowercase().contains(&q) ||
                        a.tab_context.as_ref().map(|s| s.to_lowercase().contains(&q)).unwrap_or(false) ||
                        a.reason.as_ref().map(|s| s.to_lowercase().contains(&q)).unwrap_or(false) ||
                        a.metadata.to_string().to_lowercase().contains(&q)
                    );
                }
                if let Some(ref types) = query.action_types {
                    keep = keep && types.contains(&a.action_type);
                }
                if let Some(ref entities) = query.entity_types {
                    keep = keep && entities.contains(&a.entity_type);
                }
                if let Some(ref users) = query.user_ids {
                    keep = keep && users.contains(&a.user_id);
                }
                if let Some((start, end)) = query.date_range {
                    keep = keep && a.timestamp >= start && a.timestamp <= end;
                }
                if let Some(ref tab) = query.tab_context {
                    keep = keep && a.tab_context.as_ref().map(|t| t.eq_ignore_ascii_case(tab)).unwrap_or(false);
                }
                if query.has_reason_only {
                    keep = keep && a.reason.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false);
                }
                keep
            })
            .collect()
    }

    // Unique users that appear in the history.
    pub fn history_users(&self) -> Vec<(Uuid, String, String)> {
        let mut seen = std::collections::HashSet::new();
        self.past
            .iter()
            .filter_map(|a| {
                if seen.insert(a.user_id) {
                    Some((a.user_id, a.user_name.clone(), a.user_role.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    // Unique entity types that appear in the history.
    pub fn history_entity_types(&self) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        self.past
            .iter()
            .filter_map(|a| if seen.insert(a.entity_type.clone()) { Some(a.entity_type.clone()) } else { None })
            .collect()
    }
}

/// Query object for complex history search/filtering.
#[derive(Clone, Debug, Default)]
pub struct HistoryQuery {
    pub text: String,
    pub action_types: Option<Vec<ActionType>>,
    pub entity_types: Option<Vec<String>>,
    pub user_ids: Option<Vec<Uuid>>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub tab_context: Option<String>,
    pub has_reason_only: bool,
}

// Create a signal-based store for Leptos
pub fn create_undo_redo_store() -> RwSignal<UndoRedoStore> {
    RwSignal::new(UndoRedoStore::new())
}

// Helper to format action for display
pub fn format_action_description(action: &Action) -> String {
    match action.action_type {
        ActionType::Create => format!("Created {}: {}", action.entity_type, action.description),
        ActionType::Update => format!("Updated {}: {}", action.entity_type, action.description),
        ActionType::Delete => format!("Deleted {}: {}", action.entity_type, action.description),
        ActionType::View => format!("Viewed {}: {}", action.entity_type, action.description),
        ActionType::Navigate => format!(
            "Navigated from {} to {}",
            action.navigated_from.as_deref().unwrap_or("unknown"),
            action.navigated_to.as_deref().unwrap_or("unknown")
        ),
        ActionType::Setting => format!("Changed setting: {}", action.description),
        ActionType::Payment => format!("Payment: {}", action.description),
        ActionType::Notification => format!("Notification: {}", action.description),
        ActionType::Search => format!("Search: {}", action.description),
        ActionType::Undo => format!("Undo: {}", action.description),
        ActionType::Redo => format!("Redo: {}", action.description),
        ActionType::Login => format!("Login: {}", action.description),
        ActionType::Logout => format!("Logout: {}", action.description),
    }
}

// Undo handler trait for different action types
pub trait UndoHandler: Send + Sync {
    fn can_undo(&self, action: &Action) -> bool;
    fn undo(&self, action: &Action) -> Result<(), String>;
    fn redo(&self, action: &Action) -> Result<(), String>;
}

// Registry of undo handlers
pub struct UndoHandlerRegistry {
    handlers: Vec<Box<dyn UndoHandler>>,
}

impl Default for UndoHandlerRegistry {
    fn default() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
}

impl UndoHandlerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, handler: Box<dyn UndoHandler>) {
        self.handlers.push(handler);
    }

    pub fn find_handler(&self, action: &Action) -> Option<&dyn UndoHandler> {
        self.handlers
            .iter()
            .find(|h| h.can_undo(action))
            .map(|h| h.as_ref())
    }
}

/// Apply side effects when undoing an action (e.g. navigate back to previous tab).
/// Returns true if side effects were applied.
pub fn apply_undo_side_effects(action: &Action, app_store: &mut crate::stores::AppStore) -> bool {
    if action.entity_type == "Tab" {
        if let Some(ref from) = action.navigated_from {
            if !from.is_empty() {
                if let Some(tab) = TabType::from_str(from) {
                    app_store.expand_tab(tab);
                    return true;
                }
            }
        }
    }
    false
}

/// Apply side effects when redoing an action (e.g. navigate forward to the target tab).
/// Returns true if side effects were applied.
pub fn apply_redo_side_effects(action: &Action, app_store: &mut crate::stores::AppStore) -> bool {
    if action.entity_type == "Tab" {
        if let Some(ref to) = action.navigated_to {
            if let Some(tab) = TabType::from_str(to) {
                app_store.expand_tab(tab);
                return true;
            }
        }
    }
    false
}
