use leptos::prelude::*;

#[component]
pub(crate) fn HistorySummary(
    #[prop(into)] action_count: Signal<usize>,
    #[prop(into)] total_count: Signal<usize>,
    #[prop(into)] current_user_name: Signal<String>,
    #[prop(into)] current_user_role: Signal<String>,
) -> impl IntoView {
    view! {
        <div class="welcome-header">
            <h1>"History"</h1>
            <p>{move || format!("{} matching / {} recorded", action_count.get(), total_count.get())}</p>
        </div>

        <div class="data-card">
            <div class="card-header">
                <span class="card-title">"Current User"</span>
            </div>
            <div class="card-stats">
                <div class="stat-item">
                    <div class="stat-label">"Name"</div>
                    <div class="stat-value">{move || current_user_name.get()}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">"Role"</div>
                    <div class="stat-value">{move || current_user_role.get()}</div>
                </div>
            </div>
        </div>
    }
}
