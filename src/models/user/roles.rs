use super::permissions::Perm;
use crate::types::UserRole;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Role definition and permissions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleDefinition {
    pub id: Uuid,
    pub name: String,
    pub role_type: UserRole,
    pub permissions: Vec<super::permissions::Permission>,
    pub description: Option<String>,
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
                self.name,
                self.rank,
                scope_text,
                self.member_ids.len(),
                can_count
            )
        } else {
            format!(
                "{} role. Rank {}. Applies to {}. {} members. {} permissions.",
                self.name,
                self.rank,
                scope_text,
                self.member_ids.len(),
                can_count
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
