use crate::models::{Asset, AssetGroup, Document, EntityNotificationSetting, Portfolio};
use crate::stores::notifications::Notification;
use uuid::Uuid;

/// Stage 1A/1B/1C helper for portfolio-domain logic that currently lives in AppStore.
///
/// This helper does not own `Vec<Portfolio>` yet; it operates on borrowed slices
/// so AppStore can keep `portfolios` in place while logic is extracted.
#[derive(Clone, Debug, Default)]
pub struct PortfolioStore;

impl PortfolioStore {
    pub fn new() -> Self {
        Self
    }

    /// Get notification settings for a portfolio.
    pub fn portfolio_notification_settings(
        portfolios: &[Portfolio],
        pid: Uuid,
    ) -> Vec<EntityNotificationSetting> {
        portfolios
            .iter()
            .find(|p| p.id == pid)
            .map(|p| p.notification_settings.clone())
            .unwrap_or_default()
    }

    /// Get notification settings for an asset group.
    pub fn group_notification_settings(
        portfolios: &[Portfolio],
        pid: Uuid,
        gid: Uuid,
    ) -> Vec<EntityNotificationSetting> {
        portfolios
            .iter()
            .find(|p| p.id == pid)
            .and_then(|p| p.asset_groups.iter().find(|g| g.id == gid))
            .map(|g| g.notification_settings.clone())
            .unwrap_or_default()
    }

