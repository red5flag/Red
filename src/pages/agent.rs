use leptos::prelude::*;

#[component]
pub fn AgentPage() -> impl IntoView {
    view! {
        <div class="home-screen">
            <div class="welcome-header">
                <h1>"AI Agent"</h1>
                <p>"Your intelligent business assistant"</p>
            </div>
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Agent Chat"</span>
                </div>
                <div style="padding: 20px; text-align: center; color: var(--text-secondary);">
                    <p>"AI agent interface coming soon..."</p>
                    <div style="margin-top: 20px; font-size: 48px;">"🤖"</div>
                </div>
            </div>
        </div>
    }
}
