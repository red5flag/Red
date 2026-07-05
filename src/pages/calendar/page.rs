use crate::components::calendar::{CalendarManager, CalendarScope};
use leptos::prelude::*;

#[component]
pub fn CalendarPage() -> impl IntoView {
    view! {
        <div class="calendar-page">
            <CalendarManager scope={CalendarScope::global()} />
        </div>
    }
}
