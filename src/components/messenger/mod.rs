use crate::models::{
    Asset, ContactSource, Document, Message, MessageEmbed, MessageStatus, MessengerContact,
    MessengerRequest, MessengerRequestKind, Portfolio, ServiceTask, ServiceTaskStatus,
    ServiceTaskType,
};
use crate::pages::portfolios::{marketplace_display_label, phantom_edit_scope, DocModal};
use crate::server::{
    get_messenger_state, get_secure_messages, save_messenger_state, send_secure_message,
};
use crate::stores::{use_app_store, use_messenger_store, use_organization_store, AppStore};
use crate::types::{Currency, TabType};
use chrono::{Duration, Utc};
use leptos::prelude::*;
use uuid::Uuid;

/// Display info resolved live from the app store for an embedded entity.
struct EmbedInfo {
    icon: &'static str,
    kind: &'static str,
    title: String,
    sub: String,
}

/// Item in the composer attach picker.
#[derive(Clone, PartialEq)]
struct AttachItem {
    embed: MessageEmbed,
    icon: &'static str,
    label: String,
    sub: String,
}

fn embed_key(embed: &MessageEmbed) -> String {
    match embed {
        MessageEmbed::Portfolio(id) => format!("p-{}", id),
        MessageEmbed::AssetGroup(id) => format!("g-{}", id),
        MessageEmbed::Asset(id) => format!("a-{}", id),
        MessageEmbed::Document(id) => format!("d-{}", id),
        MessageEmbed::Listing { asset_id, marketplace } => {
            format!("l-{}-{}", asset_id, marketplace)
        }
    }
}

fn portfolio_stats(p: &Portfolio) -> String {
    let assets = p.assets.len()
        + p.asset_groups
            .iter()
            .map(|g| g.assets.len())
            .sum::<usize>();
    let groups = p.asset_groups.len();
    format!(
        "{} assets · {} groups · {:.2} {}",
        assets, groups, p.total_value, p.currency
    )
}

fn group_stats(g: &crate::models::AssetGroup, p: &Portfolio) -> String {
    format!(
        "{} · {} assets · {:.2} {}",
        p.name,
        g.assets.len(),
        g.total_value,
        p.currency
    )
}

fn asset_sub(a: &Asset, portfolio_name: &str, currency: &Currency) -> String {
    let mut s = format!("{:?} · {:.2} {}", a.asset_type, a.current_value, currency);
    if let Some(loc) = a.location.as_deref().filter(|l| !l.is_empty()) {
        s.push_str(&format!(" · {}", loc));
    }
    format!("{} · {}", portfolio_name, s)
}

fn doc_sub(owner: &str, d: &Document) -> String {
    format!(
        "{} · {} · {}",
        owner,
        d.file_type,
        d.uploaded_at.format("%Y-%m-%d")
    )
}

/// Find a document and the id of the entity that owns it (portfolio, group, or asset).
fn find_document(app: &AppStore, doc_id: Uuid) -> Option<(Uuid, Document)> {
    for p in &app.portfolios {
        if let Some(d) = p.documents.iter().find(|d| d.id == doc_id) {
            return Some((p.id, d.clone()));
        }
        for g in &p.asset_groups {
            if let Some(d) = g.documents.iter().find(|d| d.id == doc_id) {
                return Some((g.id, d.clone()));
            }
            for a in &g.assets {
                if let Some(d) = a.documents.iter().find(|d| d.id == doc_id) {
                    return Some((a.id, d.clone()));
                }
            }
        }
        for a in &p.assets {
            if let Some(d) = a.documents.iter().find(|d| d.id == doc_id) {
                return Some((a.id, d.clone()));
            }
        }
    }
    None
}

/// Display name for an owning entity id (portfolio / group / asset).
fn entity_name(app: &AppStore, id: Uuid) -> Option<String> {
    for p in &app.portfolios {
        if p.id == id {
            return Some(p.name.clone());
        }
        for g in &p.asset_groups {
            if g.id == id {
                return Some(format!("{} / {}", p.name, g.name));
            }
            for a in &g.assets {
                if a.id == id {
                    return Some(a.name.clone());
                }
            }
        }
        for a in &p.assets {
            if a.id == id {
                return Some(a.name.clone());
            }
        }
    }
    None
}

