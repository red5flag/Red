use crate::types::{
    short_name_token, short_uuid_suffix, AssetType, Currency, NotificationTrigger, NotificationType,
    SearchFilters, ViewMode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Portfolio - Top level container for asset groups
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: Uuid,
    #[serde(default)]
    pub code: String,
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
        let id = Uuid::new_v4();
        let code = format!("PORT-{}-{}", currency.country_code(), short_uuid_suffix(id, 4));
        Self {
            id,
            code,
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
    #[serde(default)]
    pub code: String,
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
        let id = Uuid::new_v4();
        let code = format!("GROUP-{}-{}", short_name_token(&name), short_uuid_suffix(id, 3));
        Self {
            id,
            code,
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

fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LifecycleStatus {
    #[default]
    Draft,
    Active,
    Retired,
    Disposed,
    Archived,
}

impl LifecycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LifecycleStatus::Draft => "Draft",
            LifecycleStatus::Active => "Active",
            LifecycleStatus::Retired => "Retired",
            LifecycleStatus::Disposed => "Disposed",
            LifecycleStatus::Archived => "Archived",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "active" => Some(LifecycleStatus::Active),
            "retired" => Some(LifecycleStatus::Retired),
            "disposed" => Some(LifecycleStatus::Disposed),
            "archived" => Some(LifecycleStatus::Archived),
            _ => Some(LifecycleStatus::Draft),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AvailabilityStatus {
    #[default]
    Available,
    Reserved,
    Booked,
    Rented,
    InUse,
    Unavailable,
}

impl AvailabilityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AvailabilityStatus::Available => "Available",
            AvailabilityStatus::Reserved => "Reserved",
            AvailabilityStatus::Booked => "Booked",
            AvailabilityStatus::Rented => "Rented",
            AvailabilityStatus::InUse => "In use",
            AvailabilityStatus::Unavailable => "Unavailable",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "available" => Some(AvailabilityStatus::Available),
            "reserved" => Some(AvailabilityStatus::Reserved),
            "booked" => Some(AvailabilityStatus::Booked),
            "rented" => Some(AvailabilityStatus::Rented),
            "in use" | "inuse" => Some(AvailabilityStatus::InUse),
            "unavailable" => Some(AvailabilityStatus::Unavailable),
            _ => Some(AvailabilityStatus::Available),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConditionStatus {
    #[default]
    New,
    Excellent,
    Good,
    Fair,
    Poor,
    Damaged,
    Unsafe,
}

impl ConditionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConditionStatus::New => "New",
            ConditionStatus::Excellent => "Excellent",
            ConditionStatus::Good => "Good",
            ConditionStatus::Fair => "Fair",
            ConditionStatus::Poor => "Poor",
            ConditionStatus::Damaged => "Damaged",
            ConditionStatus::Unsafe => "Unsafe",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "new" => Some(ConditionStatus::New),
            "excellent" => Some(ConditionStatus::Excellent),
            "good" => Some(ConditionStatus::Good),
            "fair" => Some(ConditionStatus::Fair),
            "poor" => Some(ConditionStatus::Poor),
            "damaged" => Some(ConditionStatus::Damaged),
            "unsafe" => Some(ConditionStatus::Unsafe),
            _ => Some(ConditionStatus::New),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CommercialStatus {
    #[default]
    NotOffered,
    InternalUseOnly,
    ListedForSale,
    ListedForRent,
    ListedForBooking,
    SalePending,
    RentalActive,
    Sold,
    Withdrawn,
}

impl CommercialStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommercialStatus::NotOffered => "Not offered",
            CommercialStatus::InternalUseOnly => "Internal use only",
            CommercialStatus::ListedForSale => "Listed for sale",
            CommercialStatus::ListedForRent => "Listed for rent",
            CommercialStatus::ListedForBooking => "Listed for booking",
            CommercialStatus::SalePending => "Sale pending",
            CommercialStatus::RentalActive => "Rental active",
            CommercialStatus::Sold => "Sold",
            CommercialStatus::Withdrawn => "Withdrawn",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "not offered" | "notoffered" => Some(CommercialStatus::NotOffered),
            "internal use only" | "internaluseonly" => Some(CommercialStatus::InternalUseOnly),
            "listed for sale" | "listedforsale" => Some(CommercialStatus::ListedForSale),
            "listed for rent" | "listedforrent" => Some(CommercialStatus::ListedForRent),
            "listed for booking" | "listedforbooking" => Some(CommercialStatus::ListedForBooking),
            "sale pending" | "salepending" => Some(CommercialStatus::SalePending),
            "rental active" | "rentalactive" => Some(CommercialStatus::RentalActive),
            "sold" => Some(CommercialStatus::Sold),
            "withdrawn" => Some(CommercialStatus::Withdrawn),
            _ => Some(CommercialStatus::NotOffered),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AssetHistoryEventType {
    #[default]
    Created,
    IdentityChange,
    ClassificationChange,
    StatusChange,
    OwnershipChange,
    CustodianChange,
    HierarchyChange,
    DateChange,
    Disposal,
    Restored,
}

impl AssetHistoryEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetHistoryEventType::Created => "Created",
            AssetHistoryEventType::IdentityChange => "Identity change",
            AssetHistoryEventType::ClassificationChange => "Classification change",
            AssetHistoryEventType::StatusChange => "Status change",
            AssetHistoryEventType::OwnershipChange => "Ownership change",
            AssetHistoryEventType::CustodianChange => "Custodian change",
            AssetHistoryEventType::HierarchyChange => "Hierarchy change",
            AssetHistoryEventType::DateChange => "Date change",
            AssetHistoryEventType::Disposal => "Disposal",
            AssetHistoryEventType::Restored => "Restored",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetHistoryEvent {
    pub id: Uuid,
    pub event_type: AssetHistoryEventType,
    pub field: Option<String>,
    pub previous_value: Option<String>,
    pub new_value: Option<String>,
    pub reason: Option<String>,
    pub user_id: Uuid,
    pub user_name: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl AssetHistoryEvent {
    pub fn new(
        user_id: Uuid,
        user_name: Option<String>,
        event_type: AssetHistoryEventType,
        field: Option<&str>,
        previous: Option<&str>,
        new: Option<&str>,
        reason: Option<&str>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            field: field.map(|s| s.to_string()),
            previous_value: previous.map(|s| s.to_string()),
            new_value: new.map(|s| s.to_string()),
            reason: reason.map(|s| s.to_string()),
            user_id,
            user_name,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AssetRelationshipType {
    #[default]
    LegalOwner,
    BeneficialOwner,
    ManagingOrganization,
    ResponsibleDepartment,
    AssetManager,
    CurrentCustodian,
    AssignedEmployee,
    AssignedTeam,
    CostCentre,
    Supplier,
    Manufacturer,
    MaintenanceProvider,
}

impl AssetRelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetRelationshipType::LegalOwner => "Legal owner",
            AssetRelationshipType::BeneficialOwner => "Beneficial owner",
            AssetRelationshipType::ManagingOrganization => "Managing organization",
            AssetRelationshipType::ResponsibleDepartment => "Responsible department",
            AssetRelationshipType::AssetManager => "Asset manager",
            AssetRelationshipType::CurrentCustodian => "Current custodian",
            AssetRelationshipType::AssignedEmployee => "Assigned employee",
            AssetRelationshipType::AssignedTeam => "Assigned team",
            AssetRelationshipType::CostCentre => "Cost centre",
            AssetRelationshipType::Supplier => "Supplier",
            AssetRelationshipType::Manufacturer => "Manufacturer",
            AssetRelationshipType::MaintenanceProvider => "Maintenance provider",
        }
    }

    pub fn label(&self) -> &'static str {
        self.as_str()
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "legal owner" | "legalowner" => Some(AssetRelationshipType::LegalOwner),
            "beneficial owner" | "beneficialowner" => Some(AssetRelationshipType::BeneficialOwner),
            "managing organization" | "managingorganization" => Some(AssetRelationshipType::ManagingOrganization),
            "responsible department" | "responsibledepartment" => Some(AssetRelationshipType::ResponsibleDepartment),
            "asset manager" | "assetmanager" => Some(AssetRelationshipType::AssetManager),
            "current custodian" | "currentcustodian" => Some(AssetRelationshipType::CurrentCustodian),
            "assigned employee" | "assignedemployee" => Some(AssetRelationshipType::AssignedEmployee),
            "assigned team" | "assignedteam" => Some(AssetRelationshipType::AssignedTeam),
            "cost centre" | "costcenter" | "costcentre" => Some(AssetRelationshipType::CostCentre),
            "supplier" => Some(AssetRelationshipType::Supplier),
            "manufacturer" => Some(AssetRelationshipType::Manufacturer),
            "maintenance provider" | "maintenanceprovider" => Some(AssetRelationshipType::MaintenanceProvider),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AssetRelationshipPartyType {
    #[default]
    Organization,
    Team,
    Member,
    Supplier,
    ServiceProvider,
    ExternalContact,
}

impl AssetRelationshipPartyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetRelationshipPartyType::Organization => "Organization",
            AssetRelationshipPartyType::Team => "Team",
            AssetRelationshipPartyType::Member => "Member",
            AssetRelationshipPartyType::Supplier => "Supplier",
            AssetRelationshipPartyType::ServiceProvider => "Service provider",
            AssetRelationshipPartyType::ExternalContact => "External contact",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "organization" => Some(AssetRelationshipPartyType::Organization),
            "team" => Some(AssetRelationshipPartyType::Team),
            "member" => Some(AssetRelationshipPartyType::Member),
            "supplier" => Some(AssetRelationshipPartyType::Supplier),
            "service provider" | "serviceprovider" => Some(AssetRelationshipPartyType::ServiceProvider),
            "external contact" | "externalcontact" => Some(AssetRelationshipPartyType::ExternalContact),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RelatedParty {
    pub party_type: AssetRelationshipPartyType,
    pub party_id: Option<Uuid>,
    pub name: String,
    pub contact: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetRelationship {
    pub id: Uuid,
    pub relationship_type: AssetRelationshipType,
    pub related_party: RelatedParty,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub primary_contact: Option<String>,
    pub notes: Option<String>,
    pub active: bool,
}

impl AssetRelationship {
    pub fn new(
        relationship_type: AssetRelationshipType,
        related_party: RelatedParty,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            relationship_type,
            related_party,
            start_date: Some(Utc::now()),
            end_date: None,
            primary_contact: None,
            notes: None,
            active: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CustomSpecField {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AssetClassification {
    pub serial_number: Option<String>,
    pub vin: Option<String>,
    pub sku: Option<String>,
    pub registration_number: Option<String>,
    pub barcode: Option<String>,
    pub title: Option<String>,
    pub lot: Option<String>,
    pub plan: Option<String>,
    pub property_reference: Option<String>,
    pub domain: Option<String>,
    pub licence: Option<String>,
    pub version: Option<String>,
    pub repository: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub model_year: Option<String>,
    pub build_date: Option<String>,
    pub technical_specifications: Option<String>,
    pub materials: Option<String>,
    pub dimensions: Option<String>,
    pub weight: Option<String>,
    pub capacity: Option<String>,
    pub colour: Option<String>,
    pub configuration: Option<String>,
    pub odometer: Option<String>,
    pub insurance: Option<String>,
    pub quantities: Option<String>,
    pub units: Option<String>,
    pub reorder_info: Option<String>,
    pub hosting_location: Option<String>,
    pub renewal_date: Option<String>,
    pub access_info: Option<String>,
    pub custom_fields: Vec<CustomSpecField>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AssetLifecycle {
    pub commissioning_date: Option<DateTime<Utc>>,
    pub warranty_start_date: Option<DateTime<Utc>>,
    pub warranty_expiry_date: Option<DateTime<Utc>>,
    pub expected_useful_life: Option<String>,
    pub expected_retirement_date: Option<DateTime<Utc>>,
    pub actual_retirement_date: Option<DateTime<Utc>>,
    pub disposal_date: Option<DateTime<Utc>>,
    pub disposal_method: Option<String>,
    pub disposal_reason: Option<String>,
    pub disposal_value: Option<f64>,
}

// Asset - Individual asset item
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    #[serde(default)]
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub reference_code: Option<String>,
    pub asset_type: AssetType,
    pub asset_subtype: Option<String>,
    pub location: Option<String>,
    pub organization_id: Option<Uuid>,
    pub portfolio_id: Option<Uuid>,
    pub asset_group_id: Option<Uuid>,
    pub parent_asset_id: Option<Uuid>,
    pub purchase_value: f64,
    pub current_value: f64,
    pub profit_loss: f64,
    pub profit_loss_percent: f64,
    pub revenue: f64,
    pub purchase_date: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    #[serde(default = "now_utc")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "now_utc")]
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub version: u32,
    pub images: Vec<String>,
    pub documents: Vec<Document>,
    pub tags: Vec<String>,
    pub lifecycle_status: LifecycleStatus,
    pub availability_status: AvailabilityStatus,
    pub condition_status: ConditionStatus,
    pub commercial_status: CommercialStatus,
    #[serde(default)]
    pub status_reason: Option<String>,
    #[serde(default)]
    pub classification: AssetClassification,
    #[serde(default)]
    pub lifecycle: AssetLifecycle,
    #[serde(default)]
    pub relationships: Vec<AssetRelationship>,
    #[serde(default)]
    pub history: Vec<AssetHistoryEvent>,
    pub assigned_workers: Vec<Uuid>,
    pub quick_sale_enabled: bool,
    pub notification_settings: Vec<AssetNotificationSetting>,
    pub calendar_events: Vec<crate::models::CalendarEvent>,
    pub channel_ids: Vec<Uuid>,
    pub metadata: serde_json::Value,
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
        let now = Utc::now();
        let id = Uuid::now_v7();
        let code = format!("AST-{}", short_uuid_suffix(id, 6));
        Self {
            id,
            code,
            name,
            description: None,
            reference_code: None,
            asset_type,
            asset_subtype: None,
            location: None,
            organization_id: None,
            portfolio_id: None,
            asset_group_id: None,
            parent_asset_id: None,
            purchase_value,
            current_value: purchase_value,
            profit_loss: 0.0,
            profit_loss_percent: 0.0,
            revenue: 0.0,
            purchase_date: now,
            last_accessed_at: now,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
            version: 1,
            images: Vec::new(),
            documents: Vec::new(),
            tags: Vec::new(),
            lifecycle_status: LifecycleStatus::Draft,
            availability_status: AvailabilityStatus::Available,
            condition_status: ConditionStatus::New,
            commercial_status: CommercialStatus::NotOffered,
            status_reason: None,
            classification: AssetClassification::default(),
            lifecycle: AssetLifecycle::default(),
            relationships: Vec::new(),
            history: Vec::new(),
            assigned_workers: Vec::new(),
            quick_sale_enabled: false,
            notification_settings: Vec::new(),
            calendar_events: Vec::new(),
            channel_ids: Vec::new(),
            metadata: serde_json::json!({}),
        }
    }

    pub fn initialize_with_creator(&mut self, user_id: Uuid, user_name: Option<String>) {
        self.created_by = Some(user_id);
        self.updated_by = Some(user_id);
        let event = AssetHistoryEvent::new(
            user_id,
            user_name,
            AssetHistoryEventType::Created,
            None,
            None,
            None,
            Some(&format!("Asset '{}' created", self.name)),
        );
        self.push_history(user_id, event);
    }

    fn push_history(&mut self, user_id: Uuid, event: AssetHistoryEvent) {
        self.history.push(event);
        self.updated_at = Utc::now();
        self.updated_by = Some(user_id);
        self.version += 1;
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

    pub fn class_label(&self) -> String {
        self.asset_type.to_input_string()
    }

    pub fn status_badges(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("Lifecycle", self.lifecycle_status.as_str()),
            ("Availability", self.availability_status.as_str()),
            ("Condition", self.condition_status.as_str()),
            ("Commercial", self.commercial_status.as_str()),
        ]
    }

    pub fn next_important_date(&self) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        let mut candidates: Vec<DateTime<Utc>> = Vec::new();
        if let Some(d) = self.lifecycle.commissioning_date {
            if d > now {
                candidates.push(d);
            }
        }
        if let Some(d) = self.lifecycle.warranty_expiry_date {
            if d > now {
                candidates.push(d);
            }
        }
        if let Some(d) = self.lifecycle.expected_retirement_date {
            if d > now {
                candidates.push(d);
            }
        }
        candidates.into_iter().min()
    }

    pub fn current_manager(&self) -> Option<&str> {
        self.relationships
            .iter()
            .find(|r| r.relationship_type == AssetRelationshipType::AssetManager && r.active)
            .and_then(|r| {
                let name = r.related_party.name.trim();
                if name.is_empty() {
                    None
                } else {
                    Some(name)
                }
            })
    }

    pub fn current_custodian(&self) -> Option<&str> {
        self.relationships
            .iter()
            .find(|r| {
                r.relationship_type == AssetRelationshipType::CurrentCustodian && r.active
            })
            .and_then(|r| {
                let name = r.related_party.name.trim();
                if name.is_empty() {
                    None
                } else {
                    Some(name)
                }
            })
    }

    pub fn record_identity_change(
        &mut self,
        user_id: Uuid,
        user_name: Option<String>,
        field: &str,
        previous: Option<&str>,
        new: Option<&str>,
        reason: Option<&str>,
    ) {
        let event = AssetHistoryEvent::new(
            user_id,
            user_name,
            AssetHistoryEventType::IdentityChange,
            Some(field),
            previous,
            new,
            reason,
        );
        self.push_history(user_id, event);
    }

    pub fn record_classification_change(
        &mut self,
        user_id: Uuid,
        user_name: Option<String>,
        field: &str,
        previous: Option<&str>,
        new: Option<&str>,
        reason: Option<&str>,
    ) {
        let event = AssetHistoryEvent::new(
            user_id,
            user_name,
            AssetHistoryEventType::ClassificationChange,
            Some(field),
            previous,
            new,
            reason,
        );
        self.push_history(user_id, event);
    }

    pub fn record_ownership_change(
        &mut self,
        user_id: Uuid,
        user_name: Option<String>,
        relationship_type: &str,
        previous: Option<&str>,
        new: Option<&str>,
        reason: Option<&str>,
    ) {
        let event = AssetHistoryEvent::new(
            user_id,
            user_name,
            AssetHistoryEventType::OwnershipChange,
            Some(relationship_type),
            previous,
            new,
            reason,
        );
        self.push_history(user_id, event);
    }

    pub fn record_custodian_change(
        &mut self,
        user_id: Uuid,
        user_name: Option<String>,
        previous: Option<&str>,
        new: Option<&str>,
        reason: Option<&str>,
    ) {
        let event = AssetHistoryEvent::new(
            user_id,
            user_name,
            AssetHistoryEventType::CustodianChange,
            Some("Current custodian"),
            previous,
            new,
            reason,
        );
        self.push_history(user_id, event);
    }

    pub fn record_hierarchy_change(
        &mut self,
        user_id: Uuid,
        user_name: Option<String>,
        previous: Option<&str>,
        new: Option<&str>,
        reason: Option<&str>,
    ) {
        let event = AssetHistoryEvent::new(
            user_id,
            user_name,
            AssetHistoryEventType::HierarchyChange,
            Some("Parent hierarchy"),
            previous,
            new,
            reason,
        );
        self.push_history(user_id, event);
    }

    pub fn set_lifecycle_status(
        &mut self,
        user_id: Uuid,
        user_name: Option<String>,
        new: LifecycleStatus,
        reason: Option<&str>,
    ) {
        if self.lifecycle_status != new {
            let previous = self.lifecycle_status.as_str();
            self.lifecycle_status = new.clone();
            let event = AssetHistoryEvent::new(
                user_id,
                user_name,
                AssetHistoryEventType::StatusChange,
                Some("Lifecycle status"),
                Some(previous),
                Some(self.lifecycle_status.as_str()),
                reason,
            );
            self.push_history(user_id, event);
        }
    }

    pub fn set_availability_status(
        &mut self,
        user_id: Uuid,
        user_name: Option<String>,
        new: AvailabilityStatus,
        reason: Option<&str>,
    ) {
        if self.availability_status != new {
            let previous = self.availability_status.as_str();
            self.availability_status = new.clone();
            let event = AssetHistoryEvent::new(
                user_id,
                user_name,
                AssetHistoryEventType::StatusChange,
                Some("Availability status"),
                Some(previous),
                Some(self.availability_status.as_str()),
                reason,
            );
            self.push_history(user_id, event);
        }
    }

    pub fn set_condition_status(
        &mut self,
        user_id: Uuid,
        user_name: Option<String>,
        new: ConditionStatus,
        reason: Option<&str>,
    ) {
        if self.condition_status != new {
            let previous = self.condition_status.as_str();
            self.condition_status = new.clone();
            let event = AssetHistoryEvent::new(
                user_id,
                user_name,
                AssetHistoryEventType::StatusChange,
                Some("Condition status"),
                Some(previous),
                Some(self.condition_status.as_str()),
                reason,
            );
            self.push_history(user_id, event);
        }
    }

    pub fn set_commercial_status(
        &mut self,
        user_id: Uuid,
        user_name: Option<String>,
        new: CommercialStatus,
        reason: Option<&str>,
    ) {
        if self.commercial_status != new {
            let previous = self.commercial_status.as_str();
            self.commercial_status = new.clone();
            let event = AssetHistoryEvent::new(
                user_id,
                user_name,
                AssetHistoryEventType::StatusChange,
                Some("Commercial status"),
                Some(previous),
                Some(self.commercial_status.as_str()),
                reason,
            );
            self.push_history(user_id, event);
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.trim().is_empty() {
            errors.push("Asset name is required".to_string());
        }
        if let Some(ref code) = self.reference_code {
            if code.trim().is_empty() {
                errors.push("Reference code cannot be blank".to_string());
            }
        }
        if let Some(ret) = self.lifecycle.actual_retirement_date {
            if ret < self.purchase_date {
                errors.push("Retirement date cannot precede purchase date".to_string());
            }
        }
        if let (Some(start), Some(end)) = (
            self.lifecycle.warranty_start_date,
            self.lifecycle.warranty_expiry_date,
        ) {
            if end < start {
                errors.push("Warranty expiry cannot precede warranty start".to_string());
            }
        }
        if self.lifecycle_status == LifecycleStatus::Disposed {
            if self.lifecycle.disposal_date.is_none() {
                errors.push(
                    "Disposal date is required when Lifecycle status is Disposed".to_string(),
                );
            }
            if self
                .lifecycle
                .disposal_method
                .as_ref()
                .map_or(true, |s| s.trim().is_empty())
            {
                errors.push(
                    "Disposal method is required when Lifecycle status is Disposed".to_string(),
                );
            }
        }
        if matches!(
            self.condition_status,
            ConditionStatus::Damaged | ConditionStatus::Unsafe
        ) || matches!(
            self.lifecycle_status,
            LifecycleStatus::Retired | LifecycleStatus::Disposed | LifecycleStatus::Archived
        ) || self.availability_status == AvailabilityStatus::Unavailable
        {
            if self
                .status_reason
                .as_ref()
                .map_or(true, |s| s.trim().is_empty())
            {
                errors.push(
                    "A reason is required when an asset is marked Damaged, Unsafe, Retired, Disposed, Archived, or Unavailable"
                        .to_string(),
                );
            }
        }
        errors
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
