use crate::models::{OrgRole, Organization, Perm, RoleScope, User};
use crate::types::UserRole;
use leptos::prelude::*;
use uuid::Uuid;

/// Dedicated store for organization state, member management, and role/permission helpers.
/// Extracted from AppStore so organization lifecycle changes do not invalidate
/// consumers of unrelated domain state.
#[derive(Clone, Debug)]
pub struct OrganizationStore {
    pub organizations: Vec<Organization>,
    pub current_organization_id: Option<Uuid>,
    pub organization_users: Vec<User>,
}

impl Default for OrganizationStore {
    fn default() -> Self {
        Self {
            organizations: Vec::new(),
            current_organization_id: None,
            organization_users: Vec::new(),
        }
    }
}

impl OrganizationStore {
    pub fn new() -> Self {
        Self::default()
    }

    // Organization CRUD

    pub fn add_organization(&mut self, org: Organization) {
        self.organizations.push(org);
    }

    pub fn get_organization(&self, id: Uuid) -> Option<&Organization> {
        self.organizations.iter().find(|o| o.id == id)
    }

    pub fn get_organization_mut(&mut self, id: Uuid) -> Option<&mut Organization> {
        self.organizations.iter_mut().find(|o| o.id == id)
    }

    pub fn remove_organization(&mut self, id: Uuid) -> Option<Organization> {
        if let Some(pos) = self.organizations.iter().position(|o| o.id == id) {
            Some(self.organizations.remove(pos))
        } else {
            None
        }
    }

    pub fn switch_organization(&mut self, id: Uuid, current_user_name: &str) -> Option<UserRole> {
        if self.organizations.iter().any(|o| o.id == id) {
            self.current_organization_id = Some(id);
            // Return the user's role in the new organization
            if let Some(user) = self
                .organization_users
                .iter()
                .find(|u| u.organization_id == Some(id) && u.name == current_user_name)
            {
                return Some(user.role.clone());
            }
        }
        None
    }

    // Organization user management

    pub fn add_organization_user(&mut self, user: User) {
        self.organization_users.push(user);
    }

    pub fn remove_organization_user(&mut self, id: Uuid) -> Option<User> {
        if let Some(pos) = self.organization_users.iter().position(|u| u.id == id) {
            Some(self.organization_users.remove(pos))
        } else {
            None
        }
    }

