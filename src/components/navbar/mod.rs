use crate::components::messenger::MessageDrawer;
use crate::components::search::SearchFilters;
use crate::components::tabs::TabList;
use crate::models::Action;
use crate::stores::{
    apply_redo_side_effects, apply_undo_side_effects, create_action, use_app_store,
    use_messenger_store, use_notification_store, use_ui_store, use_undo_redo_store,
    Notification, NotificationDrawerFilter, NotificationDrawerSort, NotificationType,
};
use crate::types::{ActionType, TabType};
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn Navbar() -> impl IntoView {
    let app_store = use_app_store();
    let messenger_store = use_messenger_store();
    let notification_store = use_notification_store();
    let ui_store = use_ui_store();
    let undo_store = use_undo_redo_store();

    // Derived signals
    let current_user_id = move || app_store.get().current_user.id;
    let can_undo = move || undo_store.get().can_undo_by_user(current_user_id());
    let can_redo = move || undo_store.get().can_redo_by_user(current_user_id());
    let is_search_open = move || ui_store.get().is_search_open;
    let message_count = move || {
        let current_user_id = app_store.get().current_user.id;
        messenger_store.get().unread_message_count(current_user_id)
    };
    let is_message_drawer_open = move || messenger_store.get().message_drawer_open;

    // Dropdown state: (client_x, client_y, list of actions, is_redo)
    let (dropdown, set_dropdown) = signal::<Option<(i32, i32, Vec<Action>, bool)>>(None);

    // Document reader view toggle inside the notifications drawer
    let (show_doc_reader, set_show_doc_reader) = signal(false);
    // Sort dropdown inside the notifications drawer
    let (show_sort_menu, set_show_sort_menu) = signal(false);

    // Click feedback state for navbar buttons (1-second flash)
    let (clicked, set_clicked) = signal(std::collections::HashSet::<usize>::new());
    let click_feedback = {
        let set_clicked = set_clicked.clone();
        move |idx: usize| {
            set_clicked.update(|s| {
                s.insert(idx);
            });
            let set_clicked = set_clicked.clone();
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(1000).await;
                set_clicked.update(|s| {
                    s.remove(&idx);
                });
            });
        }
    };

    // Helper to get user info tuple
    fn user_info(
        app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>,
    ) -> (uuid::Uuid, String, String, Option<uuid::Uuid>) {
        let store = app_store.get();
        (
            store.current_user.id,
            store.current_user.name.clone(),
            format!("{:?}", store.current_user.role),
            store.current_user.organization_id,
        )
    }

    // Handlers
    let on_tabs_drawer = move |_| {
        ui_store.update(|ui| {
            ui.toggle_tabs_drawer();
            if ui.tabs_drawer_open {
                messenger_store.update(|store| store.set_message_drawer(false));
                ui.close_search();
            }
        });
        notification_store.update(|store| store.close_drawer());
        click_feedback(0);
    };

    let record_undo_redo = move |kind: ActionType, description: String| {
        let (uid, name, role, org) = user_info(&app_store);
        let action = create_action(kind, "Action", &description, uid, &name, &role, org, None);
        undo_store.update(|u| u.record_history_action(action));
    };

    let on_redo = move |_| {
        if dropdown.get().is_some() {
            return;
        }
        let uid = current_user_id();
        if let Some(redone) = undo_store.get().redo_by_user(uid) {
            record_undo_redo(ActionType::Redo, format!("Redid: {}", redone.description));
            tracing::info!("Redo: {:?}", redone);
            app_store.update(|store| {
                apply_redo_side_effects(&redone, store);
            });
        }
    };

    let on_undo = move |_| {
        if dropdown.get().is_some() {
            return;
        }
        let uid = current_user_id();
        if let Some(undone) = undo_store.get().undo_by_user(uid) {
            record_undo_redo(ActionType::Undo, format!("Undid: {}", undone.description));
            tracing::info!("Undo: {:?}", undone);
            app_store.update(|store| {
                apply_undo_side_effects(&undone, store);
            });
        }
        click_feedback(2);
    };

    let on_undo_context = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        let uid = current_user_id();
        let actions = undo_store
            .get()
            .undoable_by_user(uid)
            .into_iter()
            .cloned()
            .collect();
        set_dropdown.set(Some((ev.client_x(), ev.client_y(), actions, false)));
    };

    let on_redo_context = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        let uid = current_user_id();
        let actions = undo_store
            .get()
            .redoable_by_user(uid)
            .into_iter()
            .cloned()
            .collect();
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
                app_store.update(|store| {
                    apply_redo_side_effects(&redone, store);
                });
            }
        } else {
            if let Some(undone) = undo_store.get().undo_action_by_id(action_id) {
                record_undo_redo(ActionType::Undo, format!("Undid: {}", undone.description));
                tracing::info!("Undo by dropdown: {:?}", undone);
                app_store.update(|store| {
                    apply_undo_side_effects(&undone, store);
                });
            }
        }
    };

    let on_search_click = move |_| {
        ui_store.update(|s| {
            s.is_search_open = !s.is_search_open;
            if s.is_search_open {
                s.close_tabs_drawer();
                messenger_store.update(|store| store.set_message_drawer(false));
            }
        });
        notification_store.update(|s| s.close_drawer());
        click_feedback(3);
    };

    let on_message_click = move |_| {
        messenger_store.update(|store| {
            store.toggle_message_drawer();
            if store.message_drawer_open {
                ui_store.update(|ui| {
                    ui.close_tabs_drawer();
                    ui.close_search();
                });
            }
        });
        notification_store.update(|s| s.close_drawer());
        click_feedback(5);
    };

    let on_notifications_click = move |_| {
        notification_store.update(|store| {
            store.clear_drawer_scope();
            store.toggle_drawer();
            if store.drawer_open {
                messenger_store.update(|s| s.set_message_drawer(false));
                ui_store.update(|ui| {
                    ui.close_tabs_drawer();
                    ui.close_search();
                });
            }
        });
        click_feedback(6);
    };

    let is_tabs_drawer_open = move || ui_store.get().tabs_drawer_open;
    let is_notifications_drawer_open = move || notification_store.get().drawer_open;
    let notification_count = move || {
        let store = notification_store.get();
        let filter = &store.drawer_filter;
        store
            .notifications
            .iter()
            .filter(|n| matches_filter(n, filter, &store.drawer_scoped_portfolio, &store.drawer_scoped_group))
            .count()
    };

    /// Display label for the active sort mode.
    fn sort_label(sort: &NotificationDrawerSort) -> &'static str {
        match sort {
            NotificationDrawerSort::Newest => "Newest",
            NotificationDrawerSort::Oldest => "Oldest",
            NotificationDrawerSort::Type => "By Type",
        }
    }

    /// Display label for the active filter.
    fn filter_label(filter: &NotificationDrawerFilter) -> String {
        match filter {
            NotificationDrawerFilter::All => "All".to_string(),
            NotificationDrawerFilter::Portfolios => "Portfolios".to_string(),
            NotificationDrawerFilter::Networking => "Networking".to_string(),
            NotificationDrawerFilter::Files => "Files".to_string(),
            NotificationDrawerFilter::Type(t) => format!("{}", t),
        }
    }

    /// Check whether a notification matches the active drawer filter.
    fn matches_filter(
        n: &Notification,
        filter: &NotificationDrawerFilter,
        scoped_portfolio: &Option<Uuid>,
        scoped_group: &Option<(Uuid, Uuid)>,
    ) -> bool {
        let category_matches = match filter {
            NotificationDrawerFilter::All => true,
            NotificationDrawerFilter::Portfolios => n.target_tab.as_ref() == Some(&TabType::Portfolios),
            NotificationDrawerFilter::Networking => n.target_tab.as_ref() == Some(&TabType::Networking),
            NotificationDrawerFilter::Files => {
                n.linked_doc_id.is_some() || n.linked_asset_id.is_some() || n.linked_group_id.is_some()
            }
            NotificationDrawerFilter::Type(t) => &n.notification_type == t,
        };
        if !category_matches {
            return false;
        }
        if let Some(pid) = scoped_portfolio {
            if n.linked_portfolio_id.as_ref() != Some(pid) {
                return false;
            }
        }
        if let Some((_, gid)) = scoped_group {
            if n.linked_group_id.as_ref() != Some(gid) {
                return false;
            }
        }
        true
    }

    /// Apply the active sort to a list of notifications.
    fn apply_sort(notifs: &mut [Notification], sort: &NotificationDrawerSort) {
        match sort {
            NotificationDrawerSort::Newest => notifs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)),
            NotificationDrawerSort::Oldest => notifs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)),
            NotificationDrawerSort::Type => notifs.sort_by(|a, b| {
                let type_order = type_rank(&a.notification_type).cmp(&type_rank(&b.notification_type));
                if type_order != std::cmp::Ordering::Equal {
                    type_order
                } else {
                    b.timestamp.cmp(&a.timestamp)
                }
            }),
        }
    }

    fn type_rank(t: &NotificationType) -> u8 {
        match t {
            NotificationType::Error => 0,
            NotificationType::Warning => 1,
            NotificationType::Info => 2,
            NotificationType::Success => 3,
        }
    }

    /// Advance the filter to the next quick-swap option.
    fn next_filter(filter: &NotificationDrawerFilter) -> NotificationDrawerFilter {
        match filter {
            NotificationDrawerFilter::All => NotificationDrawerFilter::Portfolios,
            NotificationDrawerFilter::Portfolios => NotificationDrawerFilter::Networking,
            NotificationDrawerFilter::Networking => NotificationDrawerFilter::Files,
            NotificationDrawerFilter::Files => NotificationDrawerFilter::Type(NotificationType::Success),
            NotificationDrawerFilter::Type(NotificationType::Success) => {
                NotificationDrawerFilter::Type(NotificationType::Warning)
            }
            NotificationDrawerFilter::Type(NotificationType::Warning) => {
                NotificationDrawerFilter::Type(NotificationType::Error)
            }
            NotificationDrawerFilter::Type(NotificationType::Error) => {
                NotificationDrawerFilter::Type(NotificationType::Info)
            }
            NotificationDrawerFilter::Type(NotificationType::Info) => NotificationDrawerFilter::All,
        }
    }

    view! {
        // Main Navbar - Fixed at top, single row
        <nav class="navbar">
            // ROW 1: buttons always visible
            <div class="navbar-row navbar-row-1">
                <div class="nav-row1-left">
                    <button class="nav-btn" class:clicked={move || clicked.get().contains(&0)} on:click=on_tabs_drawer title="Tabs">"☰"</button>
                    <button class="nav-btn nav-search-btn" class:clicked={move || clicked.get().contains(&3)} on:click=on_search_click title="Search">"🔍"</button>
                    <button class="nav-btn" class:clicked={move || clicked.get().contains(&1)} on:click=on_redo
                        on:contextmenu=on_redo_context
                        disabled={move || !can_redo()} title="Redo (hold for list)">"↻"</button>
                </div>
                <div class="nav-row1-right">
                    <button class="nav-btn" class:clicked={move || clicked.get().contains(&2)} on:click=on_undo
                        on:contextmenu=on_undo_context
                        disabled={move || !can_undo()} title="Undo (hold for list)">"↺"</button>
                    <button class="nav-btn" class:clicked={move || clicked.get().contains(&5)} on:click=on_message_click title="Open messages" aria-label="Open messages">
                        <span aria-hidden="true">"💬"</span>
                        {move || {
                            let count = message_count();
                            if count > 0 {
                                view! { <span class="nav-message-badge">{count}</span> }.into_any()
                            } else { ().into_any() }
                        }}
                    </button>
                    <button class="nav-btn" class:clicked={move || clicked.get().contains(&6)} on:click=on_notifications_click title="Notifications" aria-label="Notifications">
                        <span aria-hidden="true">"🔔"</span>
                        {move || {
                            let count = notification_count();
                            if count > 0 {
                                view! { <span class="nav-message-badge nav-notif-count-badge">{count}</span> }.into_any()
                            } else { ().into_any() }
                        }}
                    </button>
                </div>
            </div>
        </nav>

        // Tabs drawer (left-side panel from ☰ button)
        {move || if is_tabs_drawer_open() {
            view! {
                <div class="tabs-drawer-overlay" on:click=move |_| ui_store.update(|ui| ui.close_tabs_drawer())>
                    <div class="tabs-drawer" on:click=|ev| ev.stop_propagation()>
                        <div class="tabs-drawer-content">
                            <TabList />
                        </div>
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
            let on_close_notif = {
                let set_show_sort_menu = set_show_sort_menu.clone();
                move |_| {
                    notification_store.update(|s| s.close_drawer());
                    set_show_sort_menu.set(false);
                }
            };
            view! {
                <div class="notif-drawer-overlay" on:click=on_close_notif>
                    <div class="notif-drawer" on:click=|ev| ev.stop_propagation()>
                        <div class="notif-drawer-header notif-drawer-header-compact">
                            <div class="notif-drawer-actions">
                                // Quick-swap filter button (e.g. All -> Portfolios -> Networking -> ...)
                                <button
                                    class="notif-drawer-mode-btn notif-filter-btn"
                                    on:click=move |_| {
                                        let mut store = notification_store.get();
                                        let next = next_filter(&store.drawer_filter);
                                        if matches!(next, NotificationDrawerFilter::All) {
                                            store.clear_drawer_scope();
                                        }
                                        store.set_drawer_filter(next);
                                        notification_store.set(store);
                                    }
                                    title="Cycle notification filter"
                                >
                                    {move || filter_label(&notification_store.get().drawer_filter)}
                                </button>

                                // Sort dropdown button with options menu
                                <div class="notif-sort-dropdown">
                                    <button
                                        class="notif-drawer-mode-btn notif-sort-btn"
                                        class:active={move || show_sort_menu.get()}
                                        on:click=move |_| set_show_sort_menu.update(|v| *v = !*v)
                                        title="Change sort order"
                                    >
                                        {move || format!("{} ▼", sort_label(&notification_store.get().drawer_sort))}
                                    </button>
                                    {move || if show_sort_menu.get() {
                                        let menu_items = [
                                            NotificationDrawerSort::Newest,
                                            NotificationDrawerSort::Oldest,
                                            NotificationDrawerSort::Type,
                                        ];
                                        view! {
                                            <div class="notif-sort-menu">
                                                {menu_items.into_iter().map(|sort| {
                                                    let label = sort_label(&sort);
                                                    let sort_for_active = sort.clone();
                                                    let sort_for_click = sort.clone();
                                                    view! {
                                                        <button
                                                            class="notif-sort-menu-item"
                                                            class:active={move || notification_store.get().drawer_sort == sort_for_active}
                                                            on:click=move |_| {
                                                                notification_store.update(|s| s.set_drawer_sort(sort_for_click.clone()));
                                                                set_show_sort_menu.set(false);
                                                            }
                                                        >
                                                            {label}
                                                        </button>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_any()
                                    } else { ().into_any() }}
                                </div>

                                // Document reader / all notifications toggle
                                <button
                                    class="notif-drawer-mode-btn"
                                    class:active={move || show_doc_reader.get()}
                                    on:click=move |_| set_show_doc_reader.update(|v| *v = !*v)
                                    title="Toggle document reader view"
                                >
                                    {move || if show_doc_reader.get() { "🔔 All" } else { "📄 Reader" }}
                                </button>
                            </div>
                        </div>
                        <div class="notif-drawer-body">
                            {move || if show_doc_reader.get() {
                                document_reader_view(app_store, notification_store, move || set_show_doc_reader.set(false)).into_any()
                            } else {
                                let store = notification_store.get();
                                let mut notifs: Vec<Notification> = store.notifications.iter()
                                    .filter(|n| matches_filter(n, &store.drawer_filter, &store.drawer_scoped_portfolio, &store.drawer_scoped_group))
                                    .cloned()
                                    .collect();
                                apply_sort(&mut notifs, &store.drawer_sort);
                                if notifs.is_empty() {
                                    view! {
                                        <div class="notif-empty">
                                            <div class="notif-empty-icon">"🔕"</div>
                                            <div class="notif-empty-text">"No notifications"</div>
                                        </div>
                                    }.into_any()
                                } else {
                                    let notifs_sorted = notifs;
                                    view! {
                                        <For
                                            each=move || notifs_sorted.clone()
                                            key=|n| n.id
                                            children=move |n| {
                                                let nid = n.id;
                                                let n_for_nav = n.clone();
                                                let n_for_nav2 = n.clone();
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
                                                    let notification_store = notification_store;
                                                    let n_for_nav = n_for_nav.clone();
                                                    move |_| {
                                                        if has_linked_origin {
                                                            app_store.update(|s| s.navigate_to_notification(&n_for_nav));
                                                        } else if let Some(tab) = target.clone() {
                                                            app_store.update(|s| s.expand_tab(tab));
                                                        }
                                                        notification_store.update(|s| s.close_drawer());
                                                    }
                                                };
                                                let on_go_to_content = {
                                                    let app_store = app_store;
                                                    let notification_store = notification_store;
                                                    let n_for_nav = n_for_nav2.clone();
                                                    move |ev: leptos::ev::MouseEvent| {
                                                        ev.stop_propagation();
                                                        if has_linked_origin {
                                                            app_store.update(|s| s.navigate_to_notification(&n_for_nav));
                                                        }
                                                        notification_store.update(|s| s.close_drawer());
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
                                                                notification_store.update(|s| s.remove_notification(nid));
                                                            }>"✕"</button>
                                                    </div>
                                                }
                                            }
                                        />
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>
                </div>
            }.into_any()
        } else { ().into_any() }}

        // Search panel - drops below navbar when open (CSS-toggled, always mounted)
        <div class="search-drop-overlay"
            class:search-visible=move || is_search_open()
            on:click=move |_| ui_store.update(|s| s.close_search())
        ></div>
        <div class="search-drop-panel"
            class:search-visible=move || is_search_open()
            on:click=|ev| ev.stop_propagation()
        >
            <SearchFilters />
        </div>

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
                        view! {
                            <For
                                each=move || actions.clone()
                                key=|action| action.id
                                children=move |action| {
                                    let action_id = action.id;
                                    let desc = format!("{} {}", action.action_type_badge(), action.description);
                                    view! {
                                        <div class="nav-dropdown-item"
                                            on:click=move |_| on_dropdown_action(action_id, is_redo)>
                                            {desc}
                                        </div>
                                    }
                                }
                            />
                        }.into_any()
                    }}
                </div>
            }.into_any()
        } else { ().into_any() }}
    }
}

/// Grouped document-notification reader view.
/// Shows the latest update per document, with a preview and an open button.
fn document_reader_view(
    app_store: leptos::prelude::RwSignal<crate::stores::AppStore>,
    notification_store: leptos::prelude::RwSignal<crate::stores::NotificationStore>,
    on_done: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    let on_done = std::sync::Arc::new(on_done);
    let doc_notifs = Memo::new(move |_| {
        let store = notification_store.get();
        let mut grouped: std::collections::HashMap<Uuid, Vec<Notification>> =
            std::collections::HashMap::new();
        for n in store
            .notifications
            .iter()
            .filter(|n| n.linked_doc_id.is_some() && n.target_tab.is_some())
        {
            grouped
                .entry(n.linked_doc_id.unwrap())
                .or_default()
                .push(n.clone());
        }
        for notifs in grouped.values_mut() {
            notifs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        }
        let mut result: Vec<_> = grouped
            .into_iter()
            .filter_map(|(doc_id, notifs)| {
                let latest = notifs.first()?.clone();
                let doc_name = app_store
                    .get()
                    .find_document(doc_id)
                    .map(|d| d.name)
                    .unwrap_or_else(|| "Document".to_string());
                Some((doc_id, doc_name, latest, notifs.len()))
            })
            .collect();
        result.sort_by(|a, b| b.2.timestamp.cmp(&a.2.timestamp));
        result
    });

    view! {
        <div class="doc-reader-view">
            <div class="doc-reader-header">
                <span class="doc-reader-title">"Document Reader"</span>
                <span class="doc-reader-subtitle">"Latest updates on documents"</span>
            </div>
            {move || {
                let items = doc_notifs.get();
                if items.is_empty() {
                    view! {
                        <div class="notif-empty">
                            <div class="notif-empty-icon">"📄"</div>
                            <div class="notif-empty-text">"No document notifications"</div>
                        </div>
                    }.into_any()
                } else {
                    items.into_iter().map(|(_doc_id, doc_name, latest, count)| {
                        let nid = latest.id;
                        let preview = latest.content_preview.clone();
                        let from = latest.from_user.clone().unwrap_or_else(|| "System".to_string());
                        let time = format!("{}", latest.timestamp.format("%b %d, %H:%M"));
                        let on_done_inner = on_done.clone();
                        view! {
                            <div class="doc-reader-card">
                                <div class="doc-reader-card-header">
                                    <span class="doc-reader-doc-name">{doc_name}</span>
                                    <span class="doc-reader-count">{format!("{} update{}", count, if count == 1 { "" } else { "s" })}</span>
                                </div>
                                <div class="doc-reader-meta">
                                    <span>{from}</span>
                                    <span>{time}</span>
                                </div>
                                {preview.map(|p| view! {
                                    <div class="doc-reader-preview">
                                        <pre>{p}</pre>
                                    </div>
                                }.into_any())}
                                <div class="doc-reader-actions">
                                    <button class="doc-reader-open-btn" on:click=move |_| {
                                        app_store.update(|s| s.navigate_to_notification(&latest));
                                        on_done_inner();
                                    }>"Open Document"</button>
                                    <button class="doc-reader-dismiss-btn" on:click=move |_| {
                                        notification_store.update(|s| s.remove_notification(nid));
                                    }>"Dismiss"</button>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>().into_any()
                }
            }}
        </div>
    }
}
