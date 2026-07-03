pub mod app_store;
pub mod credentials;
pub mod search_store;
pub mod undo_redo;

pub use app_store::*;
pub use credentials::*;
pub use search_store::*;
pub use undo_redo::{HistoryQuery, UndoRedoStore, apply_redo_side_effects, apply_undo_side_effects, create_action, create_undo_redo_store, format_action_description};

use leptos::prelude::*;

// Helper functions to consume stores from context
pub fn use_app_store() -> RwSignal<AppStore> {
    expect_context::<RwSignal<AppStore>>()
}

pub fn use_search_store() -> RwSignal<SearchStore> {
    expect_context::<RwSignal<SearchStore>>()
}

pub fn use_undo_redo_store() -> RwSignal<UndoRedoStore> {
    expect_context::<RwSignal<UndoRedoStore>>()
}
