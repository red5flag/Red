pub mod app_store;
pub mod search_store;
pub mod undo_redo;

pub use app_store::*;
pub use search_store::*;
pub use undo_redo::*;

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
