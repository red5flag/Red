use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelType {
    Test,
    Airbnb,
    BookingCom,
    Expedia,
    Vrbo,
    LinkedIn,
    Other(String),
}

impl Default for ChannelType {
    fn default() -> Self {
        ChannelType::Test
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Disconnected,
    Connected,
    Error,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        ConnectionStatus::Disconnected
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncDirection {
    ImportOnly,
    ExportOnly,
    TwoWay,
}

impl Default for SyncDirection {
    fn default() -> Self {
        SyncDirection::ImportOnly
    }
}

/// A channel connecting a real-world or local listing source to an asset.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Channel {
    pub id: Uuid,
    pub name: String,
    pub channel_type: ChannelType,
    pub linked_asset_id: Option<Uuid>,
    pub portfolio_id: Option<Uuid>,
    #[serde(default)]
    pub connection_status: ConnectionStatus,
    #[serde(default)]
    pub sync_direction: SyncDirection,
    pub nightly_rate_override: Option<f64>,
    pub minimum_nights: Option<u32>,
    pub maximum_nights: Option<u32>,
    pub commission_percent: Option<f64>,
    #[serde(default)]
    pub currency: crate::types::Currency,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_status: Option<String>,
    #[serde(default)]
    pub sync_errors: Vec<String>,
    #[serde(default)]
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Channel {
    pub fn new_test_channel(
        name: String,
        asset_id: Option<Uuid>,
        portfolio_id: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            channel_type: ChannelType::Test,
            linked_asset_id: asset_id,
            portfolio_id,
            connection_status: ConnectionStatus::Disconnected,
            sync_direction: SyncDirection::ImportOnly,
            nightly_rate_override: None,
            minimum_nights: Some(1),
            maximum_nights: None,
            commission_percent: None,
            currency: crate::types::Currency::default(),
            last_sync_at: None,
            last_sync_status: None,
            sync_errors: Vec::new(),
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_test(&self) -> bool {
        matches!(self.channel_type, ChannelType::Test)
    }

    /// Local test channel connect/disconnect toggles status without external API.
    pub fn connect(&mut self) {
        self.connection_status = ConnectionStatus::Connected;
        self.last_sync_status = Some("Connected (local test)".to_string());
        self.sync_errors.clear();
        self.updated_at = Utc::now();
    }

    pub fn disconnect(&mut self) {
        self.connection_status = ConnectionStatus::Disconnected;
        self.last_sync_status = Some("Disconnected (local test)".to_string());
        self.updated_at = Utc::now();
    }

    pub fn check_connection(&mut self) -> bool {
        if self.enabled {
            self.connection_status = ConnectionStatus::Connected;
            self.last_sync_at = Some(Utc::now());
            self.last_sync_status = Some("Connection check passed (local test)".to_string());
            self.sync_errors.clear();
        } else {
            self.connection_status = ConnectionStatus::Error;
            self.last_sync_status = Some("Connection check failed: channel disabled".to_string());
        }
        self.updated_at = Utc::now();
        matches!(self.connection_status, ConnectionStatus::Connected)
    }

    pub fn record_sync_error(&mut self, error: String) {
        self.sync_errors.push(error);
        self.connection_status = ConnectionStatus::Error;
        self.updated_at = Utc::now();
    }

    pub fn clear_sync_errors(&mut self) {
        self.sync_errors.clear();
        self.updated_at = Utc::now();
    }
}
