use crate::models::User;
use crate::types::TabType;
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

/// In-app notification displayed to the user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notification {
    pub id: Uuid,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: chrono::DateTime<Utc>,
    pub target_tab: Option<TabType>,
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

/// Classification of an in-app notification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotificationType {
    Success,
    Error,
    Warning,
    Info,
}

/// UI state, preferences, and lifecycle for the notification system.
///
/// Phase B owns the notification list and basic notification lifecycle/query
/// methods. Portfolio/group notification settings and navigation helpers remain
/// in `AppStore` until a later phase.
#[derive(Clone, Debug, Default)]
pub struct NotificationStore {
    /// The in-app notification list.
    pub notifications: Vec<Notification>,
    /// Whether the right-side notifications drawer is open.
    pub drawer_open: bool,
    /// Whether email notifications are enabled.
    pub email_notifications: bool,
    /// Whether push (in-app) notifications are enabled.
    pub push_notifications: bool,
    /// Whether notification sound effects are enabled.
    pub sound_enabled: bool,
}

impl NotificationStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_drawer(&mut self) {
        self.drawer_open = !self.drawer_open;
    }

    pub fn close_drawer(&mut self) {
        self.drawer_open = false;
    }

    pub fn set_email_notifications(&mut self, enabled: bool) {
        self.email_notifications = enabled;
    }

    pub fn set_push_notifications(&mut self, enabled: bool) {
        self.push_notifications = enabled;
    }

    pub fn set_sound_enabled(&mut self, enabled: bool) {
        self.sound_enabled = enabled;
    }

    pub fn add_notification(&mut self, message: String, notification_type: NotificationType) {
        self.add_notification_for(message, notification_type, None, None);
    }

    pub fn add_notification_for(
        &mut self,
        message: String,
        notification_type: NotificationType,
        target_tab: Option<TabType>,
        from_user: Option<String>,
    ) {
        self.notifications.push(Notification {
            id: Uuid::new_v4(),
            message,
            notification_type,
            timestamp: Utc::now(),
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
        review_tab: Option<TabType>,
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
                timestamp: Utc::now(),
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
            timestamp: Utc::now(),
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
        organization_users: &[User],
    ) {
        // Parse @username mentions from notes
        let tagged_users: Vec<(Uuid, String)> = {
            let mut found = Vec::new();
            for part in notes.split('@').skip(1) {
                // Extract username (alphanumeric + _ until whitespace)
                let username: String = part
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
                    .collect();
                if username.is_empty() {
                    continue;
                }
                // Match against organization users by name or username
                if let Some(user) = organization_users.iter().find(|u| {
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

        let preview = if notes.trim().is_empty() {
            None
        } else {
            Some(notes.to_string())
        };
        let base_msg = if notes.trim().is_empty() {
            format!("Document \"{}\" updated by {}.", doc_name, updater_name)
        } else {
            format!(
                "Document \"{}\" updated by {} — with notes.",
                doc_name, updater_name
            )
        };

        let tagged_ids: Vec<Uuid> = tagged_users.iter().map(|(id, _)| *id).collect();

        // Main notification on Portfolios tab
        self.notifications.push(Notification {
            id: Uuid::new_v4(),
            message: base_msg.clone(),
            notification_type: NotificationType::Info,
            timestamp: Utc::now(),
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
            let ping_msg = format!(
                "@{} — You were tagged in document \"{}\" by {}.",
                uname, doc_name, updater_name
            );
            self.notifications.push(Notification {
                id: Uuid::new_v4(),
                message: ping_msg,
                notification_type: NotificationType::Warning,
                timestamp: Utc::now(),
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
            self.notifications
                .drain(0..(self.notifications.len().saturating_sub(50)));
        }
    }

    pub fn remove_notification(&mut self, id: Uuid) {
        self.notifications.retain(|n| n.id != id);
    }

    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
    }

    /// Send a test notification from a given user to test the notification system.
    /// Only works when developer_mode is enabled.
    pub fn send_test_notification(
        &mut self,
        developer_mode: bool,
        from_user: &str,
        message: &str,
        target_tab: TabType,
    ) {
        if !developer_mode {
            return;
        }
        self.add_notification_for(
            message.to_string(),
            NotificationType::Info,
            Some(target_tab),
            Some(from_user.to_string()),
        );
    }

    pub fn notifications_for_tab(&self, tab: &TabType) -> usize {
        self.notifications
            .iter()
            .filter(|n| n.target_tab.as_ref() == Some(tab))
            .count()
    }

    /// Count notifications linked to a specific document.
    pub fn notifications_for_doc(&self, doc_id: Uuid) -> usize {
        self.notifications
            .iter()
            .filter(|n| n.linked_doc_id == Some(doc_id))
            .count()
    }

    /// Get the actual notifications linked to a specific document (most recent first).
    pub fn notifications_list_for_doc(&self, doc_id: Uuid) -> Vec<Notification> {
        let mut notifs: Vec<Notification> = self
            .notifications
            .iter()
            .filter(|n| n.linked_doc_id == Some(doc_id))
            .cloned()
            .collect();
        notifs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        notifs
    }
}

/// Create a new signal-based notification store.
pub fn create_notification_store() -> RwSignal<NotificationStore> {
    RwSignal::new(NotificationStore::new())
}

/// Provide the notification store to the Leptos context.
pub fn provide_notification_store() -> RwSignal<NotificationStore> {
    let store = create_notification_store();
    provide_context(store);
    store
}

/// Hook to consume the notification store from context.
pub fn use_notification_store() -> RwSignal<NotificationStore> {
    expect_context::<RwSignal<NotificationStore>>()
}
