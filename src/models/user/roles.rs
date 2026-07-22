use super::permissions::{Perm, PermGroup};
use crate::types::UserRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// Per-area view/edit permissions for the role scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ScopePermissions {
    pub view: bool,
    pub edit: bool,
}

/// Scope of a role — what functional areas it may view and edit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RoleScope {
    pub areas: HashMap<PermGroup, ScopePermissions>,
}

impl RoleScope {
    pub fn entire() -> Self {
        let mut scope = Self::default();
        for group in PermGroup::all() {
            scope.areas.insert(
                *group,
                ScopePermissions {
                    view: true,
                    edit: true,
                },
            );
        }
        scope
    }

    pub fn view_all() -> Self {
        let mut scope = Self::default();
        for group in PermGroup::all() {
            scope.areas.insert(
                *group,
                ScopePermissions {
                    view: true,
                    edit: false,
                },
            );
        }
        scope
    }

    pub fn with_group(group: PermGroup, view: bool, edit: bool) -> Self {
        let mut scope = Self::default();
        scope.areas.insert(group, ScopePermissions { view, edit });
        scope
    }

    pub fn with_groups(groups: &[(PermGroup, bool, bool)]) -> Self {
        let mut scope = Self::default();
        for (group, view, edit) in groups {
            scope.areas.insert(
                *group,
                ScopePermissions {
                    view: *view,
                    edit: *edit,
                },
            );
        }
        scope
    }

    pub fn view(&self, group: PermGroup) -> bool {
        self.areas.get(&group).map(|p| p.view).unwrap_or(false)
    }

    pub fn edit(&self, group: PermGroup) -> bool {
        self.areas.get(&group).map(|p| p.edit).unwrap_or(false)
    }

    pub fn set_view(&mut self, group: PermGroup, value: bool) {
        self.areas.entry(group).or_default().view = value;
    }

    pub fn set_edit(&mut self, group: PermGroup, value: bool) {
        self.areas.entry(group).or_default().edit = value;
    }

    pub fn toggle_view(&mut self, group: PermGroup) {
        let value = !self.view(group);
        self.set_view(group, value);
    }

    pub fn toggle_edit(&mut self, group: PermGroup) {
        let value = !self.edit(group);
        self.set_edit(group, value);
    }

