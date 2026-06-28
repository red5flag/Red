use crate::models::{Organization, Permission, Portfolio, Transaction, User};
use crate::stores::credentials::{CredentialStore, StoredCredential};
use crate::types::{TabType, Theme, UserProfile, UserRole};
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
    // UI state
    pub is_search_open: bool,
    pub search_query: String,
    pub theme: Theme,
    // Notifications
    pub notifications: Vec<Notification>,
    // Modal state
    pub active_modal: Option<ModalType>,
    // Loading states
    pub is_loading: bool,
    // Network users (for networking tab) with role and privilege management
    pub organization_users: Vec<User>,
    // Whether the networking tab add-member panel is open
    pub networking_add_member_open: bool,
    // View mode for portfolios
    pub portfolio_view_mode: crate::types::ViewMode,
    // Grid column count for portfolio grid view (2, 3, 4, 6, 8, 12)
    pub portfolio_grid_columns: usize,
    // Portfolio page UI toggles (controlled from navbar)
    pub show_add_portfolio: bool,
    pub show_top_add_group: bool,
    pub show_top_add_asset: bool,
    pub show_add_modal: bool,
    pub portfolio_sort_mode: crate::types::SortMode,
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
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: Uuid,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
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
            is_search_open: false,
            search_query: String::new(),
            theme: Theme::default(),
            notifications: Vec::new(),
            active_modal: None,
            is_loading: false,
            organization_users: Vec::new(),
            networking_add_member_open: false,
            portfolio_view_mode: crate::types::ViewMode::List,
            portfolio_grid_columns: 2,
            show_add_portfolio: false,
            show_top_add_group: false,
            show_top_add_asset: false,
            show_add_modal: false,
            portfolio_sort_mode: crate::types::SortMode::Recent,
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
        self.portfolios.push(portfolio);
    }

    pub fn get_portfolio(&self, id: Uuid) -> Option<&Portfolio> {
        self.portfolios.iter().find(|p| p.id == id)
    }

    pub fn get_portfolio_mut(&mut self, id: Uuid) -> Option<&mut Portfolio> {
        self.portfolios.iter_mut().find(|p| p.id == id)
    }

    pub fn set_portfolio_name(&mut self, id: Uuid, name: String) {
        if let Some(p) = self.get_portfolio_mut(id) {
            p.name = name;
        }
    }

    pub fn remove_portfolio(&mut self, id: Uuid) -> Option<Portfolio> {
        if let Some(pos) = self.portfolios.iter().position(|p| p.id == id) {
            Some(self.portfolios.remove(pos))
        } else {
            None
        }
    }

    pub fn update_document_name(&mut self, doc_id: Uuid, new_name: String) {
        for p in self.portfolios.iter_mut() {
            for d in &mut p.documents {
                if d.id == doc_id {
                    d.name = new_name.clone();
                    return;
                }
            }
            for g in &mut p.asset_groups {
                for d in &mut g.documents {
                    if d.id == doc_id {
                        d.name = new_name.clone();
                        return;
                    }
                }
                for a in &mut g.assets {
                    for d in &mut a.documents {
                        if d.id == doc_id {
                            d.name = new_name.clone();
                            return;
                        }
                    }
                }
            }
            for a in &mut p.assets {
                for d in &mut a.documents {
                    if d.id == doc_id {
                        d.name = new_name.clone();
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

    // Search functionality
    pub fn open_search(&mut self) {
        self.is_search_open = true;
    }

    pub fn close_search(&mut self) {
        self.is_search_open = false;
        self.search_query.clear();
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    // Theme management
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    // Portfolio grid column count
    pub fn set_portfolio_grid_columns(&mut self, columns: usize) {
        let allowed = [2, 3, 4, 6, 8, 12];
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
        self.notifications.push(Notification {
            id: Uuid::new_v4(),
            message,
            notification_type,
            timestamp: chrono::Utc::now(),
        });

        // Keep only last 10 notifications
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }

    pub fn remove_notification(&mut self, id: Uuid) {
        self.notifications.retain(|n| n.id != id);
    }

    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
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

        // Seed portfolios if none exist
        if self.portfolios.is_empty() {
            let owner_id = self.current_user.id;
            self.portfolios.push(seed_default_portfolio(owner_id));
            self.portfolios.push(seed_portfolio_2(owner_id));
            self.portfolios.push(seed_portfolio_3(owner_id));
        }

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

        // Seed portfolios if none exist
        if self.portfolios.is_empty() {
            let owner_id = self.current_user.id;
            self.portfolios.push(seed_default_portfolio(owner_id));
            self.portfolios.push(seed_portfolio_2(owner_id));
            self.portfolios.push(seed_portfolio_3(owner_id));
        }
    }

    pub fn logout(&mut self) {
        self.is_authenticated = false;
        self.current_user = UserProfile::default();
        self.collapse_all_tabs();
        self.close_search();
        self.selected_portfolio_id = None;
        self.selected_asset_group_id = None;
        self.selected_asset_id = None;
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

        // Seed a default portfolio if none exist
        if self.portfolios.is_empty() {
            let owner_id = self.current_user.id;
            self.portfolios.push(seed_default_portfolio(owner_id));
        }

    }

    // Tabs drawer
    pub fn toggle_tabs_drawer(&mut self) {
        self.tabs_drawer_open = !self.tabs_drawer_open;
    }

    pub fn close_tabs_drawer(&mut self) {
        self.tabs_drawer_open = false;
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
        }
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

    // 3 direct assets
    for i in 0..3 {
        p.assets.push(gen_asset(i, "Direct Asset", 500_000.0));
    }

    // Group 1: 4 assets
    let mut g1 = crate::models::AssetGroup::new("Group Alpha".to_string());
    g1.description = Some("First test group".to_string());
    for i in 0..4 {
        g1.assets.push(gen_asset(i, "Alpha Asset", 300_000.0));
    }
    g1.recalculate_values();

    // Group 2: 4 assets
    let mut g2 = crate::models::AssetGroup::new("Group Beta".to_string());
    g2.description = Some("Second test group".to_string());
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
