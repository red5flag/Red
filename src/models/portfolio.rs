use crate::types::{
    AssetType, Currency, NotificationTrigger, NotificationType, SearchFilters, ViewMode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Portfolio - Top level container for asset groups
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub assets: Vec<Asset>,
    pub asset_groups: Vec<AssetGroup>,
    pub currency: Currency,
    pub total_value: f64,
    pub purchase_value: f64,
    pub profit_loss: f64,
    pub profit_loss_percent: f64,
    pub revenue: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub status: PortfolioStatus,
    pub view_mode: ViewMode,
    pub documents: Vec<Document>,
    pub calendar_events: Vec<crate::models::CalendarEvent>,
    pub assigned_users: Vec<Uuid>,
    pub notification_settings: Vec<EntityNotificationSetting>,
    pub channel_ids: Vec<Uuid>,
    pub image_url: Option<String>,
    pub emoji: Option<String>,
    #[serde(default)]
    pub secondary_organization_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortfolioStatus {
    Active,
    Inactive,
    Archived,
    Sold,
}

impl Portfolio {
    pub fn new(name: String, owner_id: Uuid, currency: Currency) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            owner_id,
            organization_id: None,
            assets: Vec::new(),
            asset_groups: Vec::new(),
            currency,
            total_value: 0.0,
            purchase_value: 0.0,
            profit_loss: 0.0,
            profit_loss_percent: 0.0,
            revenue: 0.0,
            created_at: now,
            updated_at: now,
            last_accessed_at: now,
            tags: Vec::new(),
            status: PortfolioStatus::Active,
            view_mode: ViewMode::default(),
            documents: Vec::new(),
            calendar_events: Vec::new(),
            assigned_users: Vec::new(),
            notification_settings: Vec::new(),
            channel_ids: Vec::new(),
            image_url: None,
            emoji: None,
            secondary_organization_ids: Vec::new(),
        }
    }

    pub fn recalculate_values(&mut self) {
        self.total_value = 0.0;
        self.purchase_value = 0.0;
        self.revenue = 0.0;

        for asset in &self.assets {
            self.total_value += asset.current_value;
            self.purchase_value += asset.purchase_value;
            self.revenue += asset.revenue;
        }

        for group in &self.asset_groups {
            self.total_value += group.total_value;
            self.purchase_value += group.purchase_value;
            self.revenue += group.revenue;
        }

        self.profit_loss = self.total_value - self.purchase_value + self.revenue;
        if self.purchase_value > 0.0 {
            self.profit_loss_percent = (self.profit_loss / self.purchase_value) * 100.0;
        }
    }

    pub fn is_visible_to(&self, user_id: Uuid, can_view_all: bool) -> bool {
        can_view_all || self.owner_id == user_id || self.assigned_users.contains(&user_id)
    }

    pub fn get_all_assets(&self) -> Vec<&Asset> {
        self.assets
            .iter()
            .chain(self.asset_groups.iter().flat_map(|g| g.assets.iter()))
            .collect()
    }

    pub fn search(&self, query: &str, filters: &SearchFilters) -> Vec<&Asset> {
        self.get_all_assets()
            .into_iter()
            .filter(|asset| {
                let matches_query = query.is_empty()
                    || asset.name.to_lowercase().contains(&query.to_lowercase())
                    || asset
                        .description
                        .as_ref()
                        .map_or(false, |d| d.to_lowercase().contains(&query.to_lowercase()))
                    || asset
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query.to_lowercase()));

                let matches_type = filters.asset_types.is_empty()
                    || filters.asset_types.contains(&asset.asset_type);

                let matches_value = filters.value_range.map_or(true, |(min, max)| {
                    asset.current_value >= min && asset.current_value <= max
                });

                matches_query && matches_type && matches_value
            })
            .collect()
    }
}

// Asset Group - Groups related assets within a portfolio
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetGroup {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub organization_id: Option<Uuid>,
    pub assets: Vec<Asset>,
    pub total_value: f64,
    pub purchase_value: f64,
    pub profit_loss: f64,
    pub profit_loss_percent: f64,
    pub revenue: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub documents: Vec<Document>,
    pub calendar_events: Vec<crate::models::CalendarEvent>,
    pub assigned_users: Vec<Uuid>,
    pub notification_settings: Vec<EntityNotificationSetting>,
    pub channel_ids: Vec<Uuid>,
    pub image_url: Option<String>,
    pub emoji: Option<String>,
}

