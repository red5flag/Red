use crate::pages::history::{HistoryFilters, HistoryList, HistorySummary, UndoRedoDropdown};
use crate::stores::{
    apply_redo_side_effects, apply_undo_side_effects, create_action, use_app_store,
    use_undo_redo_store, HistoryQuery,
};
use crate::types::{ActionType, SortMode};
use leptos::prelude::*;

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

    let (dropdown, set_dropdown) =
        signal::<Option<(i32, i32, Vec<crate::models::Action>, bool)>>(None);

    let filtered_actions = move || {
        let mut q = HistoryQuery::default();
        q.text = search_text.get();
        if let Some(t) = type_filter.get() {
            q.action_types = Some(vec![t]);
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
    let can_undo = move || undo_store.get().can_undo_by_user(current_user_id());
    let can_redo = move || undo_store.get().can_redo_by_user(current_user_id());

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

    let on_undo = move |_| {
        if dropdown.get().is_some() {
            return;
        }
        let uid = current_user_id();
        if let Some(undone) = undo_store.get().undo_by_user(uid) {
            record_undo_redo(ActionType::Undo, format!("Undid: {}", undone.description));
            app_store.update(|store| {
                apply_undo_side_effects(&undone, store);
            });
        }
    };

    let on_redo = move |_| {
        if dropdown.get().is_some() {
            return;
        }
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

                <HistoryFilters
                    search_text={search_text}
                    set_search_text={set_search_text}
                    type_filter={type_filter}
                    set_type_filter={set_type_filter}
                    has_reason_only={has_reason_only}
                    set_has_reason_only={set_has_reason_only}
                />

                <HistoryList
                    actions={Signal::derive(filtered_actions)}
                    current_user_id={current_user_id()}
                    on_history_undo={Callback::new(move |id| on_history_undo(id))}
                />
            </div>

            <UndoRedoDropdown
                dropdown={dropdown}
                close_dropdown={Callback::new(move |_| close_dropdown(()))}
                on_dropdown_action={Callback::new(move |(id, is_redo)| on_dropdown_action(id, is_redo))}
            />
        </div>
    }
}
