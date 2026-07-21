use crate::types::short_uuid_suffix;
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookingSource {
    TestChannel,
    Manual,
}

impl Default for BookingSource {
    fn default() -> Self {
        BookingSource::Manual
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookingStatus {
    Pending,
    Confirmed,
    Changed,
    Cancelled,
    Completed,
}

impl Default for BookingStatus {
    fn default() -> Self {
        BookingStatus::Confirmed
    }
}

/// A reservation against an asset from a channel or manual entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Booking {
    pub id: Uuid,
    #[serde(default)]
    pub code: String,
    pub asset_id: Uuid,
    pub channel_id: Option<Uuid>,
    #[serde(default)]
    pub source: BookingSource,
    pub guest_name: String,
    pub start_datetime: DateTime<Utc>,
    pub end_datetime: DateTime<Utc>,
    pub nights: u32,
    pub cost_per_night: f64,
    pub cleaning_fee: f64,
    pub channel_fee: f64,
    pub tax: f64,
    pub total: f64,
    #[serde(default)]
    pub status: BookingStatus,
    pub external_reference: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_updated_source: Option<String>,
    #[serde(default)]
    pub calendar_event_ids: Vec<Uuid>,
    #[serde(default)]
    pub service_task_ids: Vec<Uuid>,
}

impl Booking {
    pub fn new(
        asset_id: Uuid,
        channel_id: Option<Uuid>,
        source: BookingSource,
        guest_name: String,
        start_datetime: DateTime<Utc>,
        end_datetime: DateTime<Utc>,
        cost_per_night: f64,
    ) -> Self {
        let now = Utc::now();
        let nights = (end_datetime.date_naive() - start_datetime.date_naive()).num_days() as u32;
        let id = Uuid::new_v4();
        let code = format!("BKG-{}-{}", start_datetime.year(), short_uuid_suffix(id, 6));
        let total = cost_per_night * nights.max(1) as f64;
        Self {
            id,
            code,
            asset_id,
            channel_id,
            guest_name,
            start_datetime,
            end_datetime,
            nights: nights.max(1),
            cost_per_night,
            cleaning_fee: 0.0,
            channel_fee: 0.0,
            tax: 0.0,
            total,
            status: BookingStatus::Confirmed,
            external_reference: None,
            notes: None,
            created_at: now,
            updated_at: now,
            last_updated_source: Some(match &source {
                BookingSource::TestChannel => "Test Channel".to_string(),
                BookingSource::Manual => "Manual".to_string(),
            }),
            calendar_event_ids: Vec::new(),
            service_task_ids: Vec::new(),
            source,
        }
    }

    pub fn recalculate_total(&mut self) {
        let nights =
            (self.end_datetime.date_naive() - self.start_datetime.date_naive()).num_days() as u32;
        self.nights = nights.max(1);
        self.total = (self.cost_per_night * self.nights as f64)
            + self.cleaning_fee
            + self.channel_fee
            + self.tax;
        self.updated_at = Utc::now();
    }

    pub fn overlaps(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> bool {
        self.start_datetime < end
            && self.end_datetime > start
            && !matches!(self.status, BookingStatus::Cancelled)
    }

    pub fn is_active(&self) -> bool {
        !matches!(
            self.status,
            BookingStatus::Cancelled | BookingStatus::Completed
        )
    }

    pub fn mark_changed(&mut self, source: &str) {
        self.status = BookingStatus::Changed;
        self.last_updated_source = Some(source.to_string());
        self.updated_at = Utc::now();
    }

    pub fn mark_cancelled(&mut self, source: &str) {
        self.status = BookingStatus::Cancelled;
        self.last_updated_source = Some(source.to_string());
        self.updated_at = Utc::now();
    }

    pub fn mark_completed(&mut self) {
        self.status = BookingStatus::Completed;
        self.updated_at = Utc::now();
    }

    pub fn channel_label(&self) -> String {
        match self.source {
            BookingSource::TestChannel => "Test Channel".to_string(),
            BookingSource::Manual => "Manual".to_string(),
        }
    }
}
