use crate::stores::{apply_redo_side_effects, apply_undo_side_effects, create_action, format_action_description, use_app_store, use_undo_redo_store, HistoryQuery};
use crate::types::{ActionType, SortMode};
use leptos::prelude::*;

fn action_type_badge(action_type: &ActionType) -> (&'static str, &'static str) {
    match action_type {
        ActionType::Create => ("Create", "badge-create"),
        ActionType::Update => ("Update", "badge-update"),
        ActionType::Delete => ("Delete", "badge-delete"),
        ActionType::View => ("View", "badge-view"),
        ActionType::Navigate => ("Navigate", "badge-nav"),
        ActionType::Setting => ("Setting", "badge-setting"),
        ActionType::Payment => ("Payment", "badge-payment"),
        ActionType::Notification => ("Notification", "badge-notif"),
        ActionType::Search => ("Search", "badge-search"),
        ActionType::Undo => ("Undo", "badge-undo"),
        ActionType::Redo => ("Redo", "badge-redo"),
        ActionType::Login => ("Login", "badge-login"),
        ActionType::Logout => ("Logout", "badge-logout"),
    }
}

#[component]
pub fn HistoryPage() -> impl IntoView {
    let undo_store = use_undo_redo_store();
    let app_store = use_app_store();

    let (search_text, set_search_text) = signal(String::new());
    let (type_filter, set_type_filter) = signal::<Option<ActionType>>(None);
    let (has_reason_only, set_has_reason_only) = signal(false);
    let (sort_mode, set_sort_mode) = signal(SortMode::Recent);

    let current_user_name = move || app_store.get().current_user.name.clone();
    let current_user_role = move || format!("{:?}", app_store.get().current_user.role);
    let current_user_id = move || app_store.get().current_user.id;

    let (dropdown, set_dropdown) = signal::<Option<(i32, i32, Vec<crate::models::Action>, bool)>>(None);

    let filtered_actions = move || {
        let mut q = HistoryQuery::default();
        q.text = search_text.get();
        if let Some(t) = type_filter.get() {
            q.action_types = Some(vec![t]);
        }
        q.has_reason_only = has_reason_only.get();
        let mut actions: Vec<_> = undo_store.get().search_actions(&q).into_iter().cloned().collect();
        match sort_mode.get() {
            SortMode::Oldest => actions.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)),
            _ => actions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)),
        }
        actions
    };

    let action_count = move || filtered_actions().len();
    let total_count = move || undo_store.get().past.len();
    let can_undo = move || undo_store.get().can_undo_by_user(current_user_id());
    let can_redo = move || undo_store.get().can_redo_by_user(current_user_id());

    let record_undo_redo = move |kind: ActionType, description: String| {
        let store = app_store.get();
        let uid = store.current_user.id;
        let name = store.current_user.name.clone();
        let role = format!("{:?}", store.current_user.role);
        let org = store.current_user.organization_id;
        drop(store);
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

    let on_undo = move |_| {
        if dropdown.get().is_some() { return; }
        let uid = current_user_id();
        if let Some(undone) = undo_store.get().undo_by_user(uid) {
            record_undo_redo(ActionType::Undo, format!("Undid: {}", undone.description));
            app_store.update(|store| {
                apply_undo_side_effects(&undone, store);
            });
        }
    };

    let on_redo = move |_| {
        if dropdown.get().is_some() { return; }
        let uid = current_user_id();
        if let Some(redone) = undo_store.get().redo_by_user(uid) {
            record_undo_redo(ActionType::Redo, format!("Redid: {}", redone.description));
            app_store.update(|store| {
                apply_redo_side_effects(&redone, store);
            });
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
                app_store.update(|store| {
                    apply_redo_side_effects(&redone, store);
                });
            }
        } else {
            if let Some(undone) = undo_store.get().undo_action_by_id(action_id) {
                record_undo_redo(ActionType::Undo, format!("Undid: {}", undone.description));
                app_store.update(|store| {
                    apply_undo_side_effects(&undone, store);
                });
            }
        }
    };

    let on_history_undo = move |action_id: uuid::Uuid| {
        if let Some(undone) = undo_store.get().undo_action_by_id(action_id) {
            record_undo_redo(ActionType::Undo, format!("Undid: {}", undone.description));
            app_store.update(|store| {
                apply_undo_side_effects(&undone, store);
            });
        }
    };

    view! {
        <div class="home-screen">
            <div class="welcome-header">
                <h1>"History"</h1>
                <p>{move || format!("{} matching / {} recorded", action_count(), total_count())}</p>
            </div>

            // Current user info
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Current User"</span>
                </div>
                <div class="card-stats">
                    <div class="stat-item">
                        <div class="stat-label">"Name"</div>
                        <div class="stat-value">{current_user_name}</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">"Role"</div>
                        <div class="stat-value">{current_user_role}</div>
                    </div>
                </div>
            </div>

            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Actions"</span>
                    <div class="history-undo-redo">
                        <button class="history-undo-btn" on:click=on_redo
                            on:contextmenu=on_redo_context
                            disabled={move || !can_redo()} title="Redo (hold for list)">"↻ Redo"</button>
                        <button class="history-undo-btn" on:click=on_undo
                            on:contextmenu=on_undo_context
                            disabled={move || !can_undo()} title="Undo (hold for list)">"↺ Undo"</button>
                    </div>
                    <select
                        class="form-select"
                        style="width: auto; min-width: 120px;"
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            let mode = match v.as_str() {
                                "oldest" => SortMode::Oldest,
                                _ => SortMode::Recent,
                            };
                            set_sort_mode.set(mode);
                        }
                    >
                        <option value="recent">"Recent"</option>
                        <option value="oldest">"Oldest"</option>
                    </select>
                </div>

                // Search / filter bar
                <div class="history-filter-bar">
                    <input
                        class="history-search-input"
                        type="text"
                        placeholder="Search who, what, where, why…"
                        prop:value=move || search_text.get()
                        on:input=move |ev| set_search_text.set(event_target_value(&ev))
                    />
                    <select
                        class="history-filter-select"
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            let f = match v.as_str() {
                                "create" => Some(ActionType::Create),
                                "update" => Some(ActionType::Update),
                                "delete" => Some(ActionType::Delete),
                                "view" => Some(ActionType::View),
                                "navigate" => Some(ActionType::Navigate),
                                "search" => Some(ActionType::Search),
                                "login" => Some(ActionType::Login),
                                "logout" => Some(ActionType::Logout),
                                _ => None,
                            };
                            set_type_filter.set(f);
                        }
                    >
                        <option value="">"All types"</option>
                        <option value="create">"Create"</option>
                        <option value="update">"Update"</option>
                        <option value="delete">"Delete"</option>
                        <option value="view">"View"</option>
                        <option value="navigate">"Navigate"</option>
                        <option value="search">"Search"</option>
                        <option value="login">"Login"</option>
                        <option value="logout">"Logout"</option>
                    </select>
                    <label class="history-filter-check">
                        <input
                            type="checkbox"
                            on:change=move |ev| set_has_reason_only.set(event_target_checked(&ev))
                        />
                        "Has reason"
                    </label>
                </div>

                {move || {
                    let actions = filtered_actions();
                    if actions.is_empty() {
                        view! {
                            <div class="history-empty">
                                <p>"No matching actions"</p>
                                <div class="history-empty-icon">"📜"</div>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class="timeline">
                                {actions
                                    .into_iter()
                                    .map(|action| {
                                        let description = format_action_description(&action);
                                        let time = action.timestamp.format("%H:%M:%S").to_string();
                                        let date = action.timestamp.format("%Y-%m-%d").to_string();
                                        let (type_label, badge_class) = action_type_badge(&action.action_type);
                                        let user_name = if action.user_name.is_empty() {
                                            "Unknown".to_string()
                                        } else {
                                            action.user_name.clone()
                                        };
                                        let user_role = if action.user_role.is_empty() {
                                            "—".to_string()
                                        } else {
                                            action.user_role.clone()
                                        };
                                        let why = action.reason.clone();
                                        let action_id = action.id;
                                        let is_current_user = action.user_id == current_user_id();

                                        view! {
                                            <div class="timeline-item">
                                                <div class="timeline-time">{time}</div>
                                                <div class="timeline-content">
                                                    <div class="timeline-action">
                                                        <span class={format!("action-badge {}", badge_class)}>{type_label}</span>
                                                        {description}
                                                    </div>
                                                    <div class="timeline-meta">
                                                        <span class="timeline-user">{user_name}</span>
                                                        <span class="timeline-role">{user_role}</span>
                                                        <span class="timeline-date">{date}</span>
                                                        {if let Some(r) = why {
                                                            if !r.trim().is_empty() {
                                                                view! { <span class="timeline-reason">{format!("Why: {}", r)}</span> }.into_any()
                                                            } else { ().into_any() }
                                                        } else { ().into_any() }}
                                                        {if is_current_user {
                                                            view! {
                                                                <button class="timeline-undo-btn"
                                                                    on:click=move |_| on_history_undo(action_id)
                                                                    title="Undo this action">
                                                                    "↺"
                                                                </button>
                                                            }.into_any()
                                                        } else { ().into_any() }}
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                        }
                            .into_any()
                    }
                }}
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
        </div>
    }
}
