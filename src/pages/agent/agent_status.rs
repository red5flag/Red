use leptos::prelude::*;

#[component]
pub(crate) fn AgentStatus() -> impl IntoView {
    view! {
        <div class="agent-topbar">
            <div class="agent-topbar-left">
                <div class="agent-avatar-sm">"🤖"</div>
                <div>
                    <div class="agent-topbar-title">"Red Agent"</div>
                    <div class="agent-topbar-status">"● Online"</div>
                </div>
            </div>
            <div class="agent-topbar-right">
                <span class="agent-badge">"AI"</span>
            </div>
        </div>
    }
}
