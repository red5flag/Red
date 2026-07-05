use crate::models::CalendarEvent;
use chrono::{Duration, Utc};
use leptos::prelude::*;
use uuid::Uuid;

/// Dedicated store for calendar event state: bookings, imported events, and
/// dev/test events. Extracted from AppStore so calendar lifecycle changes do
/// not invalidate consumers of unrelated domain state.
#[derive(Clone, Debug)]
pub struct CalendarStore {
    pub calendar_events: Vec<CalendarEvent>,
}

impl Default for CalendarStore {
    fn default() -> Self {
        Self {
            calendar_events: Vec::new(),
        }
    }
}

impl CalendarStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_calendar_event(&mut self, event: CalendarEvent) {
        self.calendar_events.push(event);
    }

    pub fn clear_calendar_events(&mut self) {
        self.calendar_events.clear();
    }

    pub fn upsert_calendar_event(&mut self, event: CalendarEvent) {
        self.calendar_events.retain(|e| e.id != event.id);
        self.calendar_events.push(event);
    }

    pub fn remove_calendar_event(&mut self, event_id: Uuid) {
        self.calendar_events.retain(|e| e.id != event_id);
    }

    // Developer/test helper
    pub fn dev_test_add_calendar_event(&mut self, title: &str, days_ahead: i64) {
        let s = Utc::now() + Duration::days(days_ahead);
        let mut ev = CalendarEvent::new(title.into(), s, s + Duration::hours(2));
        ev.source = Some("DevTest".into());
        self.add_calendar_event(ev);
    }
}

pub fn create_calendar_store() -> RwSignal<CalendarStore> {
    RwSignal::new(CalendarStore::new())
}

pub fn provide_calendar_store() -> RwSignal<CalendarStore> {
    let store = create_calendar_store();
    provide_context(store);
    store
}

pub fn use_calendar_store() -> RwSignal<CalendarStore> {
    expect_context::<RwSignal<CalendarStore>>()
}
