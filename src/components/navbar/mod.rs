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
    let on_tabs_drawer = move |_| {
        app_store.update(|store| {
            store.toggle_tabs_drawer();
            if store.tabs_drawer_open {
                store.close_notifications_drawer();
            }
        });
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

    let on_notifications_click = move |_| {
        app_store.update(|store| {
            store.toggle_notifications_drawer();
            if store.notifications_drawer_open {
                store.close_tabs_drawer();
            }
        });
    };

    let is_tabs_drawer_open = move || app_store.get().tabs_drawer_open;
    let is_notifications_drawer_open = move || app_store.get().notifications_drawer_open;
    let notification_count = move || app_store.get().notifications.iter().filter(|n| n.target_tab.is_some()).count();

    view! {
        // Main Navbar - Fixed at top, single row
        <nav class="navbar">
            // ROW 1: buttons always visible
            <div class="navbar-row navbar-row-1">
                <div class="nav-row1-left">
                    <button class="nav-btn" on:click=on_tabs_drawer title="Tabs">"☰"</button>
                    <button class="nav-btn nav-search-btn" on:click=on_search_click title="Search">"🔍"</button>
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
                    <div class="nav-message-wrap" on:click=on_message_click title="Open messages">
                        <div class="nav-message-icon">"💬"</div>
                        {move || {
                            let count = message_count();
                            if count > 0 {
                                view! { <div class="nav-message-badge">{count}</div> }.into_any()
                            } else { ().into_any() }
                        }}
                    </div>
                    <div class="nav-message-wrap" on:click=on_notifications_click title="Notifications">
                        <div class="nav-message-icon">"🔔"</div>
                        {move || {
                            let count = notification_count();
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

        // Notifications drawer (right-side panel)
        {move || if is_notifications_drawer_open() {
            let on_close_notif = move |_| app_store.update(|s| s.close_notifications_drawer());
            view! {
                <div class="notif-drawer-overlay" on:click=on_close_notif>
                    <div class="notif-drawer" on:click=|ev| ev.stop_propagation()>
                        <div class="notif-drawer-header">
                            <span class="notif-drawer-title">"Notifications"</span>
                            <button class="notif-drawer-close" on:click=on_close_notif>"✕"</button>
                        </div>
                        <div class="notif-drawer-body">
                            {move || {
                                let notifs = app_store.get().notifications.clone()
                                    .into_iter()
                                    .filter(|n| n.target_tab.is_some())
                                    .collect::<Vec<_>>();
                                if notifs.is_empty() {
                                    view! {
                                        <div class="notif-empty">
                                            <div class="notif-empty-icon">"🔕"</div>
                                            <div class="notif-empty-text">"No notifications"</div>
                                        </div>
                                    }.into_any()
                                } else {
                                    notifs.into_iter().rev().map(|n| {
                                        let nid = n.id;
                                        let icon = match n.notification_type {
                                            crate::stores::NotificationType::Success => "✅",
                                            crate::stores::NotificationType::Error => "❌",
                                            crate::stores::NotificationType::Warning => "⚠️",
                                            crate::stores::NotificationType::Info => "ℹ️",
                                        };
                                        let from = n.from_user.unwrap_or_else(|| "System".to_string());
                                        let time = format!("{}", n.timestamp.format("%b %d, %H:%M"));
                                        let msg = n.message.clone();
                                        let target = n.target_tab.clone();
                                        let target_for_map = target.clone();
                                        let has_target = target.is_some();
                                        let linked_doc = n.linked_doc_id;
                                        let content_preview = n.content_preview.clone();
                                        let has_linked_origin = n.linked_portfolio_id.is_some() || n.linked_doc_id.is_some();
                                        let on_notif_click = {
                                            let app_store = app_store;
                                            move |_| {
                                                if has_linked_origin {
                                                    app_store.update(|s| s.navigate_to_notification(nid));
                                                } else if let Some(tab) = target.clone() {
                                                    app_store.update(|s| {
                                                        s.expand_tab(tab);
                                                        s.close_notifications_drawer();
                                                    });
                                                }
                                            }
                                        };
                                        let on_go_to_content = {
                                            let app_store = app_store;
                                            move |ev: leptos::ev::MouseEvent| {
                                                ev.stop_propagation();
                                                if has_linked_origin {
                                                    app_store.update(|s| s.navigate_to_notification(nid));
                                                }
                                            }
                                        };
                                        view! {
                                            <div class="notif-item"
                                                class:clickable={has_target}
                                                on:click=on_notif_click>
                                                <div class="notif-item-icon">{icon}</div>
                                                <div class="notif-item-content">
                                                    <div class="notif-item-msg">{msg}</div>
                                                    <div class="notif-item-meta">
                                                        <span class="notif-item-from">{from}</span>
                                                        <span class="notif-item-time">{time}</span>
                                                    </div>
                                                    {target_for_map.map(|t| view! {
                                                        <div class="notif-item-tab">
                                                            {format!("→ {} (click to open)", t.as_str())}
                                                        </div>
                                                    })}
                                                    {linked_doc.map(|_| view! {
                                                        <div class="notif-item-doc-badge">"📄 Linked document"</div>
                                                    })}
                                                    {content_preview.map(|preview| view! {
                                                        <div class="notif-item-preview">
                                                            <div class="notif-item-preview-label">"Notes:"</div>
                                                            <pre class="notif-item-preview-text">{preview}</pre>
                                                        </div>
                                                    })}
                                                    {if has_linked_origin {
                                                        view! {
                                                            <button class="notif-item-go-btn" on:click=on_go_to_content>
                                                                "→ Go to content"
                                                            </button>
                                                        }.into_any()
                                                    } else { ().into_any() }}
                                                </div>
                                                <button class="notif-item-dismiss"
                                                    on:click=move |ev: leptos::ev::MouseEvent| {
                                                        ev.stop_propagation();
                                                        app_store.update(|s| s.remove_notification(nid));
                                                    }>"✕"</button>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>().into_any()
                                }
                            }}
                        </div>
                    </div>
                </div>
            }.into_any()
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
