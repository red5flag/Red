use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// User roles for organization hierarchy: Owner > Director > SeniorManager > Manager > Worker > DocumentWorker > Contractor > Guest
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Owner,
    Director,
    SeniorManager,
    Manager,
    Worker,
    DocumentWorker,
    Contractor,
    Guest,
}

impl UserRole {
    pub fn level(&self) -> u8 {
        match self {
            UserRole::Owner => 6,
            UserRole::Director => 5,
            UserRole::SeniorManager => 4,
            UserRole::Manager => 3,
            UserRole::Worker => 2,
            UserRole::DocumentWorker => 1,
            UserRole::Contractor => 1,
            UserRole::Guest => 0,
        }
    }

    pub fn can_manage(&self, other: &UserRole) -> bool {
        self.level() > other.level()
    }

    pub fn can_equal_or_manage(&self, other: &UserRole) -> bool {
        self.level() >= other.level()
    }
}

// Currency types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CAD,
    AUD,
    CNY,
    Custom(String),
}

impl Default for Currency {
    fn default() -> Self {
        Currency::USD
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::USD => write!(f, "USD"),
            Currency::EUR => write!(f, "EUR"),
            Currency::GBP => write!(f, "GBP"),
            Currency::JPY => write!(f, "JPY"),
            Currency::CAD => write!(f, "CAD"),
            Currency::AUD => write!(f, "AUD"),
            Currency::CNY => write!(f, "CNY"),
            Currency::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl Currency {
    pub fn country_code(&self) -> String {
        match self {
            Currency::USD => "US".to_string(),
            Currency::EUR => "EU".to_string(),
            Currency::GBP => "GB".to_string(),
            Currency::JPY => "JP".to_string(),
            Currency::CAD => "CA".to_string(),
            Currency::AUD => "AU".to_string(),
            Currency::CNY => "CN".to_string(),
            Currency::Custom(s) => s.chars().take(2).collect::<String>().to_uppercase(),
        }
    }
}

/// Generate a fixed-width numeric suffix from a UUID for human-readable codes.
pub fn short_uuid_suffix(id: Uuid, width: usize) -> String {
    let divisor = 10_u128.pow(width as u32);
    let n = (id.as_u128() % divisor) as u64;
    format!("{:0width$}", n, width = width)
}

/// Convert a name into an uppercase, hyphen-separated slug for codes.
pub fn name_code_slug(name: &str) -> String {
    name.to_uppercase().replace(' ', "-")
}

/// Short uppercase token taken from the first word of a name (up to 5 chars).
pub fn short_name_token(name: &str) -> String {
    name.split_whitespace()
        .next()
        .unwrap_or("")
        .to_uppercase()
        .chars()
        .take(5)
        .collect()
}

// Payment intervals
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentInterval {
    Hourly,
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
    Annually,
    OneTime,
    Custom(String),
}

impl Default for PaymentInterval {
    fn default() -> Self {
        PaymentInterval::Monthly
    }
}

// Payment methods
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentMethod {
    BankTransfer,
    CreditCard,
    DebitCard,
    DirectDeposit,
    PayPal,
    Crypto,
    Cash,
    Check,
    Custom(String),
}

// Notification types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationType {
    Push,
    Email,
    Sms,
    InApp,
}

// Notification triggers
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NotificationTrigger {
    PriceChange { percentage: f64 },
    Sale,
    Auction,
    Rent,
    Unrent,
    NoSales { days: u32 },
    Custom(String),
}

// Asset types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    RealEstate,
    Vehicle,
    Equipment,
    Stock,
    Bond,
    Commodity,
    Digital,
    IntellectualProperty,
    Channel,
    Custom(String),
}

impl AssetType {
    /// Display the asset type as it should appear in the typeahead input.
    /// Known variants use their debug name; custom variants use the user text.
    pub fn to_input_string(&self) -> String {
        match self {
            AssetType::Custom(name) => name.clone(),
            other => format!("{:?}", other),
        }
    }

