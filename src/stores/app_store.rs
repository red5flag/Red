use crate::models::{Asset, Organization, Permission, Portfolio, Rule, RuleHistoryEntry, Transaction, User};
use crate::stores::credentials::{CredentialStore, StoredCredential};
use crate::types::{AssetType, TabType, Theme, UserProfile, UserRole};
use crate::utils::crypto;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
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
    // Financial transactions for reporting
    pub transactions: Vec<Transaction>,
    // Selected portfolio/asset IDs
    pub selected_portfolio_id: Option<Uuid>,
    pub selected_asset_group_id: Option<Uuid>,
    pub selected_asset_id: Option<Uuid>,
    /// When set, PortfoliosPage should expand this portfolio, expand the group,
    /// and open the doc modal for the asset/doc — used by notification click navigation.
    pub pending_nav_target: Option<PendingNavTarget>,
    /// When set, PortfolioListItem should expand this group (set by notification navigation).
    pub pending_group_expand: Option<Uuid>,
    // UI state
    pub is_search_open: bool,
    pub search_query: String,
    pub theme: Theme,
    pub blind_mode: bool,
    pub font_size: String,
    pub reduced_motion: bool,
    pub language: String,
    // Notification preferences
    pub notifications: Vec<Notification>,
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub sound_enabled: bool,
    // Modal state
    pub active_modal: Option<ModalType>,
    // Open document modals (by entity id) - persisted across re-renders
    pub open_doc_modals: HashSet<Uuid>,
    // Loading states
    pub is_loading: bool,
    // Network users (for networking tab) with role and privilege management
    pub organization_users: Vec<User>,
    // Whether the networking tab add-member panel is open
    pub networking_add_member_open: bool,
    // Networking sort state (shared between page and navbar search)
    pub net_sort_mode: u8,      // 0=Name, 1=Company, 2=Status, 3=Risk, 4=Type, 5=Transactions
    pub net_sort_ascending: bool,
    // View mode for portfolios
    pub portfolio_view_mode: crate::types::ViewMode,
    // Grid column count for portfolio grid view (1, 2, 3, 4, 6, 8, 12)
    pub portfolio_grid_columns: usize,
    // Portfolio page UI toggles (controlled from navbar)
    pub show_add_portfolio: bool,
    pub show_top_add_group: bool,
    pub show_top_add_asset: bool,
    pub show_add_modal: bool,
    pub portfolio_sort_mode: crate::types::SortMode,
    pub sort_ascending: bool,
    // Reporting sort state
    pub reporting_sort_mode: crate::types::ReportSortMode,
    pub reporting_sort_ascending: bool,
    // Messenger drawer state
    pub message_drawer_open: bool,
    pub selected_chat_id: Option<Uuid>,
    pub messages: Vec<crate::models::Message>,
    pub messenger_contacts: Vec<crate::models::MessengerContact>,
    // Calendar events (bookings, imported data)
    pub calendar_events: Vec<crate::models::CalendarEvent>,
    // Authentication state
    pub is_authenticated: bool,
    // Organizations (multi-organization support)
    pub organizations: Vec<Organization>,
    // Currently active organization
    pub current_organization_id: Option<Uuid>,
    // Credential store for password verification
    pub credentials: CredentialStore,
    // Tabs drawer (left-side drawer for tab list)
    pub tabs_drawer_open: bool,
    // Notifications drawer (right-side drawer)
    pub notifications_drawer_open: bool,
    // Rule engine: rules and history per organization
    pub rules: Vec<Rule>,
    pub rule_history: Vec<RuleHistoryEntry>,
    // Developer mode (for testing notifications, etc.)
    pub developer_mode: bool,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: Uuid,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub target_tab: Option<crate::types::TabType>,
    pub from_user: Option<String>,
    pub linked_doc_id: Option<Uuid>,
    pub linked_portfolio_id: Option<Uuid>,
    pub linked_group_id: Option<Uuid>,
    pub linked_asset_id: Option<Uuid>,
    /// Preview content shown when the notification is clicked (e.g. notes from document update).
    pub content_preview: Option<String>,
    /// Users tagged via @username in the notes — each gets a separate notification.
    pub tagged_user_ids: Vec<Uuid>,
}

/// Navigation target for jumping to a specific portfolio/group/asset/doc from a notification click.
#[derive(Clone, Debug, PartialEq)]
pub struct PendingNavTarget {
    pub portfolio_id: Uuid,
    pub group_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub doc_id: Option<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotificationType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModalType {
    CreatePortfolio,
    EditPortfolio(Uuid),
    CreateAssetGroup(Uuid), // portfolio_id
    EditAssetGroup(Uuid),
    CreateAsset(Uuid), // group_id
    EditAsset(Uuid),
    DeleteConfirmation {
        entity_type: String,
        entity_id: Uuid,
        entity_name: String,
    },
    QuickSale(Uuid), // asset_id
    Payout {
        asset_ids: Vec<Uuid>,
        recipients: Vec<Uuid>,
    },
    Notify {
        portfolio_ids: Vec<Uuid>,
        asset_ids: Vec<Uuid>,
    },
    UserDetails(Uuid),
    PaymentSetup(Uuid),
    SettingsEditor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TabState {
    active_tabs: Vec<TabType>,
    drawer_open: bool,
    edit_mode_tabs: HashSet<TabType>,
}

#[cfg(feature = "hydrate")]
fn load_tab_state_from_local_storage() -> Option<TabState> {
    use web_sys::window;
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(json)) = storage.get_item("farley_tab_state") {
                return serde_json::from_str(&json).ok();
            }
        }
    }
    None
}

#[cfg(not(feature = "hydrate"))]
fn load_tab_state_from_local_storage() -> Option<TabState> {
    None
}

impl AppStore {
    /// Persist the current tab state to localStorage (hydrate only).
    #[cfg(feature = "hydrate")]
    fn save_tab_state(&self) {
        use web_sys::window;
        let state = TabState {
            active_tabs: self.active_tabs.clone(),
            drawer_open: self.message_drawer_open,
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
            transactions: Vec::new(),
            selected_portfolio_id: None,
            selected_asset_group_id: None,
            selected_asset_id: None,
            pending_nav_target: None,
            pending_group_expand: None,
            is_search_open: false,
            search_query: String::new(),
            theme: Theme::default(),
            blind_mode: false,
            font_size: "default".to_string(),
            reduced_motion: false,
            language: "en-AU".to_string(),
            notifications: Vec::new(),
            email_notifications: true,
            push_notifications: true,
            sound_enabled: true,
            active_modal: None,
            open_doc_modals: HashSet::new(),
            is_loading: false,
            organization_users: Vec::new(),
            networking_add_member_open: false,
            net_sort_mode: 0,
            net_sort_ascending: true,
            portfolio_view_mode: crate::types::ViewMode::List,
            portfolio_grid_columns: 2,
            show_add_portfolio: false,
            show_top_add_group: false,
            show_top_add_asset: false,
            show_add_modal: false,
            portfolio_sort_mode: crate::types::SortMode::ByOrganization,
            sort_ascending: true,
            reporting_sort_mode: crate::types::ReportSortMode::Recent,
            reporting_sort_ascending: false,
            message_drawer_open: false,
            selected_chat_id: None,
            messages: Vec::new(),
            messenger_contacts: Vec::new(),
            calendar_events: Vec::new(),
            is_authenticated: false,
            organizations: Vec::new(),
            current_organization_id: None,
            credentials,
            tabs_drawer_open: false,
            notifications_drawer_open: false,
            rules: Vec::new(),
            rule_history: Vec::new(),
            developer_mode: false,
        }
    }
}