fn resolve_embed(app: &AppStore, embed: &MessageEmbed) -> Option<EmbedInfo> {
    match embed {
        MessageEmbed::Portfolio(id) => app
            .portfolios
            .iter()
            .find(|p| p.id == *id)
            .map(|p| EmbedInfo {
                icon: "📁",
                kind: "Portfolio",
                title: p.name.clone(),
                sub: portfolio_stats(p),
            }),
        MessageEmbed::AssetGroup(id) => app.portfolios.iter().find_map(|p| {
            p.asset_groups
                .iter()
                .find(|g| g.id == *id)
                .map(|g| EmbedInfo {
                    icon: "📂",
                    kind: "Asset Group",
                    title: g.name.clone(),
                    sub: group_stats(g, p),
                })
        }),
        MessageEmbed::Asset(id) => app.portfolios.iter().find_map(|p| {
            p.assets
                .iter()
                .find(|a| a.id == *id)
                .map(|a| EmbedInfo {
                    icon: "📦",
                    kind: "Asset",
                    title: a.name.clone(),
                    sub: asset_sub(a, &p.name, &p.currency),
                })
                .or_else(|| {
                    p.asset_groups.iter().find_map(|g| {
                        g.assets.iter().find(|a| a.id == *id).map(|a| EmbedInfo {
                            icon: "📦",
                            kind: "Asset",
                            title: a.name.clone(),
                            sub: asset_sub(a, &p.name, &p.currency),
                        })
                    })
                })
        }),
        MessageEmbed::Document(id) => find_document(app, *id).map(|(entity_id, d)| {
            let owner = entity_name(app, entity_id).unwrap_or_else(|| "Unknown".to_string());
            EmbedInfo {
                icon: "📄",
                kind: "Document",
                title: d.name.clone(),
                sub: doc_sub(&owner, &d),
            }
        }),
        MessageEmbed::Listing { asset_id, marketplace } => {
            let asset = app
                .portfolios
                .iter()
                .flat_map(|p| {
                    p.assets
                        .iter()
                        .chain(p.asset_groups.iter().flat_map(|g| g.assets.iter()))
                })
                .find(|a| a.id == *asset_id)?;
            // Red Marketplace listings live at metadata["red_listing"];
            // external marketplaces under marketplace_listings[key].
            let listing = if marketplace == "Red" {
                asset.metadata.get("red_listing")?
            } else {
                asset
                    .metadata
                    .get("marketplace_listings")
                    .and_then(|v| v.get(marketplace))?
            };
            let title = listing
                .get("title")
                .and_then(|v| v.as_str())
                .filter(|t| !t.is_empty())
                .unwrap_or(&asset.name)
                .to_string();
            let price = listing
                .get("price")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let status = listing
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("draft");
            let kind = marketplace_display_label(marketplace);
            let price_str = if price.is_empty() { "0.00".to_string() } else { price.to_string() };
            Some(EmbedInfo {
                icon: "🏷️",
                kind,
                title,
                sub: format!("{} listing · ${} · {}", kind, price_str, status),
            })
        }
    }
}

