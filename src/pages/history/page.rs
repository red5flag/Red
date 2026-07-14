use crate::pages::history::{HistoryFilters, HistoryList, HistorySummary};
use crate::stores::{
    apply_redo_side_effects, apply_undo_side_effects, create_action, use_app_store,
    use_undo_redo_store, HistoryQuery,
};
use crate::types::{ActionType, ChangeSeverity, SortMode};
use leptos::prelude::*;

#[component]
pub fn HistoryPage() -> impl IntoView {
    let undo_store = use_undo_redo_store();
    let app_store = use_app_store();

    let (search_text, set_search_text) = signal(String::new());
    let (type_filter, set_type_filter) = signal::<Option<ActionType>>(None);
    let (severity_filter, set_severity_filter) = signal::<Option<ChangeSeverity>>(None);
    let (has_reason_only, set_has_reason_only) = signal(false);
    let (sort_mode, set_sort_mode) = signal(SortMode::Recent);

    let current_user_name = move || app_store.get().current_user.name.clone();
    let current_user_role = move || format!("{:?}", app_store.get().current_user.role);
    let current_user_id = move || app_store.get().current_user.id;

    let filtered_actions = move || {
        let mut q = HistoryQuery::default();
        q.text = search_text.get();
        if let Some(t) = type_filter.get() {
            q.action_types = Some(vec![t]);
        }
        if let Some(s) = severity_filter.get() {
            q.severity = Some(s);
        }
        q.has_reason_only = has_reason_only.get();
        let mut actions: Vec<_> = undo_store
            .get()
            .search_actions(&q)
            .into_iter()
            .cloned()
            .collect();
        match sort_mode.get() {
            SortMode::Oldest => actions.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)),
            _ => actions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)),
        }
        actions
    };

    let action_count = move || filtered_actions().len();
    let total_count = move || undo_store.get().past.len();

    let record_undo_redo = move |kind: ActionType, description: String| {
        let store = app_store.get();
        let uid = store.current_user.id;
        let name = store.current_user.name.clone();
        let role = format!("{:?}", store.current_user.role);
        let org = store.current_user.organization_id;
        drop(store);
        let action = create_action(kind, "Action", &description, uid, &name, &role, org, None);
        undo_store.update(|u| u.record_history_action(action));
    };

    let on_history_undo = move |action_id: uuid::Uuid| {
        if let Some(undone) = undo_store.get().undo_action_by_id(action_id) {
            record_undo_redo(ActionType::Undo, format!("Undid: {}", undone.description));
            app_store.update(|store| {
                apply_undo_side_effects(&undone, store);
            });
        }
    };

    let _on_history_redo = move |action_id: uuid::Uuid| {
        if let Some(redone) = undo_store.get().redo_action_by_id(action_id) {
            record_undo_redo(ActionType::Redo, format!("Redid: {}", redone.description));
            app_store.update(|store| {
                apply_redo_side_effects(&redone, store);
            });
        }
    };

    let can_redo = move || undo_store.get().can_redo();

    let redoable_actions = move || {
        let uid = current_user_id();
        undo_store
            .get()
            .future
            .iter()
            .filter(|a| a.user_id == uid)
            .cloned()
            .collect::<Vec<_>>()
    };

    let on_history_redo = move |action_id: uuid::Uuid| {
        if let Some(redone) = undo_store.get().redo_action_by_id(action_id) {
            record_undo_redo(ActionType::Redo, format!("Redid: {}", redone.description));
            app_store.update(|store| {
                apply_redo_side_effects(&redone, store);
            });
        }
    };

    view! {
        <div class="home-screen">
            <div class="history-page-header">
                <div class="history-page-title">
                    <span class="history-page-name">{move || current_user_name()}</span>
                    <span class="history-page-role">{move || current_user_role()}</span>
                </div>
                <HistorySummary
                    action_count={Signal::derive(action_count)}
                    total_count={Signal::derive(total_count)}
                />
            </div>

            // Redo bar: per-item redo for actions currently in the future stack
            {move || {
                let reds = redoable_actions();
                if reds.is_empty() {
                    ().into_any()
                } else {
                    view! {
                        <div class="history-redo-bar" role="region" aria-label="Redoable actions">
                            <span class="history-redo-label">"Redo:"</span>
                            <div class="history-redo-actions">
                                {reds.into_iter().map(|a| {
                                    let id = a.id;
                                    let desc = a.description.clone();
                                    view! {
                                        <button
                                            class="history-redo-btn"
                                            on:click=move |_| on_history_redo(id)
                                            title={format!("Redo: {}", desc)}
                                            aria-label={format!("Redo: {}", desc)}
                                        >
                                            {format!("↻ {}", desc)}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_any()
                }
            }}

            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Actions"</span>
                    <button
                        class="card-action-btn"
                        disabled={move || !can_redo()}
                        on:click=move |_| {
                            if let Some(redone) = undo_store.get().redo() {
                                record_undo_redo(ActionType::Redo, format!("Redid: {}", redone.description));
                                app_store.update(|store| {
                                    apply_redo_side_effects(&redone, store);
                                });
                            }
                        }
                        title="Redo last undone action"
                        aria-label="Redo last undone action"
                    >
                        "↻"
                    </button>
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

                <HistoryFilters
                    search_text={search_text}
                    set_search_text={set_search_text}
                    type_filter={type_filter}
                    set_type_filter={set_type_filter}
                    has_reason_only={has_reason_only}
                    set_has_reason_only={set_has_reason_only}
                    severity_filter={severity_filter}
                    set_severity_filter={set_severity_filter}
                />

                <HistoryList
                    actions={Signal::derive(filtered_actions)}
                    current_user_id={current_user_id()}
                    on_history_undo={Callback::new(move |id| on_history_undo(id))}
                />
            </div>
        </div>
    }
}
