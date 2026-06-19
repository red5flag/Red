use crate::stores::use_undo_redo_store;
use leptos::prelude::*;

#[component]
pub fn HistoryPage() -> impl IntoView {
    let undo_store = use_undo_redo_store();

    let action_count = move || undo_store.get().past.len();

    view! {
        <div class="home-screen">
            <div class="welcome-header">
                <h1>"History"</h1>
                <p>{move || format!("{} recorded actions", action_count())}</p>
            </div>
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Recent Actions"</span>
                </div>
                <div style="padding: 20px; text-align: center; color: var(--text-secondary);">
                    <p>"Action history will appear here..."</p>
                    <div style="margin-top: 20px; font-size: 48px;">"📜"</div>
                </div>
            </div>
        </div>
    }
}
