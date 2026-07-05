pub mod app_store;
pub mod calendar_store;
pub mod credentials;
pub mod messenger_store;
pub mod notifications;
pub mod organization_store;
pub mod rule_store;
pub mod search_store;
pub mod seed_data;
pub mod transaction_store;
pub mod ui_store;
pub mod undo_redo;

pub use app_store::*;
pub use calendar_store::*;
pub use credentials::*;
pub use messenger_store::*;
pub use notifications::*;
pub use organization_store::*;
pub use rule_store::*;
pub use search_store::*;
pub use transaction_store::*;
pub use ui_store::*;
pub use undo_redo::{
    apply_redo_side_effects, apply_undo_side_effects, create_action, create_undo_redo_store,
    format_action_description, HistoryQuery, UndoRedoStore,
};

use leptos::prelude::*;

// Helper functions to consume stores from context
pub fn use_app_store() -> RwSignal<AppStore> {
    expect_context::<RwSignal<AppStore>>()
}

pub fn use_notification_store() -> RwSignal<NotificationStore> {
    expect_context::<RwSignal<NotificationStore>>()
}

pub fn use_search_store() -> RwSignal<SearchStore> {
    expect_context::<RwSignal<SearchStore>>()
}

pub fn use_ui_store() -> RwSignal<UiStore> {
    expect_context::<RwSignal<UiStore>>()
}

pub fn use_messenger_store() -> RwSignal<MessengerStore> {
    expect_context::<RwSignal<MessengerStore>>()
}

pub fn use_calendar_store() -> RwSignal<CalendarStore> {
    expect_context::<RwSignal<CalendarStore>>()
}

pub fn use_rule_store() -> RwSignal<RuleStore> {
    expect_context::<RwSignal<RuleStore>>()
}

pub fn use_transaction_store() -> RwSignal<TransactionStore> {
    expect_context::<RwSignal<TransactionStore>>()
}

pub fn use_organization_store() -> RwSignal<OrganizationStore> {
    expect_context::<RwSignal<OrganizationStore>>()
}

pub fn use_undo_redo_store() -> RwSignal<UndoRedoStore> {
    expect_context::<RwSignal<UndoRedoStore>>()
}