impl AppStore {
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut store = Self::default();
        #[cfg(feature = "ssr")]
        {
            let db = crate::storage::message_store();
            store.messages = db.load_all_messages();
        }
        store
    }

    /// Returns the current user's effective role in the given organization,
    /// falling back to the global current role if no org-specific record exists.
    pub fn current_user_role_in_org(&self, org_id: Uuid) -> UserRole {
        self.organization_users
            .iter()
            .find(|u| u.id == self.current_user.id && u.organization_id == Some(org_id))
            .map(|u| u.role.clone())
            .unwrap_or_else(|| self.current_user.role.clone())
    }

    // Document modal state (persisted across re-renders)
    pub fn is_doc_modal_open(&self, id: Uuid) -> bool {
        self.open_doc_modals.contains(&id)
    }

    pub fn open_doc_modal(&mut self, id: Uuid) {
        self.open_doc_modals.insert(id);
    }

    pub fn close_doc_modal(&mut self, id: Uuid) {
        self.open_doc_modals.remove(&id);
    }

    pub fn toggle_doc_modal(&mut self, id: Uuid) {
        if self.open_doc_modals.contains(&id) {
            self.open_doc_modals.remove(&id);
        } else {
            self.open_doc_modals.insert(id);
        }
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
    pub fn add_portfolio(&mut self, portfolio: Portfolio) {
        let pname = portfolio.name.clone();
        let pid = portfolio.id;
        self.portfolios.push(portfolio);
        self.add_notification_for(
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

    pub fn set_portfolio_name(&mut self, id: Uuid, name: String) {
        if let Some(p) = self.get_portfolio_mut(id) {
            p.name = name.clone();
            p.updated_at = chrono::Utc::now();
        }
        self.add_notification_for(
            format!("Portfolio renamed to \"{}\" — changes pending review.", name),
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
            for g in &mut p.asset_groups {
                g.assets.retain(|a| a.id != asset_id);
            }
            removed_direct || p.asset_groups.iter().any(|g| g.assets.len() < g.assets.len())
        } else {
            false
        }
    }

    pub fn remove_document_from_asset(&mut self, portfolio_id: Uuid, asset_id: Uuid, doc_id: Uuid) -> bool {
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

    pub fn add_document_to_portfolio(&mut self, portfolio_id: Uuid, doc: crate::models::Document) {
        let dname = doc.name.clone();
        let doc_id = doc.id;
        let uploader = self.current_user.name.clone();
        if let Some(p) = self.get_portfolio_mut(portfolio_id) {
            p.documents.push(doc);
            p.updated_at = chrono::Utc::now();
        }
        self.add_document_notification(
            doc_id,
            &dname,
            &uploader.clone(),
            &format!("Document \"{}\" added to portfolio — pending review.", dname),
            NotificationType::Info,
            None,
            Some(uploader),
            Some(portfolio_id),
            None,
            None,
        );
    }

    pub fn update_document_name(&mut self, doc_id: Uuid, new_name: String) {
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
            if found { continue; }
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
                if found { continue; }
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
            if found { continue; }
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
            self.add_document_notification(
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

    // Organization user management
    pub fn add_organization_user(&mut self, user: User) {
        self.organization_users.push(user);
    }

    pub fn toggle_networking_add_member(&mut self) {
        self.networking_add_member_open = !self.networking_add_member_open;
    }

    pub fn remove_organization_user(&mut self, id: Uuid) -> Option<User> {
        if let Some(pos) = self.organization_users.iter().position(|u| u.id == id) {
            Some(self.organization_users.remove(pos))
        } else {
            None
        }
    }

    pub fn update_user_role(&mut self, id: Uuid, new_role: UserRole) -> Result<(), String> {
        let current_user_id = self.current_user.id;
        if let Some(pos) = self.organization_users.iter().position(|u| u.id == id) {
            let current_user = self
                .organization_users
                .iter()
                .find(|u| u.id == current_user_id)
                .cloned()
                .unwrap_or_else(|| User::new("Current".to_string(), String::new(), UserRole::Owner));
            let user = &mut self.organization_users[pos];
            user.update_role(new_role, &current_user)
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn toggle_user_permission(&mut self, id: Uuid, permission: Permission) {
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

    // Search functionality
    pub fn open_search(&mut self) {
        self.is_search_open = true;
    }

    pub fn close_search(&mut self) {
        self.is_search_open = false;
        self.search_query.clear();
    }

    /// Toggles search open/closed and returns the new state.
    pub fn toggle_search(&mut self) -> bool {
        if self.is_search_open {
            self.close_search();
        } else {
            self.open_search();
            self.close_tabs_drawer();
            self.close_notifications_drawer();
            self.set_message_drawer(false);
        }
        self.is_search_open
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
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

    // Theme management
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    // Portfolio grid column count
    pub fn set_portfolio_grid_columns(&mut self, columns: usize) {
        let allowed = [1, 2, 3, 4, 6, 8, 12];
        self.portfolio_grid_columns = allowed.iter().copied().find(|&c| c == columns).unwrap_or(2);
    }

    // Messenger drawer
    pub fn toggle_message_drawer(&mut self) {
        self.message_drawer_open = !self.message_drawer_open;
    }

    pub fn set_message_drawer(&mut self, open: bool) {
        self.message_drawer_open = open;
    }

    pub fn set_selected_chat(&mut self, contact_id: Option<Uuid>) {
        self.selected_chat_id = contact_id;
    }

    pub fn unread_message_count(&self) -> usize {
        let me = self.current_user.id;
        self.messages.iter().filter(|m| m.recipient_id == me && !m.read).count()
    }

    pub fn send_message(&mut self, recipient_id: Uuid, content: String) {
        let sender_id = self.current_user.id;
        let message = crate::models::Message::new(sender_id, recipient_id, content);
        #[cfg(feature = "ssr")]
        {
            let store = crate::storage::message_store();
            let _ = store.save_message(&message);
        }
        self.messages.push(message);
    }

    pub fn receive_message(&mut self, sender_id: Uuid, content: String) {
        let recipient_id = self.current_user.id;
        let message = crate::models::Message::new(sender_id, recipient_id, content);
        #[cfg(feature = "ssr")]
        {
            let store = crate::storage::message_store();
            let _ = store.save_message(&message);
        }
        self.messages.push(message);
    }

    pub fn mark_messages_read(&mut self, sender_id: Uuid) {
        let me = self.current_user.id;
        for m in self.messages.iter_mut() {
            if m.recipient_id == me && m.sender_id == sender_id {
                m.read = true;
            }
        }
    }

    pub fn add_messenger_contact(&mut self, contact: crate::models::MessengerContact) {
        if !self.messenger_contacts.iter().any(|c| c.id == contact.id) {
            self.messenger_contacts.push(contact);
        }
    }

    // Calendar events
    pub fn add_calendar_event(&mut self, event: crate::models::CalendarEvent) {
        self.calendar_events.push(event);
    }

    pub fn clear_calendar_events(&mut self) {
        self.calendar_events.clear();
    }

    pub fn upsert_calendar_event(&mut self, event: crate::models::CalendarEvent) {
        self.calendar_events.retain(|e| e.id != event.id);
        self.calendar_events.push(event.clone());

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
                let all_assets: Vec<&mut crate::models::Asset> = p.assets.iter_mut()
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

    pub fn remove_calendar_event(&mut self, event_id: Uuid) {
        self.calendar_events.retain(|e| e.id != event_id);
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

    // Notification management
    pub fn add_notification(&mut self, message: String, notification_type: NotificationType) {
        self.add_notification_for(message, notification_type, None, None);
    }

    pub fn add_notification_for(
        &mut self,
        message: String,
        notification_type: NotificationType,
        target_tab: Option<crate::types::TabType>,
        from_user: Option<String>,
    ) {
        self.notifications.push(Notification {
            id: Uuid::new_v4(),
            message,
            notification_type,
            timestamp: chrono::Utc::now(),
            target_tab,
            from_user,
            linked_doc_id: None,
            linked_portfolio_id: None,
            linked_group_id: None,
            linked_asset_id: None,
            content_preview: None,
            tagged_user_ids: Vec::new(),
        });

        // Keep only last 50 notifications
        if self.notifications.len() > 50 {
            self.notifications.remove(0);
        }
    }

    /// Add a notification linked to a specific document, with an @user ping.
    /// Sends notifications to both the document's portfolio tab and an optional review tab.
    pub fn add_document_notification(
        &mut self,
        doc_id: Uuid,
        doc_name: &str,
        ping_user: &str,
        message: &str,
        notification_type: NotificationType,
        review_tab: Option<crate::types::TabType>,
        from_user: Option<String>,
        portfolio_id: Option<Uuid>,
        group_id: Option<Uuid>,
        asset_id: Option<Uuid>,
    ) {
        let ping_msg = format!("@{} — {}", ping_user, message);
        // Notification on the review tab (e.g. Reporting)
        if let Some(tab) = review_tab {
            self.notifications.push(Notification {
                id: Uuid::new_v4(),
                message: format!("Document \"{}\": {}", doc_name, ping_msg),
                notification_type: notification_type.clone(),
                timestamp: chrono::Utc::now(),
                target_tab: Some(tab),
                from_user: from_user.clone(),
                linked_doc_id: Some(doc_id),
                linked_portfolio_id: portfolio_id,
                linked_group_id: group_id,
                linked_asset_id: asset_id,
                content_preview: None,
                tagged_user_ids: Vec::new(),
            });
        }
        // Notification on the Portfolios tab (where the document lives)
        self.notifications.push(Notification {
            id: Uuid::new_v4(),
            message: format!("Document \"{}\": {}", doc_name, ping_msg),
            notification_type: notification_type.clone(),
            timestamp: chrono::Utc::now(),
            target_tab: Some(TabType::Portfolios),
            from_user,
            linked_doc_id: Some(doc_id),
            linked_portfolio_id: portfolio_id,
            linked_group_id: group_id,
            linked_asset_id: asset_id,
            content_preview: None,
            tagged_user_ids: Vec::new(),
        });

        // Keep only last 50 notifications
        if self.notifications.len() > 50 {
            self.notifications.remove(0);
        }
    }

    /// Add a document update notification with notes content and @username parsing.
    /// Each @username mention creates a separate notification for that user with the notes as content_preview.
    pub fn add_document_update_with_notes(
        &mut self,
        doc_id: Uuid,
        doc_name: &str,
        notes: &str,
        updater_name: &str,
        portfolio_id: Option<Uuid>,
        group_id: Option<Uuid>,
        asset_id: Option<Uuid>,
    ) {
        // Parse @username mentions from notes
        let tagged_users: Vec<(Uuid, String)> = {
            let store_users = &self.organization_users;
            let mut found = Vec::new();
            for part in notes.split('@').skip(1) {
                // Extract username (alphanumeric + _ until whitespace)
                let username: String = part.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.').collect();
                if username.is_empty() { continue; }
                // Match against organization users by name or username
                if let Some(user) = store_users.iter().find(|u| {
                    u.name == username
                        || u.username.as_deref() == Some(&username)
                        || u.name.to_lowercase() == username.to_lowercase()
                }) {
                    if !found.iter().any(|(id, _)| id == &user.id) {
                        found.push((user.id, user.name.clone()));
                    }
                }
            }
            found
        };

        let preview = if notes.trim().is_empty() { None } else { Some(notes.to_string()) };
        let base_msg = if notes.trim().is_empty() {
            format!("Document \"{}\" updated by {}.", doc_name, updater_name)
        } else {
            format!("Document \"{}\" updated by {} — with notes.", doc_name, updater_name)
        };

        let tagged_ids: Vec<Uuid> = tagged_users.iter().map(|(id, _)| *id).collect();

        // Main notification on Portfolios tab
        self.notifications.push(Notification {
            id: Uuid::new_v4(),
            message: base_msg.clone(),
            notification_type: NotificationType::Info,
            timestamp: chrono::Utc::now(),
            target_tab: Some(TabType::Portfolios),
            from_user: Some(updater_name.to_string()),
            linked_doc_id: Some(doc_id),
            linked_portfolio_id: portfolio_id,
            linked_group_id: group_id,
            linked_asset_id: asset_id,
            content_preview: preview.clone(),
            tagged_user_ids: tagged_ids.clone(),
        });

        // Per-user tagged notifications
        for (uid, uname) in &tagged_users {
            let ping_msg = format!("@{} — You were tagged in document \"{}\" by {}.", uname, doc_name, updater_name);
            self.notifications.push(Notification {
                id: Uuid::new_v4(),
                message: ping_msg,
                notification_type: NotificationType::Warning,
                timestamp: chrono::Utc::now(),
                target_tab: Some(TabType::Portfolios),
                from_user: Some(updater_name.to_string()),
                linked_doc_id: Some(doc_id),
                linked_portfolio_id: portfolio_id,
                linked_group_id: group_id,
                linked_asset_id: asset_id,
                content_preview: preview.clone(),
                tagged_user_ids: vec![*uid],
            });
        }

        // Keep only last 50 notifications
        if self.notifications.len() > 50 {
            self.notifications.drain(0..(self.notifications.len().saturating_sub(50)));
        }
    }

    pub fn notifications_for_tab(&self, tab: &crate::types::TabType) -> usize {
        self.notifications
            .iter()
            .filter(|n| n.target_tab.as_ref() == Some(tab))
            .count()
    }

    /// Navigate to the origin of a notification — expands the portfolio, group,
    /// and opens the doc modal for the asset/document that the notification originated from.
    pub fn navigate_to_notification(&mut self, notif_id: Uuid) {
        if let Some(n) = self.notifications.iter().find(|n| n.id == notif_id) {
            let tab = n.target_tab.clone();
            if let Some(pid) = n.linked_portfolio_id.or_else(|| {
                // Try to find the portfolio that contains the linked doc
                n.linked_doc_id.and_then(|did| {
                    self.portfolios.iter().find(|p| {
                        p.documents.iter().any(|d| d.id == did)
                            || p.asset_groups.iter().any(|g| g.documents.iter().any(|d| d.id == did))
                            || p.assets.iter().any(|a| a.documents.iter().any(|d| d.id == did))
                            || p.asset_groups.iter().any(|g| g.assets.iter().any(|a| a.documents.iter().any(|d| d.id == did)))
                    }).map(|p| p.id)
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
                self.close_notifications_drawer();
            } else if let Some(tab) = tab {
                // No linked portfolio — just switch to the tab
                self.expand_tab(tab);
                self.close_notifications_drawer();
            }
        }
    }

    /// Count notifications linked to any document within a portfolio
    /// (portfolio-level docs, group docs, asset docs).
    pub fn doc_notifications_for_portfolio(&self, portfolio_id: Uuid) -> usize {
        let doc_ids: std::collections::HashSet<Uuid> = if let Some(p) = self.portfolios.iter().find(|p| p.id == portfolio_id) {
            let mut ids: std::collections::HashSet<Uuid> = p.documents.iter().map(|d| d.id).collect();
            for g in &p.asset_groups {
                for d in &g.documents { ids.insert(d.id); }
                for a in &g.assets {
                    for d in &a.documents { ids.insert(d.id); }
                }
            }
            for a in &p.assets {
                for d in &a.documents { ids.insert(d.id); }
            }
            ids
        } else {
            std::collections::HashSet::new()
        };
        self.notifications.iter()
            .filter(|n| n.linked_doc_id.map(|id| doc_ids.contains(&id)).unwrap_or(false))
            .count()
    }

    /// Count notifications linked to any document within an asset group.
    pub fn doc_notifications_for_group(&self, portfolio_id: Uuid, group_id: Uuid) -> usize {
        let doc_ids: std::collections::HashSet<Uuid> = if let Some(p) = self.portfolios.iter().find(|p| p.id == portfolio_id) {
            if let Some(g) = p.asset_groups.iter().find(|g| g.id == group_id) {
                let mut ids: std::collections::HashSet<Uuid> = g.documents.iter().map(|d| d.id).collect();
                for a in &g.assets {
                    for d in &a.documents { ids.insert(d.id); }
                }
                ids
            } else { std::collections::HashSet::new() }
        } else { std::collections::HashSet::new() };
        self.notifications.iter()
            .filter(|n| n.linked_doc_id.map(|id| doc_ids.contains(&id)).unwrap_or(false))
            .count()
    }

    /// Count notifications linked to any document within an asset.
    pub fn doc_notifications_for_asset(&self, asset_id: Uuid) -> usize {
        let doc_ids: std::collections::HashSet<Uuid> = {
            let mut ids = std::collections::HashSet::new();
            for p in &self.portfolios {
                for a in &p.assets {
                    if a.id == asset_id {
                        for d in &a.documents { ids.insert(d.id); }
                    }
                }
                for g in &p.asset_groups {
                    for a in &g.assets {
                        if a.id == asset_id {
                            for d in &a.documents { ids.insert(d.id); }
                        }
                    }
                }
            }
            ids
        };
        self.notifications.iter()
            .filter(|n| n.linked_doc_id.map(|id| doc_ids.contains(&id)).unwrap_or(false))
            .count()
    }

    /// Count notifications linked to a specific document.
    pub fn notifications_for_doc(&self, doc_id: Uuid) -> usize {
        self.notifications.iter()
            .filter(|n| n.linked_doc_id == Some(doc_id))
            .count()
    }

    /// Get the actual notifications linked to a specific document (most recent first).
    pub fn notifications_list_for_doc(&self, doc_id: Uuid) -> Vec<Notification> {
        let mut notifs: Vec<Notification> = self.notifications.iter()
            .filter(|n| n.linked_doc_id == Some(doc_id))
            .cloned()
            .collect();
        notifs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        notifs
    }

    // ── Entity notification settings ──────────────────────────────

    /// Get notification settings for a portfolio.
    pub fn portfolio_notification_settings(&self, pid: Uuid) -> Vec<crate::models::EntityNotificationSetting> {
        self.portfolios.iter()
            .find(|p| p.id == pid)
            .map(|p| p.notification_settings.clone())
            .unwrap_or_default()
    }

    /// Get notification settings for an asset group.
    pub fn group_notification_settings(&self, pid: Uuid, gid: Uuid) -> Vec<crate::models::EntityNotificationSetting> {
        self.portfolios.iter()
            .find(|p| p.id == pid)
            .and_then(|p| p.asset_groups.iter().find(|g| g.id == gid))
            .map(|g| g.notification_settings.clone())
            .unwrap_or_default()
    }

    /// Add a notification setting to a portfolio.
    pub fn add_portfolio_notification_setting(&mut self, pid: Uuid, setting: crate::models::EntityNotificationSetting) {
        if let Some(p) = self.get_portfolio_mut(pid) {
            p.notification_settings.push(setting);
            p.updated_at = chrono::Utc::now();
        }
    }

    /// Add a notification setting to an asset group.
    pub fn add_group_notification_setting(&mut self, pid: Uuid, gid: Uuid, setting: crate::models::EntityNotificationSetting) {
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
            if let Some(s) = p.notification_settings.iter_mut().find(|s| s.id == setting_id) {
                s.enabled = !s.enabled;
            }
            p.updated_at = chrono::Utc::now();
        }
    }

    /// Toggle the enabled state of a notification setting on a group.
    pub fn toggle_group_notification_setting(&mut self, pid: Uuid, gid: Uuid, setting_id: Uuid) {
        if let Some(p) = self.get_portfolio_mut(pid) {
            if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                if let Some(s) = g.notification_settings.iter_mut().find(|s| s.id == setting_id) {
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
    pub fn update_portfolio_notification_setting(&mut self, pid: Uuid, setting: crate::models::EntityNotificationSetting) {
        if let Some(p) = self.get_portfolio_mut(pid) {
            if let Some(s) = p.notification_settings.iter_mut().find(|s| s.id == setting.id) {
                *s = setting;
            }
            p.updated_at = chrono::Utc::now();
        }
    }

    /// Update a notification setting on a group.
    pub fn update_group_notification_setting(&mut self, pid: Uuid, gid: Uuid, setting: crate::models::EntityNotificationSetting) {
        if let Some(p) = self.get_portfolio_mut(pid) {
            if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                if let Some(s) = g.notification_settings.iter_mut().find(|s| s.id == setting.id) {
                    *s = setting;
                }
                g.updated_at = chrono::Utc::now();
            }
        }
    }

    /// Check if the current user can manage notification recipients (requires ManageUsers or ManageRoles).
    pub fn can_manage_notification_recipients(&self) -> bool {
        self.current_user.has_permission(crate::models::Permission::ManageUsers)
            || self.current_user.has_permission(crate::models::Permission::ManageRoles)
    }

    pub fn remove_notification(&mut self, id: Uuid) {
        self.notifications.retain(|n| n.id != id);
    }

    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
    }

    /// Send a test notification from a given user to test the notification system.
    /// Only works when developer_mode is enabled.
    pub fn send_test_notification(&mut self, from_user: &str, message: &str, target_tab: crate::types::TabType) {
        if !self.developer_mode {
            return;
        }
        self.add_notification_for(
            message.to_string(),
            NotificationType::Info,
            Some(target_tab),
            Some(from_user.to_string()),
        );
    }

    pub fn toggle_developer_mode(&mut self) {
        self.developer_mode = !self.developer_mode;
    }

    pub fn dev_test_message_from_bot(&mut self, content: &str) {
        if !self.developer_mode { return; }
        self.receive_message(Uuid::new_v4(), content.to_string());
        self.add_notification_for(format!("New message from Bot: \"{}\"", content), NotificationType::Info, Some(TabType::Networking), Some("Bot".into()));
    }

    pub fn dev_test_add_bot_contact(&mut self) {
        if !self.developer_mode { return; }
        self.add_messenger_contact(crate::models::MessengerContact { id: Uuid::new_v4(), name: "Bot".into(), source: crate::models::ContactSource::Bot, phone: None, email: Some("bot@farley.test".into()), unread_count: 1 });
    }

    pub fn dev_test_add_document(&mut self, doc_name: &str, file_type: &str) -> Option<Uuid> {
        if !self.developer_mode { return None; }
        let doc = crate::models::Document { id: Uuid::new_v4(), name: doc_name.into(), file_type: file_type.into(), url: "#".into(), uploaded_at: chrono::Utc::now(), uploaded_by: self.current_user.id, content: None };
        let doc_id = doc.id;
        let mut origin_pid: Option<Uuid> = None;
        if let Some(p) = self.portfolios.first() {
            origin_pid = Some(p.id);
            self.add_document_to_portfolio(p.id, doc);
        } else {
            // No portfolio exists — create one then add the doc
            let mut new_p = Portfolio::new("Dev Test Portfolio".into(), self.current_user.id, crate::types::Currency::USD);
            let pid = new_p.id;
            origin_pid = Some(pid);
            new_p.documents.push(doc);
            self.add_portfolio(new_p);
        }
        self.add_document_notification(
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

    pub fn dev_test_update_document(&mut self, doc_id: Uuid, new_name: &str) {
        if !self.developer_mode { return; }
        self.update_document_name(doc_id, new_name.into());
        self.add_document_notification(
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

    pub fn dev_test_add_transaction(&mut self, amount: f64, desc: &str) {
        if !self.developer_mode { return; }
        let from_e = crate::models::EntityReference { entity_type: crate::models::EntityType::External, entity_id: Uuid::new_v4(), name: "Bot Corp".into() };
        let to_e = crate::models::EntityReference { entity_type: crate::models::EntityType::User, entity_id: self.current_user.id, name: self.current_user.name.clone() };
        let mut tx = crate::models::Transaction::new(crate::types::TransactionType::Transfer, amount, crate::types::Currency::USD, from_e, to_e, self.current_user.id);
        tx.description = Some(desc.into()); tx.status = crate::models::TransactionStatus::Pending;
        self.transactions.push(tx);
        self.add_notification_for(format!("Transaction: ${:.2} - {}", amount, desc), NotificationType::Info, Some(TabType::Transactions), Some("Bot".into()));
    }

    pub fn dev_test_approve_last_tx(&mut self) {
        if !self.developer_mode { return; }
        if let Some(tx) = self.transactions.last_mut() { tx.approve(); }
        if let Some(tx) = self.transactions.last() { self.add_notification_for(format!("Approved: ${:.2}", tx.amount), NotificationType::Success, Some(TabType::Transactions), Some("System".into())); }
    }

    pub fn dev_test_execute_last_tx(&mut self) {
        if !self.developer_mode { return; }
        if let Some(tx) = self.transactions.last_mut() { if tx.status == crate::models::TransactionStatus::Approved { tx.execute(); } }
        if let Some(tx) = self.transactions.last() { if tx.status == crate::models::TransactionStatus::Executed { self.add_notification_for(format!("Executed: ${:.2}", tx.amount), NotificationType::Success, Some(TabType::Transactions), Some("System".into())); } }
    }

    pub fn dev_test_add_calendar_event(&mut self, title: &str, days_ahead: i64) {
        if !self.developer_mode { return; }
        use chrono::Duration;
        let s = chrono::Utc::now() + Duration::days(days_ahead);
        let mut ev = crate::models::CalendarEvent::new(title.into(), s, s + Duration::hours(2));
        ev.source = Some("DevTest".into());
        self.add_calendar_event(ev);
        self.add_notification_for(format!("Event: \"{}\"", title), NotificationType::Info, Some(TabType::Calendar), Some("System".into()));
    }

    pub fn dev_test_add_portfolio(&mut self, name: &str) -> Option<Uuid> {
        if !self.developer_mode { return None; }
        let mut p = Portfolio::new(name.into(), self.current_user.id, crate::types::Currency::USD);
        p.description = Some("Dev test".into());
        let pid = p.id;
        self.add_portfolio(p);
        self.add_notification_for(format!("Bot requested review of portfolio \"{}\"", name), NotificationType::Warning, Some(TabType::Portfolios), Some("Bot".into()));
        Some(pid)
    }

    pub fn dev_test_add_org_user(&mut self, name: &str, role: UserRole) {
        if !self.developer_mode { return; }
        let role_dbg = format!("{:?}", role);
        self.add_organization_user(User::new(name.into(), format!("{}@farley.test", name.to_lowercase()), role));
        self.add_notification_for(format!("User: \"{}\" {}", name, role_dbg), NotificationType::Info, Some(TabType::Organization), Some("System".into()));
    }

    // Modal management
    pub fn open_modal(&mut self, modal_type: ModalType) {
        self.active_modal = Some(modal_type);
    }

    pub fn close_modal(&mut self) {
        self.active_modal = None;
    }

    // Loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    // Authentication
    pub fn login_with_credentials(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<(String, String), String> {
        let cred = self
            .credentials
            .verify(username, password)
            .ok_or("Invalid username or password")?;

        if !cred.validated {
            return Err("Account not validated. Please check your email or validate via /emailvalid.".to_string());
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
            self.seed_red_family_data();
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

    pub fn login(&mut self, name: String, email: String, role: UserRole) {
        self.is_authenticated = true;
        self.current_user.name = name;
        self.current_user.email = email;
        self.current_user.role = role;

        // Seed demo organizations and portfolios if none exist
        if self.portfolios.is_empty() {
            self.seed_red_family_data();
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
        self.close_search();
        self.selected_portfolio_id = None;
        self.selected_asset_group_id = None;
        self.selected_asset_id = None;
        self.pending_nav_target = None;
        self.pending_group_expand = None;
        self.portfolios.clear();
        self.tabs_drawer_open = false;
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
    pub fn set_storage_options(
        &mut self,
        username: &str,
        store_local: bool,
        store_cloud: bool,
    ) {
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
    pub fn save_password_to_credentials(&mut self, username: &str, password: &str) {
        let display_name = self.current_user.name.clone();
        let email = self.current_user.email.clone();
        self.credentials.save_password(username, password, &display_name, &email);
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
        let (existing_store_local, existing_store_cloud) = self
            .credentials
            .credentials
            .get(username)
            .map(|c| (c.store_local, c.store_cloud))
            .unwrap_or((true, false));
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
            };
            self.credentials.credentials.insert(username.to_string(), cred);
            #[cfg(feature = "hydrate")]
            self.credentials.save_to_local_storage();
        }
    }

    /// Login a server-validated user (from /api/login after email validation)
    pub fn login_server_validated(&mut self, display_name: &str, email: &str) {
        // Set user profile
        self.is_authenticated = true;
        self.current_user.name = display_name.to_string();
        self.current_user.email = email.to_string();
        self.current_user.role = UserRole::Owner;

        // Also mark this user as validated locally so future local logins work
        if !display_name.is_empty() {
            self.credentials.mark_validated(display_name);
            #[cfg(feature = "hydrate")]
            self.credentials.save_to_local_storage();
        }

        // Seed demo organizations and portfolios if none exist
        if self.portfolios.is_empty() {
            self.seed_red_family_data();
            // Mixed Investments is NOT part of any organization
            self.portfolios.push(seed_portfolio_2(self.current_user.id));
        }

        // Navigate to Overview after login
        self.expand_tab(TabType::Overview);
    }

    // Sort direction
    pub fn toggle_sort_direction(&mut self) {
        self.sort_ascending = !self.sort_ascending;
    }

    pub fn reversed_sort_mode(&self) -> crate::types::SortMode {
        use crate::types::SortMode;
        match self.portfolio_sort_mode {
            SortMode::Recent => SortMode::Oldest,
            SortMode::Oldest => SortMode::Recent,
            SortMode::HighestValue => SortMode::LowestValue,
            SortMode::LowestValue => SortMode::HighestValue,
            SortMode::HighestProfit => SortMode::LowestProfit,
            SortMode::LowestProfit => SortMode::HighestProfit,
            SortMode::HighestRevenue => SortMode::LowestRevenue,
            SortMode::LowestRevenue => SortMode::HighestRevenue,
            SortMode::ByOrganization => SortMode::ByOrganization,
        }
    }

    // Reporting sort helpers
    pub fn toggle_reporting_sort_direction(&mut self) {
        self.reporting_sort_ascending = !self.reporting_sort_ascending;
    }

    pub fn effective_reporting_sort_mode(&self) -> crate::types::ReportSortMode {
        use crate::types::ReportSortMode;
        if self.reporting_sort_ascending {
            match &self.reporting_sort_mode {
                ReportSortMode::Recent => ReportSortMode::Oldest,
                ReportSortMode::Oldest => ReportSortMode::Recent,
                ReportSortMode::HighestValue => ReportSortMode::LowestValue,
                ReportSortMode::LowestValue => ReportSortMode::HighestValue,
                other => other.clone(),
            }
        } else {
            self.reporting_sort_mode.clone()
        }
    }

    // Tabs drawer
    pub fn toggle_tabs_drawer(&mut self) {
        self.tabs_drawer_open = !self.tabs_drawer_open;
    }

    pub fn close_tabs_drawer(&mut self) {
        self.tabs_drawer_open = false;
    }

    // Notifications drawer
    pub fn toggle_notifications_drawer(&mut self) {
        self.notifications_drawer_open = !self.notifications_drawer_open;
    }

    pub fn close_notifications_drawer(&mut self) {
        self.notifications_drawer_open = false;
    }

    // Rule engine management
    pub fn add_rule(&mut self, rule: Rule) {
        let entry = crate::models::RuleHistoryEntry::new(
            rule.id,
            rule.name.clone(),
            "Created".to_string(),
            rule.created_by,
            self.current_user.name.clone(),
            format!("Rule '{}' was created", rule.name),
        );
        self.rule_history.push(entry);
        self.rules.push(rule);
    }

    pub fn update_rule(&mut self, rule: Rule, updated_by: Uuid) {
        if let Some(existing) = self.rules.iter_mut().find(|r| r.id == rule.id) {
            let entry = crate::models::RuleHistoryEntry::new(
                rule.id,
                rule.name.clone(),
                "Updated".to_string(),
                updated_by,
                self.current_user.name.clone(),
                format!("Rule '{}' was updated", rule.name),
            );
            self.rule_history.push(entry);
            *existing = rule;
            existing.updated_by = updated_by;
            existing.updated_at = chrono::Utc::now();
        }
    }

    pub fn delete_rule(&mut self, rule_id: Uuid, deleted_by: Uuid) {
        if let Some(rule) = self.rules.iter().find(|r| r.id == rule_id) {
            let entry = crate::models::RuleHistoryEntry::new(
                rule_id,
                rule.name.clone(),
                "Deleted".to_string(),
                deleted_by,
                self.current_user.name.clone(),
                format!("Rule '{}' was deleted", rule.name),
            );
            self.rule_history.push(entry);
        }
        self.rules.retain(|r| r.id != rule_id);
    }

    pub fn toggle_rule(&mut self, rule_id: Uuid, toggled_by: Uuid) {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = !rule.enabled;
            let action = if rule.enabled { "Enabled" } else { "Disabled" };
            let entry = crate::models::RuleHistoryEntry::new(
                rule_id,
                rule.name.clone(),
                action.to_string(),
                toggled_by,
                self.current_user.name.clone(),
                format!("Rule '{}' was {}", rule.name, action.to_lowercase()),
            );
            self.rule_history.push(entry);
            rule.updated_by = toggled_by;
            rule.updated_at = chrono::Utc::now();
        }
    }

    pub fn duplicate_rule(&mut self, rule_id: Uuid, duplicated_by: Uuid) -> Option<Rule> {
        let rule = self.rules.iter().find(|r| r.id == rule_id)?.clone();
        let mut new_rule = rule.clone();
        new_rule.id = Uuid::new_v4();
        new_rule.name = format!("{} (Copy)", rule.name);
        new_rule.enabled = false;
        new_rule.created_by = duplicated_by;
        new_rule.created_at = chrono::Utc::now();
        new_rule.updated_by = duplicated_by;
        new_rule.updated_at = chrono::Utc::now();
        let entry = crate::models::RuleHistoryEntry::new(
            new_rule.id,
            new_rule.name.clone(),
            "Duplicated".to_string(),
            duplicated_by,
            self.current_user.name.clone(),
            format!("Rule '{}' was duplicated from '{}'", new_rule.name, rule.name),
        );
        self.rule_history.push(entry);
        self.rules.push(new_rule.clone());
        Some(new_rule)
    }

    pub fn rules_for_org(&self, org_id: Uuid) -> Vec<&Rule> {
        self.rules.iter().filter(|r| r.organization_id == org_id).collect()
    }

    pub fn rule_history_for_rule(&self, rule_id: Uuid) -> Vec<&RuleHistoryEntry> {
        self.rule_history.iter().filter(|h| h.rule_id == rule_id).collect()
    }

    // Organization management
    pub fn add_organization(&mut self, org: Organization) {
        self.organizations.push(org);
    }

    pub fn get_organization(&self, id: Uuid) -> Option<&Organization> {
        self.organizations.iter().find(|o| o.id == id)
    }

    pub fn switch_organization(&mut self, id: Uuid) {
        if self.organizations.iter().any(|o| o.id == id) {
            self.current_organization_id = Some(id);
            // When Red switches orgs, update the current user's role to match
            // the seeded Red membership for that organization.
            if let Some(red_user) = self
                .organization_users
                .iter()
                .find(|u| u.organization_id == Some(id) && u.name == self.current_user.name)
            {
                self.current_user.role = red_user.role.clone();
            }
        }
    }

    /// Seed the Red family demo organizations and role-based test data.
    fn seed_red_family_data(&mut self) {
        let owner_id = self.current_user.id;
        let owner_email = self.current_user.email.clone();

        // RedOrg is owned by the current user (Red).
        let red_org = Organization::new("RedOrg".to_string(), owner_id);
        let red_org_id = red_org.id;

        let mut red_corp = Organization::new("RedDirector".to_string(), owner_id);
        let red_corp_id = red_corp.id;
        red_corp.description = Some("Red Director - Director role testbed".to_string());
        red_corp.settings.color = Some("#ef4444".to_string());

        let mut red_comp = Organization::new("RedManager".to_string(), owner_id);
        let red_comp_id = red_comp.id;
        red_comp.description = Some("Red Manager - Manager role testbed".to_string());
        red_comp.settings.color = Some("#f97316".to_string());

        let mut red_co = Organization::new("RedWorker".to_string(), owner_id);
        let red_co_id = red_co.id;
        red_co.description = Some("Red Worker - Worker role testbed".to_string());
        red_co.settings.color = Some("#3b82f6".to_string());

        self.organizations.push(red_org);
        self.organizations.push(red_corp);
        self.organizations.push(red_comp);
        self.organizations.push(red_co);
        self.current_organization_id = Some(red_org_id);

        // Red as Owner in RedOrg (same ID as current user).
        let mut red_owner = User::new("Red".to_string(), owner_email.clone(), UserRole::Owner);
        red_owner.id = owner_id;
        red_owner.organization_id = Some(red_org_id);

        // Red seeded into the other orgs with the requested roles.
        let mut red_director = User::new("Red".to_string(), "red@reddirector.com".to_string(), UserRole::Director);
        red_director.id = owner_id;
        red_director.organization_id = Some(red_corp_id);

        let mut red_manager = User::new("Red".to_string(), "red@redmanager.com".to_string(), UserRole::Manager);
        red_manager.id = owner_id;
        red_manager.organization_id = Some(red_comp_id);

        let mut red_worker = User::new("Red".to_string(), "red@redworker.com".to_string(), UserRole::Worker);
        red_worker.id = owner_id;
        red_worker.organization_id = Some(red_co_id);

        self.organization_users.push(red_owner);
        self.organization_users.push(red_director);
        self.organization_users.push(red_manager);
        self.organization_users.push(red_worker);

        // Add Red to every organization's member list.
        for org in &mut self.organizations {
            org.add_member(owner_id);
        }

        // One portfolio + one asset for each organization to test role access.
        self.portfolios.push(Self::seed_org_portfolio(red_org_id, owner_id, "RedOrg Portfolio", "RedOrg HQ Asset"));
        self.portfolios.push(Self::seed_org_portfolio(red_corp_id, owner_id, "RedDirector Portfolio", "RedDirector Fleet Asset"));
        self.portfolios.push(Self::seed_org_portfolio(red_comp_id, owner_id, "RedManager Portfolio", "RedManager Equipment Asset"));
        self.portfolios.push(Self::seed_org_portfolio(red_co_id, owner_id, "RedWorker Portfolio", "RedWorker Equipment Asset"));

        self.seed_notred_data();

        // Start Red as Owner of RedOrg; role updates when switching organizations.
        self.current_user.role = UserRole::Owner;
    }

    fn seed_org_portfolio(org_id: Uuid, owner_id: Uuid, name: &str, asset_name: &str) -> Portfolio {
        let mut p = Portfolio::new(name.to_string(), owner_id, crate::types::Currency::USD);
        p.organization_id = Some(org_id);
        p.description = Some(format!("{} - role testing portfolio", name));
        let mut asset = Asset::new(asset_name.to_string(), AssetType::Equipment, 10000.0);
        asset.organization_id = Some(org_id);
        p.assets.push(asset);
        p.recalculate_values();
        p
    }

    /// Seed a separate organization where the current user is a Guest.
    /// Used to test that a guest cannot edit organization, portfolio, asset, or document info.
    fn seed_notred_data(&mut self) {
        let guest_id = self.current_user.id;
        let guest_name = self.current_user.name.clone();
        let guest_email = self.current_user.email.clone();

        let notred_owner = Uuid::new_v4();
        let mut notred = Organization::new("NotRed".to_string(), notred_owner);
        let notred_id = notred.id;
        notred.description = Some("NotRed - Guest role testbed".to_string());
        notred.settings.color = Some("#10b981".to_string());
        notred.add_member(guest_id);
        self.organizations.push(notred);

        // Current user as a Guest in NotRed.
        let mut notred_guest = User::new(guest_name, guest_email, UserRole::Guest);
        notred_guest.id = guest_id;
        notred_guest.organization_id = Some(notred_id);
        self.organization_users.push(notred_guest);

        // Portfolio assigned to the guest so it is visible to them, but owned by the org owner.
        let mut p = Portfolio::new("NotRed Portfolio".to_string(), notred_owner, crate::types::Currency::USD);
        p.organization_id = Some(notred_id);
        p.description = Some("NotRed portfolio - Guest view-only testbed".to_string());
        p.assigned_users.push(guest_id);

        let mut asset = Asset::new("NotRed Office Equipment".to_string(), AssetType::Equipment, 5000.0);
        asset.organization_id = Some(notred_id);
        asset.assigned_workers.push(guest_id);
        // Pre-existing documents the guest can read but cannot edit (nil owner = legacy shared).
        asset.documents.push(make_doc("NotRed Welcome", "pdf"));
        asset.documents.push(make_doc("NotRed Policy", "pdf"));

        // Real audit document with @red ping in notes, uploaded by NotRed owner.
        let audit_doc = crate::models::Document {
            id: Uuid::new_v4(),
            name: "NotRed Q3 Audit Report".to_string(),
            file_type: "pdf".to_string(),
            url: "#".to_string(),
            uploaded_at: chrono::Utc::now(),
            uploaded_by: notred_owner,
            content: Some("@red — Please review this audit report for compliance. Notes: financial statements verified, tax filings current, 2 discrepancies flagged for follow-up. @red ping for approval.".to_string()),
        };
        let audit_doc_id = audit_doc.id;
        let notred_pid = p.id;
        let notred_aid = asset.id;
        asset.documents.push(audit_doc);
        p.assets.push(asset);
        p.recalculate_values();
        self.portfolios.push(p);

        // Linked document notifications: one on Reporting tab, one on Portfolios tab.
        self.add_document_notification(
            audit_doc_id,
            "NotRed Q3 Audit Report",
            "red",
            "Red1 (NotRed Owner) has listed a new document and requested audit review by Red (Auditor).",
            NotificationType::Warning,
            Some(crate::types::TabType::Reporting),
            Some("Red1".to_string()),
            Some(notred_pid),
            None,
            Some(notred_aid),
        );
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

    // ── Discord-style role management ──────────────────────────────────

    pub fn add_role_to_org(&mut self, org_id: Uuid, role: crate::models::OrgRole) {
        if let Some(org) = self.get_organization_mut(org_id) {
            org.roles.push(role);
            org.roles.sort_by(|a, b| b.rank.cmp(&a.rank));
            org.updated_at = chrono::Utc::now();
        }
    }

    pub fn update_org_role(&mut self, org_id: Uuid, role_id: Uuid, name: String, description: String, color: Option<String>, rank: u32, scope: crate::models::RoleScope) {
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

    pub fn toggle_role_permission(&mut self, org_id: Uuid, role_id: Uuid, perm: crate::models::Perm) {
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

fn make_doc(name: &str, ext: &str) -> crate::models::Document {
    crate::models::Document {
        id: Uuid::new_v4(),
        name: name.to_string(),
        file_type: ext.to_string(),
        url: "#".to_string(),
        uploaded_at: chrono::Utc::now(),
        uploaded_by: Uuid::nil(),
        content: None,
    }
}

fn make_asset(name: &str, desc: &str, location: &str, purchase: f64, current: f64, docs: Vec<crate::models::Document>) -> crate::models::Asset {
    use crate::types::AssetType;
    let mut a = crate::models::Asset::new(name.to_string(), AssetType::RealEstate, purchase);
    a.description = Some(desc.to_string());
    a.location = Some(location.to_string());
    a.update_value(current);
    a.documents = docs;
    a
}

pub fn seed_default_portfolio(owner_id: Uuid) -> Portfolio {
    let mut p = Portfolio::new("Commercial Real Estate".to_string(), owner_id, crate::types::Currency::USD);
    p.description = Some("Office buildings and retail spaces".to_string());
    p.tags = vec!["real-estate".to_string(), "commercial".to_string()];
    p.documents = vec![
        make_doc("Portfolio Overview", "pdf"),
        make_doc("Annual Report 2024", "xlsx"),
        make_doc("Investment Strategy", "docx"),
    ];

    let mut hq = make_asset(
        "Headquarters",
        "Main corporate headquarters building, 8 floors, 4200 sqm.",
        "123 Collins St, Melbourne VIC 3000",
        5_000_000.0, 6_200_000.0,
        vec![make_doc("Title Deed", "pdf"), make_doc("Valuation Report", "pdf"), make_doc("Insurance Certificate", "pdf")],
    );
    hq.images = vec![format!("https://placehold.co/400x400/2d3748/FFF?text=HQ")];
    p.assets.push(hq);

    // Downtown Properties group
    let mut group1 = crate::models::AssetGroup::new("Downtown Properties".to_string());
    group1.description = Some("Central business district commercial properties".to_string());
    group1.documents = vec![
        make_doc("Group Overview", "pdf"),
        make_doc("CBD Market Analysis", "xlsx"),
    ];

    let mut a1 = make_asset(
        "Main Office Building",
        "12-storey premium grade-A office tower, fully leased to blue-chip tenants.",
        "1 George St, Sydney NSW 2000",
        2_500_000.0, 3_200_000.0,
        vec![make_doc("Title Deed", "pdf"), make_doc("Lease Agreements", "docx"), make_doc("Floor Plans", "pdf"), make_doc("Valuation 2024", "xlsx")],
    );
    a1.images = vec!["https://placehold.co/400x400/1a365d/FFF?text=Main+Office".to_string()];

    let mut a2 = make_asset(
        "Retail Plaza",
        "Street-level retail complex with 14 tenancies, high foot traffic corner site.",
        "88 Queen St, Brisbane QLD 4000",
        1_200_000.0, 1_450_000.0,
        vec![make_doc("Title Deed", "pdf"), make_doc("Tenant Schedule", "xlsx"), make_doc("Inspection Report", "pdf")],
    );
    a2.images = vec!["https://placehold.co/400x400/2a4365/FFF?text=Retail+Plaza".to_string()];

    let mut a3 = make_asset(
        "Meridian Tower Suite 9",
        "Premium sublease office suite on level 9, panoramic harbour views, 420 sqm.",
        "100 Barangaroo Ave, Sydney NSW 2000",
        980_000.0, 1_150_000.0,
        vec![make_doc("Sublease Agreement", "docx"), make_doc("Fit-Out Schedule", "pdf"), make_doc("Building Compliance", "pdf")],
    );
    a3.images = vec!["https://placehold.co/400x400/2c5282/FFF?text=Meridian".to_string()];

    let mut a4 = make_asset(
        "Exchange Court Carpark",
        "Multi-deck 280-bay commercial carpark adjacent to main office tower.",
        "3 Exchange Court, Sydney NSW 2000",
        750_000.0, 820_000.0,
        vec![make_doc("Carpark Licence", "pdf"), make_doc("Revenue Report", "xlsx")],
    );
    a4.images = vec!["https://placehold.co/400x400/2d3748/FFF?text=Carpark".to_string()];

    let mut a5 = make_asset(
        "12345 Tan St Residence",
        "Residential property on Tan St, Gold Coast. 4 bed, 2 bath, double garage.",
        "12345 Tan St, Gold Coast, QLD 4000",
        850_000.0, 920_000.0,
        vec![
            make_doc("Contract of Sale", "pdf"),
            make_doc("Building & Pest Inspection", "pdf"),
            make_doc("Title Search", "pdf"),
            make_doc("Rental Appraisal", "docx"),
        ],
    );
    a5.images = vec!["https://placehold.co/400x400/744210/FFF?text=Tan+St".to_string()];

    let mut a6 = make_asset(
        "567 Modl Ct Residence",
        "Residential property on Modl Ct, Gold Coast. 3 bed, 2 bath, single garage.",
        "567 Modl Ct, Gold Coast, QLD 4001",
        720_000.0, 780_000.0,
        vec![
            make_doc("Contract of Sale", "pdf"),
            make_doc("Strata Report", "pdf"),
            make_doc("Tenant Lease", "docx"),
            make_doc("Depreciation Schedule", "xlsx"),
        ],
    );
    a6.images = vec!["https://placehold.co/400x400/7b341e/FFF?text=Modl+Ct".to_string()];

    group1.assets = vec![a1, a2, a3, a4, a5, a6];
    group1.recalculate_values();

    // Suburban Offices group
    let mut group2 = crate::models::AssetGroup::new("Suburban Offices".to_string());
    group2.description = Some("Technology park and suburban office campus holdings".to_string());
    group2.documents = vec![
        make_doc("Campus Master Plan", "pdf"),
        make_doc("Occupancy Report Q4", "xlsx"),
    ];

    let mut b1 = make_asset(
        "Tech Park Building A",
        "Modern 4-storey office building, open-plan, 2800 sqm NLA, NBN connected.",
        "15 Innovation Dr, Macquarie Park NSW 2113",
        1_800_000.0, 2_100_000.0,
        vec![make_doc("Title Deed", "pdf"), make_doc("Lease Roll", "xlsx"), make_doc("Energy Audit", "pdf"), make_doc("Fitout Specs", "docx")],
    );
    b1.images = vec!["https://placehold.co/400x400/276749/FFF?text=Tech+Park+A".to_string()];

    let mut b2 = make_asset(
        "Tech Park Building B",
        "Companion building to Building A, shared amenities, 2400 sqm NLA.",
        "17 Innovation Dr, Macquarie Park NSW 2113",
        1_600_000.0, 1_850_000.0,
        vec![make_doc("Title Deed", "pdf"), make_doc("Lease Roll", "xlsx"), make_doc("NABERS Rating", "pdf")],
    );
    b2.images = vec!["https://placehold.co/400x400/2f855a/FFF?text=Tech+Park+B".to_string()];

    let mut b3 = make_asset(
        "Parkside Annex",
        "Single-storey annex building used as a training centre and boardroom facility.",
        "19 Innovation Dr, Macquarie Park NSW 2113",
        620_000.0, 710_000.0,
        vec![make_doc("Building Survey", "pdf"), make_doc("Maintenance Schedule", "docx")],
    );
    b3.images = vec!["https://placehold.co/400x400/285e61/FFF?text=Parkside".to_string()];

    let mut b4 = make_asset(
        "North Business Hub",
        "Boutique 6-suite business centre, fully serviced, short-term leases.",
        "7 Rosebery Ave, Rosebery NSW 2018",
        890_000.0, 975_000.0,
        vec![make_doc("Lease Summary", "xlsx"), make_doc("Services Agreement", "docx"), make_doc("Insurance", "pdf")],
    );
    b4.images = vec!["https://placehold.co/400x400/322659/FFF?text=North+Hub".to_string()];

    let mut b5 = make_asset(
        "5454 Matter St Commercial",
        "Commercial real estate on Matter St, Gold Coast. Ground-floor retail with office above.",
        "5454 Matter St, Gold Coast, QLD 4000",
        1_250_000.0, 1_380_000.0,
        vec![
            make_doc("Title Deed", "pdf"),
            make_doc("Commercial Lease Agreement", "docx"),
            make_doc("Council Zoning Certificate", "pdf"),
            make_doc("Outgoings Schedule", "xlsx"),
        ],
    );
    b5.images = vec!["https://placehold.co/400x400/2c3e50/FFF?text=Matter+St".to_string()];

    let mut b6 = make_asset(
        "321 Porks Crescent Residence",
        "Residential property on Porks Crescent, Gold Coast. 4 bed, 3 bath, pool.",
        "321 Porks Crescent, Gold Coast, QLD 4001",
        980_000.0, 1_050_000.0,
        vec![
            make_doc("Contract of Sale", "pdf"),
            make_doc("Pool Compliance Certificate", "pdf"),
            make_doc("Body Corporate Disclosure", "docx"),
            make_doc("Rental Income History", "xlsx"),
        ],
    );
    b6.images = vec!["https://placehold.co/400x400/6b2737/FFF?text=Porks+Crsct".to_string()];

    group2.assets = vec![b1, b2, b3, b4, b5, b6];
    group2.recalculate_values();

    p.asset_groups = vec![group1, group2];
    p.recalculate_values();
    p
}

/// Generate a simple asset with a random-ish name and value.
fn gen_asset(idx: usize, prefix: &str, base_value: f64) -> crate::models::Asset {
    use crate::types::AssetType;
    let asset_types = [
        AssetType::RealEstate,
        AssetType::Vehicle,
        AssetType::Equipment,
        AssetType::Stock,
        AssetType::Bond,
        AssetType::Commodity,
        AssetType::Digital,
        AssetType::IntellectualProperty,
    ];
    let at = asset_types[idx % asset_types.len()].clone();
    let purchase = base_value + (idx as f64 * 10_000.0);
    let current = purchase * (1.0 + ((idx % 7) as f64 * 0.03));
    let mut a = crate::models::Asset::new(format!("{} #{}", prefix, idx + 1), at, purchase);
    a.description = Some(format!("Test asset {} for portfolio testing.", idx + 1));
    a.location = Some(format!("Test Location {}", idx + 1));
    a.documents = vec![
        make_doc(&format!("{} #{} Title Deed", prefix, idx + 1), "pdf"),
        make_doc(&format!("{} #{} Valuation", prefix, idx + 1), "xlsx"),
        make_doc(&format!("{} #{} Inspection", prefix, idx + 1), "pdf"),
    ];
    a.update_value(current);
    a
}

/// Portfolio 2: mixed assets — 3 direct + 8 assets across 2 groups
pub fn seed_portfolio_2(owner_id: Uuid) -> Portfolio {
    let mut p = Portfolio::new(
        "Mixed Investments".to_string(),
        owner_id,
        crate::types::Currency::USD,
    );
    p.description = Some("Diverse asset collection for testing".to_string());
    p.tags = vec!["mixed".to_string(), "test".to_string()];
    p.documents = vec![
        make_doc("Portfolio Summary", "pdf"),
        make_doc("Asset Allocation Report", "xlsx"),
        make_doc("Investment Policy Statement", "docx"),
    ];

    // 3 direct assets
    for i in 0..3 {
        p.assets.push(gen_asset(i, "Direct Asset", 500_000.0));
    }

    // Group 1: 4 assets
    let mut g1 = crate::models::AssetGroup::new("Group Alpha".to_string());
    g1.description = Some("First test group".to_string());
    g1.documents = vec![
        make_doc("Group Alpha Overview", "pdf"),
        make_doc("Alpha Performance Report", "xlsx"),
    ];
    for i in 0..4 {
        g1.assets.push(gen_asset(i, "Alpha Asset", 300_000.0));
    }
    g1.recalculate_values();

    // Group 2: 4 assets
    let mut g2 = crate::models::AssetGroup::new("Group Beta".to_string());
    g2.description = Some("Second test group".to_string());
    g2.documents = vec![
        make_doc("Group Beta Overview", "pdf"),
        make_doc("Beta Performance Report", "xlsx"),
    ];
    for i in 0..4 {
        g2.assets.push(gen_asset(i + 4, "Beta Asset", 250_000.0));
    }
    g2.recalculate_values();

    p.asset_groups = vec![g1, g2];
    p.recalculate_values();
    p
}

/// Portfolio 3: 5 asset groups with 3, 10, 19, 37, 98 assets
pub fn seed_portfolio_3(owner_id: Uuid) -> Portfolio {
    let mut p = Portfolio::new(
        "Large Scale Portfolio".to_string(),
        owner_id,
        crate::types::Currency::USD,
    );
    p.description = Some("Stress test portfolio with large asset groups".to_string());
    p.tags = vec!["large-scale".to_string(), "stress-test".to_string()];
    p.documents = vec![
        make_doc("Portfolio Master Plan", "pdf"),
        make_doc("Risk Assessment Report", "pdf"),
        make_doc("Quarterly Performance Summary", "xlsx"),
        make_doc("Compliance Certificate", "docx"),
    ];

    let group_specs: [(usize, &str); 5] = [
        (3, "Mini Group"),
        (10, "Small Group"),
        (19, "Medium Group"),
        (37, "Large Group"),
        (98, "Mega Group"),
    ];

    let mut groups = Vec::new();
    for (count, name) in group_specs {
        let mut g = crate::models::AssetGroup::new(name.to_string());
        g.description = Some(format!("{} with {} assets", name, count));
        g.documents = vec![
            make_doc(&format!("{} Overview", name), "pdf"),
            make_doc(&format!("{} Asset Register", name), "xlsx"),
        ];
        for i in 0..count {
            g.assets.push(gen_asset(i, name, 100_000.0));
        }
        g.recalculate_values();
        groups.push(g);
    }

    p.asset_groups = groups;
    p.recalculate_values();
    p
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