    pub fn update_user_role(
        &mut self,
        id: Uuid,
        new_role: UserRole,
        current_user_id: Uuid,
    ) -> Result<(), String> {
        if let Some(pos) = self.organization_users.iter().position(|u| u.id == id) {
            let current_user = self
                .organization_users
                .iter()
                .find(|u| u.id == current_user_id)
                .cloned()
                .unwrap_or_else(|| {
                    User::new("Current".to_string(), String::new(), UserRole::Owner)
                });
            let user = &mut self.organization_users[pos];
            user.update_role(new_role, &current_user)
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn toggle_user_permission(&mut self, id: Uuid, permission: crate::models::Permission) {
        if let Some(user) = self.organization_users.iter_mut().find(|u| u.id == id) {
            user.toggle_permission(permission);
        }
    }

    pub fn update_user_name(&mut self, id: Uuid, name: String) -> Result<(), String> {
        if let Some(user) = self.organization_users.iter_mut().find(|u| u.id == id) {
            user.name = name;
            user.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn current_user_role_in_org(
        &self,
        org_id: Uuid,
        current_user_id: Uuid,
        fallback_role: UserRole,
    ) -> UserRole {
        self.organization_users
            .iter()
            .find(|u| u.id == current_user_id && u.organization_id == Some(org_id))
            .map(|u| u.role.clone())
            .unwrap_or(fallback_role)
    }

    // Discord-style role management

    pub fn add_role_to_org(&mut self, org_id: Uuid, role: OrgRole) {
        if let Some(org) = self.get_organization_mut(org_id) {
            org.roles.push(role);
            org.roles.sort_by(|a, b| b.rank.cmp(&a.rank));
            org.updated_at = chrono::Utc::now();
        }
    }

    pub fn update_org_role(
        &mut self,
        org_id: Uuid,
        role_id: Uuid,
        name: String,
        description: String,
        color: Option<String>,
        rank: u32,
        scope: RoleScope,
    ) {
        if let Some(org) = self.get_organization_mut(org_id) {
            if let Some(role) = org.roles.iter_mut().find(|r| r.id == role_id) {
                role.name = name;
                role.description = description;
                role.color = color;
                role.rank = rank;
                role.scope = scope;
            }
            org.roles.sort_by(|a, b| b.rank.cmp(&a.rank));
            org.updated_at = chrono::Utc::now();
        }
    }

    pub fn delete_org_role(&mut self, org_id: Uuid, role_id: Uuid) {
        if let Some(org) = self.get_organization_mut(org_id) {
            org.roles.retain(|r| r.id != role_id || r.is_system);
            org.updated_at = chrono::Utc::now();
        }
    }

    pub fn reorder_org_role(&mut self, org_id: Uuid, role_id: Uuid, new_rank: u32) {
        if let Some(org) = self.get_organization_mut(org_id) {
            if let Some(role) = org.roles.iter_mut().find(|r| r.id == role_id) {
                role.rank = new_rank;
            }
            org.roles.sort_by(|a, b| b.rank.cmp(&a.rank));
            org.updated_at = chrono::Utc::now();
        }
    }

    pub fn drag_role(&mut self, org_id: Uuid, dragged_id: Uuid, target_id: Uuid) {
        if dragged_id == target_id {
            return;
        }
        if let Some(org) = self.get_organization_mut(org_id) {
            let mut roles = org.roles.clone();
            let Some(dragged_pos) = roles.iter().position(|r| r.id == dragged_id) else {
                return;
            };
            let Some(target_pos) = roles.iter().position(|r| r.id == target_id) else {
                return;
            };
            let dragged = roles.remove(dragged_pos);
            let target_pos = if dragged_pos < target_pos {
                target_pos - 1
            } else {
                target_pos
            };
            roles.insert(target_pos, dragged);

            let base = 1000u32;
            for (i, role) in roles.iter_mut().enumerate() {
                role.rank = base.saturating_sub(i as u32 * 10);
            }
            org.roles = roles;
            org.updated_at = chrono::Utc::now();
        }
    }

    pub fn toggle_role_permission(&mut self, org_id: Uuid, role_id: Uuid, perm: Perm) {
        if let Some(org) = self.get_organization_mut(org_id) {
            if let Some(role) = org.roles.iter_mut().find(|r| r.id == role_id) {
                if role.permissions.contains(&perm) {
                    role.permissions.retain(|p| p != &perm);
                } else {
                    role.permissions.push(perm);
                }
            }
            org.updated_at = chrono::Utc::now();
        }
    }

    pub fn assign_member_to_role(&mut self, org_id: Uuid, role_id: Uuid, user_id: Uuid) {
        if let Some(org) = self.get_organization_mut(org_id) {
            if let Some(role) = org.roles.iter_mut().find(|r| r.id == role_id) {
                if !role.member_ids.contains(&user_id) {
                    role.member_ids.push(user_id);
                }
            }
            org.updated_at = chrono::Utc::now();
        }
    }

    pub fn remove_member_from_role(&mut self, org_id: Uuid, role_id: Uuid, user_id: Uuid) {
        if let Some(org) = self.get_organization_mut(org_id) {
            if let Some(role) = org.roles.iter_mut().find(|r| r.id == role_id) {
                role.member_ids.retain(|&id| id != user_id);
            }
            org.updated_at = chrono::Utc::now();
        }
    }

    pub fn dev_test_add_org_user(
        &mut self,
        name: &str,
        role: UserRole,
        notification_store: &mut crate::stores::notifications::NotificationStore,
    ) {
        let role_dbg = format!("{:?}", role);
        self.add_organization_user(User::new(
            name.into(),
            format!("{}@farley.test", name.to_lowercase()),
            role,
        ));
        notification_store.add_notification_for(
            format!("User: \"{}\" {}", name, role_dbg),
            crate::stores::notifications::NotificationType::Info,
            Some(crate::types::TabType::Organization),
            Some("System".into()),
        );
    }

    pub fn duplicate_org_role(&mut self, org_id: Uuid, role_id: Uuid) -> Option<Uuid> {
        let new_id = Uuid::new_v4();
        if let Some(org) = self.get_organization_mut(org_id) {
            if let Some(role) = org.roles.iter().find(|r| r.id == role_id).cloned() {
                let mut new_role = role;
                new_role.id = new_id;
                new_role.name = format!("{} (Copy)", new_role.name);
                new_role.is_system = false;
                new_role.member_ids = Vec::new();
                org.roles.push(new_role);
                org.roles.sort_by(|a, b| b.rank.cmp(&a.rank));
                org.updated_at = chrono::Utc::now();
                return Some(new_id);
            }
        }
        None
    }
}

pub fn create_organization_store() -> RwSignal<OrganizationStore> {
    RwSignal::new(OrganizationStore::new())
}

pub fn provide_organization_store() -> RwSignal<OrganizationStore> {
    let store = create_organization_store();
    provide_context(store);
    store
}

pub fn use_organization_store() -> RwSignal<OrganizationStore> {
    expect_context::<RwSignal<OrganizationStore>>()
}