    /// Parse a typed-in or selected asset type string.
    /// Known variants are recognised case/space-insensitively; anything else
    /// becomes a custom type.
    pub fn from_input(s: &str) -> Self {
        let normalized = s.trim().replace(' ', "").to_lowercase();
        match normalized.as_str() {
            "realestate" => AssetType::RealEstate,
            "vehicle" => AssetType::Vehicle,
            "equipment" => AssetType::Equipment,
            "stock" => AssetType::Stock,
            "bond" => AssetType::Bond,
            "commodity" => AssetType::Commodity,
            "digital" => AssetType::Digital,
            "intellectualproperty" => AssetType::IntellectualProperty,
            "channel" => AssetType::Channel,
            _ => AssetType::Custom(s.trim().to_string()),
        }
    }

    /// Known asset type labels (matches from_input/to_input_string for non-custom variants).
    pub fn all_labels() -> Vec<&'static str> {
        vec![
            "RealEstate",
            "Vehicle",
            "Equipment",
            "Stock",
            "Bond",
            "Commodity",
            "Digital",
            "IntellectualProperty",
            "Channel",
        ]
    }

    /// Common subtype/build options for this asset type.
    pub fn common_subtypes(&self) -> Vec<&'static str> {
        match self {
            AssetType::RealEstate => vec![
                "House",
                "Apartment",
                "Unit",
                "Townhouse",
                "Land",
                "Commercial",
                "Industrial",
                "Retail",
            ],
            AssetType::Vehicle => vec![
                "Car",
                "Truck",
                "Van",
                "Motorcycle",
                "Boat",
                "Trailer",
                "Bus",
                "RV",
            ],
            AssetType::Equipment => vec![
                "Machinery",
                "Tools",
                "Electronics",
                "Furniture",
                "Appliances",
                "IT Hardware",
            ],
            AssetType::Stock => vec!["Shares", "Bonds", "ETF", "Options", "Futures"],
            AssetType::Bond => vec!["Government", "Corporate", "Municipal", "Junk"],
            AssetType::Commodity => vec!["Agricultural", "Metals", "Energy", "Livestock"],
            AssetType::Digital => vec!["Cryptocurrency", "Software", "Domain", "NFT", "License"],
            AssetType::IntellectualProperty => {
                vec!["Patent", "Trademark", "Copyright", "Trade Secret"]
            }
            AssetType::Channel => vec![
                "Airbnb",
                "BookingCom",
                "Expedia",
                "Vrbo",
                "Direct",
                "Website",
            ],
            AssetType::Custom(_) => vec!["Custom"],
        }
    }
}

// Transaction types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Purchase,
    Sale,
    Rent,
    Lease,
    Payout,
    Dividend,
    Fee,
    Tax,
    Transfer,
    Adjustment,
}

// View modes for display
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    List,
    Grid,
    Chart,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::List
    }
}

// View-count page size for paginated lists/grids
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewCount {
    V1,
    V10,
    V20,
    V50,
    V100,
    Custom(usize),
}

impl ViewCount {
    pub fn all() -> [ViewCount; 5] {
        [
            ViewCount::V1,
            ViewCount::V10,
            ViewCount::V20,
            ViewCount::V50,
            ViewCount::V100,
        ]
    }

    pub fn as_usize(self) -> usize {
        match self {
            ViewCount::V1 => 1,
            ViewCount::V10 => 10,
            ViewCount::V20 => 20,
            ViewCount::V50 => 50,
            ViewCount::V100 => 100,
            ViewCount::Custom(n) => n,
        }
    }

    pub fn label(self) -> String {
        format!("{}", self.as_usize())
    }
}

impl Default for ViewCount {
    fn default() -> Self {
        ViewCount::V10
    }
}

// Sort modes for overview dashboard sections
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OverviewSortMode {
    #[default]
    Selected,
    Recent,
    Trending,
}

// Sort modes for lists
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SortMode {
    #[default]
    Recent,
    Oldest,
    HighestValue,
    LowestValue,
    HighestProfit,
    LowestProfit,
    HighestRevenue,
    LowestRevenue,
    ByOrganization,
}

// Sort modes for reporting views
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReportSortMode {
    #[default]
    Recent,
    Oldest,
    HighestValue,
    LowestValue,
    ByStatus,
    ByName,
    ByDocumentType,
    ByCalendarDate,
    // Document category sort
    BySales,
    ByPurchases,
    ByBills,
    ByInvoices,
    ByNotices,
    ByStatements,
    BySummaries,
    ByCompliance,
    // Parent entity sort
    ByOrganization,
    ByPortfolio,
    ByAssetGroup,
    ByDirectAsset,
    ByRole,
    ByUser,
}

