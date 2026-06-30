use crate::components::messenger::MessageDrawer;
use crate::components::search::SearchFilters;
use crate::components::tabs::TabList;
use crate::models::Action;
use crate::stores::{create_action, use_app_store, use_search_store, use_undo_redo_store};
use crate::types::{ActionType, TabType};
use leptos::prelude::*;

#[component]
pub fn Navbar() -> impl IntoView {
    let app_store = use_app_store();
    let undo_store = use_undo_redo_store();
    let search_store = use_search_store();

    // Derived signals
    let current_user_id = move || app_store.get().current_user.id;
    let can_undo = move || undo_store.get().can_undo_by_user(current_user_id());
    let can_redo = move || undo_store.get().can_redo_by_user(current_user_id());
    let is_search_open = move || app_store.get().is_search_open;
    let profile_name = move || app_store.get().current_user.name.clone();
    let message_count = move || app_store.get().unread_message_count();
    let is_message_drawer_open = move || app_store.get().message_drawer_open;

    // Dropdown state: (client_x, client_y, list of actions, is_redo)
    let (dropdown, set_dropdown) = signal::<Option<(i32, i32, Vec<Action>, bool)>>(None);

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
            store.expand_tab(TabType::Overview);
            store.close_search();
            store.close_tabs_drawer();
        });
    };

    let on_tabs_drawer = move |_| {
        app_store.update(|store| store.toggle_tabs_drawer());
    };

    let record_undo_redo = move |kind: ActionType, description: String| {
        let (uid, name, role, org) = user_info(&app_store);
        let action = create_action(
            kind,
            "Action",
            &description,
            uid,
            &name,
            &role,
            org,
            None,
        );
        undo_store.update(|u| u.record_action(action));
    };

    let on_redo = move |_| {
        if dropdown.get().is_some() { return; }
        let uid = current_user_id();
        if let Some(redone) = undo_store.get().redo_by_user(uid) {
            record_undo_redo(ActionType::Redo, format!("Redid: {}", redone.description));
            tracing::info!("Redo: {:?}", redone);
        }
    };

    let on_undo = move |_| {
        if dropdown.get().is_some() { return; }
        let uid = current_user_id();
        if let Some(undone) = undo_store.get().undo_by_user(uid) {
            record_undo_redo(ActionType::Undo, format!("Undid: {}", undone.description));
            tracing::info!("Undo: {:?}", undone);

            if let Some(ref from) = undone.navigated_from {
                tracing::info!("Navigating back to: {}", from);
            }
        }
    };

    let on_undo_context = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        let uid = current_user_id();
        let actions = undo_store.get().undoable_by_user(uid).into_iter().cloned().collect();
        set_dropdown.set(Some((ev.client_x(), ev.client_y(), actions, false)));
    };

    let on_redo_context = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        let uid = current_user_id();
        let actions = undo_store.get().redoable_by_user(uid).into_iter().cloned().collect();
        set_dropdown.set(Some((ev.client_x(), ev.client_y(), actions, true)));
    };

    let close_dropdown = move |_| {
        set_dropdown.set(None);
    };

    let on_dropdown_action = move |action_id: uuid::Uuid, is_redo: bool| {
        set_dropdown.set(None);
        if is_redo {
            if let Some(redone) = undo_store.get().redo_action_by_id(action_id) {
                record_undo_redo(ActionType::Redo, format!("Redid: {}", redone.description));
                tracing::info!("Redo by dropdown: {:?}", redone);
            }
        } else {
            if let Some(undone) = undo_store.get().undo_action_by_id(action_id) {
                record_undo_redo(ActionType::Undo, format!("Undid: {}", undone.description));
                tracing::info!("Undo by dropdown: {:?}", undone);
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
                    .active_tabs
                    .first()
                    .cloned()
                    .unwrap_or(TabType::Overview),
            );
        });
    };

    let on_message_click = move |_| {
        app_store.update(|store| store.toggle_message_drawer());
    };

    let is_tabs_drawer_open = move || app_store.get().tabs_drawer_open;

    view! {
        // Main Navbar - Fixed at top, single row
        <nav class="navbar">
            // ROW 1: buttons always visible
            <div class="navbar-row navbar-row-1">
                <div class="nav-row1-left">
                    <button class="nav-btn" on:click=on_tabs_drawer title="Tabs">"☰"</button>
                    <button class="nav-btn" on:click=on_home title="Home">"⌂"</button>
                    <button class="nav-btn" on:click=on_redo
                        on:contextmenu=on_redo_context
                        disabled={move || !can_redo()} title="Redo (hold for list)">"↻"</button>
                </div>
                <div class="nav-row1-centre">
                    <span class="nav-profile-name-top">{profile_name}</span>
                </div>
                <div class="nav-row1-right">
                    <button class="nav-btn" on:click=on_undo
                        on:contextmenu=on_undo_context
                        disabled={move || !can_undo()} title="Undo (hold for list)">"↺"</button>
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

        // Tabs drawer (left-side panel from ☰ button)
        {move || if is_tabs_drawer_open() {
            view! {
                <div class="tabs-drawer">
                    <div class="tabs-drawer-content">
                        <TabList />
                    </div>
                </div>
            }.into_any()
        } else { ().into_any() }}

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

        // Undo/Redo dropdown (tap-and-hold on the buttons)
        {move || if let Some((x, y, actions, is_redo)) = dropdown.get() {
            let title = if is_redo { "Redo" } else { "Undo" };
            view! {
                <div class="nav-dropdown-overlay" on:click=close_dropdown></div>
                <div class="nav-dropdown-menu" style={format!("left:{}px;top:{}px;", x, y)}>
                    <div class="nav-dropdown-title">{title}" actions"</div>
                    {if actions.is_empty() {
                        view! { <div class="nav-dropdown-empty">"No actions"</div> }.into_any()
                    } else {
                        actions.into_iter().map(|action| {
                            let action_id = action.id;
                            let desc = format!("{} {}", action.action_type_badge(), action.description);
                            let is_redo = is_redo;
                            view! {
                                <div class="nav-dropdown-item"
                                    on:click=move |_| on_dropdown_action(action_id, is_redo)>
                                    {desc}
                                </div>
                            }
                        }).collect::<Vec<_>>().into_any()
                    }}
                </div>
            }.into_any()
        } else { ().into_any() }}
    }
}
