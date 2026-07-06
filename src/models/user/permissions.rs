use crate::types::UserRole;
use serde::{Deserialize, Serialize};

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
            Perm::ManageSuppliers => "Manage partners/suppliers/vendors",
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
            Perm::ViewOrganization
            | Perm::EditOrganizationDetails
            | Perm::ManageOrgSettings
            | Perm::ManageMembers
            | Perm::InviteMembers
            | Perm::RemoveMembers
            | Perm::AssignRoles
            | Perm::CreateRoles
            | Perm::EditRolesBelowOwn
            | Perm::DeleteRolesBelowOwn
            | Perm::ReorderRoles
            | Perm::TransferOwnership
            | Perm::DeleteOrganization => PermGroup::Organization,

            Perm::ViewPortfolios
            | Perm::CreatePortfolios
            | Perm::EditAssignedPortfolios
            | Perm::DeleteAssignedPortfolios
            | Perm::ManagePortfolioAccess
            | Perm::CreateAssetGroups
            | Perm::EditAssetGroups
            | Perm::DeleteAssetGroups
            | Perm::CreateAssets
            | Perm::EditAssets
            | Perm::DeleteAssets
            | Perm::MoveAssetsBetweenGroups
            | Perm::CreateDirectAssets
            | Perm::EditDirectAssets
            | Perm::DeleteDirectAssets
            | Perm::MoveDirectAssetsIntoGroups => PermGroup::Portfolio,

            Perm::ViewNetworkingContacts
            | Perm::CreateContacts
            | Perm::EditContacts
            | Perm::DeleteContacts
            | Perm::ViewExternalOrganizations
            | Perm::CreateExternalOrganizations
            | Perm::EditExternalOrganizations
            | Perm::DeleteExternalOrganizations
            | Perm::ViewChannels
            | Perm::CreateChannels
            | Perm::EditChannels
            | Perm::DeleteChannels
            | Perm::ManagePartners
            | Perm::ManageSuppliers
            | Perm::ManageIntegrations
            | Perm::LinkContactsToPortfolios => PermGroup::Networking,

            Perm::ViewReports
            | Perm::CreateReports
            | Perm::EditOwnReports
            | Perm::EditAnyDraftReport
            | Perm::SubmitReports
            | Perm::ApproveReports
            | Perm::PublishReports
            | Perm::ArchiveReports
            | Perm::DeleteOwnDraftReports
            | Perm::DeleteAnyDraftReports
            | Perm::ViewReportingHistory
            | Perm::ViewDocuments
            | Perm::UploadDocuments
            | Perm::EditOwnDraftDocuments
            | Perm::EditAnyDraftDocument
            | Perm::SubmitDocuments
            | Perm::ApproveDocuments
            | Perm::LockFinalDocuments
            | Perm::UnlockFinalDocuments
            | Perm::ArchiveDocuments
            | Perm::CreateDocumentRevision
            | Perm::ViewLockedDocuments
            | Perm::DeleteOwnDraftDocuments
            | Perm::DeleteAnyDraftDocuments
            | Perm::DeleteLockedDocuments
            | Perm::RestoreArchivedDocuments
            | Perm::ManageDocumentCategories
            | Perm::ManageDocumentVisibility
            | Perm::ManageDocumentOwnership => PermGroup::Reporting,

            Perm::ViewCalendar
            | Perm::CreateCalendarEvents
            | Perm::EditOwnEvents
            | Perm::EditOrgEvents
            | Perm::DeleteOwnEvents
            | Perm::DeleteOrgEvents
            | Perm::AssignEventsToPortfolios
            | Perm::ManageReminders
            | Perm::ViewPrivateCalendar
            | Perm::ManageSharedCalendarVisibility => PermGroup::Calendar,

            Perm::ViewTransactions
            | Perm::CreateTransactions
            | Perm::EditOwnDraftTransactions
            | Perm::EditAnyDraftTransaction
            | Perm::SubmitTransactions
            | Perm::ApproveTransactions
            | Perm::RejectTransactions
            | Perm::LockFinalizedTransactions
            | Perm::ExportTransactions
            | Perm::DeleteDraftTransactions
            | Perm::ViewTransactionHistory
            | Perm::ManageTransactionCategories => PermGroup::Transaction,

            Perm::ViewOwnActivityHistory
            | Perm::ViewOrgAuditLog
            | Perm::ViewPortfolioHistory
            | Perm::ViewAssetHistory
            | Perm::ViewDocumentHistory
            | Perm::ViewTransactionHistoryLog
            | Perm::ExportHistoryLogs
            | Perm::RestorePreviousVersions
            | Perm::ViewDeletedArchivedItems
            | Perm::RestoreDeletedArchivedItems => PermGroup::History,
        }
    }

    pub fn all() -> Vec<Perm> {
        vec![
            // Organization
            Perm::ViewOrganization,
            Perm::EditOrganizationDetails,
            Perm::ManageOrgSettings,
            Perm::ManageMembers,
            Perm::InviteMembers,
            Perm::RemoveMembers,
            Perm::AssignRoles,
            Perm::CreateRoles,
            Perm::EditRolesBelowOwn,
            Perm::DeleteRolesBelowOwn,
            Perm::ReorderRoles,
            Perm::TransferOwnership,
            Perm::DeleteOrganization,
            // Portfolio
            Perm::ViewPortfolios,
            Perm::CreatePortfolios,
            Perm::EditAssignedPortfolios,
            Perm::DeleteAssignedPortfolios,
            Perm::ManagePortfolioAccess,
            Perm::CreateAssetGroups,
            Perm::EditAssetGroups,
            Perm::DeleteAssetGroups,
            Perm::CreateAssets,
            Perm::EditAssets,
            Perm::DeleteAssets,
            Perm::MoveAssetsBetweenGroups,
            Perm::CreateDirectAssets,
            Perm::EditDirectAssets,
            Perm::DeleteDirectAssets,
            Perm::MoveDirectAssetsIntoGroups,
            // Networking
            Perm::ViewNetworkingContacts,
            Perm::CreateContacts,
            Perm::EditContacts,
            Perm::DeleteContacts,
            Perm::ViewExternalOrganizations,
            Perm::CreateExternalOrganizations,
            Perm::EditExternalOrganizations,
            Perm::DeleteExternalOrganizations,
            Perm::ViewChannels,
            Perm::CreateChannels,
            Perm::EditChannels,
            Perm::DeleteChannels,
            Perm::ManagePartners,
            Perm::ManageSuppliers,
            Perm::ManageIntegrations,
            Perm::LinkContactsToPortfolios,
            // Reporting
            Perm::ViewReports,
            Perm::CreateReports,
            Perm::EditOwnReports,
            Perm::EditAnyDraftReport,
            Perm::SubmitReports,
            Perm::ApproveReports,
            Perm::PublishReports,
            Perm::ArchiveReports,
            Perm::DeleteOwnDraftReports,
            Perm::DeleteAnyDraftReports,
            Perm::ViewReportingHistory,
            // Document Controls
            Perm::ViewDocuments,
            Perm::UploadDocuments,
            Perm::EditOwnDraftDocuments,
            Perm::EditAnyDraftDocument,
            Perm::SubmitDocuments,
            Perm::ApproveDocuments,
            Perm::LockFinalDocuments,
            Perm::UnlockFinalDocuments,
            Perm::ArchiveDocuments,
            Perm::CreateDocumentRevision,
            Perm::ViewLockedDocuments,
            Perm::DeleteOwnDraftDocuments,
            Perm::DeleteAnyDraftDocuments,
            Perm::DeleteLockedDocuments,
            Perm::RestoreArchivedDocuments,
            Perm::ManageDocumentCategories,
            Perm::ManageDocumentVisibility,
            Perm::ManageDocumentOwnership,
            // Calendar
            Perm::ViewCalendar,
            Perm::CreateCalendarEvents,
            Perm::EditOwnEvents,
            Perm::EditOrgEvents,
            Perm::DeleteOwnEvents,
            Perm::DeleteOrgEvents,
            Perm::AssignEventsToPortfolios,
            Perm::ManageReminders,
            Perm::ViewPrivateCalendar,
            Perm::ManageSharedCalendarVisibility,
            // Transaction
            Perm::ViewTransactions,
            Perm::CreateTransactions,
            Perm::EditOwnDraftTransactions,
            Perm::EditAnyDraftTransaction,
            Perm::SubmitTransactions,
            Perm::ApproveTransactions,
            Perm::RejectTransactions,
            Perm::LockFinalizedTransactions,
            Perm::ExportTransactions,
            Perm::DeleteDraftTransactions,
            Perm::ViewTransactionHistory,
            Perm::ManageTransactionCategories,
            // History
            Perm::ViewOwnActivityHistory,
            Perm::ViewOrgAuditLog,
            Perm::ViewPortfolioHistory,
            Perm::ViewAssetHistory,
            Perm::ViewDocumentHistory,
            Perm::ViewTransactionHistoryLog,
            Perm::ExportHistoryLogs,
            Perm::RestorePreviousVersions,
            Perm::ViewDeletedArchivedItems,
            Perm::RestoreDeletedArchivedItems,
        ]
    }

    pub fn for_group(group: &PermGroup) -> Vec<Perm> {
        Self::all()
            .into_iter()
            .filter(|p| p.group() == *group)
            .collect()
    }
}