// Search filters
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    pub asset_types: Vec<AssetType>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub value_range: Option<(f64, f64)>,
    pub tags: Vec<String>,
    pub status: Option<String>,
    pub owner: Option<Uuid>,
    pub portfolio: Option<Uuid>,
}

// Action types for undo/redo system
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Create,
    Update,
    Delete,
    View,
    Navigate,
    Setting,
    Payment,
    Notification,
    Search,
    Undo,
    Redo,
    Login,
    Logout,
}

// Severity classification for audit/history actions
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ChangeSeverity {
    Major,
    #[default]
    Minor,
    System,
}

impl ChangeSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeSeverity::Major => "major",
            ChangeSeverity::Minor => "minor",
            ChangeSeverity::System => "system",
        }
    }

    /// Map an action type to its default severity.
    /// Major actions create, delete, or have significant side effects.
    /// Minor actions are reads, updates, or low-impact settings.
    /// System actions are auth, undo, redo, and other internal bookkeeping.
    pub fn from_action_type(action_type: &ActionType) -> Self {
        match action_type {
            ActionType::Create | ActionType::Delete => ChangeSeverity::Major,
            ActionType::Update
            | ActionType::View
            | ActionType::Navigate
            | ActionType::Setting
            | ActionType::Search => ChangeSeverity::Minor,
            ActionType::Payment | ActionType::Notification => ChangeSeverity::Minor,
            ActionType::Undo | ActionType::Redo | ActionType::Login | ActionType::Logout => {
                ChangeSeverity::System
            }
        }
    }
}

// Viewport / navigation context captured with an action for audit and replay
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ViewportContext {
    pub page: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub scroll_position: Option<f64>,
    pub tab: Option<String>,
}

impl ViewportContext {
    pub fn new(page: impl Into<String>, entity_type: impl Into<String>) -> Self {
        Self {
            page: page.into(),
            entity_type: entity_type.into(),
            entity_id: None,
            scroll_position: None,
            tab: None,
        }
    }

    pub fn with_entity_id(mut self, entity_id: Uuid) -> Self {
        self.entity_id = Some(entity_id);
        self
    }

    pub fn with_scroll_position(mut self, scroll_position: f64) -> Self {
        self.scroll_position = Some(scroll_position);
        self
    }

    pub fn with_tab(mut self, tab: impl Into<String>) -> Self {
        self.tab = Some(tab.into());
        self
    }
}

// Theme/accessibility options
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    HighContrast,
    // Major colorblindness types
    Deuteranopia, // Green-blind
    Protanopia,   // Red-blind
    Tritanopia,   // Blue-blind
    // Anomalous trichromacy (less severe / common forms)
    Deuteranomaly, // Green-weak (most common)
    Protanomaly,   // Red-weak
    Tritanomaly,   // Blue-weak
    // Dichromacy / monochromacy
    Achromatopsia, // Total colour blindness (grayscale)
    Achromatomaly, // Reduced colour with blue-yellow weakness
    // Low vision / legally blind support
    LowVision, // High contrast + larger UI elements
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Light
    }
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::HighContrast => "high-contrast",
            Theme::Deuteranopia => "deuteranopia",
            Theme::Protanopia => "protanopia",
            Theme::Tritanopia => "tritanopia",
            Theme::Deuteranomaly => "deuteranomaly",
            Theme::Protanomaly => "protanomaly",
            Theme::Tritanomaly => "tritanomaly",
            Theme::Achromatopsia => "achromatopsia",
            Theme::Achromatomaly => "achromatomaly",
            Theme::LowVision => "low-vision",
        }
    }
}

// Visual edge style for controls and cards
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeStyle {
    Square,
    Rounded,
    Pill,
}

impl Default for EdgeStyle {
    fn default() -> Self {
        EdgeStyle::Square
    }
}

impl EdgeStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeStyle::Square => "square",
            EdgeStyle::Rounded => "rounded",
            EdgeStyle::Pill => "pill",
        }
    }
}

// Button fill style for the UI surface
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonStyle {
    Filled,
    Outline,
    Ghost,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        ButtonStyle::Filled
    }
}

impl ButtonStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            ButtonStyle::Filled => "filled",
            ButtonStyle::Outline => "outline",
            ButtonStyle::Ghost => "ghost",
        }
    }
}