#[component]
pub fn MessageDrawer() -> impl IntoView {
    let app_store = use_app_store();
    let messenger_store = use_messenger_store();
    let organization_store = use_organization_store();
    let (draft, set_draft) = signal(String::new());
    let (search, set_search) = signal(String::new());
    // Embedded content (portfolios, groups, assets, documents)
    let (pending_embeds, set_pending_embeds) = signal(Vec::<MessageEmbed>::new());
    let (show_attach, set_show_attach) = signal(false);
    let (attach_search, set_attach_search) = signal(String::new());
    let (doc_view, set_doc_view) = signal(Option::<(Uuid, Document)>::None);

    // Sidebar resize / collapse state: width in px, plus which side is hidden
    let (sidebar_w, set_sidebar_w) = signal(280.0_f64);
    // Compact "square" view: contacts render as a grid of square tiles.
    let (sidebar_compact, set_sidebar_compact) = signal(false);
    let (left_collapsed, set_left_collapsed) = signal(false);
    let (right_collapsed, set_right_collapsed) = signal(false);
    // Some((start_x, start_width, body_width)) while a resize drag is active
    let (resize_drag, set_resize_drag) = signal(Option::<(f64, f64, f64)>::None);

    let install_resize_listeners = move || {
        thread_local! {
            static RESIZE_INSTALLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
        }
        RESIZE_INSTALLED.with(|installed| {
            if installed.replace(true) {
                return;
            }
            #[cfg(target_arch = "wasm32")]
            if let Some(w) = web_sys::window() {
                use wasm_bindgen::JsCast;
                let mv = wasm_bindgen::closure::Closure::wrap(Box::new(
                    move |ev: web_sys::MouseEvent| {
                        // Self-heal: if the button was released outside the
                        // window the mouseup never arrived — stop resizing
                        // instead of collapsing panels on every mousemove.
                        if ev.buttons() & 1 == 0 {
                            set_resize_drag.set(None);
                            return;
                        }
                        let Some((start_x, start_w, body_w)) = resize_drag.get_untracked()
                        else {
                            return;
                        };
                        let w = start_w + (ev.client_x() as f64 - start_x);
                        if w < 60.0 {
                            set_left_collapsed.set(true);
                            set_right_collapsed.set(false);
                        } else if w > body_w - 60.0 {
                            set_right_collapsed.set(true);
                            set_left_collapsed.set(false);
                        } else if w < 190.0 {
                            // Snap zone: compact square ("box") view
                            set_left_collapsed.set(false);
                            set_right_collapsed.set(false);
                            set_sidebar_compact.set(true);
                        } else {
                            set_left_collapsed.set(false);
                            set_right_collapsed.set(false);
                            set_sidebar_compact.set(false);
                            set_sidebar_w.set(w);
                        }
                    },
                ) as Box<dyn FnMut(web_sys::MouseEvent)>);
                let _ = w.add_event_listener_with_callback(
                    "mousemove",
                    mv.as_ref().unchecked_ref(),
                );
                mv.forget();

                let up = wasm_bindgen::closure::Closure::wrap(Box::new(
                    move |_: web_sys::MouseEvent| {
                        set_resize_drag.set(None);
                        if let Some(d) =
                            web_sys::window().and_then(|win| win.document())
                        {
                            if let Some(b) = d.body() {
                                let _ = b.style().set_property("cursor", "");
                                let _ = b.style().set_property("user-select", "");
                            }
                        }
                    },
                ) as Box<dyn FnMut(web_sys::MouseEvent)>);
                let _ = w.add_event_listener_with_callback(
                    "mouseup",
                    up.as_ref().unchecked_ref(),
                );
                up.forget();
            }
        });
    };

    let on_resizer_down = move |ev: leptos::ev::MouseEvent| {
        #[cfg(target_arch = "wasm32")]
        {
            ev.prevent_default();
            ev.stop_propagation();
            let body_w = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.document_element())
                .map(|e| e.client_width() as f64)
                .unwrap_or(1280.0);
            // In box view the displayed width is the fixed compact size, so
            // the drag must measure from that — not the last name-view width.
            let start_w = if sidebar_compact.get() { 150.0 } else { sidebar_w.get() };
            set_resize_drag.set(Some((ev.client_x() as f64, start_w, body_w)));
            if let Some(d) = web_sys::window().and_then(|win| win.document()) {
                if let Some(b) = d.body() {
                    let _ = b.style().set_property("cursor", "col-resize");
                    let _ = b.style().set_property("user-select", "none");
                }
            }
            install_resize_listeners();
        }
    };

    let restore_panels = move |_| {
        set_sidebar_w.set(280.0);
        set_left_collapsed.set(false);
        set_right_collapsed.set(false);
    };

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

        let group = contacts
            .iter()
            .find(|c| c.source == ContactSource::Group)
            .cloned();

        if contacts != messenger_store.get_untracked().messenger_contacts {
            messenger_store.update(|s| s.messenger_contacts = contacts);
        }

        // Seed incoming requests once contacts have been populated.
        let app = app_store.get_untracked();
        let current_user_id = app.current_user.id;
        let mut requests = messenger_store.get_untracked().messenger_requests.clone();

        if requests.is_empty() && !org.organization_users.is_empty() {
            if let Some(other) = org
                .organization_users
                .iter()
                .find(|u| u.id != current_user_id)
            {
                requests.push(
                    MessengerRequest::new(MessengerRequestKind::FriendRequest, other.id, other.id)
                        .with_message("Would like to connect"),
                );
            }

            if let Some(group) = group.as_ref() {
                if let Some(manager) = org.organization_users.iter().find(|u| {
                    matches!(
                        u.role,
                        crate::types::UserRole::Owner
                            | crate::types::UserRole::Director
                            | crate::types::UserRole::SeniorManager
                            | crate::types::UserRole::Manager
                    )
                }) {
                    requests.push(
                        MessengerRequest::new(
                            MessengerRequestKind::GroupInvite,
                            manager.id,
                            group.id,
                        )
                        .with_message(format!("You have been invited to {}", group.name)),
                    );
                }
            }

            if let Some(manager) = org.organization_users.iter().find(|u| {
                matches!(
                    u.role,
                    crate::types::UserRole::Owner
                        | crate::types::UserRole::Director
                        | crate::types::UserRole::SeniorManager
                        | crate::types::UserRole::Manager
                )
            }) {
                let asset_id = app
                    .portfolios
                    .iter()
                    .flat_map(|p| p.assets.iter())
                    .next()
                    .map(|a| a.id)
                    .unwrap_or(Uuid::nil());
                let mut task = ServiceTask::new(
                    asset_id,
                    None,
                    ServiceTaskType::Maintenance,
                    Utc::now(),
                    Utc::now() + Duration::hours(2),
                );
                task.assigned_to = Some(current_user_id);
                task.status = ServiceTaskStatus::Pending;
                let task_id = task.id;
                app_store.update(|s| s.add_service_task(task));
                requests.push(
                    MessengerRequest::new(MessengerRequestKind::WorkRequest, manager.id, task_id)
                        .with_message("New maintenance job order"),
                );
            }
        }

        if requests != messenger_store.get_untracked().messenger_requests {
            messenger_store.update(|s| s.messenger_requests = requests);
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

    // Pull the PQC-sealed message store + messenger directory from the
    // server each time the drawer opens. Restoring contacts is what keeps
    // group ids stable so stored group messages still thread correctly.
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            if let Ok(msgs) = get_secure_messages().await {
                messenger_store.update(|s| s.merge_messages(msgs));
            }
            if let Ok(Some(state)) = get_messenger_state().await {
                messenger_store.update(|s| {
                    // Merge rather than replace: contacts/requests created
                    // locally while the drawer was closed (e.g. marketplace
                    // listing threads) must survive the server restore.
                    let local_contacts = std::mem::take(&mut s.messenger_contacts);
                    let mut contacts = state.contacts;
                    for c in local_contacts {
                        if !contacts.iter().any(|m| m.id == c.id) {
                            contacts.push(c);
                        }
                    }
                    s.messenger_contacts = contacts;
                    let local_requests = std::mem::take(&mut s.messenger_requests);
                    let mut requests = state.requests;
                    for r in local_requests {
                        if !requests.iter().any(|m| m.id == r.id) {
                            requests.push(r);
                        }
                    }
                    s.messenger_requests = requests;
                });
            }
        });
    });

    // Persist the messenger directory whenever contacts or requests actually
    // change (Memo compare keeps this from firing on unrelated store updates).
    #[cfg(feature = "hydrate")]
    {
        let contacts_memo = Memo::new(move |_| messenger_store.get().messenger_contacts.clone());
        let requests_memo = Memo::new(move |_| messenger_store.get().messenger_requests.clone());
        Effect::new(move |_| {
            let state = crate::models::MessengerState {
                contacts: contacts_memo.get(),
                requests: requests_memo.get(),
            };
            leptos::task::spawn_local(async move {
                let _ = save_messenger_state(state).await;
            });
        });
    }

    // Persist a message through the secure server path: the server PQC-seals
    // it into the designated store. Status stays Pending until the round-trip
    // confirms — a message that never reaches Sent means sealing failed.
    let persist_message = move |msg: Message| {
        let mid = msg.id;
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let res = send_secure_message(msg).await;
            messenger_store.update(|s| {
                s.set_message_status(
                    mid,
                    if res.is_ok() {
                        MessageStatus::Sent
                    } else {
                        MessageStatus::Failed
                    },
                );
            });
        });
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = msg;
            messenger_store.update(|s| s.set_message_status(mid, MessageStatus::Sent));
        }
    };

    let send_message = move |recipient_id: Uuid| {
        let text = draft.get();
        let embeds = pending_embeds.get();
        let current_user_id = app_store.get().current_user.id;
        if text.trim().is_empty() && embeds.is_empty() {
            return;
        }
        let mut sent_slot = None;
        messenger_store.update(|s| {
            sent_slot = Some(s.send_message_with_embeds(
                current_user_id,
                recipient_id,
                text.clone(),
                embeds.clone(),
            ));
        });
        if let Some(sent_msg) = sent_slot {
            persist_message(sent_msg);
        }
        // Echo from bot for testing
        if recipient_id == Uuid::nil() {
            let reply = format!("Bot received: {}", text);
            let mut reply_slot = None;
            messenger_store.update(|s| {
                reply_slot = Some(s.receive_message_with_embeds(
                    Uuid::nil(),
                    current_user_id,
                    reply,
                    Vec::new(),
                ));
            });
            if let Some(reply_msg) = reply_slot {
                persist_message(reply_msg);
            }
        }
        set_draft.set(String::new());
        set_pending_embeds.set(Vec::new());
    };

    let (left_tab, set_left_tab) = signal("recent");
    let (chat_search, set_chat_search) = signal(String::new());

    // Group creation / editing signals
    let (show_new_group, set_show_new_group) = signal(false);
    let (new_group_name, set_new_group_name) = signal(String::from("New Group"));
    let (new_group_icon, set_new_group_icon) = signal(String::from("👥"));
    let (new_group_members, set_new_group_members) = signal(Vec::<Uuid>::new());

    let (show_edit_group, set_show_edit_group) = signal(false);
    let (edit_group_name, set_edit_group_name) = signal(String::new());
    let (edit_group_icon, set_edit_group_icon) = signal(String::new());

    // Phantom outside-tap dismissal for the attach picker (same pattern as inline editors)
    let attach_scope_alive = phantom_edit_scope(
        "messenger-attach".to_string(),
        move || show_attach.get(),
        move || set_show_attach.set(false),
    );
    on_cleanup(move || {
        attach_scope_alive.store(false, std::sync::atomic::Ordering::Relaxed);
    });

    let attach_items: Memo<Vec<AttachItem>> = Memo::new(move |_| {
        let q = attach_search.get().trim().to_lowercase();
        let app = app_store.get();
        let mut items: Vec<AttachItem> = Vec::new();
        let mut push = |embed: MessageEmbed, icon: &'static str, label: String, sub: String| {
            if q.is_empty()
                || label.to_lowercase().contains(&q)
                || sub.to_lowercase().contains(&q)
            {
                items.push(AttachItem {
                    embed,
                    icon,
                    label,
                    sub,
                });
            }
        };
        for p in &app.portfolios {
            push(
                MessageEmbed::Portfolio(p.id),
                "📁",
                p.name.clone(),
                portfolio_stats(p),
            );
            for g in &p.asset_groups {
                push(
                    MessageEmbed::AssetGroup(g.id),
                    "📂",
                    g.name.clone(),
                    group_stats(g, p),
                );
                for a in &g.assets {
                    push(
                        MessageEmbed::Asset(a.id),
                        "📦",
                        a.name.clone(),
                        asset_sub(a, &p.name, &p.currency),
                    );
                }
                for d in &g.documents {
                    push(
                        MessageEmbed::Document(d.id),
                        "📄",
                        d.name.clone(),
                        doc_sub(&format!("{} / {}", p.name, g.name), d),
                    );
                }
            }
            for a in &p.assets {
                push(
                    MessageEmbed::Asset(a.id),
                    "📦",
                    a.name.clone(),
                    asset_sub(a, &p.name, &p.currency),
                );
            }
            for d in &p.documents {
                push(
                    MessageEmbed::Document(d.id),
                    "📄",
                    d.name.clone(),
                    doc_sub(&p.name, d),
                );
            }
        }
        // Marketplace listings — each saved listing is attachable so a
        // conversation can reference the exact posting.
        for p in &app.portfolios {
            for a in p
                .assets
                .iter()
                .chain(p.asset_groups.iter().flat_map(|g| g.assets.iter()))
            {
                if let Some(listings) = a
                    .metadata
                    .get("marketplace_listings")
                    .and_then(|v| v.as_object())
                {
                for (key, l) in listings {
                    let title = l
                        .get("title")
                        .and_then(|v| v.as_str())
                        .filter(|t| !t.is_empty())
                        .unwrap_or(&a.name)
                        .to_string();
                    let price = l
                        .get("price")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let status = l
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("draft");
                    let kind = marketplace_display_label(key);
                    let price_str =
                        if price.is_empty() { "0.00".to_string() } else { price.to_string() };
                    push(
                        MessageEmbed::Listing {
                            asset_id: a.id,
                            marketplace: key.clone(),
                        },
                        "🏷️",
                        title,
                        format!("{} listing · ${} · {}", kind, price_str, status),
                    );
                }
                }
                // Internal Red Marketplace listing (separate metadata slot).
                if let Some(l) = a.metadata.get("red_listing") {
                    let title = l
                        .get("title")
                        .and_then(|v| v.as_str())
                        .filter(|t| !t.is_empty())
                        .unwrap_or(&a.name)
                        .to_string();
                    let price = l
                        .get("price")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let status = l
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("draft");
                    let price_str =
                        if price.is_empty() { "0.00".to_string() } else { price.to_string() };
                    push(
                        MessageEmbed::Listing {
                            asset_id: a.id,
                            marketplace: "Red".to_string(),
                        },
                        "🏪",
                        title,
                        format!("Red Marketplace listing · ${} · {}", price_str, status),
                    );
                }
            }
        }
        items
    });

    let toggle_embed = move |embed: MessageEmbed| {
        set_pending_embeds.update(|v| {
            if let Some(pos) = v.iter().position(|e| *e == embed) {
                v.remove(pos);
            } else {
                v.push(embed);
            }
        });
    };

    let open_portfolios_tab = move || {
        app_store.update(|s| s.expand_tab(TabType::Portfolios));
        messenger_store.update(|s| s.set_message_drawer(false));
    };

    let render_embed_card = move |embed: MessageEmbed| {
        view! {
            {move || {
                let app = app_store.get();
                match resolve_embed(&app, &embed) {
                    Some(info) => {
                        let kind = info.kind;
                        let title = info.title.clone();
                        let sub = info.sub.clone();
                        let icon = info.icon;
                        let card_embed = embed.clone();
                        let open_docs = open_portfolios_tab;
                        view! {
                            <button
                                class="messenger-embed-card"
                                title={format!("Open {}", kind)}
                                on:click=move |_| {
                                    match &card_embed {
                                        MessageEmbed::Document(doc_id) => {
                                            if let Some((entity_id, doc)) =
                                                find_document(&app_store.get(), *doc_id)
                                            {
                                                set_doc_view.set(Some((entity_id, doc)));
                                            }
                                        }
                                        _ => open_docs(),
                                    }
                                }
                            >
                                <span class="messenger-embed-icon">{icon}</span>
                                <span class="messenger-embed-body">
                                    <span class="messenger-embed-kind">{kind}</span>
                                    <span class="messenger-embed-title">{title}</span>
                                    <span class="messenger-embed-sub">{sub}</span>
                                </span>
                            </button>
                        }
                        .into_any()
                    }
                    None => view! {
                        <div class="messenger-embed-card messenger-embed-missing">
                            <span class="messenger-embed-icon">{"❓"}</span>
                            <span class="messenger-embed-body">
                                <span class="messenger-embed-kind">"Unavailable"</span>
                                <span class="messenger-embed-title">"This item is no longer available"</span>
                            </span>
                        </div>
                    }
                    .into_any(),
                }
            }}
        }
        .into_any()
    };

    let friend_contacts: Memo<Vec<MessengerContact>> = Memo::new(move |_| {
        contacts
            .get()
            .into_iter()
            .filter(|c| {
                c.source != ContactSource::Recommended
                    && c.source != ContactSource::Group
                    && c.source != ContactSource::Marketplace
            })
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
                    messages
                        .iter()
                        .filter(|m| m.recipient_id == c.id)
                        .max_by_key(|m| m.timestamp)
                } else {
                    messages
                        .iter()
                        .filter(|m| {
                            (m.sender_id == c.id && m.recipient_id == current_user_id)
                                || (m.sender_id == current_user_id && m.recipient_id == c.id)
                        })
                        .max_by_key(|m| m.timestamp)
                };
                last.map(|m| (c.clone(), m.timestamp))
            })
            .collect();
        with_last.sort_by(|a, b| b.1.cmp(&a.1));
        with_last.into_iter().map(|(c, _)| c).collect()
    });

    // Contact resolution deliberately excludes messages: the chat pane's
    // mount gate must not re-run on every send/receive, or the whole chat
    // DOM would be recreated (the "view resets" bug). Message updates flow
    // through `thread_messages` → the keyed <For>, which diffs granularly.
    let active_contact = Memo::new(move |_| {
        let all = contacts.get();
        let cid = messenger_store.get().selected_chat_id?;
        all.iter().find(|c| c.id == cid).cloned()
    });

    let thread_messages = Memo::new(move |_| {
        let Some(contact) = active_contact.get() else {
            return Vec::new();
        };
        let cid = contact.id;
        let current_user_id = app_store.get().current_user.id;
        let q = chat_search.get().to_lowercase();
        let messages = messenger_store.get().messages.clone();
        let is_group = contact.source == ContactSource::Group;
        messages
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
            .collect::<Vec<_>>()
    });

    let friend_requests: Memo<Vec<MessengerRequest>> = Memo::new(move |_| {
        messenger_store
            .get()
            .messenger_requests
            .iter()
            .filter(|r| matches!(r.kind, MessengerRequestKind::FriendRequest))
            .cloned()
            .collect()
    });
    let group_invites: Memo<Vec<MessengerRequest>> = Memo::new(move |_| {
        messenger_store
            .get()
            .messenger_requests
            .iter()
            .filter(|r| matches!(r.kind, MessengerRequestKind::GroupInvite))
            .cloned()
            .collect()
    });
    let work_requests: Memo<Vec<MessengerRequest>> = Memo::new(move |_| {
        messenger_store
            .get()
            .messenger_requests
            .iter()
            .filter(|r| matches!(r.kind, MessengerRequestKind::WorkRequest))
            .cloned()
            .collect()
    });

    #[derive(Clone, PartialEq)]
    enum SidebarListItem {
        Contact(MessengerContact),
        Request(MessengerRequest),
    }

    let friend_items: Memo<Vec<SidebarListItem>> = Memo::new(move |_| {
        let mut items: Vec<_> = friend_requests
            .get()
            .into_iter()
            .map(SidebarListItem::Request)
            .collect();
        items.extend(
            friend_contacts
                .get()
                .into_iter()
                .map(SidebarListItem::Contact),
        );
        items
    });
    let recent_items: Memo<Vec<SidebarListItem>> = Memo::new(move |_| {
        let mut items: Vec<_> = recent_contacts
            .get()
            .into_iter()
            .map(SidebarListItem::Contact)
            .collect();
        let mut seen: std::collections::HashSet<uuid::Uuid> = items
            .iter()
            .filter_map(|item| match item {
                SidebarListItem::Contact(c) => Some(c.id),
                SidebarListItem::Request(_) => None,
            })
            .collect();
        for group in group_contacts.get() {
            if seen.insert(group.id) {
                items.push(SidebarListItem::Contact(group));
            }
        }
        items
    });
    let group_items: Memo<Vec<SidebarListItem>> = Memo::new(move |_| {
        let mut items: Vec<_> = group_invites
            .get()
            .into_iter()
            .map(SidebarListItem::Request)
            .collect();
        items.extend(
            group_contacts
                .get()
                .into_iter()
                .map(SidebarListItem::Contact),
        );
        items
    });
    let request_items: Memo<Vec<SidebarListItem>> = Memo::new(move |_| {
        let mut items: Vec<_> = group_invites
            .get()
            .into_iter()
            .map(SidebarListItem::Request)
            .collect();
        items.extend(
            work_requests
                .get()
                .into_iter()
                .map(SidebarListItem::Request),
        );
        items.extend(
            request_contacts
                .get()
                .into_iter()
                .map(SidebarListItem::Contact),
        );
        items
    });

    let accept_request = move |req: MessengerRequest| {
        let current_user_id = app_store.get().current_user.id;
        match req.kind {
            MessengerRequestKind::FriendRequest => {
                let org = organization_store.get();
                if let Some(user) = org
                    .organization_users
                    .iter()
                    .find(|u| u.id == req.target_id)
                {
                    let contact = MessengerContact {
                        id: user.id,
                        name: user.name.clone(),
                        source: ContactSource::Imported,
                        phone: user.phone.clone(),
                        email: Some(user.email.clone()),
                        icon: user.icon.clone().or(user.avatar_url.clone()),
                        members: Vec::new(),
                        unread_count: 0,
                    };
                    messenger_store.update(|s| s.accept_friend_request(req.id, contact));
                }
            }
            MessengerRequestKind::GroupInvite => {
                messenger_store.update(|s| s.accept_group_invite(req.id, current_user_id));
            }
            MessengerRequestKind::WorkRequest => {
                app_store.update(|s| {
                    if let Some(t) = s.get_service_task_mut(req.target_id) {
                        t.status = ServiceTaskStatus::Scheduled;
                        t.updated_at = Utc::now();
                    }
                });
                messenger_store.update(|s| {
                    s.remove_request(req.id);
                });
            }
        }
    };

    let decline_request = move |id: Uuid| {
        if let Some(req) = messenger_store
            .get()
            .messenger_requests
            .iter()
            .find(|r| r.id == id)
            .cloned()
        {
            if matches!(req.kind, MessengerRequestKind::WorkRequest) {
                app_store.update(|s| {
                    if let Some(t) = s.get_service_task_mut(req.target_id) {
                        t.status = ServiceTaskStatus::Cancelled;
                        t.updated_at = Utc::now();
                    }
                });
            }
            messenger_store.update(|s| s.decline_request(id));
        }
    };

    let render_request = move |req: MessengerRequest| {
        let req_id = req.id;
        let org = organization_store.get();
        let store = messenger_store.get();
        let app = app_store.get();
        let from_user = org
            .organization_users
            .iter()
            .find(|u| u.id == req.from_user_id);
        let (avatar, title, sub) = match req.kind {
            MessengerRequestKind::FriendRequest => {
                let name = from_user
                    .map(|u| u.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let avatar = from_user
                    .and_then(|u| u.icon.clone().or(u.avatar_url.clone()))
                    .unwrap_or_else(|| "👤".to_string());
                (
                    avatar,
                    format!("Friend request from {}", name),
                    req.message.clone().unwrap_or_default(),
                )
            }
            MessengerRequestKind::GroupInvite => {
                let group = store
                    .messenger_contacts
                    .iter()
                    .find(|c| c.id == req.target_id);
                let group_name = group
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let avatar = group
                    .and_then(|c| c.icon.clone())
                    .unwrap_or_else(|| "👥".to_string());
                let from_name = from_user
                    .map(|u| u.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let sub = format!(
                    "Invited by {}{}",
                    from_name,
                    req.message
                        .as_ref()
                        .map(|m| format!(" · {}", m))
                        .unwrap_or_default()
                );
                (avatar, format!("Group invite: {}", group_name), sub)
            }
            MessengerRequestKind::WorkRequest => {
                let task = app.service_tasks.iter().find(|t| t.id == req.target_id);
                let task_title = task
                    .map(|t| t.display_title())
                    .unwrap_or_else(|| "Work request".to_string());
                let from_name = from_user
                    .map(|u| u.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let sub = format!(
                    "Job order from {}{}",
                    from_name,
                    req.message
                        .as_ref()
                        .map(|m| format!(" · {}", m))
                        .unwrap_or_default()
                );
                ("🛠".to_string(), task_title, sub)
            }
        };
        let accept_req = req.clone();
        view! {
            <div class="messenger-request">
                <div class="messenger-request-avatar">{avatar}</div>
                <div class="messenger-request-info">
                    <div class="messenger-request-title">{title}</div>
                    <div class="messenger-request-sub">{sub}</div>
                </div>
                <div class="messenger-request-actions">
                    <button
                        class="messenger-request-accept"
                        on:click=move |_| accept_request(accept_req.clone())
                    >
                        "Accept"
                    </button>
                    <button
                        class="messenger-request-decline"
                        on:click=move |_| decline_request(req_id)
                    >
                        "Decline"
                    </button>
                </div>
            </div>
        }
        .into_any()
    };

    view! {
        <div class="messenger-drawer-overlay" on:click=on_close>
            <div class="messenger-screen" on:click=|ev| ev.stop_propagation()>
                <div class="messenger-controls-bar">
                    <button
                        class="messenger-tab"
                        class:active={move || left_tab.get() == "recent"}
                        on:click=move |_| set_left_tab.set("recent")>
                        "Recent"
                    </button>
                    <button
                        class="messenger-tab"
                        class:active={move || left_tab.get() == "friends"}
                        on:click=move |_| set_left_tab.set("friends")>
                        "Friends"
                    </button>
                    <button
                        class="messenger-tab"
                        class:active={move || left_tab.get() == "groups"}
                        on:click=move |_| set_left_tab.set("groups")>
                        "Groups"
                    </button>
                    <button
                        class="messenger-tab"
                        class:active={move || left_tab.get() == "requests"}
                        on:click=move |_| set_left_tab.set("requests")>
                        "Requests"
                    </button>
                    <button
                        class="messenger-tab messenger-square-toggle"
                        class:active={move || sidebar_compact.get()}
                        title="Square box view — drag the divider right for names"
                        on:click=move |_| {
                            if left_collapsed.get() {
                                // Open straight into the box view
                                set_left_collapsed.set(false);
                                set_sidebar_compact.set(true);
                            } else {
                                set_sidebar_compact.update(|v| *v = !*v);
                            }
                        }>
                        "▦"
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
                <div class="messenger-body">
                    <div
                        class="messenger-sidebar"
                        class:msb-hidden={move || left_collapsed.get()}
                        class:msb-compact={move || sidebar_compact.get() && !right_collapsed.get()}
                        style={move || {
                            if right_collapsed.get() {
                                "flex:1 1 auto;width:auto;min-width:0;max-width:none;"
                                    .to_string()
                            } else if sidebar_compact.get() {
                                "width:150px;min-width:150px;max-width:150px;"
                                    .to_string()
                            } else {
                                let w = sidebar_w.get();
                                format!(
                                    "width:{}px;min-width:{}px;max-width:{}px;",
                                    w, w, w
                                )
                            }
                        }}
                    >
                        <div class="messenger-sidebar-search">
                            <input
                                class="messenger-search-input"
                                type="text"
                                placeholder="Search people..."
                                prop:value={move || search.get()}
                                on:input=move |ev| set_search.set(event_target_value(&ev))
                            />
                        </div>
                    <div class="messenger-sidebar-list">
                        <For
                            each=move || match left_tab.get() {
                                "recent" => recent_items.get(),
                                "friends" => friend_items.get(),
                                "groups" => group_items.get(),
                                _ => request_items.get(),
                            }
                            key=|item| match item.clone() {
                                SidebarListItem::Contact(c) => format!("c-{}", c.id),
                                SidebarListItem::Request(r) => format!("r-{}", r.id),
                            }
                            children=move |item: SidebarListItem| {
                                match item {
                                    SidebarListItem::Contact(c) => {
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
                                                ContactSource::Marketplace => "Marketplace listing".to_string(),
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
                                        }.into_any()
                                    }
                                    SidebarListItem::Request(req) => render_request(req),
                                }
                            }
                        />
                    </div>
                </div>
                <div
                    class="messenger-resizer"
                    class:msb-hidden={move || left_collapsed.get() || right_collapsed.get()}
                    on:mousedown=move |ev| {
                        on_resizer_down(ev);
                    }
                    title="Drag to resize — push to an edge to hide"
                ></div>
                <div
                    class="messenger-chat"
                    class:msb-hidden={move || right_collapsed.get()}
                >
                    {move || {
                        if let Some(contact) = active_contact.get() {
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
                                        each=move || thread_messages.get()
                                        key=|m| m.id
                                        children=move |m| {
                                            let text = m.content.clone();
                                            let embeds = m.embeds.clone();
                                            let ts = format!("{:?}", m.timestamp.format("%H:%M"));
                                            let is_me = m.sender_id == current_user_id;
                                            let sender = if contact_is_group {
                                                if is_me { "You".to_string() } else {
                                                    contacts.get().iter().find(|c| c.id == m.sender_id).map(|c| c.name.clone()).unwrap_or_else(|| "Unknown".to_string())
                                                }
                                            } else { String::new() };
                                            let cls = if is_me { "messenger-message messenger-message-me" } else { "messenger-message" };
                                            let (status_icon, status_text) = match m.status {
                                                MessageStatus::Pending => ("⏳", "Pending — PQC seal in progress"),
                                                MessageStatus::Sent => ("✓", "Sent — PQC sealed & stored"),
                                                MessageStatus::Failed => ("⚠", "Failed — secure store rejected"),
                                            };
                                            view! {
                                                <div class={cls}>
                                                    {if contact_is_group {
                                                        view! { <div class="messenger-message-sender">{sender.clone()}</div> }.into_any()
                                                    } else { ().into_any() }}
                                                    {if !text.is_empty() {
                                                        view! { <div class="messenger-message-text">{text}</div> }.into_any()
                                                    } else { ().into_any() }}
                                                    {if !embeds.is_empty() {
                                                        view! {
                                                            <div class="messenger-embeds">
                                                                <For
                                                                    each=move || embeds.clone()
                                                                    key=|e| embed_key(e)
                                                                    children=move |e| render_embed_card(e)
                                                                />
                                                            </div>
                                                        }.into_any()
                                                    } else { ().into_any() }}
                                                    <div class="messenger-message-meta">
                                                        {ts}
                                                        <span class="messenger-msg-status" title={status_text}>{status_icon}</span>
                                                    </div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                                {move || {
                                    let pending = pending_embeds.get();
                                    if pending.is_empty() {
                                        ().into_any()
                                    } else {
                                        view! {
                                            <div class="messenger-pending-embeds">
                                                <For
                                                    each=move || pending.clone()
                                                    key=|e| embed_key(e)
                                                    children=move |e| {
                                                        let app = app_store.get();
                                                        let (icon, title) = resolve_embed(&app, &e)
                                                            .map(|i| (i.icon.to_string(), i.title))
                                                            .unwrap_or_else(|| ("❓".to_string(), "Unavailable".to_string()));
                                                        let rm = e.clone();
                                                        view! {
                                                            <span class="messenger-embed-chip">
                                                                <span class="messenger-embed-chip-icon">{icon}</span>
                                                                <span class="messenger-embed-chip-label">{title}</span>
                                                                <button
                                                                    class="messenger-embed-chip-x"
                                                                    title="Remove"
                                                                    on:click=move |_| {
                                                                        set_pending_embeds.update(|v| {
                                                                            v.retain(|x| x != &rm);
                                                                        });
                                                                    }
                                                                >
                                                                    "×"
                                                                </button>
                                                            </span>
                                                        }
                                                    }
                                                />
                                            </div>
                                        }
                                        .into_any()
                                    }
                                }}
                                <div class="messenger-composer">
                                    <div
                                        class="messenger-attach-wrap"
                                        attr:data-pf-edit-scope="messenger-attach"
                                    >
                                        <button
                                            class="messenger-attach-btn"
                                            title="Attach portfolio, asset, or document"
                                            on:click=move |_| set_show_attach.update(|v| *v = !*v)
                                        >
                                            "📎"
                                        </button>
                                        {move || if show_attach.get() {
                                            view! {
                                                <div class="messenger-attach-menu">
                                                    <input
                                                        class="messenger-attach-search"
                                                        type="text"
                                                        placeholder="Search portfolios, assets, docs…"
                                                        prop:value={move || attach_search.get()}
                                                        on:input=move |ev| set_attach_search.set(event_target_value(&ev))
                                                    />
                                                    <div class="messenger-attach-list">
                                                        <For
                                                            each=move || attach_items.get()
                                                            key=|i| embed_key(&i.embed)
                                                            children=move |item| {
                                                                let e = item.embed.clone();
                                                                let e_sel = e.clone();
                                                                let e_click = e.clone();
                                                                let e_mark = e.clone();
                                                                view! {
                                                                    <button
                                                                        class="messenger-attach-item"
                                                                        class:selected={move || pending_embeds.get().contains(&e_sel)}
                                                                        on:click=move |_| toggle_embed(e_click.clone())
                                                                    >
                                                                        <span class="messenger-attach-icon">{item.icon}</span>
                                                                        <span class="messenger-attach-text">
                                                                            <span class="messenger-attach-label">{item.label}</span>
                                                                            <span class="messenger-attach-sub">{item.sub}</span>
                                                                        </span>
                                                                        {move || if pending_embeds.get().contains(&e_mark) { "✓".into_any() } else { ().into_any() }}
                                                                    </button>
                                                                }
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            }
                                            .into_any()
                                        } else {
                                            ().into_any()
                                        }}
                                    </div>
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
                {move || {
                    if left_collapsed.get() {
                        view! {
                            <button
                                class="messenger-restore-arrow messenger-restore-left"
                                on:click=restore_panels
                                title="Show conversations"
                            >
                                "▶"
                            </button>
                        }
                        .into_any()
                    } else if right_collapsed.get() {
                        view! {
                            <button
                                class="messenger-restore-arrow messenger-restore-right"
                                on:click=restore_panels
                                title="Show chat"
                            >
                                "◀"
                            </button>
                        }
                        .into_any()
                    } else {
                        ().into_any()
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
            {move || doc_view.get().map(|(entity_id, doc)| {
                let title = doc.name.clone();
                view! {
                    <DocModal
                        entity_id={entity_id}
                        title={title}
                        on_close=move || set_doc_view.set(None)
                        can_edit={false}
                        on_add={None::<leptos::prelude::Callback<String>>}
                        initial_doc={Some(doc)}
                    />
                }
                .into_any()
            })}
            </div>
        </div>
    }
}
