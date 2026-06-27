use crate::components::calendar::{CalendarManager, CalendarScope};
use crate::stores::use_app_store;
use leptos::prelude::*;

#[component]
pub fn CalendarPage() -> impl IntoView {
    let app_store = use_app_store();

    view! {
        <div class="calendar-page">
            <CalendarManager scope={CalendarScope::global()} />
            {move || {
                let store = app_store.get();
                let commercial = store.portfolios.iter().find(|p| p.name == "Commercial Real Estate");
                commercial.map(|p| {
                    let pid = p.id;
                    view! {
                        <CalendarManager scope={CalendarScope { portfolio_id: Some(pid), group_id: None, asset_id: None, title: "Commercial Real Estate Calendar".to_string() }} />
                    }.into_any()
                }).unwrap_or(().into_any())
            }}
        </div>
    }
}
