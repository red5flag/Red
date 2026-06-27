use crate::components::messenger::MessageDrawer;
use crate::components::search::SearchFilters;
use crate::stores::{create_action, use_app_store, use_search_store, use_undo_redo_store};
use crate::types::{ActionType, TabType};
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
    let profile_name = move || app_store.get().current_user.name.clone();
    let message_count = move || app_store.get().unread_message_count();
    let is_message_drawer_open = move || app_store.get().message_drawer_open;

    // Helper to get user info tuple
    fn user_info(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>) -> (uuid::Uuid, String, String, Option<uuid::Uuid>) {
        let store = app_store.get();
        (store.current_user.id, store.current_user.name.clone(), format!("{:?}", store.current_user.role), store.current_user.organization_id)
    }

    // Handlers
    let on_home = move |_| {
        let (uid, name, role, org) = user_info(&app_store);
        let action = create_action(
            ActionType::Navigate,
            "App",
            "Returned home",
            uid,
            &name,
            &role,
            org,
            None,
        );
        undo_store.update(|u| u.record_action(action));
        app_store.update(|store| {
            store.collapse_all_tabs();
            store.close_search();
        });
    };

    let on_redo = move |_| {
        if let Some(redone) = undo_store.get().redo() {
            let (uid, name, role, org) = user_info(&app_store);
            let action = create_action(
                ActionType::Redo,
                "Action",
                &format!("Redid: {}", redone.description),
                uid,
                &name,
                &role,
                org,
                None,
            );
            undo_store.update(|u| u.record_action(action));
            tracing::info!("Redo: {:?}", redone);
        }
    };

    let on_undo = move |_| {
        if let Some(undone) = undo_store.get().undo() {
            let (uid, name, role, org) = user_info(&app_store);
            let action = create_action(
                ActionType::Undo,
                "Action",
                &format!("Undid: {}", undone.description),
                uid,
                &name,
                &role,
                org,
                None,
            );
            undo_store.update(|u| u.record_action(action));
            tracing::info!("Undo: {:?}", undone);

            if let Some(ref from) = undone.navigated_from {
                tracing::info!("Navigating back to: {}", from);
            }
        }
    };

    let on_search_click = move |_| {
        let (uid, name, role, org) = user_info(&app_store);
        app_store.update(|store| {
            if store.is_search_open {
                store.close_search();
                let action = create_action(
                    ActionType::Search,
                    "Search",
                    "Closed search",
                    uid,
                    &name,
                    &role,
                    org,
                    None,
                );
                undo_store.update(|u| u.record_action(action));
            } else {
                store.open_search();
                let action = create_action(
                    ActionType::Search,
                    "Search",
                    "Opened search",
                    uid,
                    &name,
                    &role,
                    org,
                    None,
                );
                undo_store.update(|u| u.record_action(action));
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

    let on_message_click = move |_| {
        app_store.update(|store| store.toggle_message_drawer());
    };

    let on_drawer_toggle = move |_| {
        app_store.update(|store| store.toggle_drawer());
    };

    let on_overview_pen = move |_| {
        app_store.update(|store| { store.toggle_tab_edit_mode(&TabType::Overview); });
    };

    let overview_edit_active = move || app_store.get().is_tab_edit_mode(&TabType::Overview);
    let drawer_open = move || app_store.get().drawer_open;

    view! {
        // Main Navbar - Fixed at top, single row
        <nav class="navbar">
            // ROW 1: buttons always visible
            <div class="navbar-row navbar-row-1">
                <div class="nav-row1-left">
                    <button class="nav-btn" on:click=on_drawer_toggle title="Toggle menu" class:nav-btn-active=drawer_open>"☰"</button>
                    <button class="nav-btn" on:click=on_home title="Home">"⌂"</button>
                    <button class="nav-btn" on:click=on_redo
                        disabled={move || !can_redo()} title="Redo">"↻"</button>
                </div>
                <div class="nav-row1-centre">
                    <span class="nav-profile-name-top">{profile_name}</span>
                </div>
                <div class="nav-row1-right">
                    <button class="nav-btn" on:click=on_undo
                        disabled={move || !can_undo()} title="Undo">"↺"</button>
                    <button class="nav-btn" on:click=on_overview_pen
                        class:nav-btn-active=overview_edit_active title="Edit Overview">"✎"</button>
                    <button class="nav-btn nav-search-btn" on:click=on_search_click title="Search">"🔍"</button>
                    <div class="nav-message-wrap" on:click=on_message_click title="Open messages">
                        <div class="nav-message-icon">"💬"</div>
                        {move || {
                            let count = message_count();
                            if count > 0 {
                                view! { <div class="nav-message-badge">{count}</div> }.into_any()
                            } else { ().into_any() }
                        }}
                    </div>
                </div>
            </div>
        </nav>

        // Message drawer overlay
        {move || if is_message_drawer_open() {
            view! { <MessageDrawer /> }.into_any()
        } else { ().into_any() }}

        // Search panel - drops below navbar when open
        {move || if is_search_open() {
            view! {
                <div class="search-drop-panel">
                    <SearchFilters />
                </div>
            }.into_any()
        } else { ().into_any() }}
    }
}
