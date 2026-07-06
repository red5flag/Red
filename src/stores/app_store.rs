use crate::models::{Document, Portfolio};
use crate::stores::credentials::{CredentialStore, StoredCredential};
use crate::stores::notifications::{Notification, NotificationStore, NotificationType};
use crate::stores::seed_data::seed_portfolio_2;
use crate::types::{TabType, UserProfile, UserRole};
use crate::utils::crypto;
use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use serde::Serialize;
use std::collections::HashSet;
use uuid::Uuid;

// Main application state store
#[derive(Clone, Debug)]
pub struct AppStore {
    // Current user
    pub current_user: UserProfile,
    // Currently active tabs (single unless edit mode is on)
    pub active_tabs: Vec<TabType>,
    // Tabs that have their edit pen enabled (per-tab edit state)
    pub edit_mode_tabs: HashSet<TabType>,
    // All portfolios
    pub portfolios: Vec<Portfolio>,
    // Selected portfolio/asset IDs
    pub selected_portfolio_id: Option<Uuid>,
    pub selected_asset_group_id: Option<Uuid>,
    pub selected_asset_id: Option<Uuid>,
    /// When set, PortfoliosPage should expand this portfolio, expand the group,
    /// and open the doc modal for the asset/doc — used by notification click navigation.
    pub pending_nav_target: Option<PendingNavTarget>,
    /// When set, PortfolioListItem should expand this group (set by notification navigation).
    pub pending_group_expand: Option<Uuid>,
    // Authentication state
    pub is_authenticated: bool,
    // Credential store for password verification
    pub credentials: CredentialStore,
    // Notifications drawer state moved to NotificationStore in Phase A
    // Developer mode (for testing notifications, etc.)
    pub developer_mode: bool,
}

/// Navigation target for jumping to a specific portfolio/group/asset/doc from a notification click.
#[derive(Clone, Debug, PartialEq)]
pub struct PendingNavTarget {
    pub portfolio_id: Uuid,
    pub group_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub doc_id: Option<Uuid>,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Serialize)]
struct TabState {
    active_tabs: Vec<TabType>,
    edit_mode_tabs: HashSet<TabType>,
}

impl AppStore {
    /// Persist the current tab state to localStorage (hydrate only).
    #[cfg(feature = "hydrate")]
    fn save_tab_state(&self) {
        use web_sys::window;
        let state = TabState {
            active_tabs: self.active_tabs.clone(),
            edit_mode_tabs: self.edit_mode_tabs.clone(),
        };
        if let Ok(json) = serde_json::to_string(&state) {
            if let Some(window) = window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("farley_tab_state", &json);
                }
            }
        }
    }
}

impl Default for AppStore {
    fn default() -> Self {
        #[allow(unused_mut)]
        let mut credentials = CredentialStore::with_defaults();

        #[cfg(feature = "hydrate")]
        credentials.merge_from_local_storage();

        Self {
            current_user: UserProfile::default(),
            active_tabs: vec![TabType::Overview],
            edit_mode_tabs: HashSet::new(),
            portfolios: Vec::new(),
            selected_portfolio_id: None,
            selected_asset_group_id: None,
            selected_asset_id: None,
            pending_nav_target: None,
            pending_group_expand: None,
            is_authenticated: false,
            credentials,
            developer_mode: false,
        }
    }
}

impl AppStore {
    pub fn new() -> Self {
        Self::default()
    }

    // Tab management
    /// True if any currently active tab is in edit mode.
    fn is_any_tab_editing(&self) -> bool {
        self.active_tabs.iter().any(|t| self.is_tab_edit_mode(t))
    }

    pub fn expand_tab(&mut self, tab: TabType) {
        if self.is_any_tab_editing() {
            if !self.active_tabs.contains(&tab) {
                self.active_tabs.push(tab);
            }
        } else {
            self.active_tabs = vec![tab];
        }
        #[cfg(feature = "hydrate")]
        self.save_tab_state();
    }

    pub fn collapse_tab(&mut self, tab: &TabType) {
        self.active_tabs.retain(|t| t != tab);
        #[cfg(feature = "hydrate")]
        self.save_tab_state();
    }

