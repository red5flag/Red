use super::{Booking, BookingStatus, ServiceTask, ServiceTaskStatus, ServiceTaskType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalendarEventType {
    Meeting,
    AssetBooking,
    ChannelBooking,
    Cleaning,
    Maintenance,
    Inspection,
    Linen,
    CheckIn,
    CheckOut,
    Conflict,
}

impl Default for CalendarEventType {
    fn default() -> Self {
        CalendarEventType::Meeting
    }
}

impl CalendarEventType {
    pub fn all() -> &'static [CalendarEventType] {
        &[
            CalendarEventType::Meeting,
            CalendarEventType::AssetBooking,
            CalendarEventType::ChannelBooking,
            CalendarEventType::Cleaning,
            CalendarEventType::Maintenance,
            CalendarEventType::Inspection,
            CalendarEventType::Linen,
            CalendarEventType::CheckIn,
            CalendarEventType::CheckOut,
            CalendarEventType::Conflict,
        ]
    }

    pub fn display(&self) -> &'static str {
        match self {
            CalendarEventType::Meeting => "Meeting",
            CalendarEventType::AssetBooking => "Asset Booking",
            CalendarEventType::ChannelBooking => "Channel Booking",
            CalendarEventType::Cleaning => "Cleaning",
            CalendarEventType::Maintenance => "Maintenance",
            CalendarEventType::Inspection => "Inspection",
            CalendarEventType::Linen => "Linen",
            CalendarEventType::CheckIn => "Check-in",
            CalendarEventType::CheckOut => "Check-out",
            CalendarEventType::Conflict => "Conflict",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalendarEventStatus {
    Active,
    Cancelled,
    Completed,
    Tentative,
}

impl Default for CalendarEventStatus {
    fn default() -> Self {
        CalendarEventStatus::Active
    }
}

/// A single encrypted message in the secure messenger.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub recipient_id: Uuid,
    pub content: String,
    pub encrypted: bool,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
}

impl Message {
    pub fn new(sender_id: Uuid, recipient_id: Uuid, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender_id,
            recipient_id,
            content,
            encrypted: true,
            timestamp: Utc::now(),
            read: false,
        }
    }
}

/// A contact/person entry for the messenger drawer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MessengerContact {
    pub id: Uuid,
    pub name: String,
    pub source: ContactSource,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub unread_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactSource {
    Organization,
    Imported,
    Recommended,
    Bot,
}

/// Calendar/booking event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub all_day: bool,
    pub source: Option<String>,
    pub related_portfolio_id: Option<Uuid>,
    pub related_group_id: Option<Uuid>,
    pub related_asset_id: Option<Uuid>,
    pub related_channel_id: Option<Uuid>,
    pub related_booking_id: Option<Uuid>,
    pub related_service_task_id: Option<Uuid>,
    pub category: Option<String>,
    #[serde(default)]
    pub event_type: CalendarEventType,
    #[serde(default)]
    pub status: CalendarEventStatus,
}

impl CalendarEvent {
    pub fn new(title: String, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description: None,
            start,
            end,
            all_day: false,
            source: None,
            related_portfolio_id: None,
            related_group_id: None,
            related_asset_id: None,
            related_channel_id: None,
            related_booking_id: None,
            related_service_task_id: None,
            category: None,
            event_type: CalendarEventType::Meeting,
            status: CalendarEventStatus::Active,
        }
    }

    pub fn for_booking(
        title: String,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        portfolio_id: Option<Uuid>,
        asset_id: Uuid,
        channel_id: Option<Uuid>,
        booking_id: Uuid,
        source: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description: None,
            start,
            end,
            all_day: false,
            source: Some(source.to_string()),
            related_portfolio_id: portfolio_id,
            related_group_id: None,
            related_asset_id: Some(asset_id),
            related_channel_id: channel_id,
            related_booking_id: Some(booking_id),
            related_service_task_id: None,
            category: Some("Booking".to_string()),
            event_type: if channel_id.is_some() {
                CalendarEventType::ChannelBooking
            } else {
                CalendarEventType::AssetBooking
            },
            status: CalendarEventStatus::Active,
        }
    }

    pub fn for_service_task(
        title: String,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        portfolio_id: Option<Uuid>,
        asset_id: Uuid,
        task_id: Uuid,
        task_type: &ServiceTaskType,
    ) -> Self {
        let event_type = match task_type {
            ServiceTaskType::Cleaning => CalendarEventType::Cleaning,
            ServiceTaskType::Maintenance => CalendarEventType::Maintenance,
            ServiceTaskType::Inspection => CalendarEventType::Inspection,
            ServiceTaskType::Linen => CalendarEventType::Linen,
            ServiceTaskType::CheckIn => CalendarEventType::CheckIn,
            ServiceTaskType::CheckOut => CalendarEventType::CheckOut,
        };
        Self {
            id: Uuid::new_v4(),
            title,
            description: None,
            start,
            end,
            all_day: false,
            source: None,
            related_portfolio_id: portfolio_id,
            related_group_id: None,
            related_asset_id: Some(asset_id),
            related_channel_id: None,
            related_booking_id: None,
            related_service_task_id: Some(task_id),
            category: Some(task_type.display().to_string()),
            event_type,
            status: CalendarEventStatus::Active,
        }
    }

    pub fn update_from_booking(&mut self, booking: &Booking, source: &str) {
        self.title = format!("{} - {}", booking.guest_name, booking.channel_label());
        self.start = booking.start_datetime;
        self.end = booking.end_datetime;
        self.source = Some(source.to_string());
        self.status = match booking.status {
            BookingStatus::Cancelled => CalendarEventStatus::Cancelled,
            BookingStatus::Completed => CalendarEventStatus::Completed,
            _ => CalendarEventStatus::Active,
        };
        self.related_booking_id = Some(booking.id);
    }

    pub fn update_from_service_task(&mut self, task: &ServiceTask) {
        self.title = task.display_title();
        self.start = task.start_datetime;
        self.end = task.end_datetime;
        self.status = match task.status {
            ServiceTaskStatus::Cancelled => CalendarEventStatus::Cancelled,
            ServiceTaskStatus::Done => CalendarEventStatus::Completed,
            _ => CalendarEventStatus::Active,
        };
        self.related_service_task_id = Some(task.id);
    }
}
