use uuid::Uuid;

pub mod channels;
pub mod contact_card;
pub mod contact_list;
pub mod external_orgs;
pub mod integrations;
pub mod networking_forms;
pub mod page;
pub mod partners;
pub mod relationship_history;

pub use networking_forms::AddTeamMemberPage;
pub use page::NetworkingPage;

/// Active tab in the networking page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum NetTab {
    Contacts,
    ExternalOrgs,
    Channels,
    Partners,
    Clients,
    Suppliers,
    Integrations,
    RelationshipMap,
    RelationshipHistory,
}

impl NetTab {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            NetTab::Contacts => "Contacts",
            NetTab::ExternalOrgs => "External Organizations",
            NetTab::Channels => "Channels",
            NetTab::Partners => "Partners",
            NetTab::Clients => "Clients",
            NetTab::Suppliers => "Suppliers / Vendors",
            NetTab::Integrations => "Integrations",
            NetTab::RelationshipMap => "Relationship Map",
            NetTab::RelationshipHistory => "Relationship History",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RelationshipStatus {
    Active,
    #[allow(dead_code)]
    Inactive,
    Pending,
    Suspended,
    #[allow(dead_code)]
    Archived,
}

impl RelationshipStatus {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            RelationshipStatus::Active => "Active",
            RelationshipStatus::Inactive => "Inactive",
            RelationshipStatus::Pending => "Pending",
            RelationshipStatus::Suspended => "Suspended",
            RelationshipStatus::Archived => "Archived",
        }
    }
    pub(crate) fn css_class(&self) -> &'static str {
        match self {
            RelationshipStatus::Active => "net-status-active",
            RelationshipStatus::Inactive => "net-status-inactive",
            RelationshipStatus::Pending => "net-status-pending",
            RelationshipStatus::Suspended => "net-status-suspended",
            RelationshipStatus::Archived => "net-status-archived",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RiskLevel {
    Low,
    Medium,
    #[allow(dead_code)]
    High,
    #[allow(dead_code)]
    Unknown,
}

impl RiskLevel {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            RiskLevel::Low => "Low",
            RiskLevel::Medium => "Medium",
            RiskLevel::High => "High",
            RiskLevel::Unknown => "Unknown",
        }
    }
    pub(crate) fn css_class(&self) -> &'static str {
        match self {
            RiskLevel::Low => "net-risk-low",
            RiskLevel::Medium => "net-risk-medium",
            RiskLevel::High => "net-risk-high",
            RiskLevel::Unknown => "net-risk-unknown",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NetSort {
    Name,
    Company,
    Status,
    Risk,
    Type,
    Transactions,
}

impl NetSort {
    #[allow(dead_code)]
    pub(crate) fn label(&self) -> &'static str {
        match self {
            NetSort::Name => "Name",
            NetSort::Company => "Company",
            NetSort::Status => "Status",
            NetSort::Risk => "Risk",
            NetSort::Type => "Type",
            NetSort::Transactions => "Transactions",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Channel {
    #[allow(dead_code)]
    pub id: Uuid,
    pub name: String,
    pub channel_type: String,
    pub address: Option<String>,
    pub status: RelationshipStatus,
    pub linked_contact: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ExternalContact {
    pub id: Uuid,
    pub name: String,
    pub title: String,
    pub company: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub relationship_type: String,
    pub status: RelationshipStatus,
    pub risk_level: RiskLevel,
    pub linked_portfolios: Vec<String>,
    pub linked_transactions: Vec<String>,
    pub linked_reports: Vec<String>,
    pub last_message: Option<String>,
    pub last_transaction: Option<String>,
    pub channels: Vec<String>,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ExternalOrganization {
    pub id: Uuid,
    pub name: String,
    pub org_type: String,
    pub primary_contact: Option<String>,
    pub status: RelationshipStatus,
    pub risk_level: RiskLevel,
    pub linked_portfolios: Vec<String>,
    pub transaction_count: usize,
    pub document_count: usize,
    pub channels: Vec<String>,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Integration {
    #[allow(dead_code)]
    pub id: Uuid,
    pub name: String,
    pub integration_type: String,
    pub status: RelationshipStatus,
    pub last_sync: Option<String>,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RelationshipEvent {
    #[allow(dead_code)]
    pub id: Uuid,
    pub entity_name: String,
    pub event_description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
}
