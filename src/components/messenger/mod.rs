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

    // One-time seed: organization users, debug bot, and the Red5flag group
    Effect::new(move |_| {
        let org = organization_store.get_untracked();
        let mut contacts = messenger_store.get_untracked().messenger_contacts.clone();

        if contacts.is_empty() && !org.organization_users.is_empty() {
            for user in &org.organization_users {
                contacts.push(MessengerContact {
                    id: user.id,
                    name: user.name.clone(),
                    source: ContactSource::Organization,
                    phone: user.phone.clone(),
                    email: Some(user.email.clone()),
                    icon: user.icon.clone(),
                    members: Vec::new(),
                    unread_count: 0,
                });
            }
        }

        if !contacts.iter().any(|c| c.id == Uuid::nil()) {
            contacts.push(MessengerContact {
                id: Uuid::nil(),
                name: "Debug Bot".to_string(),
                source: ContactSource::Bot,
                phone: None,
                email: None,
                icon: None,
                members: Vec::new(),
                unread_count: 0,
            });
        }

        const RED_EMAILS: [&str; 3] = [
            "Red5flag.bus@gmail.com",
            "red@reddirector.com",
            "red@redmanager.com",
        ];

        let has_red_group = contacts.iter().any(|c| c.source == ContactSource::Group);
        if !has_red_group {
            let mut members = Vec::new();
            for email in RED_EMAILS {
                let email_lower = email.to_lowercase();
                if let Some(c) = contacts.iter().find(|c| {
                    c.email.as_deref().map(|e| e.to_lowercase()) == Some(email_lower.clone())
                }) {
                    members.push(c.id);
                } else {
                    let id = Uuid::new_v4();
                    contacts.push(MessengerContact {
                        id,
                        name: email.to_string(),
                        source: ContactSource::Imported,
                        phone: None,
                        email: Some(email.to_string()),
                        icon: None,
                        members: Vec::new(),
                        unread_count: 0,
                    });
                    members.push(id);
                }
            }
            contacts.push(MessengerContact::group("Red5flag Group", "🚩", members));
        }

        if contacts != messenger_store.get_untracked().messenger_contacts {
            messenger_store.update(|s| s.messenger_contacts = contacts);
        }
    });

    let contacts = Memo::new(move |_| {
        let q = search.get().to_lowercase();
        let all = messenger_store.get().messenger_contacts.clone();
        if q.is_empty() {
            all
        } else {
            all.into_iter()
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

    // Group creation / editing signals
    let (show_new_group, set_show_new_group) = signal(false);
    let (new_group_name, set_new_group_name) = signal(String::from("New Group"));
    let (new_group_icon, set_new_group_icon) = signal(String::from("👥"));
    let (new_group_members, set_new_group_members) = signal(Vec::<Uuid>::new());

    let (show_edit_group, set_show_edit_group) = signal(false);
    let (edit_group_name, set_edit_group_name) = signal(String::new());
    let (edit_group_icon, set_edit_group_icon) = signal(String::new());

    let friend_contacts: Memo<Vec<MessengerContact>> = Memo::new(move |_| {
        contacts
            .get()
            .into_iter()
            .filter(|c| c.source != ContactSource::Recommended && c.source != ContactSource::Group)
            .collect::<Vec<_>>()
    });
    let group_contacts: Memo<Vec<MessengerContact>> = Memo::new(move |_| {
        contacts
            .get()
            .into_iter()
            .filter(|c| c.source == ContactSource::Group)
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
                let last = if c.source == ContactSource::Group {
                    messages.iter().filter(|m| m.recipient_id == c.id).max_by_key(|m| m.timestamp)
                } else {
                    messages.iter().filter(|m| {
                        (m.sender_id == c.id && m.recipient_id == current_user_id)
                            || (m.sender_id == current_user_id && m.recipient_id == c.id)
                    }).max_by_key(|m| m.timestamp)
                };
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
        let is_group = contact.source == ContactSource::Group;
        let thread: Vec<_> = messages
            .iter()
            .filter(|m| {
                let in_thread = if is_group {
                    m.recipient_id == cid
                } else {
                    (m.sender_id == cid && m.recipient_id == current_user_id)
                        || (m.sender_id == current_user_id && m.recipient_id == cid)
                };
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
                            class:active={move || left_tab.get() == "groups"}
                            on:click=move |_| set_left_tab.set("groups")
                        >
                            "Groups"
                        </button>
                        <button
                            class="messenger-tab"
                            class:active={move || left_tab.get() == "requests"}
                            on:click=move |_| set_left_tab.set("requests")
                        >
                            "Requests"
                        </button>
                        <button
                            class="messenger-new-group-btn"
                            on:click=move |_| {
                                set_new_group_name.set("New Group".to_string());
                                set_new_group_icon.set("👥".to_string());
                                set_new_group_members.set(Vec::new());
                                set_show_new_group.set(true);
                            }
                        >
                            "+ New Group"
                        </button>
                    </div>
                    <div class="messenger-sidebar-list">
                        <For
                            each=move || match left_tab.get() {
                                "friends" => friend_contacts.get(),
                                "groups" => group_contacts.get(),
                                _ => request_contacts.get(),
                            }
                            key=|c| c.id
                            children=move |c| {
                                let id = c.id;
                                let name = c.name.clone();
                                let avatar = c.icon.clone().unwrap_or_else(|| name.chars().next().unwrap_or('?').to_string());
                                let sub = if c.source == ContactSource::Group {
                                    let n = c.members.len();
                                    format!("{} member{}", n, if n == 1 { "" } else { "s" })
                                } else {
                                    c.phone.clone().or(c.email.clone()).unwrap_or_else(|| match c.source {
                                        ContactSource::Bot => "PQC-secured test bot".to_string(),
                                        ContactSource::Recommended => "Request".to_string(),
                                        _ => "Friend".to_string(),
                                    })
                                };
                                view! {
                                    <div class="messenger-contact" on:click=move |_| set_selected_contact(Some(id))>
                                        <div class="messenger-contact-avatar">{avatar}</div>
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
                                let avatar = c.icon.clone().unwrap_or_else(|| name.chars().next().unwrap_or('?').to_string());
                                let sub = if c.source == ContactSource::Group {
                                    let n = c.members.len();
                                    format!("{} member{}", n, if n == 1 { "" } else { "s" })
                                } else {
                                    c.phone.clone().or(c.email.clone()).unwrap_or_else(|| match c.source {
                                        ContactSource::Bot => "PQC-secured test bot".to_string(),
                                        ContactSource::Recommended => "Request".to_string(),
                                        _ => "Friend".to_string(),
                                    })
                                };
                                view! {
                                    <div class="messenger-contact" on:click=move |_| set_selected_contact(Some(id))>
                                        <div class="messenger-contact-avatar">{avatar}</div>
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
                            let contact_icon = contact.icon.clone().unwrap_or_default();
                            let contact_is_group = contact.source == ContactSource::Group;
                            let contact_members = contact.members.clone();
                            let edit_name_init = contact_name.clone();
                            let edit_icon_init = contact_icon.clone();
                            let cid = contact.id;
                            let current_user_id = app_store.get().current_user.id;
                            view! {
                                <div class="messenger-chat-navbar">
                                    <div class="messenger-chat-title-block">
                                        <span class="messenger-chat-icon">{contact_icon}</span>
                                        <span class="messenger-chat-title">{contact_name}</span>
                                        {if contact_is_group {
                                            view! {
                                                <button
                                                    class="messenger-chat-action"
                                                    title="Edit group"
                                                    on:click=move |_| {
                                                        set_edit_group_name.set(edit_name_init.clone());
                                                        set_edit_group_icon.set(edit_icon_init.clone());
                                                        set_show_edit_group.set(true);
                                                    }
                                                >
                                                    "✏️"
                                                </button>
                                            }.into_any()
                                        } else { ().into_any() }}
                                    </div>
                                    <div class="messenger-chat-actions">
                                        <button class="messenger-chat-action" title="Call">"📞"</button>
                                        <button class="messenger-chat-action" title="Video">"🎥"</button>
                                        <button class="messenger-chat-action" title="Pin">"�"</button>
                                        <input
                                            class="messenger-chat-search"
                                            type="text"
                                            placeholder="Search chat..."
                                            prop:value={move || chat_search.get()}
                                            on:input=move |ev| set_chat_search.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>
                                {if contact_is_group {
                                    view! {
                                        <div class="messenger-chat-members">
                                            {contact_members.iter().map(|mid| {
                                                let name = contacts.get().iter().find(|c| c.id == *mid).map(|c| c.name.clone()).unwrap_or_else(|| "Unknown".to_string());
                                                view! { <span class="messenger-chat-member">{name}</span> }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                } else { ().into_any() }}
                                {move || if show_edit_group.get() {
                                    view! {
                                        <div class="messenger-group-edit">
                                            <input
                                                type="text"
                                                placeholder="Group name"
                                                prop:value={move || edit_group_name.get()}
                                                on:input=move |ev| set_edit_group_name.set(event_target_value(&ev))
                                            />
                                            <input
                                                type="text"
                                                placeholder="Icon emoji"
                                                prop:value={move || edit_group_icon.get()}
                                                on:input=move |ev| set_edit_group_icon.set(event_target_value(&ev))
                                            />
                                            <button
                                                on:click=move |_| {
                                                    let n = edit_group_name.get();
                                                    let i = edit_group_icon.get();
                                                    messenger_store.update(|s| s.update_group(cid, Some(n), Some(i)));
                                                    set_show_edit_group.set(false);
                                                }
                                            >
                                                "Save"
                                            </button>
                                            <button on:click=move |_| set_show_edit_group.set(false)>"Cancel"</button>
                                        </div>
                                    }.into_any()
                                } else { ().into_any() }}
                                <div class="messenger-messages">
                                    <For
                                        each=move || thread.clone()
                                        key=|m| m.id
                                        children=move |m| {
                                            let text = m.content.clone();
                                            let ts = format!("{:?}", m.timestamp.format("%H:%M"));
                                            let is_me = m.sender_id == current_user_id;
                                            let sender = if contact_is_group {
                                                if is_me { "You".to_string() } else {
                                                    contacts.get().iter().find(|c| c.id == m.sender_id).map(|c| c.name.clone()).unwrap_or_else(|| "Unknown".to_string())
                                                }
                                            } else { String::new() };
                                            let cls = if is_me { "messenger-message messenger-message-me" } else { "messenger-message" };
                                            view! {
                                                <div class={cls}>
                                                    {if contact_is_group {
                                                        view! { <div class="messenger-message-sender">{sender.clone()}</div> }.into_any()
                                                    } else { ().into_any() }}
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
                                        placeholder=if contact_is_group { "Message group..." } else { "Type a secure message..." }
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
                {move || if show_new_group.get() {
                    view! {
                        <div class="messenger-modal-overlay" on:click=move |_| set_show_new_group.set(false)>
                            <div class="messenger-modal" on:click=|ev| ev.stop_propagation()>
                                <div class="messenger-modal-header">"New Group"</div>
                                <input
                                    type="text"
                                    placeholder="Group name"
                                    prop:value={move || new_group_name.get()}
                                    on:input=move |ev| set_new_group_name.set(event_target_value(&ev))
                                />
                                <input
                                    type="text"
                                    placeholder="Icon emoji"
                                    prop:value={move || new_group_icon.get()}
                                    on:input=move |ev| set_new_group_icon.set(event_target_value(&ev))
                                />
                                <div class="messenger-modal-members">
                                    <For
                                        each=move || friend_contacts.get()
                                        key=|c| c.id
                                        children=move |c| {
                                            let id = c.id;
                                            view! {
                                                <label class="messenger-modal-member">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked={move || new_group_members.get().contains(&id)}
                                                        on:click=move |_| set_new_group_members.update(|v| {
                                                            if v.contains(&id) {
                                                                v.retain(|x| x != &id);
                                                            } else {
                                                                v.push(id);
                                                            }
                                                        })
                                                    />
                                                    {c.name.clone()}
                                                </label>
                                            }
                                        }
                                    />
                                </div>
                                <div class="messenger-modal-actions">
                                    <button
                                        on:click=move |_| {
                                            let name = new_group_name.get().trim().to_string();
                                            if !name.is_empty() {
                                                let icon = new_group_icon.get().trim().to_string();
                                                let members = new_group_members.get();
                                                messenger_store.update(|s| { let _ = s.create_group(name, Some(icon), members); });
                                                set_show_new_group.set(false);
                                            }
                                        }
                                    >
                                        "Create"
                                    </button>
                                    <button on:click=move |_| set_show_new_group.set(false)>"Cancel"</button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else { ().into_any() }}
            </div>
        </div>
    }
}
