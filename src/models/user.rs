use crate::models::portfolio::Document;
use crate::types::{
    Currency, NotificationTrigger, NotificationType, OrganizationSettings, PaymentInterval,
    PaymentMethod, UserProfile, UserRole,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// User/Organization member
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub username: Option<String>,
    pub email: String,
    pub role: UserRole,
    pub organization_id: Option<Uuid>,
    pub department: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub hire_date: Option<DateTime<Utc>>,
    pub base_salary: Option<f64>,
    pub avatar_url: Option<String>,
    pub payment_settings: PaymentSettings,
    pub notification_preferences: Vec<(NotificationTrigger, Vec<NotificationType>)>,
    pub permissions: Vec<Permission>,
    pub assignments: Vec<UserAssignment>,
    pub activity_log: Vec<UserActivity>,
    pub documents: Vec<crate::models::Document>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserAssignment {
    pub target_type: String,
    pub target_id: Uuid,
    pub target_name: String,
    pub assigned_at: DateTime<Utc>,
    pub duration_days: Option<i64>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserActivity {
    pub action: String,
    pub target_type: String,
    pub target_name: String,
    pub timestamp: DateTime<Utc>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaymentSettings {
    pub payment_method: PaymentMethod,
    pub account_details: String,
    pub payment_interval: PaymentInterval,
    pub currency: Currency,
    pub automatic_payout: bool,
    pub payout_threshold: Option<f64>,
}

impl Default for PaymentSettings {
    fn default() -> Self {
        Self {
            payment_method: PaymentMethod::BankTransfer,
            account_details: String::new(),
            payment_interval: PaymentInterval::Monthly,
            currency: Currency::USD,
            automatic_payout: true,
            payout_threshold: None,
        }
    }
}

impl User {
    pub fn new(name: String, email: String, role: UserRole) -> Self {
        let now = Utc::now();
        let permissions = default_permissions_for_role(&role);
        Self {
            id: Uuid::new_v4(),
            name,
            username: None,
            email,
            role,
            organization_id: None,
            department: None,
            phone: None,
            address: None,
            hire_date: None,
            base_salary: None,
            avatar_url: None,
            payment_settings: PaymentSettings::default(),
            notification_preferences: Vec::new(),
            permissions,
            assignments: Vec::new(),
            activity_log: Vec::new(),
            documents: Vec::new(),
            created_at: now,
            updated_at: now,
            last_login: None,
            is_active: true,
        }
    }

    pub fn can_manage(&self, other: &User) -> bool {
        // Must be in same organization
        if self.organization_id != other.organization_id {
            return false;
        }
        self.role.can_manage(&other.role)
    }

    pub fn update_role(&mut self, new_role: UserRole, changed_by: &User) -> Result<(), String> {
        if !changed_by.can_manage(self) {
            return Err("Insufficient permissions to change role".to_string());
        }
        self.role = new_role;
        self.permissions = default_permissions_for_role(&self.role);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    pub fn toggle_permission(&mut self, permission: Permission) {
        if self.permissions.contains(&permission) {
            self.permissions.retain(|p| p != &permission);
        } else {
            self.permissions.push(permission);
        }
        self.updated_at = Utc::now();
    }
}

pub fn default_permissions_for_role(role: &UserRole) -> Vec<Permission> {
    match role {
        // Owner: full unrestricted access across the organization and its data.
        UserRole::Owner => vec![
            Permission::ViewAll,
            Permission::CreateOrganization,
            Permission::EditAll,
            Permission::DeleteAll,
            Permission::ManageUsers,
            Permission::ManageRoles,
            Permission::ManagePayments,
            Permission::ManageSettings,
            Permission::ExportData,
            Permission::ImportData,
            Permission::EditDocuments,
        ],
        // Director: can manage nearly everything but cannot delete the organization or change global settings.
        UserRole::Director | UserRole::SeniorManager => vec![
            Permission::ViewAll,
            Permission::EditAll,
            Permission::CreateOrganization,
            Permission::EditOrganization,
            Permission::ManageUsers,
            Permission::ManageRoles,
            Permission::ManagePayments,
            Permission::ExportData,
            Permission::ImportData,
            Permission::EditDocuments,
        ],
        // Manager: full control over their portfolio purview (organization, users, payments, docs) but not global delete/export.
        UserRole::Manager => vec![
            Permission::ViewAll,
            Permission::ViewOrganization,
            Permission::CreateOrganization,
            Permission::EditOrganization,
            Permission::EditAll,
            Permission::ManageUsers,
            Permission::ManagePayments,
            Permission::EditDocuments,
        ],
        // Worker: can only manage their own assigned assets and documents.
        UserRole::Worker => vec![
            Permission::ViewOwn,
            Permission::ViewOrganization,
            Permission::CreateOwn,
            Permission::EditOwn,
            Permission::EditDocuments,
            Permission::DeleteOwn,
        ],
        // DocumentWorker: can view organization portfolios/assets but only edit documentation.
        UserRole::DocumentWorker => vec![
            Permission::ViewOrganization,
            Permission::ViewAll,
            Permission::EditDocuments,
        ],
        // Contractor: limited to their own assignments.
        UserRole::Contractor => vec![
            Permission::ViewOwn,
            Permission::CreateOwn,
            Permission::EditOwn,
        ],
        // Guest: view-only access to their own assignments.
        UserRole::Guest => vec![Permission::ViewOwn],
    }
}

impl UserProfile {
    pub fn has_permission(&self, permission: Permission) -> bool {
        default_permissions_for_role(&self.role).contains(&permission)
    }

    /// Can this user upload new documents to a portfolio/group/asset?
    pub fn can_upload_documents(&self) -> bool {
        self.has_permission(Permission::EditDocuments)
    }

    /// Can this user edit the given document? Full editors can edit any document;
    /// workers with EditOwn can only edit documents they uploaded (or legacy nil-owner docs).
    pub fn can_edit_document(&self, doc: &Document) -> bool {
        if self.has_permission(Permission::EditAll) {
            return true;
        }
        if self.has_permission(Permission::EditDocuments) {
            if self.has_permission(Permission::EditOwn) {
                return doc.uploaded_by == self.id || doc.uploaded_by == Uuid::nil();
            }
            return true;
        }
        false
    }
}

// Organization
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub members: Vec<Uuid>,
    pub settings: OrganizationSettings,
    pub roles: Vec<OrgRole>,
    pub documents: Vec<crate::models::Document>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Organization {
    pub fn new(name: String, owner_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            owner_id,
            members: vec![owner_id],
            settings: OrganizationSettings::default(),
            roles: default_org_roles(),
            documents: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_member(&mut self, user_id: Uuid) {
        if !self.members.contains(&user_id) {
            self.members.push(user_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_member(&mut self, user_id: Uuid) {
        self.members.retain(|&id| id != user_id);
        self.updated_at = Utc::now();
    }
}

// Payment/Transaction record
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub amount: f64,
    pub currency: Currency,
    pub payment_method: PaymentMethod,
    pub description: Option<String>,
    pub related_asset_id: Option<Uuid>,
    pub related_portfolio_id: Option<Uuid>,
    pub status: PaymentStatus,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub executed_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub is_recurring: bool,
    pub recurrence_rule: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Scheduled,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl Payment {
    pub fn new(
        from_user_id: Uuid,
        to_user_id: Uuid,
        amount: f64,
        currency: Currency,
        payment_method: PaymentMethod,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_user_id,
            to_user_id,
            amount,
            currency,
            payment_method,
            description: None,
            related_asset_id: None,
            related_portfolio_id: None,
            status: PaymentStatus::Pending,
            scheduled_date: None,
            executed_date: None,
            created_at: Utc::now(),
            is_recurring: false,
            recurrence_rule: None,
        }
    }

    pub fn schedule(&mut self, date: DateTime<Utc>) {
        self.scheduled_date = Some(date);
        self.status = PaymentStatus::Scheduled;
    }

    pub fn mark_completed(&mut self) {
        self.status = PaymentStatus::Completed;
        self.executed_date = Some(Utc::now());
    }
}

// Role definition and permissions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleDefinition {
    pub id: Uuid,
    pub name: String,
    pub role_type: UserRole,
    pub permissions: Vec<Permission>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    ViewOwn,
    ViewOrganization,
    ViewAll,
    CreateOwn,
    CreateOrganization,
    EditOwn,
    EditOrganization,
    EditAll,
    DeleteOwn,
    DeleteOrganization,
    DeleteAll,
    ManageUsers,
    ManageRoles,
    ManagePayments,
    ManageSettings,
    ExportData,
    ImportData,
    EditDocuments,
    Custom(String),
}

// ── Discord-style role system ─────────────────────────────────────────────

/// Scope of a role — what it applies to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleScope {
    EntireOrganization,
    SelectedPortfolios(Vec<Uuid>),
    ReportingOnly,
    CalendarOnly,
    TransactionsOnly,
    NetworkingOnly,
    HistoryOnly,
}

impl Default for RoleScope {
    fn default() -> Self {
        RoleScope::EntireOrganization
    }
}

impl RoleScope {
    pub fn display(&self) -> &'static str {
        match self {
            RoleScope::EntireOrganization => "Entire organization",
            RoleScope::SelectedPortfolios(_) => "Selected portfolios",
            RoleScope::ReportingOnly => "Reporting only",
            RoleScope::CalendarOnly => "Calendar only",
            RoleScope::TransactionsOnly => "Transactions only",
            RoleScope::NetworkingOnly => "Networking only",
            RoleScope::HistoryOnly => "History/audit only",
        }
    }
}

/// Permission groups for the role editor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermGroup {
    Organization,
    Portfolio,
    Networking,
    Reporting,
    Calendar,
    Transaction,
    History,
}

impl PermGroup {
    pub fn label(&self) -> &'static str {
        match self {
            PermGroup::Organization => "Organization Controls",
            PermGroup::Portfolio => "Portfolio Controls",
            PermGroup::Networking => "Networking Controls",
            PermGroup::Reporting => "Reporting Controls",
            PermGroup::Calendar => "Calendar Controls",
            PermGroup::Transaction => "Transaction Controls",
            PermGroup::History => "History Controls",
        }
    }

    pub fn all() -> &'static [PermGroup] {
        &[
            PermGroup::Organization,
            PermGroup::Portfolio,
            PermGroup::Networking,
            PermGroup::Reporting,
            PermGroup::Calendar,
            PermGroup::Transaction,
            PermGroup::History,
        ]
    }
}

/// Granular permission enum for the Discord-style role system.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Perm {
    // Organization
    ViewOrganization,
    EditOrganizationDetails,
    ManageOrgSettings,
    ManageMembers,
    InviteMembers,
    RemoveMembers,
    AssignRoles,
    CreateRoles,
    EditRolesBelowOwn,
    DeleteRolesBelowOwn,
    ReorderRoles,
    TransferOwnership,
    DeleteOrganization,
    // Portfolio
    ViewPortfolios,
    CreatePortfolios,
    EditAssignedPortfolios,
    DeleteAssignedPortfolios,
    ManagePortfolioAccess,
    CreateAssetGroups,
    EditAssetGroups,
    DeleteAssetGroups,
    CreateAssets,
    EditAssets,
    DeleteAssets,
    MoveAssetsBetweenGroups,
    CreateDirectAssets,
    EditDirectAssets,
    DeleteDirectAssets,
    MoveDirectAssetsIntoGroups,
    // Networking
    ViewNetworkingContacts,
    CreateContacts,
    EditContacts,
    DeleteContacts,
    ViewExternalOrganizations,
    CreateExternalOrganizations,
    EditExternalOrganizations,
    DeleteExternalOrganizations,
    ViewChannels,
    CreateChannels,
    EditChannels,
    DeleteChannels,
    ManagePartners,
    ManageSuppliers,
    ManageIntegrations,
    LinkContactsToPortfolios,
    // Reporting
    ViewReports,
    CreateReports,
    EditOwnReports,
    EditAnyDraftReport,
    SubmitReports,
    ApproveReports,
    PublishReports,
    ArchiveReports,
    DeleteOwnDraftReports,
    DeleteAnyDraftReports,
    ViewReportingHistory,
    // Document Controls (under Reporting)
    ViewDocuments,
    UploadDocuments,
    EditOwnDraftDocuments,
    EditAnyDraftDocument,
    SubmitDocuments,
    ApproveDocuments,
    LockFinalDocuments,
    UnlockFinalDocuments,
    ArchiveDocuments,
    CreateDocumentRevision,
    ViewLockedDocuments,
    DeleteOwnDraftDocuments,
    DeleteAnyDraftDocuments,
    DeleteLockedDocuments,
    RestoreArchivedDocuments,
    ManageDocumentCategories,
    ManageDocumentVisibility,
    ManageDocumentOwnership,
    // Calendar
    ViewCalendar,
    CreateCalendarEvents,
    EditOwnEvents,
    EditOrgEvents,
    DeleteOwnEvents,
    DeleteOrgEvents,
    AssignEventsToPortfolios,
    ManageReminders,
    ViewPrivateCalendar,
    ManageSharedCalendarVisibility,
    // Transaction
    ViewTransactions,
    CreateTransactions,
    EditOwnDraftTransactions,
    EditAnyDraftTransaction,
    SubmitTransactions,
    ApproveTransactions,
    RejectTransactions,
    LockFinalizedTransactions,
    ExportTransactions,
    DeleteDraftTransactions,
    ViewTransactionHistory,
    ManageTransactionCategories,
    // History
    ViewOwnActivityHistory,
    ViewOrgAuditLog,
    ViewPortfolioHistory,
    ViewAssetHistory,
    ViewDocumentHistory,
    ViewTransactionHistoryLog,
    ExportHistoryLogs,
    RestorePreviousVersions,
    ViewDeletedArchivedItems,
    RestoreDeletedArchivedItems,
}