    pub fn collapse_all_tabs(&mut self) {
        self.active_tabs.clear();
        #[cfg(feature = "hydrate")]
        self.save_tab_state();
    }

    pub fn toggle_tab(&mut self, tab: TabType) {
        if self.is_any_tab_editing() {
            if self.active_tabs.contains(&tab) {
                self.active_tabs.retain(|t| t != &tab);
            } else {
                self.active_tabs.push(tab);
            }
        } else {
            self.active_tabs = vec![tab];
        }
        #[cfg(feature = "hydrate")]
        self.save_tab_state();
    }

    pub fn is_tab_expanded(&self, tab: &TabType) -> bool {
        self.active_tabs.contains(tab)
    }

    pub fn toggle_tab_edit_mode(&mut self, tab: &TabType) -> bool {
        let result = if self.edit_mode_tabs.contains(tab) {
            self.edit_mode_tabs.remove(tab);
            false
        } else {
            self.edit_mode_tabs.insert(tab.clone());
            true
        };
        #[cfg(feature = "hydrate")]
        self.save_tab_state();
        result
    }

    pub fn is_tab_edit_mode(&self, tab: &TabType) -> bool {
        self.edit_mode_tabs.contains(tab)
    }

    pub fn clear_tab_edit_modes(&mut self) {
        self.edit_mode_tabs.clear();
        #[cfg(feature = "hydrate")]
        self.save_tab_state();
    }

    // Portfolio management
    pub fn add_portfolio(
        &mut self,
        portfolio: Portfolio,
        notification_store: &mut NotificationStore,
    ) {
        let pname = portfolio.name.clone();
        let pid = portfolio.id;
        self.portfolios.push(portfolio);
        notification_store.add_notification_for(
            format!("Portfolio \"{}\" created — ready for review.", pname),
            NotificationType::Success,
            Some(TabType::Portfolios),
            Some(self.current_user.name.clone()),
        );
        let _ = pid;
    }

    pub fn get_portfolio(&self, id: Uuid) -> Option<&Portfolio> {
        self.portfolios.iter().find(|p| p.id == id)
    }

    pub fn get_portfolio_mut(&mut self, id: Uuid) -> Option<&mut Portfolio> {
        self.portfolios.iter_mut().find(|p| p.id == id)
    }

    pub fn set_portfolio_name(
        &mut self,
        id: Uuid,
        name: String,
        notification_store: &mut NotificationStore,
    ) {
        if let Some(p) = self.get_portfolio_mut(id) {
            p.name = name.clone();
            p.updated_at = chrono::Utc::now();
        }
        notification_store.add_notification_for(
            format!(
                "Portfolio renamed to \"{}\" — changes pending review.",
                name
            ),
            NotificationType::Warning,
            Some(TabType::Portfolios),
            Some(self.current_user.name.clone()),
        );
    }

    pub fn remove_portfolio(&mut self, id: Uuid) -> Option<Portfolio> {
        if let Some(pos) = self.portfolios.iter().position(|p| p.id == id) {
            Some(self.portfolios.remove(pos))
        } else {
            None
        }
    }

    pub fn remove_asset_group(&mut self, portfolio_id: Uuid, group_id: Uuid) -> bool {
        if let Some(p) = self.get_portfolio_mut(portfolio_id) {
            let before = p.asset_groups.len();
            p.asset_groups.retain(|g| g.id != group_id);
            p.asset_groups.len() < before
        } else {
            false
        }
    }

