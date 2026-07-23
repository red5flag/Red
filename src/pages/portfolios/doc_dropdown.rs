use crate::models::Document;
use uuid::Uuid;

#[derive(Clone, PartialEq)]
pub struct DocEntry {
    pub doc: Document,
    pub portfolio_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
}
