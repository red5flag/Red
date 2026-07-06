use leptos::prelude::*;

#[component]
pub(crate) fn HistorySummary(
    #[prop(into)] action_count: Signal<usize>,
    #[prop(into)] total_count: Signal<usize>,
) -> impl IntoView {
    view! {
        <div class="history-summary-count">
            {move || format!("{} matching / {} recorded", action_count.get(), total_count.get())}
        </div>
    }
}