    pub fn remove_asset(&mut self, portfolio_id: Uuid, asset_id: Uuid) -> bool {
        if let Some(p) = self.get_portfolio_mut(portfolio_id) {
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

    pub fn remove_document_from_asset(
        &mut self,
        portfolio_id: Uuid,
        asset_id: Uuid,
        doc_id: Uuid,
    ) -> bool {
        if let Some(p) = self.get_portfolio_mut(portfolio_id) {
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

    pub fn add_document_to_portfolio(
        &mut self,
        portfolio_id: Uuid,
        doc: crate::models::Document,
        notification_store: &mut NotificationStore,
    ) {
        let dname = doc.name.clone();
        let doc_id = doc.id;
        let uploader = self.current_user.name.clone();
        if let Some(p) = self.get_portfolio_mut(portfolio_id) {
            p.documents.push(doc);
            p.updated_at = chrono::Utc::now();
        }
        notification_store.add_document_notification(
            doc_id,
            &dname,
            &uploader.clone(),
            &format!(
                "Document \"{}\" added to portfolio — pending review.",
                dname
            ),
            NotificationType::Info,
            None,
            Some(uploader),
            Some(portfolio_id),
            None,
            None,
        );
    }

    pub fn update_document_name(
        &mut self,
        doc_id: Uuid,
        new_name: String,
        notification_store: &mut NotificationStore,
    ) {
        let mut found = false;
        let mut origin_pid: Option<Uuid> = None;
        let mut origin_gid: Option<Uuid> = None;
        let mut origin_aid: Option<Uuid> = None;
        for p in self.portfolios.iter_mut() {
            for d in &mut p.documents {
                if d.id == doc_id {
                    d.name = new_name.clone();
                    p.updated_at = chrono::Utc::now();
                    found = true;
                    origin_pid = Some(p.id);
                    break;
                }
            }
            if found {
                continue;
            }
            for g in &mut p.asset_groups {
                for d in &mut g.documents {
                    if d.id == doc_id {
                        d.name = new_name.clone();
                        p.updated_at = chrono::Utc::now();
                        found = true;
                        origin_pid = Some(p.id);
                        origin_gid = Some(g.id);
                        break;
                    }
                }
                if found {
                    continue;
                }
                for a in &mut g.assets {
                    for d in &mut a.documents {
                        if d.id == doc_id {
                            d.name = new_name.clone();
                            p.updated_at = chrono::Utc::now();
                            found = true;
                            origin_pid = Some(p.id);
                            origin_gid = Some(g.id);
                            origin_aid = Some(a.id);
                            break;
                        }
                    }
                }
            }
            if found {
                continue;
            }
            for a in &mut p.assets {
                for d in &mut a.documents {
                    if d.id == doc_id {
                        d.name = new_name.clone();
                        p.updated_at = chrono::Utc::now();
                        found = true;
                        origin_pid = Some(p.id);
                        origin_aid = Some(a.id);
                        break;
                    }
                }
            }
        }
        if found {
            let user_name = self.current_user.name.clone();
            notification_store.add_document_notification(
                doc_id,
                &new_name,
                &user_name.clone(),
                &format!("Document renamed to \"{}\" — review requested.", new_name),
                NotificationType::Warning,
                None,
                Some(user_name),
                origin_pid,
                origin_gid,
                origin_aid,
            );
        }
    }

    pub fn update_document_file_type(&mut self, doc_id: Uuid, new_file_type: String) {
        for p in self.portfolios.iter_mut() {
            for d in &mut p.documents {
                if d.id == doc_id {
                    d.file_type = new_file_type.clone();
                    p.updated_at = chrono::Utc::now();
                    return;
                }
            }
            for g in &mut p.asset_groups {
                for d in &mut g.documents {
                    if d.id == doc_id {
                        d.file_type = new_file_type.clone();
                        p.updated_at = chrono::Utc::now();
                        return;
                    }
                }
                for a in &mut g.assets {
                    for d in &mut a.documents {
                        if d.id == doc_id {
                            d.file_type = new_file_type.clone();
                            p.updated_at = chrono::Utc::now();
                            return;
                        }
                    }
                }
            }
            for a in &mut p.assets {
                for d in &mut a.documents {
                    if d.id == doc_id {
                        d.file_type = new_file_type.clone();
                        p.updated_at = chrono::Utc::now();
                        return;
                    }
                }
            }
        }
    }

    /// Mark a portfolio as recently accessed.
    pub fn touch_portfolio(&mut self, id: Uuid) {
        let now = chrono::Utc::now();
        if let Some(p) = self.get_portfolio_mut(id) {
            p.last_accessed_at = now;
        }
    }

    /// Mark an asset as recently accessed.
    pub fn touch_asset(&mut self, id: Uuid) {
        let now = chrono::Utc::now();
        for p in self.portfolios.iter_mut() {
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

    // Portfolio-level calendar event synchronization helpers.
    // These keep portfolio/asset/group embedded calendar_events in sync with
    // CalendarStore, which now owns the top-level calendar_events list.
    pub fn sync_calendar_event_to_portfolios(&mut self, event: crate::models::CalendarEvent) {
        if let Some(pid) = event.related_portfolio_id {
            if let Some(p) = self.portfolios.iter_mut().find(|p| p.id == pid) {
                let mut found = false;
                for e in &mut p.calendar_events {
                    if e.id == event.id {
                        *e = event.clone();
                        found = true;
                        break;
                    }
                }
                if !found {
                    p.calendar_events.push(event.clone());
                }
                p.updated_at = chrono::Utc::now();
            }
            return;
        }

        if let Some(gid) = event.related_group_id {
            for p in self.portfolios.iter_mut() {
                if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                    let mut found = false;
                    for e in &mut g.calendar_events {
                        if e.id == event.id {
                            *e = event.clone();
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        g.calendar_events.push(event.clone());
                    }
                    g.updated_at = chrono::Utc::now();
                    p.updated_at = chrono::Utc::now();
                    break;
                }
            }
            return;
        }

        if let Some(aid) = event.related_asset_id {
            for p in self.portfolios.iter_mut() {
                let all_assets: Vec<&mut crate::models::Asset> = p
                    .assets
                    .iter_mut()
                    .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                    .collect();
                for a in all_assets {
                    if a.id == aid {
                        let mut found = false;
                        for e in &mut a.calendar_events {
                            if e.id == event.id {
                                *e = event.clone();
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            a.calendar_events.push(event.clone());
                        }
                        p.updated_at = chrono::Utc::now();
                        break;
                    }
                }
            }
        }
    }

    pub fn remove_calendar_event_from_portfolios(&mut self, event_id: Uuid) {
        for p in self.portfolios.iter_mut() {
            p.calendar_events.retain(|e| e.id != event_id);
            for g in p.asset_groups.iter_mut() {
                g.calendar_events.retain(|e| e.id != event_id);
                for a in g.assets.iter_mut() {
                    a.calendar_events.retain(|e| e.id != event_id);
                }
            }
            for a in p.assets.iter_mut() {
                a.calendar_events.retain(|e| e.id != event_id);
            }
            p.updated_at = chrono::Utc::now();
        }
    }

    // Notification lifecycle and query methods moved to NotificationStore in Phase B

    /// Navigate to the origin of a notification — expands the portfolio, group,
    /// and opens the doc modal for the asset/document that the notification originated from.
    pub fn navigate_to_notification(&mut self, n: &Notification) {
        let tab = n.target_tab.clone();
        if let Some(pid) = n.linked_portfolio_id.or_else(|| {
            // Try to find the portfolio that contains the linked doc
            n.linked_doc_id.and_then(|did| {
                self.portfolios
                    .iter()
                    .find(|p| {
                        p.documents.iter().any(|d| d.id == did)
                            || p.asset_groups
                                .iter()
                                .any(|g| g.documents.iter().any(|d| d.id == did))
                            || p.assets
                                .iter()
                                .any(|a| a.documents.iter().any(|d| d.id == did))
                            || p.asset_groups.iter().any(|g| {
                                g.assets
                                    .iter()
                                    .any(|a| a.documents.iter().any(|d| d.id == did))
                            })
                    })
                    .map(|p| p.id)
            })
        }) {
            self.pending_nav_target = Some(PendingNavTarget {
                portfolio_id: pid,
                group_id: n.linked_group_id,
                asset_id: n.linked_asset_id,
                doc_id: n.linked_doc_id,
            });
            // Auto-expand the group if the notification originated from within a group
            if let Some(gid) = n.linked_group_id {
                self.pending_group_expand = Some(gid);
            }
            // Always switch to Portfolios tab — that's where the document lives and
            // where the pending_nav_target Effect will open the doc modal.
            self.expand_tab(TabType::Portfolios);
        } else if let Some(tab) = tab {
            // No linked portfolio — just switch to the tab
            self.expand_tab(tab);
        }
    }

    /// Count notifications linked to any document within a portfolio
    /// (portfolio-level docs, group docs, asset docs).
    pub fn doc_notifications_for_portfolio(
        &self,
        portfolio_id: Uuid,
        notifications: &[Notification],
    ) -> usize {
        let doc_ids: std::collections::HashSet<Uuid> =
            if let Some(p) = self.portfolios.iter().find(|p| p.id == portfolio_id) {
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
        &self,
        portfolio_id: Uuid,
        group_id: Uuid,
        notifications: &[Notification],
    ) -> usize {
        let doc_ids: std::collections::HashSet<Uuid> =
            if let Some(p) = self.portfolios.iter().find(|p| p.id == portfolio_id) {
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
        &self,
        asset_id: Uuid,
        notifications: &[Notification],
    ) -> usize {
        let doc_ids: std::collections::HashSet<Uuid> = {
            let mut ids = std::collections::HashSet::new();
            for p in &self.portfolios {
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

    /// Find a document by ID across all portfolios, groups, and assets.
    pub fn find_document(&self, doc_id: Uuid) -> Option<Document> {
        for p in &self.portfolios {
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

    // ── Entity notification settings ──────────────────────────────

    /// Get notification settings for a portfolio.
    pub fn portfolio_notification_settings(
        &self,
        pid: Uuid,
    ) -> Vec<crate::models::EntityNotificationSetting> {
        self.portfolios
            .iter()
            .find(|p| p.id == pid)
            .map(|p| p.notification_settings.clone())
            .unwrap_or_default()
    }

    /// Get notification settings for an asset group.
    pub fn group_notification_settings(
        &self,
        pid: Uuid,
        gid: Uuid,
    ) -> Vec<crate::models::EntityNotificationSetting> {
        self.portfolios
            .iter()
            .find(|p| p.id == pid)
            .and_then(|p| p.asset_groups.iter().find(|g| g.id == gid))
            .map(|g| g.notification_settings.clone())
            .unwrap_or_default()
    }

    /// Add a notification setting to a portfolio.
    pub fn add_portfolio_notification_setting(
        &mut self,
        pid: Uuid,
        setting: crate::models::EntityNotificationSetting,
    ) {
        if let Some(p) = self.get_portfolio_mut(pid) {
            p.notification_settings.push(setting);
            p.updated_at = chrono::Utc::now();
        }
    }

    /// Add a notification setting to an asset group.
    pub fn add_group_notification_setting(
        &mut self,
        pid: Uuid,
        gid: Uuid,
        setting: crate::models::EntityNotificationSetting,
    ) {
        if let Some(p) = self.get_portfolio_mut(pid) {
            if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                g.notification_settings.push(setting);
                g.updated_at = chrono::Utc::now();
            }
        }
    }

    /// Toggle the enabled state of a notification setting on a portfolio.
    pub fn toggle_portfolio_notification_setting(&mut self, pid: Uuid, setting_id: Uuid) {
        if let Some(p) = self.get_portfolio_mut(pid) {
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
    pub fn toggle_group_notification_setting(&mut self, pid: Uuid, gid: Uuid, setting_id: Uuid) {
        if let Some(p) = self.get_portfolio_mut(pid) {
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
    pub fn remove_portfolio_notification_setting(&mut self, pid: Uuid, setting_id: Uuid) {
        if let Some(p) = self.get_portfolio_mut(pid) {
            p.notification_settings.retain(|s| s.id != setting_id);
            p.updated_at = chrono::Utc::now();
        }
    }

    /// Remove a notification setting from a group.
    pub fn remove_group_notification_setting(&mut self, pid: Uuid, gid: Uuid, setting_id: Uuid) {
        if let Some(p) = self.get_portfolio_mut(pid) {
            if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                g.notification_settings.retain(|s| s.id != setting_id);
                g.updated_at = chrono::Utc::now();
            }
        }
    }

    /// Update a notification setting on a portfolio.
    pub fn update_portfolio_notification_setting(
        &mut self,
        pid: Uuid,
        setting: crate::models::EntityNotificationSetting,
    ) {
        if let Some(p) = self.get_portfolio_mut(pid) {
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
        &mut self,
        pid: Uuid,
        gid: Uuid,
        setting: crate::models::EntityNotificationSetting,
    ) {
        if let Some(p) = self.get_portfolio_mut(pid) {
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

    /// Check if the current user can manage notification recipients (requires ManageUsers or ManageRoles).
    pub fn can_manage_notification_recipients(&self) -> bool {
        self.current_user
            .has_permission(crate::models::Permission::ManageUsers)
            || self
                .current_user
                .has_permission(crate::models::Permission::ManageRoles)
    }

    pub fn toggle_developer_mode(&mut self) {
        self.developer_mode = !self.developer_mode;
    }

    pub fn dev_test_add_document(
        &mut self,
        doc_name: &str,
        file_type: &str,
        notification_store: &mut NotificationStore,
    ) -> Option<Uuid> {
        if !self.developer_mode {
            return None;
        }
        let doc = crate::models::Document {
            id: Uuid::new_v4(),
            name: doc_name.into(),
            file_type: file_type.into(),
            url: "#".into(),
            uploaded_at: chrono::Utc::now(),
            uploaded_by: self.current_user.id,
            content: None,
        };
        let doc_id = doc.id;
        let origin_pid: Option<Uuid>;
        if let Some(p) = self.portfolios.first() {
            origin_pid = Some(p.id);
            self.add_document_to_portfolio(p.id, doc, notification_store);
        } else {
            // No portfolio exists — create one then add the doc
            let mut new_p = Portfolio::new(
                "Dev Test Portfolio".into(),
                self.current_user.id,
                crate::types::Currency::USD,
            );
            let pid = new_p.id;
            origin_pid = Some(pid);
            new_p.documents.push(doc);
            self.add_portfolio(new_p, notification_store);
        }
        notification_store.add_document_notification(
            doc_id,
            doc_name,
            "red",
            &format!("Bot requested review of document \"{}\"", doc_name),
            NotificationType::Warning,
            Some(TabType::Reporting),
            Some("Bot".into()),
            origin_pid,
            None,
            None,
        );
        Some(doc_id)
    }

    pub fn dev_test_update_document(
        &mut self,
        doc_id: Uuid,
        new_name: &str,
        notification_store: &mut NotificationStore,
    ) {
        if !self.developer_mode {
            return;
        }
        self.update_document_name(doc_id, new_name.into(), notification_store);
        notification_store.add_document_notification(
            doc_id,
            new_name,
            "red",
            &format!("Bot updated document for review: \"{}\"", new_name),
            NotificationType::Warning,
            Some(TabType::Reporting),
            Some("Bot".into()),
            None,
            None,
            None,
        );
    }

    pub fn dev_test_add_portfolio(
        &mut self,
        name: &str,
        notification_store: &mut NotificationStore,
    ) -> Option<Uuid> {
        if !self.developer_mode {
            return None;
        }
        let mut p = Portfolio::new(
            name.into(),
            self.current_user.id,
            crate::types::Currency::USD,
        );
        p.description = Some("Dev test".into());
        let pid = p.id;
        self.add_portfolio(p, notification_store);
        notification_store.add_notification_for(
            format!("Bot requested review of portfolio \"{}\"", name),
            NotificationType::Warning,
            Some(TabType::Portfolios),
            Some("Bot".into()),
        );
        Some(pid)
    }

    // Authentication
    pub fn login_with_credentials(
        &mut self,
        username: &str,
        password: &str,
        _notification_store: &mut NotificationStore,
    ) -> Result<(String, String), String> {
        let cred = self
            .credentials
            .verify(username, password)
            .ok_or("Invalid username or password")?;

        if !cred.validated {
            return Err(
                "Account not validated. Please check your email or validate via /emailvalid."
                    .to_string(),
            );
        }

        let display_name = cred.display_name.clone();
        let email = cred.email.clone();

        // Set user profile
        self.is_authenticated = true;
        self.current_user.name = display_name.clone();
        self.current_user.email = email.clone();
        self.current_user.role = UserRole::Owner;

        // Seed demo organizations and portfolios if none exist
        if self.portfolios.is_empty() {
            // Seeding requires OrganizationStore — handled by login() and login_server_validated()
            // which have access to organization_store. login_with_credentials is the local-only
            // path that still needs it passed in.
            // For now, skip seeding here — the caller should use login() instead.
            // Mixed Investments is NOT part of any organization
            self.portfolios.push(seed_portfolio_2(self.current_user.id));
        }

        // Navigate to Overview after login
        self.expand_tab(TabType::Overview);

        Ok((display_name, format!("{:?}", self.current_user.role)))
    }

    pub fn set_user_name(&mut self, name: String) {
        self.current_user.name = name;
    }

    pub fn login(
        &mut self,
        name: String,
        email: String,
        role: UserRole,
        notification_store: &mut NotificationStore,
        organization_store: &mut crate::stores::OrganizationStore,
    ) {
        self.is_authenticated = true;
        self.current_user.name = name;
        self.current_user.email = email;
        self.current_user.role = role;

        // Seed demo organizations and portfolios if none exist
        if self.portfolios.is_empty() {
            crate::stores::seed_data::seed_red_family_data(
                self,
                organization_store,
                notification_store,
            );
            // Mixed Investments is NOT part of any organization
            self.portfolios.push(seed_portfolio_2(self.current_user.id));
        }

        // Navigate to Overview after login
        self.expand_tab(TabType::Overview);
    }

    pub fn logout(&mut self) {
        self.is_authenticated = false;
        self.current_user = UserProfile::default();
        self.collapse_all_tabs();
        self.selected_portfolio_id = None;
        self.selected_asset_group_id = None;
        self.selected_asset_id = None;
        self.pending_nav_target = None;
        self.pending_group_expand = None;
        self.portfolios.clear();
        let _ = crypto::clear_cached("farley_home_cache");
    }

    /// Register a new user in the credential store
    pub fn register_user(
        &mut self,
        username: &str,
        password: &str,
        email: &str,
        store_local: bool,
        store_cloud: bool,
    ) -> Result<(), String> {
        let display_name = username;
        let result = self.credentials.register_user(
            username,
            password,
            display_name,
            email,
            store_local,
            store_cloud,
        );

        #[cfg(feature = "hydrate")]
        if result.is_ok() {
            self.credentials.save_to_local_storage();
        }

        result
    }

    /// Check if password matches locally (regardless of validation)
    pub fn check_password(&self, username: &str, password: &str) -> bool {
        self.credentials.verify_password_only(username, password)
    }

    /// Check if user is validated locally
    pub fn is_user_validated(&self, username: &str) -> bool {
        self.credentials.is_validated(username)
    }

    /// Mark a user as validated locally
    pub fn mark_user_validated(&mut self, username: &str) {
        self.credentials.mark_validated(username);
    }

    /// Check if a user exists in local credentials
    pub fn user_exists(&self, username: &str) -> bool {
        self.credentials.user_exists(username)
    }

    /// Set local and cloud storage preferences for a local credential and persist
    pub fn set_storage_options(&mut self, username: &str, store_local: bool, store_cloud: bool) {
        self.credentials
            .set_storage_options(username, store_local, store_cloud);
        #[cfg(feature = "hydrate")]
        self.credentials.save_to_local_storage();
    }

    /// Save current credentials to localStorage (hydrate only)
    #[cfg(feature = "hydrate")]
    pub fn save_credentials_to_local_storage(&self) {
        self.credentials.save_to_local_storage();
    }

    /// Save password to credential store (for "Remember Password" feature).
    /// 2FA requirement is stubbed — for now, password is saved without 2FA.
    /// When remember_password is true, the plain password is also retained so saved profiles can auto-fill it.
    pub fn save_password_to_credentials(
        &mut self,
        username: &str,
        password: &str,
        remember_password: bool,
    ) {
        let display_name = self.current_user.name.clone();
        let email = self.current_user.email.clone();
        let remembered = if remember_password {
            Some(password.to_string())
        } else {
            None
        };
        self.credentials
            .save_password(username, password, &display_name, &email, remembered);
        #[cfg(feature = "hydrate")]
        self.credentials.save_to_local_storage();
    }

    /// Add or update a local credential after successful login and persist it
    pub fn upsert_credential_from_login(
        &mut self,
        username: &str,
        password: &str,
        display_name: &str,
        email: &str,
        validated: bool,
        store_local: bool,
        store_cloud: bool,
    ) {
        let (existing_store_local, existing_store_cloud, existing_remembered) = self
            .credentials
            .credentials
            .get(username)
            .map(|c| (c.store_local, c.store_cloud, c.remembered_password.clone()))
            .unwrap_or((true, false, None));
        let store_local = store_local || existing_store_local;
        let store_cloud = store_cloud || existing_store_cloud;
        if let Ok(hash) = CredentialStore::hash_password(password) {
            let cred = StoredCredential {
                username: username.to_string(),
                password_hash: hash,
                display_name: display_name.to_string(),
                email: email.to_string(),
                validated,
                totp_secret: None,
                totp_enabled: false,
                email_2fa_enabled: false,
                store_local,
                store_cloud,
                remembered_password: existing_remembered,
            };
            self.credentials
                .credentials
                .insert(username.to_string(), cred);
            #[cfg(feature = "hydrate")]
            self.credentials.save_to_local_storage();
        }
    }

    /// Login a server-validated user (from /api/login after email validation).
    /// `username` is the login identifier used as the credential store key.
    /// `display_name` is the human-readable name shown in the UI.
    pub fn login_server_validated(
        &mut self,
        username: &str,
        display_name: &str,
        email: &str,
        notification_store: &mut NotificationStore,
        organization_store: &mut crate::stores::OrganizationStore,
    ) {
        // Set user profile
        self.is_authenticated = true;
        self.current_user.name = display_name.to_string();
        self.current_user.email = email.to_string();
        self.current_user.role = UserRole::Owner;

        // Also mark this user as validated locally so future local logins work
        if !username.is_empty() {
            self.credentials.mark_validated(username);
            #[cfg(feature = "hydrate")]
            self.credentials.save_to_local_storage();
        }

        // Seed demo organizations and portfolios if none exist
        if self.portfolios.is_empty() {
            crate::stores::seed_data::seed_red_family_data(
                self,
                organization_store,
                notification_store,
            );
            // Mixed Investments is NOT part of any organization
            self.portfolios.push(seed_portfolio_2(self.current_user.id));
        }

        // Navigate to Overview after login
        self.expand_tab(TabType::Overview);
    }

    // Get location name for navbar
    pub fn get_current_location(&self) -> String {
        let tab = self.active_tabs.first().cloned();
        if let Some(ref tab) = tab {
            match tab {
                TabType::Overview => "Overview".to_string(),
                TabType::Portfolios => {
                    if let Some(id) = self.selected_portfolio_id {
                        if let Some(p) = self.get_portfolio(id) {
                            return format!("Portfolio: {}", p.name);
                        }
                    }
                    "Portfolios".to_string()
                }
                TabType::Networking => "Networking".to_string(),
                TabType::NetworkingAddMember => "Add Team".to_string(),
                TabType::Organization => "Organization".to_string(),
                TabType::Reporting => "Reporting".to_string(),
                TabType::Calendar => "Calendar".to_string(),
                TabType::Transactions => "Transactions".to_string(),
                TabType::History => "History".to_string(),
                TabType::Settings => "Settings".to_string(),
                TabType::Agent => "Agent".to_string(),
            }
        } else {
            "Home".to_string()
        }
    }
}

// Create a signal-based store for Leptos
pub fn create_app_store() -> RwSignal<AppStore> {
    RwSignal::new(AppStore::new())
}

// Context provider for the app store
pub fn provide_app_store() -> RwSignal<AppStore> {
    let store = create_app_store();
    provide_context(store);
    store
}

// Hook to use the app store
pub fn use_app_store() -> RwSignal<AppStore> {
    expect_context::<RwSignal<AppStore>>()
}
