use crate::models::ApprovalAction;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub(crate) fn PrintRecordButton() -> impl IntoView {
    let on_print = move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.print();
        }
    };
    view! {
        <button class="tx-print-btn" on:click=on_print>"🖨 Print Record"</button>
    }
}

#[component]
pub(crate) fn TransactionApprovalBar(
    tx_id: Uuid,
    can_submit: bool,
    can_approve: bool,
    can_reject: bool,
    can_withdraw: bool,
    can_execute: bool,
    can_lock: bool,
    on_action: Callback<(Uuid, ApprovalAction, Option<String>)>,
) -> impl IntoView {
    let (reject_comment, set_reject_comment) = signal(String::new());
    let action_btn = |label: &'static str,
                      action: ApprovalAction,
                      enabled: bool,
                      class: &'static str| {
        let on_action = on_action.clone();
        let tx_id = tx_id;
        view! {
            <button
                class={format!("tx-action-btn {}", class)}
                disabled={move || !enabled}
                aria-label={label}
                on:click=move |_| {
                    let comment = if action == ApprovalAction::Reject { Some(reject_comment.get()) } else { None };
                    on_action.run((tx_id, action, comment));
                }
            >
                {label}
            </button>
        }
    };
    view! {
        <div class="tx-approval-bar" role="group" aria-label="Transaction approval actions">
            {action_btn("Submit", ApprovalAction::Submit, can_submit, "tx-action-btn-submit")}
            {action_btn("Approve", ApprovalAction::Approve, can_approve, "tx-action-btn-approve")}
            {action_btn("Reject", ApprovalAction::Reject, can_reject, "tx-action-btn-reject")}
            {action_btn("Withdraw", ApprovalAction::Withdraw, can_withdraw, "tx-action-btn-withdraw")}
            {action_btn("Execute", ApprovalAction::Execute, can_execute, "tx-action-btn-execute")}
            {action_btn("Lock", ApprovalAction::Lock, can_lock, "tx-action-btn-lock")}
            {if can_reject {
                view! {
                    <input
                        class="form-input tx-reject-input"
                        type="text"
                        placeholder="Reason for rejection"
                        prop:value={move || reject_comment.get()}
                        on:input=move |ev| set_reject_comment.set(event_target_value(&ev))
                    />
                }.into_any()
            } else { ().into_any() }}
        </div>
    }
}
