use super::permissions::{default_permissions_for_role, Permission};
use crate::models::portfolio::Document;
use crate::types::UserProfile;
use uuid::Uuid;

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
