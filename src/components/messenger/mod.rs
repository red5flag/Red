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

    let selected_contact = move || messenger_store.get().selected_chat_id;
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

    let sections = [
        ("Inbox", ContactSource::Imported),
        ("Direct Messages", ContactSource::Organization),
        ("Recommended", ContactSource::Recommended),
        ("Bot", ContactSource::Bot),
    ];

    view! {
        <div class="messenger-drawer-overlay" on:click=on_close>
            <div class="messenger-drawer" on:click=|ev| ev.stop_propagation()>
                <div class="messenger-drawer-search">
                    <input
                        class="messenger-search-input"
                        type="text"
                        placeholder="Search people..."
                        prop:value={move || search.get()}
                        on:input=move |ev| set_search.set(event_target_value(&ev))
                    />
                </div>
                <div class="messenger-drawer-body">
                    {move || {
                        let all = contacts.get();
                        let active = selected_contact();
                        let current_user_id = app_store.get().current_user.id;
                        if let Some(cid) = active {
                            let contact = all.iter().find(|c| c.id == cid).cloned();
                            let messages = messenger_store.get().messages.clone();
                            let thread = messages.iter().filter(|m| {
                                (m.sender_id == cid && m.recipient_id == current_user_id) ||
                                (m.sender_id == current_user_id && m.recipient_id == cid)
                            }).cloned().collect::<Vec<_>>();
                            view! {
                                <div class="messenger-thread">
                                    <button class="messenger-back" on:click=move |_| set_selected_contact(None)>"← Back"</button>
                                    <div class="messenger-thread-header">
                                        {contact.as_ref().map(|c| c.name.clone()).unwrap_or_default()}
                                    </div>
                                    <div class="messenger-messages">
                                        {thread.into_iter().map(|m| {
                                            let is_me = m.sender_id == current_user_id;
                                            let cls = if is_me { "messenger-message messenger-message-me" } else { "messenger-message" };
                                            view! {
                                                <div class={cls}>
                                                    <div class="messenger-message-text">{m.content}</div>
                                                    <div class="messenger-message-meta">{format!("{:?}", m.timestamp.format("%H:%M"))}</div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
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
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="messenger-sections">
                                    {sections.iter().map(|(title, source)| {
                                        let section_contacts: Vec<_> = all.iter().filter(|c| c.source == *source).cloned().collect();
                                        if section_contacts.is_empty() {
                                            ().into_any()
                                        } else {
                                            view! {
                                                <div class="messenger-section">
                                                    <div class="messenger-section-title">{*title}</div>
                                                    {section_contacts.into_iter().map(|c| {
                                                        let id = c.id;
                                                        let name = c.name.clone();
                                                        let sub = c.phone.clone().or(c.email.clone()).unwrap_or_else(|| match c.source {
                                                            ContactSource::Bot => "PQC-secured test bot".to_string(),
                                                            ContactSource::Recommended => "Recommended for you".to_string(),
                                                            _ => "Organization member".to_string(),
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
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            }.into_any()
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
                <div class="messenger-drawer-footer">
                    <span class="messenger-encryption-note">
                        "Plain text (development mode)"
                    </span>
                </div>
            </div>
        </div>
    }
}
