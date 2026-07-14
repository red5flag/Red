use crate::models::Permission;
use crate::types::UserRole;

pub mod member_card;
pub mod members;
pub mod organization_card;
pub mod organization_forms;
pub mod organization_list;
pub mod organization_summary;
pub mod page;
pub mod permissions;
pub mod portfolio_access;
pub mod role_card;
pub mod roles;

pub use page::OrganizationPage;
pub(crate) use permissions::PermissionGroups;
pub(crate) use role_card::RoleCard;

pub(crate) fn permission_label(p: &Permission) -> &'static str {
    match p {
        Permission::ViewOwn => "View own",
        Permission::ViewOrganization => "View organization",
        Permission::ViewAll => "View all",
        Permission::CreateOwn => "Create own",
        Permission::CreateOrganization => "Create organization",
        Permission::EditOwn => "Edit own",
        Permission::EditOrganization => "Edit organization",
        Permission::EditAll => "Edit all",
        Permission::DeleteOwn => "Delete own",
        Permission::DeleteOrganization => "Delete organization",
        Permission::DeleteAll => "Delete all",
        Permission::ManageUsers => "Manage users",
        Permission::ManageRoles => "Manage roles",
        Permission::ManagePayments => "Manage payments",
        Permission::ManageSettings => "Manage settings",
        Permission::ExportData => "Export data",
        Permission::ImportData => "Import data",
        Permission::EditDocuments => "Edit documents",
        Permission::Custom(s) => return Box::leak(format!("Custom: {}", s).into_boxed_str()),
    }
}

pub(crate) fn role_from_str(s: &str) -> UserRole {
    match s {
        "Owner" => UserRole::Owner,
        "Director" => UserRole::Director,
        "SeniorManager" => UserRole::SeniorManager,
        "Manager" => UserRole::Manager,
        "Worker" => UserRole::Worker,
        "DocumentWorker" => UserRole::DocumentWorker,
        "Contractor" => UserRole::Contractor,
        _ => UserRole::Guest,
    }
}

pub(crate) fn role_display(role: &UserRole) -> &'static str {
    match role {
        UserRole::Owner => "Owner",
        UserRole::Director => "Director",
        UserRole::SeniorManager => "Senior Manager",
        UserRole::Manager => "Manager",
        UserRole::Worker => "Worker",
        UserRole::DocumentWorker => "Document Worker",
        UserRole::Contractor => "Contractor",
        UserRole::Guest => "Guest",
    }
}

pub(crate) fn scope_display(s: &crate::models::RoleScope) -> String {
    s.display()
}
