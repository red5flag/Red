use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub source: Option<String>,
    pub related_portfolio_id: Option<Uuid>,
    pub related_group_id: Option<Uuid>,
    pub related_asset_id: Option<Uuid>,
    pub category: Option<String>,
}

impl CalendarEvent {
    pub fn new(title: String, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            start,
            end,
            source: None,
            related_portfolio_id: None,
            related_group_id: None,
            related_asset_id: None,
            category: None,
        }
    }
}
