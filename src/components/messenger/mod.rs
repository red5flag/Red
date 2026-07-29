use crate::models::{ContactSource, MessengerContact};
use crate::stores::{use_app_store, use_messenger_store, use_organization_store};
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn MessageDrawer() -> impl IntoView {
    let app_store = use_app_store();
    let messenger_store = use_messenger_store();
    let organization_store = use_organization_store();
    let (draft, set_draft) = signal(String::new());
    let (search, set_search) = signal(String::new());

    let _selected_contact = move || messenger_store.get().selected_chat_id;
    let set_selected_contact =
        move |id: Option<Uuid>| messenger_store.update(|s| s.set_selected_chat(id));

    let on_close = move |_| messenger_store.update(|s| s.set_message_drawer(false));

    let contacts = Memo::new(move |_| {
        let org = organization_store.get();
        let mut contacts = messenger_store.get().messenger_contacts.clone();
        // Seed organization users if no contacts exist
        if contacts.is_empty() {
            for user in &org.organization_users {
                contacts.push(MessengerContact {
                    id: user.id,
                    name: user.name.clone(),
                    source: ContactSource::Organization,
                    phone: user.phone.clone(),
                    email: Some(user.email.clone()),
                    unread_count: 0,
                });
            }
        }
        // Always include the debug bot
        if !contacts.iter().any(|c| c.id == Uuid::nil()) {
            contacts.push(MessengerContact {
                id: Uuid::nil(),
                name: "Debug Bot".to_string(),
                source: ContactSource::Bot,
                phone: None,
                email: None,
                unread_count: 0,
            });
        }
        let q = search.get().to_lowercase();
        if q.is_empty() {
            contacts
        } else {
            contacts
                .into_iter()
                .filter(|c| c.name.to_lowercase().contains(&q))
                .collect()
        }
    });

    let send_message = move |recipient_id: Uuid| {
        let text = draft.get();
        let current_user_id = app_store.get().current_user.id;
        if text.trim().is_empty() {
            return;
        }
        messenger_store.update(|s| s.send_message(current_user_id, recipient_id, text.clone()));
        // Echo from bot for testing
        if recipient_id == Uuid::nil() {
            let reply = format!("Bot received: {}", text);
            messenger_store.update(|s| s.receive_message(Uuid::nil(), current_user_id, reply));
        }
        set_draft.set(String::new());
    };

    let (left_tab, set_left_tab) = signal("friends");
    let (chat_search, set_chat_search) = signal(String::new());

    let friend_contacts: Memo<Vec<MessengerContact>> = Memo::new(move |_| {
        contacts
            .get()
            .into_iter()
            .filter(|c| c.source != ContactSource::Recommended)
            .collect::<Vec<_>>()
    });
    let request_contacts: Memo<Vec<MessengerContact>> = Memo::new(move |_| {
        contacts
            .get()
            .into_iter()
            .filter(|c| c.source == ContactSource::Recommended)
            .collect::<Vec<_>>()
    });

    let recent_contacts: Memo<Vec<MessengerContact>> = Memo::new(move |_| {
        let all = contacts.get();
        let current_user_id = app_store.get().current_user.id;
        let messages = messenger_store.get().messages.clone();
        let mut with_last: Vec<_> = all
            .iter()
            .filter_map(|c| {
                let last = messages.iter().filter(|m| {
                    (m.sender_id == c.id && m.recipient_id == current_user_id)
                        || (m.sender_id == current_user_id && m.recipient_id == c.id)
                }).max_by_key(|m| m.timestamp);
                last.map(|m| (c.clone(), m.timestamp))
            })
            .collect();
        with_last.sort_by(|a, b| b.1.cmp(&a.1));
        with_last.into_iter().map(|(c, _)| c).collect()
    });

    let active_thread = Memo::new(move |_| {
        let all = contacts.get();
        let cid = messenger_store.get().selected_chat_id?;
        let contact = all.iter().find(|c| c.id == cid).cloned()?;
        let current_user_id = app_store.get().current_user.id;
        let q = chat_search.get().to_lowercase();
        let messages = messenger_store.get().messages.clone();
        let thread: Vec<_> = messages
            .iter()
            .filter(|m| {
                let in_thread = (m.sender_id == cid && m.recipient_id == current_user_id)
                    || (m.sender_id == current_user_id && m.recipient_id == cid);
                in_thread && (q.is_empty() || m.content.to_lowercase().contains(&q))
            })
            .cloned()
            .collect();
        Some((contact, thread))
    });

    view! {
        <div class="messenger-drawer-overlay" on:click=on_close>
            <div class="messenger-screen" on:click=|ev| ev.stop_propagation()>
                <div class="messenger-sidebar">
                    <div class="messenger-sidebar-search">
                        <input
                            class="messenger-search-input"
                            type="text"
                            placeholder="Search people..."
                            prop:value={move || search.get()}
                            on:input=move |ev| set_search.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="messenger-sidebar-tabs">
                        <button
                            class="messenger-tab"
                            class:active={move || left_tab.get() == "friends"}
                            on:click=move |_| set_left_tab.set("friends")
                        >
                            "Friends"
                        </button>
                        <button
                            class="messenger-tab"
                            class:active={move || left_tab.get() == "requests"}
                            on:click=move |_| set_left_tab.set("requests")
                        >
                            "Requests"
                        </button>
                    </div>
                    <div class="messenger-sidebar-list">
                        <For
                            each=move || if left_tab.get() == "friends" { friend_contacts.get() } else { request_contacts.get() }
                            key=|c| c.id
                            children=move |c| {
                                let id = c.id;
                                let name = c.name.clone();
                                let sub = c.phone.clone().or(c.email.clone()).unwrap_or_else(|| match c.source {
                                    ContactSource::Bot => "PQC-secured test bot".to_string(),
                                    ContactSource::Recommended => "Request".to_string(),
                                    _ => "Friend".to_string(),
                                });
                                view! {
                                    <div class="messenger-contact" on:click=move |_| set_selected_contact(Some(id))>
                                        <div class="messenger-contact-avatar">{name.chars().next().unwrap_or('?').to_string()}</div>
                                        <div class="messenger-contact-info">
                                            <div class="messenger-contact-name">{name}</div>
                                            <div class="messenger-contact-sub">{sub}</div>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>
                    <div class="messenger-recent-title">"Recent Messages"</div>
                    <div class="messenger-recent-list">
                        <For
                            each=move || recent_contacts.get()
                            key=|c: &MessengerContact| c.id
                            children=move |c| {
                                let id = c.id;
                                let name = c.name.clone();
                                let sub = c.phone.clone().or(c.email.clone()).unwrap_or_else(|| match c.source {
                                    ContactSource::Bot => "PQC-secured test bot".to_string(),
                                    ContactSource::Recommended => "Request".to_string(),
                                    _ => "Friend".to_string(),
                                });
                                view! {
                                    <div class="messenger-contact" on:click=move |_| set_selected_contact(Some(id))>
                                        <div class="messenger-contact-avatar">{name.chars().next().unwrap_or('?').to_string()}</div>
                                        <div class="messenger-contact-info">
                                            <div class="messenger-contact-name">{name}</div>
                                            <div class="messenger-contact-sub">{sub}</div>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>
                </div>
                <div class="messenger-chat">
                    {move || {
                        if let Some((contact, thread)) = active_thread.get() {
                            let contact_name = contact.name.clone();
                            let cid = contact.id;
                            let current_user_id = app_store.get().current_user.id;
                            view! {
                                <div class="messenger-chat-navbar">
                                    <div class="messenger-chat-title">{contact_name}</div>
                                    <div class="messenger-chat-actions">
                                        <button class="messenger-chat-action" title="Call">"📞"</button>
                                        <button class="messenger-chat-action" title="Video">"🎥"</button>
                                        <button class="messenger-chat-action" title="Pin">"📌"</button>
                                        <button class="messenger-chat-action" title="Groups">"👥"</button>
                                        <input
                                            class="messenger-chat-search"
                                            type="text"
                                            placeholder="Search chat..."
                                            prop:value={move || chat_search.get()}
                                            on:input=move |ev| set_chat_search.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>
                                <div class="messenger-messages">
                                    <For
                                        each=move || thread.clone()
                                        key=|m| m.id
                                        children=move |m| {
                                            let text = m.content.clone();
                                            let ts = format!("{:?}", m.timestamp.format("%H:%M"));
                                            let is_me = m.sender_id == current_user_id;
                                            let cls = if is_me { "messenger-message messenger-message-me" } else { "messenger-message" };
                                            view! {
                                                <div class={cls}>
                                                    <div class="messenger-message-text">{text}</div>
                                                    <div class="messenger-message-meta">{ts}</div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                                <div class="messenger-composer">
                                    <input
                                        class="messenger-composer-input"
                                        type="text"
                                        placeholder="Type a secure message..."
                                        prop:value={move || draft.get()}
                                        on:input=move |ev| set_draft.set(event_target_value(&ev))
                                        on:keyup=move |ev: leptos::ev::KeyboardEvent| {
                                            if ev.key() == "Enter" { send_message(cid); }
                                        }
                                    />
                                    <button class="messenger-composer-btn" on:click=move |_| send_message(cid)>
                                        "Send"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="messenger-empty-chat">
                                    "Select a chat to view messages"
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