impl Perm {
    pub fn label(&self) -> &'static str {
        match self {
            Perm::ViewOrganization => "View organization",
            Perm::EditOrganizationDetails => "Edit organization details",
            Perm::ManageOrgSettings => "Manage organization settings",
            Perm::ManageMembers => "Manage members",
            Perm::InviteMembers => "Invite members",
            Perm::RemoveMembers => "Remove members",
            Perm::AssignRoles => "Assign roles",
            Perm::CreateRoles => "Create roles",
            Perm::EditRolesBelowOwn => "Edit roles below own rank",
            Perm::DeleteRolesBelowOwn => "Delete roles below own rank",
            Perm::ReorderRoles => "Reorder roles",
            Perm::TransferOwnership => "Transfer ownership",
            Perm::DeleteOrganization => "Delete organization",
            Perm::ViewPortfolios => "View portfolios",
            Perm::CreatePortfolios => "Create portfolios",
            Perm::EditAssignedPortfolios => "Edit assigned portfolios",
            Perm::DeleteAssignedPortfolios => "Delete assigned portfolios",
            Perm::ManagePortfolioAccess => "Manage portfolio access",
            Perm::CreateAssetGroups => "Create asset groups",
            Perm::EditAssetGroups => "Edit asset groups",
            Perm::DeleteAssetGroups => "Delete asset groups",
            Perm::CreateAssets => "Create assets",
            Perm::EditAssets => "Edit assets",
            Perm::DeleteAssets => "Delete assets",
            Perm::MoveAssetsBetweenGroups => "Move assets between groups",
            Perm::CreateDirectAssets => "Create direct assets",
            Perm::EditDirectAssets => "Edit direct assets",
            Perm::DeleteDirectAssets => "Delete direct assets",
            Perm::MoveDirectAssetsIntoGroups => "Move direct assets into groups",
            Perm::ViewNetworkingContacts => "View networking contacts",
            Perm::CreateContacts => "Create contacts",
            Perm::EditContacts => "Edit contacts",
            Perm::DeleteContacts => "Delete contacts",
            Perm::ViewExternalOrganizations => "View external organizations",
            Perm::CreateExternalOrganizations => "Create external organizations",
            Perm::EditExternalOrganizations => "Edit external organizations",
            Perm::DeleteExternalOrganizations => "Delete external organizations",
            Perm::ViewChannels => "View channels",
            Perm::CreateChannels => "Create channels",
            Perm::EditChannels => "Edit channels",
            Perm::DeleteChannels => "Delete channels",
            Perm::ManagePartners => "Manage partners",
            Perm::ManageSuppliers => "Manage suppliers/vendors",
            Perm::ManageIntegrations => "Manage integrations",
            Perm::LinkContactsToPortfolios => "Link contacts to portfolios/assets",
            Perm::ViewReports => "View reports",
            Perm::CreateReports => "Create reports",
            Perm::EditOwnReports => "Edit own reports",
            Perm::EditAnyDraftReport => "Edit any draft report",
            Perm::SubmitReports => "Submit reports for approval",
            Perm::ApproveReports => "Approve reports",
            Perm::PublishReports => "Publish reports",
            Perm::ArchiveReports => "Archive reports",
            Perm::DeleteOwnDraftReports => "Delete own draft reports",
            Perm::DeleteAnyDraftReports => "Delete any draft reports",
            Perm::ViewReportingHistory => "View reporting history",
            Perm::ViewDocuments => "View documents",
            Perm::UploadDocuments => "Upload/add documents",
            Perm::EditOwnDraftDocuments => "Edit own draft documents",
            Perm::EditAnyDraftDocument => "Edit any draft document",
            Perm::SubmitDocuments => "Submit documents for approval",
            Perm::ApproveDocuments => "Approve documents",
            Perm::LockFinalDocuments => "Lock final documents",
            Perm::UnlockFinalDocuments => "Unlock final documents",
            Perm::ArchiveDocuments => "Archive documents",
            Perm::CreateDocumentRevision => "Create new document revision",
            Perm::ViewLockedDocuments => "View locked documents",
            Perm::DeleteOwnDraftDocuments => "Delete own draft documents",
            Perm::DeleteAnyDraftDocuments => "Delete any draft documents",
            Perm::DeleteLockedDocuments => "Delete locked documents",
            Perm::RestoreArchivedDocuments => "Restore archived/deleted documents",
            Perm::ManageDocumentCategories => "Manage document categories",
            Perm::ManageDocumentVisibility => "Manage document visibility",
            Perm::ManageDocumentOwnership => "Manage document ownership",
            Perm::ViewCalendar => "View calendar",
            Perm::CreateCalendarEvents => "Create calendar events",
            Perm::EditOwnEvents => "Edit own events",
            Perm::EditOrgEvents => "Edit organization events",
            Perm::DeleteOwnEvents => "Delete own events",
            Perm::DeleteOrgEvents => "Delete organization events",
            Perm::AssignEventsToPortfolios => "Assign events to portfolios/assets",
            Perm::ManageReminders => "Manage reminders",
            Perm::ViewPrivateCalendar => "View private/internal calendar items",
            Perm::ManageSharedCalendarVisibility => "Manage shared calendar visibility",
            Perm::ViewTransactions => "View transactions",
            Perm::CreateTransactions => "Create transactions",
            Perm::EditOwnDraftTransactions => "Edit own draft transactions",
            Perm::EditAnyDraftTransaction => "Edit any draft transaction",
            Perm::SubmitTransactions => "Submit transactions for approval",
            Perm::ApproveTransactions => "Approve transactions",
            Perm::RejectTransactions => "Reject transactions",
            Perm::LockFinalizedTransactions => "Lock finalized transactions",
            Perm::ExportTransactions => "Export transactions",
            Perm::DeleteDraftTransactions => "Delete draft transactions",
            Perm::ViewTransactionHistory => "View transaction history",
            Perm::ManageTransactionCategories => "Manage transaction categories",
            Perm::ViewOwnActivityHistory => "View own activity history",
            Perm::ViewOrgAuditLog => "View organization audit log",
            Perm::ViewPortfolioHistory => "View portfolio history",
            Perm::ViewAssetHistory => "View asset history",
            Perm::ViewDocumentHistory => "View document history",
            Perm::ViewTransactionHistoryLog => "View transaction history log",
            Perm::ExportHistoryLogs => "Export history/audit logs",
            Perm::RestorePreviousVersions => "Restore previous versions",
            Perm::ViewDeletedArchivedItems => "View deleted/archived items",
            Perm::RestoreDeletedArchivedItems => "Restore deleted/archived items",
        }
    }

    pub fn group(&self) -> PermGroup {
        match self {
            Perm::ViewOrganization | Perm::EditOrganizationDetails | Perm::ManageOrgSettings
            | Perm::ManageMembers | Perm::InviteMembers | Perm::RemoveMembers
            | Perm::AssignRoles | Perm::CreateRoles | Perm::EditRolesBelowOwn
            | Perm::DeleteRolesBelowOwn | Perm::ReorderRoles | Perm::TransferOwnership
            | Perm::DeleteOrganization => PermGroup::Organization,

            Perm::ViewPortfolios | Perm::CreatePortfolios | Perm::EditAssignedPortfolios
            | Perm::DeleteAssignedPortfolios | Perm::ManagePortfolioAccess
            | Perm::CreateAssetGroups | Perm::EditAssetGroups | Perm::DeleteAssetGroups
            | Perm::CreateAssets | Perm::EditAssets | Perm::DeleteAssets
            | Perm::MoveAssetsBetweenGroups | Perm::CreateDirectAssets
            | Perm::EditDirectAssets | Perm::DeleteDirectAssets
            | Perm::MoveDirectAssetsIntoGroups => PermGroup::Portfolio,

            Perm::ViewNetworkingContacts | Perm::CreateContacts | Perm::EditContacts
            | Perm::DeleteContacts | Perm::ViewExternalOrganizations
            | Perm::CreateExternalOrganizations | Perm::EditExternalOrganizations
            | Perm::DeleteExternalOrganizations | Perm::ViewChannels
            | Perm::CreateChannels | Perm::EditChannels | Perm::DeleteChannels
            | Perm::ManagePartners | Perm::ManageSuppliers | Perm::ManageIntegrations
            | Perm::LinkContactsToPortfolios => PermGroup::Networking,

            Perm::ViewReports | Perm::CreateReports | Perm::EditOwnReports
            | Perm::EditAnyDraftReport | Perm::SubmitReports | Perm::ApproveReports
            | Perm::PublishReports | Perm::ArchiveReports | Perm::DeleteOwnDraftReports
            | Perm::DeleteAnyDraftReports | Perm::ViewReportingHistory
            | Perm::ViewDocuments | Perm::UploadDocuments | Perm::EditOwnDraftDocuments
            | Perm::EditAnyDraftDocument | Perm::SubmitDocuments | Perm::ApproveDocuments
            | Perm::LockFinalDocuments | Perm::UnlockFinalDocuments | Perm::ArchiveDocuments
            | Perm::CreateDocumentRevision | Perm::ViewLockedDocuments
            | Perm::DeleteOwnDraftDocuments | Perm::DeleteAnyDraftDocuments
            | Perm::DeleteLockedDocuments | Perm::RestoreArchivedDocuments
            | Perm::ManageDocumentCategories | Perm::ManageDocumentVisibility
            | Perm::ManageDocumentOwnership => PermGroup::Reporting,

            Perm::ViewCalendar | Perm::CreateCalendarEvents | Perm::EditOwnEvents
            | Perm::EditOrgEvents | Perm::DeleteOwnEvents | Perm::DeleteOrgEvents
            | Perm::AssignEventsToPortfolios | Perm::ManageReminders
            | Perm::ViewPrivateCalendar | Perm::ManageSharedCalendarVisibility => PermGroup::Calendar,

            Perm::ViewTransactions | Perm::CreateTransactions | Perm::EditOwnDraftTransactions
            | Perm::EditAnyDraftTransaction | Perm::SubmitTransactions | Perm::ApproveTransactions
            | Perm::RejectTransactions | Perm::LockFinalizedTransactions
            | Perm::ExportTransactions | Perm::DeleteDraftTransactions
            | Perm::ViewTransactionHistory | Perm::ManageTransactionCategories => PermGroup::Transaction,

            Perm::ViewOwnActivityHistory | Perm::ViewOrgAuditLog | Perm::ViewPortfolioHistory
            | Perm::ViewAssetHistory | Perm::ViewDocumentHistory
            | Perm::ViewTransactionHistoryLog | Perm::ExportHistoryLogs
            | Perm::RestorePreviousVersions | Perm::ViewDeletedArchivedItems
            | Perm::RestoreDeletedArchivedItems => PermGroup::History,
        }
    }

    pub fn all() -> Vec<Perm> {
        vec![
            // Organization
            Perm::ViewOrganization, Perm::EditOrganizationDetails, Perm::ManageOrgSettings,
            Perm::ManageMembers, Perm::InviteMembers, Perm::RemoveMembers,
            Perm::AssignRoles, Perm::CreateRoles, Perm::EditRolesBelowOwn,
            Perm::DeleteRolesBelowOwn, Perm::ReorderRoles, Perm::TransferOwnership,
            Perm::DeleteOrganization,
            // Portfolio
            Perm::ViewPortfolios, Perm::CreatePortfolios, Perm::EditAssignedPortfolios,
            Perm::DeleteAssignedPortfolios, Perm::ManagePortfolioAccess,
            Perm::CreateAssetGroups, Perm::EditAssetGroups, Perm::DeleteAssetGroups,
            Perm::CreateAssets, Perm::EditAssets, Perm::DeleteAssets,
            Perm::MoveAssetsBetweenGroups, Perm::CreateDirectAssets,
            Perm::EditDirectAssets, Perm::DeleteDirectAssets, Perm::MoveDirectAssetsIntoGroups,
            // Networking
            Perm::ViewNetworkingContacts, Perm::CreateContacts, Perm::EditContacts,
            Perm::DeleteContacts, Perm::ViewExternalOrganizations,
            Perm::CreateExternalOrganizations, Perm::EditExternalOrganizations,
            Perm::DeleteExternalOrganizations, Perm::ViewChannels,
            Perm::CreateChannels, Perm::EditChannels, Perm::DeleteChannels,
            Perm::ManagePartners, Perm::ManageSuppliers, Perm::ManageIntegrations,
            Perm::LinkContactsToPortfolios,
            // Reporting
            Perm::ViewReports, Perm::CreateReports, Perm::EditOwnReports,
            Perm::EditAnyDraftReport, Perm::SubmitReports, Perm::ApproveReports,
            Perm::PublishReports, Perm::ArchiveReports, Perm::DeleteOwnDraftReports,
            Perm::DeleteAnyDraftReports, Perm::ViewReportingHistory,
            // Document Controls
            Perm::ViewDocuments, Perm::UploadDocuments, Perm::EditOwnDraftDocuments,
            Perm::EditAnyDraftDocument, Perm::SubmitDocuments, Perm::ApproveDocuments,
            Perm::LockFinalDocuments, Perm::UnlockFinalDocuments, Perm::ArchiveDocuments,
            Perm::CreateDocumentRevision, Perm::ViewLockedDocuments,
            Perm::DeleteOwnDraftDocuments, Perm::DeleteAnyDraftDocuments,
            Perm::DeleteLockedDocuments, Perm::RestoreArchivedDocuments,
            Perm::ManageDocumentCategories, Perm::ManageDocumentVisibility,
            Perm::ManageDocumentOwnership,
            // Calendar
            Perm::ViewCalendar, Perm::CreateCalendarEvents, Perm::EditOwnEvents,
            Perm::EditOrgEvents, Perm::DeleteOwnEvents, Perm::DeleteOrgEvents,
            Perm::AssignEventsToPortfolios, Perm::ManageReminders,
            Perm::ViewPrivateCalendar, Perm::ManageSharedCalendarVisibility,
            // Transaction
            Perm::ViewTransactions, Perm::CreateTransactions, Perm::EditOwnDraftTransactions,
            Perm::EditAnyDraftTransaction, Perm::SubmitTransactions, Perm::ApproveTransactions,
            Perm::RejectTransactions, Perm::LockFinalizedTransactions,
            Perm::ExportTransactions, Perm::DeleteDraftTransactions,
            Perm::ViewTransactionHistory, Perm::ManageTransactionCategories,
            // History
            Perm::ViewOwnActivityHistory, Perm::ViewOrgAuditLog, Perm::ViewPortfolioHistory,
            Perm::ViewAssetHistory, Perm::ViewDocumentHistory, Perm::ViewTransactionHistoryLog,
            Perm::ExportHistoryLogs, Perm::RestorePreviousVersions,
            Perm::ViewDeletedArchivedItems, Perm::RestoreDeletedArchivedItems,
        ]
    }

    pub fn for_group(group: &PermGroup) -> Vec<Perm> {
        Self::all().into_iter().filter(|p| p.group() == *group).collect()
    }
}

