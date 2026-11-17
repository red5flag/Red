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
    EBay,
    Gumtree,
    FacebookMarketplace,
    Amazon,
    Etsy,
    Kogan,
    Other(String),
}

impl ChannelType {
    pub fn label(&self) -> &str {
        match self {
            ChannelType::Test => "Test Channel",
            ChannelType::Airbnb => "Airbnb",
            ChannelType::BookingCom => "Booking.com",
            ChannelType::Expedia => "Expedia",
            ChannelType::Vrbo => "Vrbo",
            ChannelType::LinkedIn => "LinkedIn",
            ChannelType::EBay => "eBay",
            ChannelType::Gumtree => "Gumtree",
            ChannelType::FacebookMarketplace => "Facebook Marketplace",
            ChannelType::Amazon => "Amazon",
            ChannelType::Etsy => "Etsy",
            ChannelType::Kogan => "Kogan",
            ChannelType::Other(s) => s.as_str(),
        }
    }
}

/// An eBay marketplace (site) a listing can be published to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EbaySite {
    /// eBay `X-EBAY-C-MARKETPLACE-ID` value.
    pub id: &'static str,
    pub label: &'static str,
    pub domain: &'static str,
    pub currency_code: &'static str,
}

impl EbaySite {
    pub fn currency(&self) -> crate::types::Currency {
        use crate::types::Currency;
        match self.currency_code {
            "USD" => Currency::USD,
            "EUR" => Currency::EUR,
            "GBP" => Currency::GBP,
            "CAD" => Currency::CAD,
            "AUD" => Currency::AUD,
            other => Currency::Custom(other.to_string()),
        }
    }
}

pub static EBAY_SITES: &[EbaySite] = &[
    EbaySite { id: "EBAY_AU", label: "Australia", domain: "ebay.com.au", currency_code: "AUD" },
    EbaySite { id: "EBAY_US", label: "United States", domain: "ebay.com", currency_code: "USD" },
    EbaySite { id: "EBAY_GB", label: "United Kingdom", domain: "ebay.co.uk", currency_code: "GBP" },
    EbaySite { id: "EBAY_CA", label: "Canada", domain: "ebay.ca", currency_code: "CAD" },
    EbaySite { id: "EBAY_DE", label: "Germany", domain: "ebay.de", currency_code: "EUR" },
    EbaySite { id: "EBAY_FR", label: "France", domain: "ebay.fr", currency_code: "EUR" },
    EbaySite { id: "EBAY_IT", label: "Italy", domain: "ebay.it", currency_code: "EUR" },
    EbaySite { id: "EBAY_ES", label: "Spain", domain: "ebay.es", currency_code: "EUR" },
    EbaySite { id: "EBAY_IE", label: "Ireland", domain: "ebay.ie", currency_code: "EUR" },
    EbaySite { id: "EBAY_AT", label: "Austria", domain: "ebay.at", currency_code: "EUR" },
    EbaySite { id: "EBAY_CH", label: "Switzerland", domain: "ebay.ch", currency_code: "CHF" },
    EbaySite { id: "EBAY_PL", label: "Poland", domain: "ebay.pl", currency_code: "PLN" },
];

pub fn ebay_site(id: &str) -> Option<&'static EbaySite> {
    EBAY_SITES.iter().find(|s| s.id == id)
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
    /// eBay marketplace id (e.g. "EBAY_AU") this channel publishes to.
    #[serde(default)]
    pub marketplace_site: Option<String>,
    /// Account the channel is connected through (shown in the Marketplace tab).
    #[serde(default)]
    pub account_label: Option<String>,
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
            marketplace_site: None,
            account_label: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// eBay site channel: account-level (no linked asset), managed from
    /// the Marketplace tab and selectable when publishing an asset.
    pub fn new_ebay_site_channel(site: &EbaySite, account_label: Option<String>) -> Self {
        let mut c = Self::new_test_channel(format!("eBay {}", site.label), None, None);
        c.channel_type = ChannelType::EBay;
        c.sync_direction = SyncDirection::ExportOnly;
        c.minimum_nights = None;
        c.currency = site.currency();
        c.marketplace_site = Some(site.id.to_string());
        c.account_label = account_label;
        c
    }

    /// Account-level channels are not tied to a single asset.
    pub fn is_account_level(&self) -> bool {
        self.linked_asset_id.is_none()
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