impl AssetGroup {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            organization_id: None,
            assets: Vec::new(),
            total_value: 0.0,
            purchase_value: 0.0,
            profit_loss: 0.0,
            profit_loss_percent: 0.0,
            revenue: 0.0,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            documents: Vec::new(),
            calendar_events: Vec::new(),
            assigned_users: Vec::new(),
            notification_settings: Vec::new(),
            channel_ids: Vec::new(),
            image_url: None,
            emoji: None,
        }
    }

    pub fn recalculate_values(&mut self) {
        self.total_value = 0.0;
        self.purchase_value = 0.0;
        self.revenue = 0.0;

        for asset in &self.assets {
            self.total_value += asset.current_value;
            self.purchase_value += asset.purchase_value;
            self.revenue += asset.revenue;
        }

        self.profit_loss = self.total_value - self.purchase_value + self.revenue;
        if self.purchase_value > 0.0 {
            self.profit_loss_percent = (self.profit_loss / self.purchase_value) * 100.0;
        }
    }

    pub fn is_visible_to(&self, user_id: Uuid, can_view_all: bool) -> bool {
        can_view_all || self.assigned_users.contains(&user_id)
    }
}

// Asset - Individual asset item
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub asset_type: AssetType,
    pub location: Option<String>,
    pub organization_id: Option<Uuid>,
    pub purchase_value: f64,
    pub current_value: f64,
    pub profit_loss: f64,
    pub profit_loss_percent: f64,
    pub revenue: f64,
    pub purchase_date: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub images: Vec<String>,
    pub documents: Vec<Document>,
    pub tags: Vec<String>,
    pub status: AssetStatus,
    pub metadata: serde_json::Value, // Flexible metadata storage
    pub assigned_workers: Vec<Uuid>,
    pub quick_sale_enabled: bool,
    pub notification_settings: Vec<AssetNotificationSetting>,
    pub calendar_events: Vec<crate::models::CalendarEvent>,
    pub channel_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetStatus {
    Active,
    Inactive,
    ForSale,
    Sold,
    Rented,
    Maintenance,
    Archived,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetNotificationSetting {
    pub trigger: crate::types::NotificationTrigger,
    pub enabled: bool,
    pub recipients: Vec<Uuid>,
}

/// Unified notification setting for portfolios, groups, and assets.
/// Supports configuring triggers, delivery types, recipients (users + roles),
/// and optional conditions — gated by the acting user's permissions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EntityNotificationSetting {
    pub id: Uuid,
    pub trigger: NotificationTrigger,
    pub notification_types: Vec<NotificationType>,
    pub enabled: bool,
    /// User IDs to also notify (beyond self). Requires ManageUsers permission.
    pub recipients: Vec<Uuid>,
    /// Roles to notify. Requires ManageRoles permission.
    pub recipient_roles: Vec<crate::types::UserRole>,
    /// Optional condition description (e.g. "Only PDF documents", "Value > $10k").
    pub condition: Option<String>,
}

impl EntityNotificationSetting {
    pub fn new(trigger: NotificationTrigger) -> Self {
        Self {
            id: Uuid::new_v4(),
            trigger,
            notification_types: vec![NotificationType::InApp],
            enabled: true,
            recipients: Vec::new(),
            recipient_roles: Vec::new(),
            condition: None,
        }
    }
}

impl Asset {
    pub fn is_visible_to(&self, user_id: Uuid, can_view_all: bool) -> bool {
        can_view_all || self.assigned_workers.contains(&user_id)
    }

    pub fn new(name: String, asset_type: AssetType, purchase_value: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            asset_type,
            location: None,
            organization_id: None,
            purchase_value,
            current_value: purchase_value,
            profit_loss: 0.0,
            profit_loss_percent: 0.0,
            revenue: 0.0,
            purchase_date: Utc::now(),
            last_accessed_at: Utc::now(),
            images: Vec::new(),
            documents: Vec::new(),
            tags: Vec::new(),
            status: AssetStatus::Active,
            metadata: serde_json::json!({}),
            assigned_workers: Vec::new(),
            quick_sale_enabled: false,
            notification_settings: Vec::new(),
            calendar_events: Vec::new(),
            channel_ids: Vec::new(),
        }
    }

    pub fn update_value(&mut self, new_value: f64) {
        self.current_value = new_value;
        self.recalculate_profit_loss();
    }

    pub fn recalculate_profit_loss(&mut self) {
        self.profit_loss = self.current_value - self.purchase_value + self.revenue;
        if self.purchase_value > 0.0 {
            self.profit_loss_percent = (self.profit_loss / self.purchase_value) * 100.0;
        }
    }

    pub fn add_revenue(&mut self, amount: f64) {
        self.revenue += amount;
        self.recalculate_profit_loss();
    }
}

// Document - Attachments for portfolios, groups, and assets
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub name: String,
    pub file_type: String,
    pub url: String,
    pub uploaded_at: DateTime<Utc>,
    pub uploaded_by: Uuid,
    pub content: Option<String>,
}

// Trending data for overview
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrendingData {
    pub portfolio_id: Uuid,
    pub portfolio_name: String,
    pub change_percent: f64,
    pub volume: f64,
    pub trend: TrendDirection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

// Recent change for overview
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecentChange {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub change_type: ChangeType,
    pub entity_name: String,
    pub entity_type: String,
    pub value_change: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    ValueUpdated,
    StatusChanged,
    DocumentAdded,
}
