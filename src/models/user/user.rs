use super::activity::UserActivity;
use super::payment::PaymentSettings;
use super::permissions::{default_permissions_for_role, Permission};
use crate::types::{NotificationTrigger, NotificationType, UserRole};
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
