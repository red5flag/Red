use crate::pages::history::action_type_badge;
use crate::stores::format_action_description;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub(crate) fn HistoryCard(
    action: crate::models::Action,
    current_user_id: Uuid,
    on_undo: Callback<Uuid>,
) -> impl IntoView {
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
    let is_current_user = action.user_id == current_user_id;

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
                                on:click=move |_| on_undo.run(action_id)
                                title="Undo this action">
                                "↺"
                            </button>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
            </div>
        </div>
    }
}
