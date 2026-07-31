use crate::models::{ContactSource, Message, MessengerContact, MessengerRequest};
use leptos::prelude::*;
use uuid::Uuid;

/// Dedicated store for messenger state: drawer visibility, selected chat,
/// messages, and contacts. Extracted from AppStore so messenger lifecycle
/// changes do not invalidate consumers of unrelated domain state.
#[derive(Clone, Debug)]
pub struct MessengerStore {
    pub message_drawer_open: bool,
    pub selected_chat_id: Option<Uuid>,
    pub messages: Vec<Message>,
    pub messenger_contacts: Vec<MessengerContact>,
    pub messenger_requests: Vec<MessengerRequest>,
}

impl Default for MessengerStore {
    fn default() -> Self {
        Self {
            message_drawer_open: false,
            selected_chat_id: None,
            messages: Vec::new(),
            messenger_contacts: Vec::new(),
            messenger_requests: Vec::new(),
        }
    }
}

impl MessengerStore {
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

    // Drawer
    pub fn toggle_message_drawer(&mut self) {
        self.message_drawer_open = !self.message_drawer_open;
    }

    pub fn set_message_drawer(&mut self, open: bool) {
        self.message_drawer_open = open;
    }

    // Chat selection
    pub fn set_selected_chat(&mut self, contact_id: Option<Uuid>) {
        self.selected_chat_id = contact_id;
    }

    // Queries
    pub fn unread_message_count(&self, current_user_id: Uuid) -> usize {
        self.messages
            .iter()
            .filter(|m| m.recipient_id == current_user_id && !m.read)
            .count()
    }

    // Lifecycle
    pub fn send_message(&mut self, sender_id: Uuid, recipient_id: Uuid, content: String) {
        let message = Message::new(sender_id, recipient_id, content);
        #[cfg(feature = "ssr")]
        {
            let store = crate::storage::message_store();
            let _ = store.save_message(&message);
        }
        self.messages.push(message);
    }

    pub fn receive_message(&mut self, sender_id: Uuid, recipient_id: Uuid, content: String) {
        let message = Message::new(sender_id, recipient_id, content);
        #[cfg(feature = "ssr")]
        {
            let store = crate::storage::message_store();
            let _ = store.save_message(&message);
        }
        self.messages.push(message);
    }

    pub fn mark_messages_read(&mut self, recipient_id: Uuid, sender_id: Uuid) {
        for m in self.messages.iter_mut() {
            if m.recipient_id == recipient_id && m.sender_id == sender_id {
                m.read = true;
            }
        }
    }

    pub fn add_messenger_contact(&mut self, contact: MessengerContact) {
        if !self.messenger_contacts.iter().any(|c| c.id == contact.id) {
            self.messenger_contacts.push(contact);
        }
    }

    pub fn create_group(
        &mut self,
        name: String,
        icon: Option<String>,
        members: Vec<Uuid>,
    ) -> Uuid {
        let group = MessengerContact::group(name, icon.unwrap_or_default(), members);
        let id = group.id;
        self.add_messenger_contact(group);
        id
    }

    pub fn update_group(
        &mut self,
        id: Uuid,
        name: Option<String>,
        icon: Option<String>,
    ) {
        if let Some(g) = self.messenger_contacts.iter_mut().find(|c| c.id == id && c.source == ContactSource::Group) {
            if let Some(name) = name {
                g.name = name;
            }
            if let Some(icon) = icon {
                g.icon = Some(icon);
            }
        }
    }

    pub fn add_group_member(&mut self, id: Uuid, member_id: Uuid) {
        if let Some(g) = self.messenger_contacts.iter_mut().find(|c| c.id == id && c.source == ContactSource::Group) {
            if !g.members.contains(&member_id) {
                g.members.push(member_id);
            }
        }
    }

    // Requests

    pub fn add_messenger_request(&mut self, request: MessengerRequest) {
        if !self.messenger_requests.iter().any(|r| r.id == request.id) {
            self.messenger_requests.push(request);
        }
    }

    pub fn remove_request(&mut self, id: Uuid) -> Option<MessengerRequest> {
        if let Some(pos) = self.messenger_requests.iter().position(|r| r.id == id) {
            Some(self.messenger_requests.remove(pos))
        } else {
            None
        }
    }

    pub fn accept_friend_request(&mut self, request_id: Uuid, contact: MessengerContact) {
        self.add_messenger_contact(contact);
        self.remove_request(request_id);
    }

    pub fn accept_group_invite(&mut self, request_id: Uuid, current_user_id: Uuid) {
        if let Some(req) = self.remove_request(request_id) {
            self.add_group_member(req.target_id, current_user_id);
        }
    }

    pub fn decline_request(&mut self, request_id: Uuid) {
        let _ = self.remove_request(request_id);
    }

    // Developer/test helpers
    pub fn dev_test_message_from_bot(&mut self, current_user_id: Uuid, content: &str) {
        self.receive_message(Uuid::new_v4(), current_user_id, content.to_string());
    }

    pub fn dev_test_add_bot_contact(&mut self) {
        self.add_messenger_contact(MessengerContact {
            id: Uuid::new_v4(),
            name: "Bot".into(),
            source: crate::models::ContactSource::Bot,
            phone: None,
            email: Some("bot@farley.test".into()),
            icon: None,
            members: Vec::new(),
            unread_count: 1,
        });
    }
}

pub fn create_messenger_store() -> RwSignal<MessengerStore> {
    RwSignal::new(MessengerStore::new())
}

pub fn provide_messenger_store() -> RwSignal<MessengerStore> {
    let store = create_messenger_store();
    provide_context(store);
    store
}

pub fn use_messenger_store() -> RwSignal<MessengerStore> {
    expect_context::<RwSignal<MessengerStore>>()
}
