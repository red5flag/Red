use crate::types::ActionType;
use leptos::prelude::*;

#[component]
pub(crate) fn HistoryFilters(
    search_text: ReadSignal<String>,
    set_search_text: WriteSignal<String>,
    #[allow(unused_variables)] type_filter: ReadSignal<Option<ActionType>>,
    set_type_filter: WriteSignal<Option<ActionType>>,
    #[allow(unused_variables)] has_reason_only: ReadSignal<bool>,
    set_has_reason_only: WriteSignal<bool>,
) -> impl IntoView {
    view! {
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
    }
}