    pub fn display(&self) -> String {
        let all_groups: Vec<_> = PermGroup::all().iter().copied().collect();
        let all_view = all_groups.iter().all(|g| self.view(*g));
        let all_edit = all_groups.iter().all(|g| self.edit(*g));

        if all_view && all_edit {
            return "Entire organization".to_string();
        }
        if all_view && !all_edit {
            return "View entire organization".to_string();
        }

        let active: Vec<String> = all_groups
            .iter()
            .filter(|&&g| self.view(g) || self.edit(g))
            .map(|&g| {
                let label = g.label();
                match (self.view(g), self.edit(g)) {
                    (true, true) => format!("{}: view/edit", label),
                    (true, false) => format!("{}: view", label),
                    (false, true) => format!("{}: edit", label),
                    (false, false) => unreachable!(),
                }
            })
            .collect();

        if active.is_empty() {
            "No scope".to_string()
        } else {
            active.join(", ")
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
            scope: RoleScope::default(),
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
                "{} role. Applies to {}. {} members. Has {} permissions.",
                self.name,
                scope_text,
                self.member_ids.len(),
                can_count
            )
        } else {
            format!(
                "{} role. Applies to {}. {} members. {} permissions.",
                self.name,
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
            scope: RoleScope::entire(),
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
            scope: RoleScope::entire(),
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
            scope: RoleScope::entire(),
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
                Perm::ViewBookings, Perm::CreateBookings, Perm::EditBookings, Perm::CancelBookings, Perm::CompleteBookings,
                Perm::ViewServiceTasks, Perm::CreateServiceTasks, Perm::EditServiceTasks, Perm::CompleteServiceTasks, Perm::AssignServiceTasks,
                Perm::ViewChannels, Perm::CreateChannels, Perm::EditChannels,
                Perm::ViewDirectAssetLinking, Perm::EditDirectAssetLinking,
                Perm::ViewAssetGroupLinking, Perm::EditAssetGroupLinking,
                Perm::ViewPortfolioLinking, Perm::EditPortfolioLinking,
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
            scope: RoleScope::with_group(PermGroup::Reporting, true, true),
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
            scope: RoleScope::with_group(PermGroup::Transaction, true, true),
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
            scope: RoleScope::with_groups(&[
                (PermGroup::Portfolio, true, true),
                (PermGroup::Reporting, true, true),
                (PermGroup::Calendar, true, true),
                (PermGroup::Transaction, true, true),
            ]),
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
            name: "Channel Manager".to_string(),
            rank: 50,
            color: Some("#f97316".to_string()),
            description: "Can link channels to assets, edit channel settings, and manage test-channel bookings. Cannot approve financial transactions unless separately granted.".to_string(),
            scope: RoleScope::with_groups(&[
                (PermGroup::Organization, true, false),
                (PermGroup::Portfolio, true, false),
                (PermGroup::Networking, true, true),
                (PermGroup::AssetLinking, true, true),
                (PermGroup::Calendar, true, true),
                (PermGroup::Booking, true, true),
                (PermGroup::ServiceTasks, true, true),
                (PermGroup::History, true, false),
            ]),
            permissions: vec![
                Perm::ViewOrganization, Perm::ViewPortfolios,
                Perm::ViewChannels, Perm::CreateChannels, Perm::EditChannels, Perm::DeleteChannels,
                Perm::ViewDirectAssetLinking, Perm::EditDirectAssetLinking,
                Perm::ViewAssetGroupLinking, Perm::EditAssetGroupLinking,
                Perm::ViewPortfolioLinking, Perm::EditPortfolioLinking,
                Perm::ViewBookings, Perm::CreateBookings, Perm::EditBookings, Perm::CancelBookings, Perm::ManageBookings,
                Perm::ViewDirectAssetBookings, Perm::CreateDirectAssetBookings, Perm::EditDirectAssetBookings, Perm::CancelDirectAssetBookings,
                Perm::ViewAssetGroupBookings, Perm::CreateAssetGroupBookings, Perm::EditAssetGroupBookings, Perm::CancelAssetGroupBookings,
                Perm::ViewPortfolioBookings, Perm::CreatePortfolioBookings, Perm::EditPortfolioBookings, Perm::CancelPortfolioBookings,
                Perm::ViewCalendar, Perm::CreateCalendarEvents, Perm::EditOwnEvents,
                Perm::ViewServiceTasks, Perm::CreateServiceTasks, Perm::EditServiceTasks, Perm::AssignServiceTasks,
                Perm::ViewOwnActivityHistory, Perm::ViewPortfolioHistory, Perm::ViewAssetHistory,
            ],
            member_ids: Vec::new(),
            documents: Vec::new(),
            is_system: true,
        },
        OrgRole {
            id: Uuid::new_v4(),
            name: "Cleaner".to_string(),
            rank: 20,
            color: Some("#06b6d4".to_string()),
            description: "Can view assigned cleaning and service tasks, update task status, and view the calendar. Cannot edit bookings or channel settings.".to_string(),
            scope: RoleScope::with_groups(&[
                (PermGroup::Organization, true, false),
                (PermGroup::Portfolio, true, false),
                (PermGroup::ServiceTasks, true, true),
                (PermGroup::Calendar, true, false),
                (PermGroup::History, true, false),
            ]),
            permissions: vec![
                Perm::ViewOrganization, Perm::ViewPortfolios,
                Perm::ViewServiceTasks, Perm::CompleteServiceTasks, Perm::AssignServiceTasks,
                Perm::ViewCalendar,
                Perm::ViewOwnActivityHistory,
            ],
            member_ids: Vec::new(),
            documents: Vec::new(),
            is_system: true,
        },
        OrgRole {
            id: Uuid::new_v4(),
            name: "Finance Manager".to_string(),
            rank: 60,
            color: Some("#d946ef".to_string()),
            description: "Can view booking financials, reports, and approve transactions. Cannot manage channel connections unless separately granted.".to_string(),
            scope: RoleScope::with_groups(&[
                (PermGroup::Organization, true, false),
                (PermGroup::Portfolio, true, false),
                (PermGroup::Reporting, true, true),
                (PermGroup::Booking, true, false),
                (PermGroup::Transaction, true, true),
                (PermGroup::History, true, false),
            ]),
            permissions: vec![
                Perm::ViewOrganization, Perm::ViewPortfolios,
                Perm::ViewReports, Perm::CreateReports, Perm::EditOwnReports, Perm::ApproveReports,
                Perm::ViewBookings, Perm::ViewBookingFinancials,
                Perm::ViewDirectAssetBookings, Perm::ViewAssetGroupBookings,
                Perm::ViewPortfolioBookings, Perm::ViewOrgBookings,
                Perm::ViewTransactions, Perm::CreateTransactions, Perm::EditOwnDraftTransactions,
                Perm::SubmitTransactions, Perm::ApproveTransactions, Perm::RejectTransactions, Perm::LockFinalizedTransactions,
                Perm::ViewTransactionHistory, Perm::ViewTransactionHistoryLog,
                Perm::ViewOwnActivityHistory, Perm::ViewPortfolioHistory, Perm::ViewAssetHistory,
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
            scope: RoleScope::view_all(),
            permissions: vec![
                Perm::ViewOrganization, Perm::ViewPortfolios,
                Perm::ViewReports, Perm::ViewDocuments, Perm::ViewLockedDocuments,
                Perm::ViewCalendar, Perm::ViewTransactions,
                Perm::ViewBookings, Perm::ViewServiceTasks,
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