// UI density / spacing scale
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Density {
    Compact,
    Comfortable,
    Spacious,
}

impl Default for Density {
    fn default() -> Self {
        Density::Comfortable
    }
}

impl Density {
    pub fn as_str(&self) -> &'static str {
        match self {
            Density::Compact => "compact",
            Density::Comfortable => "comfortable",
            Density::Spacious => "spacious",
        }
    }
}

// Named preset that bundles theme, edges, buttons, density, and accent colour.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsPreset {
    pub name: String,
    pub theme: Theme,
    pub edge_style: EdgeStyle,
    pub button_style: ButtonStyle,
    pub density: Density,
    pub accent_color: String,
}

impl SettingsPreset {
    pub fn new(
        name: impl Into<String>,
        theme: Theme,
        edge_style: EdgeStyle,
        button_style: ButtonStyle,
        density: Density,
        accent_color: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            theme,
            edge_style,
            button_style,
            density,
            accent_color: accent_color.into(),
        }
    }
}

/// Built-in display presets.
pub fn default_presets() -> Vec<SettingsPreset> {
    vec![
        SettingsPreset::new(
            "Classic",
            Theme::Light,
            EdgeStyle::Square,
            ButtonStyle::Filled,
            Density::Comfortable,
            "#3b82f6",
        ),
        SettingsPreset::new(
            "Compact",
            Theme::Dark,
            EdgeStyle::Square,
            ButtonStyle::Outline,
            Density::Compact,
            "#10b981",
        ),
        SettingsPreset::new(
            "Accessible",
            Theme::HighContrast,
            EdgeStyle::Rounded,
            ButtonStyle::Filled,
            Density::Spacious,
            "#f59e0b",
        ),
        SettingsPreset::new(
            "Minimal",
            Theme::Light,
            EdgeStyle::Pill,
            ButtonStyle::Ghost,
            Density::Comfortable,
            "#6366f1",
        ),
    ]
}

// Tab types
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabType {
    Overview,
    Portfolios,
    Networking,
    NetworkingAddMember,
    Organization,
    Reporting,
    Calendar,
    Transactions,
    History,
    Settings,
    Agent,
}

impl TabType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TabType::Overview => "Overview",
            TabType::Portfolios => "Portfolios",
            TabType::Networking => "Networking",
            TabType::NetworkingAddMember => "Add Team",
            TabType::Organization => "Organization",
            TabType::Reporting => "Reporting",
            TabType::Calendar => "Calendar",
            TabType::Transactions => "Transactions",
            TabType::History => "History",
            TabType::Settings => "Settings",
            TabType::Agent => "Agent",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Overview" => Some(TabType::Overview),
            "Portfolios" => Some(TabType::Portfolios),
            "Networking" => Some(TabType::Networking),
            "Add Team" => Some(TabType::NetworkingAddMember),
            "Organization" => Some(TabType::Organization),
            "Reporting" => Some(TabType::Reporting),
            "Calendar" => Some(TabType::Calendar),
            "Transactions" => Some(TabType::Transactions),
            "History" => Some(TabType::History),
            "Settings" => Some(TabType::Settings),
            "Agent" => Some(TabType::Agent),
            _ => None,
        }
    }
}

// Organization settings
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OrganizationSettings {
    pub default_currency: Currency,
    pub default_payment_interval: PaymentInterval,
    pub notification_preferences: Vec<(NotificationTrigger, Vec<NotificationType>)>,
    pub theme: Theme,
    pub custom_fields: Vec<String>,
    pub color: Option<String>,
}

// User profile
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub organization_id: Option<Uuid>,
    pub settings: OrganizationSettings,
    pub created_at: DateTime<Utc>,
}

impl UserProfile {
    pub fn can_view_all(&self) -> bool {
        !matches!(
            self.role,
            UserRole::Worker | UserRole::Contractor | UserRole::Guest
        )
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            username: String::new(),
            name: "Guest".to_string(),
            email: String::new(),
            role: UserRole::Worker,
            organization_id: None,
            settings: OrganizationSettings::default(),
            created_at: Utc::now(),
        }
    }
}

// API response wrapper
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
        }
    }
}

// Pagination
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

impl Pagination {
    pub fn new(page: u32, per_page: u32) -> Self {
        Self {
            page,
            per_page,
            total: 0,
        }
    }

    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }
}
