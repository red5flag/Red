use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceTaskType {
    #[default]
    Cleaning,
    Maintenance,
    Inspection,
    Linen,
    CheckIn,
    CheckOut,
}

impl ServiceTaskType {
    pub fn display(&self) -> &'static str {
        match self {
            ServiceTaskType::Cleaning => "Cleaning",
            ServiceTaskType::Maintenance => "Maintenance",
            ServiceTaskType::Inspection => "Inspection",
            ServiceTaskType::Linen => "Linen",
            ServiceTaskType::CheckIn => "Check-in",
            ServiceTaskType::CheckOut => "Check-out",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceTaskStatus {
    Pending,
    Scheduled,
    InProgress,
    Done,
    Cancelled,
}

impl Default for ServiceTaskStatus {
    fn default() -> Self {
        ServiceTaskStatus::Scheduled
    }
}

/// Cleaner/maintenance/check-in work linked to an asset or booking.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServiceTask {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub booking_id: Option<Uuid>,
    #[serde(default)]
    pub task_type: ServiceTaskType,
    pub assigned_to: Option<Uuid>,
    pub start_datetime: DateTime<Utc>,
    pub end_datetime: DateTime<Utc>,
    #[serde(default)]
    pub status: ServiceTaskStatus,
    pub notes: Option<String>,
    pub calendar_event_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ServiceTask {
    pub fn new(
        asset_id: Uuid,
        booking_id: Option<Uuid>,
        task_type: ServiceTaskType,
        start_datetime: DateTime<Utc>,
        end_datetime: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            asset_id,
            booking_id,
            task_type,
            assigned_to: None,
            start_datetime,
            end_datetime,
            status: ServiceTaskStatus::Scheduled,
            notes: None,
            calendar_event_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn display_title(&self) -> String {
        format!("{} task", self.task_type.display())
    }

    pub fn mark_done(&mut self) {
        self.status = ServiceTaskStatus::Done;
        self.updated_at = Utc::now();
    }

    pub fn mark_cancelled(&mut self) {
        self.status = ServiceTaskStatus::Cancelled;
        self.updated_at = Utc::now();
    }

    pub fn assign(&mut self, user_id: Option<Uuid>) {
        self.assigned_to = user_id;
        self.updated_at = Utc::now();
    }
}
