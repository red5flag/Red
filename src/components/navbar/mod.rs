use crate::stores::{use_app_store, use_search_store, use_undo_redo_store};
use crate::types::TabType;
use leptos::prelude::*;

#[component]
pub fn Navbar() -> impl IntoView {
    let app_store = use_app_store();
    let undo_store = use_undo_redo_store();
    let search_store = use_search_store();

    // Derived signals
    let can_undo = move || undo_store.get().can_undo();
    let can_redo = move || undo_store.get().can_redo();
    let is_search_open = move || app_store.get().is_search_open;
    let current_location = move || app_store.get().get_current_location();
    let profile_name = move || app_store.get().current_user.name.clone();

    // Handlers
    let on_home = move |_| {
        app_store.update(|store| {
            store.collapse_tab();
            store.close_search();
        });
    };

    let on_redo = move |_| {
        if let Some(action) = undo_store.get().redo() {
            // Handle redo action
            tracing::info!("Redo: {:?}", action);
        }
    };

    let on_undo = move |_| {
        if let Some(action) = undo_store.get().undo() {
            // Handle undo action
            tracing::info!("Undo: {:?}", action);

            // If it was a navigation action, navigate back
            if let Some(ref from) = action.navigated_from {
                tracing::info!("Navigating back to: {}", from);
            }
        }
    };

    let on_search_click = move |_| {
        app_store.update(|store| {
            if store.is_search_open {
                store.close_search();
            } else {
                store.open_search();
            }
        });
        search_store.update(|store| {
            store.set_context_tab(
                app_store
                    .get()
                    .active_tab
                    .clone()
                    .unwrap_or(TabType::Overview),
            );
        });
    };

    let on_search_close = move |_| {
        app_store.update(|store| {
            store.close_search();
        });
    };

    view! {
        // Search Overlay
        <div
            class="search-overlay"
            class:active=is_search_open
        >
            <input
                type="text"
                class="search-input"
                placeholder="Search across all data..."
                prop:value={move || search_store.get().query}
                on:input=move |ev| {
                    let value = event_target_value(&ev);
                    search_store.update(|store| store.set_query(value));
                }
            />
            <button
                class="search-close-btn"
                on:click=on_search_close
            >
                "✕"
            </button>
        </div>

        // Main Navbar - Fixed at bottom
        <nav class="navbar">
            // Left section: Home button
            <div class="navbar-section">
                <button
                    class="nav-btn"
                    on:click=on_home
                    title="Home"
                >
                    "⌂"
                </button>
            </div>

            // Second from left: Redo button
            <div class="navbar-section">
                <button
                    class="nav-btn"
                    on:click=on_redo
                    disabled={move || !can_redo()}
                    title="Redo"
                >
                    "↻"
                </button>
            </div>

            // Middle: Location and Profile
            <div class="nav-location">
                <div class="nav-location-text">{current_location}</div>
                <div class="nav-profile-name">{profile_name}</div>
            </div>

            // Second from right: Undo button
            <div class="navbar-section">
                <button
                    class="nav-btn"
                    on:click=on_undo
                    disabled={move || !can_undo()}
                    title="Undo"
                >
                    "↺"
                </button>
            </div>

            // Right: Search button
            <div class="navbar-section">
                <button
                    class="nav-btn"
                    class:active=is_search_open
                    on:click=on_search_click
                    title="Search"
                >
                    "🔍"
                </button>
            </div>
        </nav>
    }
}
