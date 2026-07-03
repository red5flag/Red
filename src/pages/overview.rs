use crate::components::editable_text::EditableText;
use crate::stores::use_app_store;
use leptos::prelude::*;

fn fmt_time(ts: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(ts);
    if diff.num_minutes() < 1 { "now".to_string() }
    else if diff.num_hours() < 1 { format!("{}m", diff.num_minutes()) }
    else if diff.num_days() < 1 { format!("{}h", diff.num_hours()) }
    else if diff.num_days() < 30 { format!("{}d", diff.num_days()) }
    else { ts.format("%d %b").to_string() }
}

#[component]
pub fn OverviewPage() -> impl IntoView {
    let app_store = use_app_store();

    let user_name = move || app_store.get().current_user.name.clone();
    let on_name_commit = move |name: String| {
        app_store.update(|s| s.set_user_name(name));
    };

    let unread_message_count = move || app_store.get().unread_message_count();

    let recent_messages = move || {
        let mut messages = app_store.get().messages.clone();
        messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        messages.into_iter().take(4).collect::<Vec<_>>()
    };

    let sender_name_for_message = move |msg: &crate::models::Message| {
        let store = app_store.get();
        store.messenger_contacts.iter().find(|c| c.id == msg.sender_id).map(|c| c.name.clone())
            .or_else(|| store.organization_users.iter().find(|u| u.id == msg.sender_id).map(|u| u.name.clone()))
            .unwrap_or_else(|| "Unknown".to_string())
    };

    let on_open_messages = move |_| {
        app_store.update(|s| s.set_message_drawer(true));
    };

    let on_open_bookings = move |_| {
        app_store.update(|s| s.expand_tab(crate::types::TabType::Calendar));
    };

    let on_open_transactions = move |_| {
        app_store.update(|s| s.expand_tab(crate::types::TabType::Transactions));
    };

    let on_open_portfolios = move |_| {
        app_store.update(|s| s.expand_tab(crate::types::TabType::Portfolios));
    };

    // Recent bookings = upcoming/recent calendar events
    let recent_bookings = move || {
        let mut events: Vec<_> = app_store.get().calendar_events.clone();
        events.sort_by(|a, b| b.start.cmp(&a.start));
        events.into_iter().take(4).collect::<Vec<_>>()
    };

    // Updated investments = recently updated portfolios
    let recent_investments = move || {
        let mut portfolios: Vec<_> = app_store.get().portfolios.clone();
        portfolios.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        portfolios.into_iter().take(4).collect::<Vec<_>>()
    };

    // Recent contacts = most recently updated users
    let recent_contacts = move || {
        let mut users: Vec<_> = app_store.get().organization_users.clone();
        users.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        users.into_iter().take(4).collect::<Vec<_>>()
    };

    // Recent transactions
    let recent_transactions = move || {
        let mut txns: Vec<_> = app_store.get().transactions.clone();
        txns.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        txns.into_iter().take(4).collect::<Vec<_>>()
    };

    // Recent notifications
    let recent_notifications = move || {
        let mut notifs: Vec<_> = app_store.get().notifications.clone();
        notifs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        notifs.into_iter().take(4).collect::<Vec<_>>()
    };

    let notif_icon = |t: &crate::stores::app_store::NotificationType| {
        match t {
            crate::stores::app_store::NotificationType::Success => "✅",
            crate::stores::app_store::NotificationType::Error => "❌",
            crate::stores::app_store::NotificationType::Warning => "⚠",
            crate::stores::app_store::NotificationType::Info => "ℹ",
        }
    };

    let txn_type_label = |t: &crate::types::TransactionType| {
        match t {
            crate::types::TransactionType::Purchase => "Buy",
            crate::types::TransactionType::Sale => "Sell",
            crate::types::TransactionType::Rent => "Rent",
            crate::types::TransactionType::Lease => "Lease",
            crate::types::TransactionType::Payout => "Payout",
            crate::types::TransactionType::Dividend => "Div",
            crate::types::TransactionType::Fee => "Fee",
            crate::types::TransactionType::Tax => "Tax",
            crate::types::TransactionType::Transfer => "Xfer",
            crate::types::TransactionType::Adjustment => "Adj",
        }
    };

    view! {
        <div class="overview-content">
            <div class="overview-greeting">
                "Welcome, "
                <EditableText value=Signal::derive(user_name) on_commit=on_name_commit />
            </div>

            <div class="overview-square-grid">
                // Recent Messages
                <div class="overview-square overview-square-messages" on:click=on_open_messages>
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"💬"</span>
                        <span class="overview-square-label">"Recent Messages"</span>
                        <span class="overview-square-count">{move || format!("{}", unread_message_count())}</span>
                    </div>
                    <div class="overview-square-messages">
                        {move || {
                            let messages = recent_messages();
                            if messages.is_empty() {
                                view! { <div class="overview-square-empty">"No messages yet"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {messages.into_iter().map(|m| {
                                            let sender = sender_name_for_message(&m);
                                            let preview = if m.content.len() > 30 { format!("{}…", &m.content[..30]) } else { m.content.clone() };
                                            let time = fmt_time(m.timestamp);
                                            let unread = !m.read;
                                            view! {
                                                <div class="overview-message-row-compact" class:unread=unread>
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{sender}</span>
                                                        <span class="overview-message-time">{time}</span>
                                                    </div>
                                                    <div class="overview-message-preview">{preview}</div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // Recent Bookings
                <div class="overview-square overview-square-booking clickable" on:click=on_open_bookings>
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"📅"</span>
                        <span class="overview-square-label">"Recent Bookings"</span>
                        <span class="overview-square-count">{move || app_store.get().calendar_events.len()}</span>
                    </div>
                    <div class="overview-square-messages">
                        {move || {
                            let events = recent_bookings();
                            if events.is_empty() {
                                view! { <div class="overview-square-empty">"No bookings yet"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {events.into_iter().map(|e| {
                                            let title = if e.title.len() > 28 { format!("{}…", &e.title[..28]) } else { e.title.clone() };
                                            let time = fmt_time(e.start);
                                            view! {
                                                <div class="overview-message-row-compact">
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{title}</span>
                                                        <span class="overview-message-time">{time}</span>
                                                    </div>
                                                    <div class="overview-message-preview">{e.start.format("%d %b %H:%M").to_string()}</div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // Updated Investments
                <div class="overview-square overview-square-change">
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"📈"</span>
                        <span class="overview-square-label">"Updated Investments"</span>
                        <span class="overview-square-count">{move || app_store.get().portfolios.len()}</span>
                    </div>
                    <div class="overview-square-messages">
                        {move || {
                            let portfolios = recent_investments();
                            if portfolios.is_empty() {
                                view! { <div class="overview-square-empty">"No portfolios yet"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {portfolios.into_iter().map(|p| {
                                            let name = if p.name.len() > 24 { format!("{}…", &p.name[..24]) } else { p.name.clone() };
                                            let time = fmt_time(p.updated_at);
                                            let assets = p.get_all_assets().len();
                                            view! {
                                                <div class="overview-message-row-compact">
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{name}</span>
                                                        <span class="overview-message-time">{time}</span>
                                                    </div>
                                                    <div class="overview-message-preview">{format!("{} assets · ${:.0}K", assets, p.total_value / 1_000.0)}</div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // Recent Transactions
                <div class="overview-square overview-square-report clickable" on:click=on_open_transactions>
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"💰"</span>
                        <span class="overview-square-label">"Recent Transactions"</span>
                        <span class="overview-square-count">{move || app_store.get().transactions.len()}</span>
                    </div>
                    <div class="overview-square-messages">
                        {move || {
                            let txns = recent_transactions();
                            if txns.is_empty() {
                                view! { <div class="overview-square-empty">"No transactions yet"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {txns.into_iter().map(|t| {
                                            let label = txn_type_label(&t.transaction_type).to_string();
                                            let desc = t.description.clone().unwrap_or_else(|| t.to_entity.name.clone());
                                            let desc_short = if desc.len() > 24 { format!("{}…", &desc[..24]) } else { desc };
                                            let time = fmt_time(t.created_at);
                                            view! {
                                                <div class="overview-message-row-compact">
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{format!("{} · ${:.0}", label, t.amount)}</span>
                                                        <span class="overview-message-time">{time}</span>
                                                    </div>
                                                    <div class="overview-message-preview">{desc_short}</div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // Recent Contacts
                <div class="overview-square overview-square-contact">
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"�"</span>
                        <span class="overview-square-label">"Recent Contacts"</span>
                        <span class="overview-square-count">{move || app_store.get().organization_users.len()}</span>
                    </div>
                    <div class="overview-square-messages">
                        {move || {
                            let users = recent_contacts();
                            if users.is_empty() {
                                view! { <div class="overview-square-empty">"No contacts yet"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {users.into_iter().map(|u| {
                                            let name = u.name.clone();
                                            let role = format!("{:?}", u.role);
                                            let time = fmt_time(u.updated_at);
                                            view! {
                                                <div class="overview-message-row-compact">
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{name}</span>
                                                        <span class="overview-message-time">{time}</span>
                                                    </div>
                                                    <div class="overview-message-preview">{role}</div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // Notifications
                <div class="overview-square overview-square-notifications">
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"🔔"</span>
                        <span class="overview-square-label">"Notifications"</span>
                        <span class="overview-square-count">{move || app_store.get().notifications.len()}</span>
                    </div>
                    <div class="overview-square-messages">
                        {move || {
                            let notifs = recent_notifications();
                            if notifs.is_empty() {
                                view! { <div class="overview-square-empty">"No notifications"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {notifs.into_iter().map(|n| {
                                            let icon = notif_icon(&n.notification_type).to_string();
                                            let msg = if n.message.len() > 32 { format!("{}…", &n.message[..32]) } else { n.message.clone() };
                                            let time = fmt_time(n.timestamp);
                                            view! {
                                                <div class="overview-message-row-compact">
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{icon} " " {msg}</span>
                                                        <span class="overview-message-time">{time}</span>
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // Property Overview (full width)
                <div class="overview-square overview-square-wide clickable" on:click=on_open_portfolios>
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"🏠"</span>
                        <span class="overview-square-label">"Property Overview"</span>
                    </div>
                    {move || {
                        let store = app_store.get();
                        let portfolio = store.portfolios.iter().max_by_key(|p| p.updated_at);
                        if let Some(p) = portfolio {
                            let img = p.get_all_assets().into_iter().find(|a| !a.images.is_empty()).and_then(|a| a.images.first().cloned());
                            view! {
                                <div class="overview-property-overview">
                                    <div class="overview-property-text">
                                        <div class="overview-property-title">{p.name.clone()}</div>
                                        <div class="overview-property-address">{p.description.clone().unwrap_or_default()}</div>
                                        <div class="overview-property-summary">{format!("{} assets · ${:.2}M", p.get_all_assets().len(), p.total_value / 1_000_000.0)}</div>
                                    </div>
                                    {match img {
                                        Some(url) => view! { <img class="overview-property-img" src={url} alt="Property" /> }.into_any(),
                                        None => view! { <div class="overview-property-img-placeholder">"🏞"</div> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            view! { <div class="overview-square-empty">"No property overview available"</div> }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
