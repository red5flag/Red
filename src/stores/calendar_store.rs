use crate::models::{Booking, CalendarEvent, CalendarEventStatus, CalendarEventType, ServiceTask};
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

    pub fn remove_events_for_booking(&mut self, booking_id: Uuid) {
        self.calendar_events
            .retain(|e| e.related_booking_id != Some(booking_id));
    }

    pub fn remove_events_for_service_task(&mut self, task_id: Uuid) {
        self.calendar_events
            .retain(|e| e.related_service_task_id != Some(task_id));
    }

    /// Sync a single calendar event for a booking, creating or updating it.
    pub fn sync_booking_event(
        &mut self,
        booking: &Booking,
        asset_name: &str,
        channel_name: Option<&str>,
        portfolio_id: Option<Uuid>,
    ) -> Uuid {
        let existing = self
            .calendar_events
            .iter_mut()
            .find(|e| e.related_booking_id == Some(booking.id));
        let source = match booking.source {
            crate::models::BookingSource::TestChannel => "Test Channel",
            crate::models::BookingSource::Manual => "Manual",
        };
        if let Some(ev) = existing {
            ev.update_from_booking(booking, source);
            ev.id
        } else {
            let title = format!(
                "{} - {} ({})",
                booking.guest_name,
                asset_name,
                channel_name.unwrap_or("Manual")
            );
            let event = CalendarEvent::for_booking(
                title,
                booking.start_datetime,
                booking.end_datetime,
                portfolio_id,
                booking.asset_id,
                booking.channel_id,
                booking.id,
                source,
            );
            let id = event.id;
            self.calendar_events.push(event);
            id
        }
    }

    /// Sync a single calendar event for a service task, creating or updating it.
    pub fn sync_service_task_event(
        &mut self,
        task: &ServiceTask,
        asset_name: &str,
        portfolio_id: Option<Uuid>,
    ) -> Uuid {
        let existing = self
            .calendar_events
            .iter_mut()
            .find(|e| e.related_service_task_id == Some(task.id));
        if let Some(ev) = existing {
            ev.update_from_service_task(task);
            ev.id
        } else {
            let title = format!("{} - {}", task.task_type.display(), asset_name);
            let event = CalendarEvent::for_service_task(
                title,
                task.start_datetime,
                task.end_datetime,
                portfolio_id,
                task.asset_id,
                task.id,
                &task.task_type,
            );
            let id = event.id;
            self.calendar_events.push(event);
            id
        }
    }

    pub fn add_conflict_event(
        &mut self,
        asset_id: Uuid,
        portfolio_id: Option<Uuid>,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
        description: String,
    ) -> Uuid {
        let event = CalendarEvent {
            id: Uuid::new_v4(),
            title: "Conflict".to_string(),
            description: Some(description),
            start,
            end,
            all_day: false,
            source: Some("Conflict detection".to_string()),
            related_portfolio_id: portfolio_id,
            related_group_id: None,
            related_asset_id: Some(asset_id),
            related_channel_id: None,
            related_booking_id: None,
            related_service_task_id: None,
            category: Some("Conflict".to_string()),
            event_type: CalendarEventType::Conflict,
            status: CalendarEventStatus::Active,
        };
        let id = event.id;
        self.calendar_events.push(event);
        id
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
