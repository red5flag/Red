use crate::pages::history::{action_type_badge, severity_badge};
use crate::stores::format_action_description;
use crate::types::ActionType;
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
    let (severity_label, severity_class) = severity_badge(&action.change_severity);
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
    let is_undoable = action.is_undoable();
    let viewport_context = action.viewport_context.clone();
    let notarised = action.notarised;

    view! {
        <div class="timeline-item">
            <div class="timeline-time">{time}</div>
            <div class="timeline-content">
                <div class="timeline-action">
                    <span class={format!("action-badge {}", badge_class)}>{type_label}</span>
                    <span class={format!("severity-badge {}", severity_class)}>{severity_label}</span>
                    {description}
                    {if notarised {
                        view! { <span class="notarised-indicator" title="Notarised">"🔒"</span> }.into_any()
                    } else { ().into_any() }}
                </div>
                <div class="timeline-meta">
                    <span class="timeline-user">{user_name}</span>
                    <span class="timeline-role">{user_role}</span>
                    <span class="timeline-date">{date}</span>
                    {if let Some(ref ctx) = viewport_context {
                        let page = ctx.page.clone();
                        let entity_type = ctx.entity_type.clone();
                        let entity_id = ctx.entity_id.map(|id| id.to_string()).unwrap_or_else(|| "—".to_string());
                        let tab = ctx.tab.clone().unwrap_or_else(|| "—".to_string());
                        view! {
                            <span class="timeline-context">
                                {format!("{} / {} / {} / tab: {}", page, entity_type, entity_id, tab)}
                            </span>
                        }.into_any()
                    } else { ().into_any() }}
                    {if let Some(r) = why {
                        if !r.trim().is_empty() {
                            view! { <span class="timeline-reason">{format!("Why: {}", r)}</span> }.into_any()
                        } else { ().into_any() }
                    } else { ().into_any() }}
                    {if is_current_user && is_undoable {
                        view! {
                            <button class="timeline-undo-btn"
                                on:click=move |_| on_undo.run(action_id)
                                title="Undo this action"
                                aria-label="Undo this action">
                                "↺"
                            </button>
                        }.into_any()
                    } else if action.action_type == ActionType::Login || action.action_type == ActionType::Logout {
                        view! {
                            <span class="timeline-locked-indicator" title="Login/logout actions cannot be undone">"🔒"</span>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
            </div>
        </div>
    }
}
