use leptos::prelude::*;
use leptos_router::components::Redirect;

#[component]
pub fn HomePage() -> impl IntoView {
    // Redirect to overview by default
    view! {
        <Redirect path="/overview" />
    }
}