/// Discord-style role definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrgRole {
    pub id: Uuid,
    pub name: String,
    pub rank: u32,
    pub color: Option<String>,
    pub description: String,
    pub scope: RoleScope,
    pub permissions: Vec<Perm>,
    pub member_ids: Vec<Uuid>,
    pub documents: Vec<crate::models::Document>,
    pub is_system: bool,
}

impl OrgRole {
    pub fn new(name: String, rank: u32, description: String, permissions: Vec<Perm>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            rank,
            color: None,
            description,
            scope: RoleScope::EntireOrganization,
            permissions,
            member_ids: Vec::new(),
            documents: Vec::new(),
            is_system: false,
        }
    }

    pub fn summary(&self) -> String {
        let can_count = self.permissions.len();
        let scope_text = self.scope.display();
        if can_count > 10 {
            format!(
                "{} role. Rank {}. Applies to {}. {} members. Has {} permissions.",
                self.name, self.rank, scope_text, self.member_ids.len(), can_count
            )
        } else {
            format!(
                "{} role. Rank {}. Applies to {}. {} members. {} permissions.",
                self.name, self.rank, scope_text, self.member_ids.len(), can_count
            )
        }
    }
}

/// Create the 7 default Discord-style roles for a new organization.
pub fn default_org_roles() -> Vec<OrgRole> {
    vec![
        OrgRole {
            id: Uuid::new_v4(),
            name: "Owner".to_string(),
            rank: 100,
            color: Some("#f59e0b".to_string()),
            description: "Full control of the organization. Can manage all roles, members, portfolios, reporting, documents, calendar, transactions, networking, and history. Cannot be removed if they are the last owner.".to_string(),
            scope: RoleScope::EntireOrganization,
            permissions: Perm::all(),
            member_ids: Vec::new(),
            documents: Vec::new(),
            is_system: true,
        },
        OrgRole {
            id: Uuid::new_v4(),
            name: "Organization Admin".to_string(),
            rank: 90,
            color: Some("#3b82f6".to_string()),
            description: "Can manage organization settings, portfolios, members, roles below their rank, reporting, calendar, transactions, networking, and history. Cannot manage owners.".to_string(),
            scope: RoleScope::EntireOrganization,
            permissions: Perm::all().into_iter().filter(|p| !matches!(p,
                Perm::TransferOwnership | Perm::DeleteOrganization
            )).collect(),
            member_ids: Vec::new(),
            documents: Vec::new(),
            is_system: true,
        },
        OrgRole {
            id: Uuid::new_v4(),
            name: "Portfolio Manager".to_string(),
            rank: 70,
            color: Some("#10b981".to_string()),
            description: "Can manage assigned portfolios, asset groups, assets, direct assets, and portfolio-level access. Can add reports/documents and edit own drafts. Cannot unlock final documents unless explicitly given that permission.".to_string(),
            scope: RoleScope::EntireOrganization,
            permissions: vec![
                Perm::ViewOrganization, Perm::ViewPortfolios, Perm::CreatePortfolios,
                Perm::EditAssignedPortfolios, Perm::ManagePortfolioAccess,
                Perm::CreateAssetGroups, Perm::EditAssetGroups, Perm::DeleteAssetGroups,
                Perm::CreateAssets, Perm::EditAssets, Perm::DeleteAssets,
                Perm::MoveAssetsBetweenGroups, Perm::CreateDirectAssets,
                Perm::EditDirectAssets, Perm::DeleteDirectAssets, Perm::MoveDirectAssetsIntoGroups,
                Perm::ViewReports, Perm::CreateReports, Perm::EditOwnReports,
                Perm::ViewDocuments, Perm::UploadDocuments, Perm::EditOwnDraftDocuments,
                Perm::ViewLockedDocuments, Perm::CreateDocumentRevision,
                Perm::ViewCalendar, Perm::CreateCalendarEvents, Perm::EditOwnEvents,
                Perm::ViewTransactions, Perm::CreateTransactions, Perm::EditOwnDraftTransactions,
                Perm::ViewOwnActivityHistory, Perm::ViewPortfolioHistory, Perm::ViewAssetHistory,
            ],
            member_ids: Vec::new(),
            documents: Vec::new(),
            is_system: true,
        },
        OrgRole {
            id: Uuid::new_v4(),
            name: "Reporting Manager".to_string(),
            rank: 60,
            color: Some("#8b5cf6".to_string()),
            description: "Can manage reports and document workflows. Can approve, publish, lock, archive, and create new revisions of documents if granted. Cannot manage organization roles unless separately granted.".to_string(),
            scope: RoleScope::ReportingOnly,
            permissions: vec![
                Perm::ViewOrganization, Perm::ViewPortfolios,
                Perm::ViewReports, Perm::CreateReports, Perm::EditOwnReports,
                Perm::EditAnyDraftReport, Perm::SubmitReports, Perm::ApproveReports,
                Perm::PublishReports, Perm::ArchiveReports, Perm::DeleteOwnDraftReports,
                Perm::ViewReportingHistory,
                Perm::ViewDocuments, Perm::UploadDocuments, Perm::EditOwnDraftDocuments,
                Perm::EditAnyDraftDocument, Perm::SubmitDocuments, Perm::ApproveDocuments,
                Perm::LockFinalDocuments, Perm::ArchiveDocuments,
                Perm::CreateDocumentRevision, Perm::ViewLockedDocuments,
                Perm::DeleteOwnDraftDocuments, Perm::RestoreArchivedDocuments,
                Perm::ManageDocumentCategories, Perm::ManageDocumentVisibility,
                Perm::ViewOwnActivityHistory, Perm::ViewDocumentHistory,
            ],
            member_ids: Vec::new(),
            documents: Vec::new(),
            is_system: true,
        },
        OrgRole {
            id: Uuid::new_v4(),
            name: "Transaction Manager".to_string(),
            rank: 55,
            color: Some("#ec4899".to_string()),
            description: "Can create, edit, submit, approve, lock, and export transactions depending on permission level. Cannot manage documents or organization roles unless separately granted.".to_string(),
            scope: RoleScope::TransactionsOnly,
            permissions: vec![
                Perm::ViewOrganization, Perm::ViewPortfolios,
                Perm::ViewTransactions, Perm::CreateTransactions,
                Perm::EditOwnDraftTransactions, Perm::EditAnyDraftTransaction,
                Perm::SubmitTransactions, Perm::ApproveTransactions,
                Perm::RejectTransactions, Perm::LockFinalizedTransactions,
                Perm::ExportTransactions, Perm::DeleteDraftTransactions,
                Perm::ViewTransactionHistory, Perm::ManageTransactionCategories,
                Perm::ViewOwnActivityHistory, Perm::ViewTransactionHistoryLog,
            ],
            member_ids: Vec::new(),
            documents: Vec::new(),
            is_system: true,
        },
        OrgRole {
            id: Uuid::new_v4(),
            name: "Contributor".to_string(),
            rank: 30,
            color: Some("#6366f1".to_string()),
            description: "Can view assigned areas, add assets or documents where allowed, and edit their own draft content. Cannot approve, lock, unlock, or delete final records.".to_string(),
            scope: RoleScope::EntireOrganization,
            permissions: vec![
                Perm::ViewOrganization, Perm::ViewPortfolios,
                Perm::CreateAssets, Perm::EditAssets,
                Perm::CreateDirectAssets, Perm::EditDirectAssets,
                Perm::ViewReports, Perm::CreateReports, Perm::EditOwnReports,
                Perm::ViewDocuments, Perm::UploadDocuments, Perm::EditOwnDraftDocuments,
                Perm::SubmitDocuments, Perm::SubmitReports,
                Perm::ViewCalendar, Perm::CreateCalendarEvents, Perm::EditOwnEvents,
                Perm::ViewTransactions, Perm::CreateTransactions, Perm::EditOwnDraftTransactions,
                Perm::ViewOwnActivityHistory,
            ],
            member_ids: Vec::new(),
            documents: Vec::new(),
            is_system: true,
        },
        OrgRole {
            id: Uuid::new_v4(),
            name: "Viewer / Auditor".to_string(),
            rank: 10,
            color: Some("#6b7280".to_string()),
            description: "Can view assigned content and history. Cannot edit, add, delete, approve, or lock anything unless explicitly granted.".to_string(),
            scope: RoleScope::EntireOrganization,
            permissions: vec![
                Perm::ViewOrganization, Perm::ViewPortfolios,
                Perm::ViewReports, Perm::ViewDocuments, Perm::ViewLockedDocuments,
                Perm::ViewCalendar, Perm::ViewTransactions,
                Perm::ViewOwnActivityHistory, Perm::ViewOrgAuditLog,
                Perm::ViewPortfolioHistory, Perm::ViewAssetHistory,
                Perm::ViewDocumentHistory, Perm::ViewTransactionHistoryLog,
            ],
            member_ids: Vec::new(),
            documents: Vec::new(),
            is_system: true,
        },
    ]
}
