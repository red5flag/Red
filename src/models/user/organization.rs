use super::roles::{default_org_roles, OrgRole};
use crate::types::OrganizationSettings;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    // Business details (optional)
    pub abn: Option<String>,           // Australian Business Number
    pub lei: Option<String>,           // Legal Entity Identifier (global)
    pub business_type: Option<String>, // e.g., "Sole Trader", "Company", "Partnership", etc.
    pub business_address: Option<String>,
    pub business_phone: Option<String>,
    pub business_email: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
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
            abn: None,
            lei: None,
            business_type: None,
            business_address: None,
            business_phone: None,
            business_email: None,
            image_url: None,
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