    /// Add a notification setting to a portfolio.
    pub fn add_portfolio_notification_setting(
        portfolios: &mut [Portfolio],
        pid: Uuid,
        setting: EntityNotificationSetting,
    ) {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == pid) {
            p.notification_settings.push(setting);
            p.updated_at = chrono::Utc::now();
        }
    }

    /// Add a notification setting to an asset group.
    pub fn add_group_notification_setting(
        portfolios: &mut [Portfolio],
        pid: Uuid,
        gid: Uuid,
        setting: EntityNotificationSetting,
    ) {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == pid) {
            if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                g.notification_settings.push(setting);
                g.updated_at = chrono::Utc::now();
            }
        }
    }

    /// Toggle the enabled state of a notification setting on a portfolio.
    pub fn toggle_portfolio_notification_setting(
        portfolios: &mut [Portfolio],
        pid: Uuid,
        setting_id: Uuid,
    ) {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == pid) {
            if let Some(s) = p
                .notification_settings
                .iter_mut()
                .find(|s| s.id == setting_id)
            {
                s.enabled = !s.enabled;
            }
            p.updated_at = chrono::Utc::now();
        }
    }

    /// Toggle the enabled state of a notification setting on a group.
    pub fn toggle_group_notification_setting(
        portfolios: &mut [Portfolio],
        pid: Uuid,
        gid: Uuid,
        setting_id: Uuid,
    ) {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == pid) {
            if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                if let Some(s) = g
                    .notification_settings
                    .iter_mut()
                    .find(|s| s.id == setting_id)
                {
                    s.enabled = !s.enabled;
                }
                g.updated_at = chrono::Utc::now();
            }
        }
    }

    /// Remove a notification setting from a portfolio.
    pub fn remove_portfolio_notification_setting(
        portfolios: &mut [Portfolio],
        pid: Uuid,
        setting_id: Uuid,
    ) {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == pid) {
            p.notification_settings.retain(|s| s.id != setting_id);
            p.updated_at = chrono::Utc::now();
        }
    }

    /// Remove a notification setting from a group.
    pub fn remove_group_notification_setting(
        portfolios: &mut [Portfolio],
        pid: Uuid,
        gid: Uuid,
        setting_id: Uuid,
    ) {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == pid) {
            if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                g.notification_settings.retain(|s| s.id != setting_id);
                g.updated_at = chrono::Utc::now();
            }
        }
    }

    /// Update a notification setting on a portfolio.
    pub fn update_portfolio_notification_setting(
        portfolios: &mut [Portfolio],
        pid: Uuid,
        setting: EntityNotificationSetting,
    ) {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == pid) {
            if let Some(s) = p
                .notification_settings
                .iter_mut()
                .find(|s| s.id == setting.id)
            {
                *s = setting;
            }
            p.updated_at = chrono::Utc::now();
        }
    }

    /// Update a notification setting on a group.
    pub fn update_group_notification_setting(
        portfolios: &mut [Portfolio],
        pid: Uuid,
        gid: Uuid,
        setting: EntityNotificationSetting,
    ) {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == pid) {
            if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                if let Some(s) = g
                    .notification_settings
                    .iter_mut()
                    .find(|s| s.id == setting.id)
                {
                    *s = setting;
                }
                g.updated_at = chrono::Utc::now();
            }
        }
    }

    // ── Stage 1B: read/query/touch helpers ─────────────────────────

    /// Find a document by ID across all portfolios, groups, and assets.
    pub fn find_document(portfolios: &[Portfolio], doc_id: Uuid) -> Option<Document> {
        for p in portfolios {
            if let Some(d) = p.documents.iter().find(|d| d.id == doc_id) {
                return Some(d.clone());
            }
            for g in &p.asset_groups {
                if let Some(d) = g.documents.iter().find(|d| d.id == doc_id) {
                    return Some(d.clone());
                }
                for a in &g.assets {
                    if let Some(d) = a.documents.iter().find(|d| d.id == doc_id) {
                        return Some(d.clone());
                    }
                }
            }
            for a in &p.assets {
                if let Some(d) = a.documents.iter().find(|d| d.id == doc_id) {
                    return Some(d.clone());
                }
            }
        }
        None
    }

    /// Count notifications linked to any document within a portfolio.
    pub fn doc_notifications_for_portfolio(
        portfolios: &[Portfolio],
        portfolio_id: Uuid,
        notifications: &[Notification],
    ) -> usize {
        let doc_ids: std::collections::HashSet<Uuid> =
            if let Some(p) = portfolios.iter().find(|p| p.id == portfolio_id) {
                let mut ids: std::collections::HashSet<Uuid> =
                    p.documents.iter().map(|d| d.id).collect();
                for g in &p.asset_groups {
                    for d in &g.documents {
                        ids.insert(d.id);
                    }
                    for a in &g.assets {
                        for d in &a.documents {
                            ids.insert(d.id);
                        }
                    }
                }
                for a in &p.assets {
                    for d in &a.documents {
                        ids.insert(d.id);
                    }
                }
                ids
            } else {
                std::collections::HashSet::new()
            };
        notifications
            .iter()
            .filter(|n| {
                n.linked_doc_id
                    .map(|id| doc_ids.contains(&id))
                    .unwrap_or(false)
            })
            .count()
    }

    /// Count notifications linked to any document within an asset group.
    pub fn doc_notifications_for_group(
        portfolios: &[Portfolio],
        portfolio_id: Uuid,
        group_id: Uuid,
        notifications: &[Notification],
    ) -> usize {
        let doc_ids: std::collections::HashSet<Uuid> =
            if let Some(p) = portfolios.iter().find(|p| p.id == portfolio_id) {
                if let Some(g) = p.asset_groups.iter().find(|g| g.id == group_id) {
                    let mut ids: std::collections::HashSet<Uuid> =
                        g.documents.iter().map(|d| d.id).collect();
                    for a in &g.assets {
                        for d in &a.documents {
                            ids.insert(d.id);
                        }
                    }
                    ids
                } else {
                    std::collections::HashSet::new()
                }
            } else {
                std::collections::HashSet::new()
            };
        notifications
            .iter()
            .filter(|n| {
                n.linked_doc_id
                    .map(|id| doc_ids.contains(&id))
                    .unwrap_or(false)
            })
            .count()
    }

    /// Count notifications linked to any document within an asset.
    pub fn doc_notifications_for_asset(
        portfolios: &[Portfolio],
        asset_id: Uuid,
        notifications: &[Notification],
    ) -> usize {
        let doc_ids: std::collections::HashSet<Uuid> = {
            let mut ids = std::collections::HashSet::new();
            for p in portfolios {
                for a in &p.assets {
                    if a.id == asset_id {
                        for d in &a.documents {
                            ids.insert(d.id);
                        }
                    }
                }
                for g in &p.asset_groups {
                    for a in &g.assets {
                        if a.id == asset_id {
                            for d in &a.documents {
                                ids.insert(d.id);
                            }
                        }
                    }
                }
            }
            ids
        };
        notifications
            .iter()
            .filter(|n| {
                n.linked_doc_id
                    .map(|id| doc_ids.contains(&id))
                    .unwrap_or(false)
            })
            .count()
    }

    /// Mark a portfolio as recently accessed.
    pub fn touch_portfolio(portfolios: &mut [Portfolio], id: Uuid) {
        let now = chrono::Utc::now();
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == id) {
            p.last_accessed_at = now;
        }
    }

    /// Mark an asset as recently accessed.
    pub fn touch_asset(portfolios: &mut [Portfolio], id: Uuid) {
        let now = chrono::Utc::now();
        for p in portfolios.iter_mut() {
            for a in p.assets.iter_mut() {
                if a.id == id {
                    a.last_accessed_at = now;
                    return;
                }
            }
            for g in p.asset_groups.iter_mut() {
                for a in g.assets.iter_mut() {
                    if a.id == id {
                        a.last_accessed_at = now;
                        return;
                    }
                }
            }
        }
    }

    // ── Stage 1C: safe portfolio CRUD helpers ──────────────────────

    /// Find a portfolio by ID (immutable).
    pub fn get_portfolio(portfolios: &[Portfolio], id: Uuid) -> Option<&Portfolio> {
        portfolios.iter().find(|p| p.id == id)
    }

    /// Find a portfolio by ID (mutable).
    pub fn get_portfolio_mut(portfolios: &mut [Portfolio], id: Uuid) -> Option<&mut Portfolio> {
        portfolios.iter_mut().find(|p| p.id == id)
    }

    /// Remove a portfolio by ID, returning it if found.
    pub fn remove_portfolio(portfolios: &mut Vec<Portfolio>, id: Uuid) -> Option<Portfolio> {
        if let Some(pos) = portfolios.iter().position(|p| p.id == id) {
            Some(portfolios.remove(pos))
        } else {
            None
        }
    }

    /// Remove an asset group from a portfolio.
    pub fn remove_asset_group(
        portfolios: &mut [Portfolio],
        portfolio_id: Uuid,
        group_id: Uuid,
    ) -> bool {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == portfolio_id) {
            let before = p.asset_groups.len();
            p.asset_groups.retain(|g| g.id != group_id);
            p.asset_groups.len() < before
        } else {
            false
        }
    }

    /// Remove an asset from a portfolio (direct assets or inside groups).
    pub fn remove_asset(portfolios: &mut [Portfolio], portfolio_id: Uuid, asset_id: Uuid) -> bool {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == portfolio_id) {
            let before = p.assets.len();
            p.assets.retain(|a| a.id != asset_id);
            let removed_direct = p.assets.len() < before;
            let mut removed_from_group = false;
            for g in &mut p.asset_groups {
                let g_before = g.assets.len();
                g.assets.retain(|a| a.id != asset_id);
                if g.assets.len() < g_before {
                    removed_from_group = true;
                }
            }
            removed_direct || removed_from_group
        } else {
            false
        }
    }

    /// Remove a document from an asset within a portfolio.
    pub fn remove_document_from_asset(
        portfolios: &mut [Portfolio],
        portfolio_id: Uuid,
        asset_id: Uuid,
        doc_id: Uuid,
    ) -> bool {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == portfolio_id) {
            for a in &mut p.assets {
                if a.id == asset_id {
                    let before = a.documents.len();
                    a.documents.retain(|d| d.id != doc_id);
                    return a.documents.len() < before;
                }
            }
            for g in &mut p.asset_groups {
                for a in &mut g.assets {
                    if a.id == asset_id {
                        let before = a.documents.len();
                        a.documents.retain(|d| d.id != doc_id);
                        return a.documents.len() < before;
                    }
                }
            }
        }
        false
    }

    // ── Move helpers ──────────────────────────────────────────────────────────

    /// Move a direct asset or an asset inside a group to another portfolio as a direct asset.
    pub fn move_asset_to_portfolio(
        portfolios: &mut [Portfolio],
        asset_id: Uuid,
        target_pid: Uuid,
    ) -> bool {
        let source_pid = portfolios.iter().find_map(|p| {
            if p.assets.iter().any(|a| a.id == asset_id) || p.asset_groups.iter().any(|g| g.assets.iter().any(|a| a.id == asset_id)) {
                Some(p.id)
            } else {
                None
            }
        });
        let Some(source_pid) = source_pid else { return false; };
        if source_pid == target_pid {
            // ensure it is a direct asset
            let already_direct = portfolios
                .iter()
                .find(|p| p.id == source_pid)
                .map(|p| p.assets.iter().any(|a| a.id == asset_id))
                .unwrap_or(false);
            return already_direct;
        }

        let mut asset: Option<Asset> = None;
        {
            let source = portfolios.iter_mut().find(|p| p.id == source_pid).unwrap();
            if let Some(pos) = source.assets.iter().position(|a| a.id == asset_id) {
                asset = Some(source.assets.remove(pos));
            } else if let Some(gpos) = source.asset_groups.iter().position(|g| g.assets.iter().any(|a| a.id == asset_id)) {
                if let Some(apos) = source.asset_groups[gpos].assets.iter().position(|a| a.id == asset_id) {
                    asset = Some(source.asset_groups[gpos].assets.remove(apos));
                    source.asset_groups[gpos].recalculate_values();
                }
            }
            if asset.is_some() {
                source.recalculate_values();
            }
        }
        let Some(asset) = asset else { return false; };
        let target = portfolios.iter_mut().find(|p| p.id == target_pid).unwrap();
        target.assets.push(asset);
        target.recalculate_values();
        true
    }

    /// Move a direct asset or an asset inside a group into a specific asset group.
    pub fn move_asset_to_group(
        portfolios: &mut [Portfolio],
        asset_id: Uuid,
        target_pid: Uuid,
        target_gid: Uuid,
    ) -> bool {
        let source_pid = portfolios.iter().find_map(|p| {
            if p.assets.iter().any(|a| a.id == asset_id) || p.asset_groups.iter().any(|g| g.assets.iter().any(|a| a.id == asset_id)) {
                Some(p.id)
            } else {
                None
            }
        });
        let Some(source_pid) = source_pid else { return false; };

        let mut asset: Option<Asset> = None;
        {
            let source = portfolios.iter_mut().find(|p| p.id == source_pid).unwrap();
            if let Some(pos) = source.assets.iter().position(|a| a.id == asset_id) {
                asset = Some(source.assets.remove(pos));
            } else if let Some(gpos) = source.asset_groups.iter().position(|g| g.assets.iter().any(|a| a.id == asset_id)) {
                if source_pid == target_pid && source.asset_groups[gpos].id == target_gid {
                    // already in the target group
                    return true;
                }
                if let Some(apos) = source.asset_groups[gpos].assets.iter().position(|a| a.id == asset_id) {
                    asset = Some(source.asset_groups[gpos].assets.remove(apos));
                    source.asset_groups[gpos].recalculate_values();
                }
            }
            if asset.is_some() {
                source.recalculate_values();
            }
        }
        let Some(asset) = asset else { return false; };
        let target = portfolios.iter_mut().find(|p| p.id == target_pid).unwrap();
        if let Some(g) = target.asset_groups.iter_mut().find(|g| g.id == target_gid) {
            g.assets.push(asset);
            g.recalculate_values();
            target.recalculate_values();
            true
        } else {
            false
        }
    }

    /// Move an asset group (and its assets) to another portfolio.
    pub fn move_group_to_portfolio(
        portfolios: &mut [Portfolio],
        group_id: Uuid,
        target_pid: Uuid,
    ) -> bool {
        let source_pid = portfolios.iter().find_map(|p| {
            if p.asset_groups.iter().any(|g| g.id == group_id) {
                Some(p.id)
            } else {
                None
            }
        });
        let Some(source_pid) = source_pid else { return false; };
        if source_pid == target_pid {
            return true;
        }

        let mut group: Option<AssetGroup> = None;
        {
            let source = portfolios.iter_mut().find(|p| p.id == source_pid).unwrap();
            if let Some(pos) = source.asset_groups.iter().position(|g| g.id == group_id) {
                group = Some(source.asset_groups.remove(pos));
                source.recalculate_values();
            }
        }
        let Some(group) = group else { return false; };
        let target = portfolios.iter_mut().find(|p| p.id == target_pid).unwrap();
        target.asset_groups.push(group);
        target.recalculate_values();
        true
    }

    // ── Organization helpers ────────────────────────────────────────────────

    pub fn set_portfolio_organization(
        portfolios: &mut [Portfolio],
        portfolio_id: Uuid,
        org_id: Option<Uuid>,
    ) {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == portfolio_id) {
            p.organization_id = org_id;
            p.updated_at = chrono::Utc::now();
        }
    }

    pub fn add_portfolio_secondary_org(
        portfolios: &mut [Portfolio],
        portfolio_id: Uuid,
        org_id: Uuid,
    ) {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == portfolio_id) {
            if !p.secondary_organization_ids.contains(&org_id) {
                p.secondary_organization_ids.push(org_id);
            }
            p.updated_at = chrono::Utc::now();
        }
    }

    pub fn remove_portfolio_secondary_org(
        portfolios: &mut [Portfolio],
        portfolio_id: Uuid,
        org_id: Uuid,
    ) {
        if let Some(p) = portfolios.iter_mut().find(|p| p.id == portfolio_id) {
            p.secondary_organization_ids.retain(|&id| id != org_id);
            p.updated_at = chrono::Utc::now();
        }
    }

    pub fn set_asset_organization(
        portfolios: &mut [Portfolio],
        asset_id: Uuid,
        org_id: Option<Uuid>,
    ) {
        for p in portfolios.iter_mut() {
            for a in &mut p.assets {
                if a.id == asset_id {
                    a.organization_id = org_id;
                    p.updated_at = chrono::Utc::now();
                    return;
                }
            }
            for g in &mut p.asset_groups {
                for a in &mut g.assets {
                    if a.id == asset_id {
                        a.organization_id = org_id;
                        p.updated_at = chrono::Utc::now();
                        return;
                    }
                }
            }
        }
    }

    pub fn set_asset_group_organization(
        portfolios: &mut [Portfolio],
        group_id: Uuid,
        org_id: Option<Uuid>,
    ) {
        for p in portfolios.iter_mut() {
            if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == group_id) {
                g.organization_id = org_id;
                p.updated_at = chrono::Utc::now();
                return;
            }
        }
    }
}
