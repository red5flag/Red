use crate::components::editable_text::EditableText;
use crate::models::ConnectionStatus;
use crate::stores::{
    use_app_store, use_calendar_store, use_messenger_store, use_notification_store,
    use_organization_store, use_transaction_store,
};
use leptos::prelude::*;

fn fmt_time(ts: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(ts);
    if diff.num_minutes() < 1 {
        "now".to_string()
    } else if diff.num_hours() < 1 {
        format!("{}m", diff.num_minutes())
    } else if diff.num_days() < 1 {
        format!("{}h", diff.num_hours())
    } else if diff.num_days() < 30 {
        format!("{}d", diff.num_days())
    } else {
        ts.format("%d %b").to_string()
    }
}

#[component]
pub fn OverviewPage() -> impl IntoView {
    let app_store = use_app_store();
    let organization_store = use_organization_store();
    let calendar_store = use_calendar_store();
    let messenger_store = use_messenger_store();
    let notification_store = use_notification_store();
    let transaction_store = use_transaction_store();

    let user_name = move || app_store.get().current_user.name.clone();
    let on_name_commit = move |name: String| {
        app_store.update(|s| s.set_user_name(name));
    };

    let unread_message_count = move || {
        let current_user_id = app_store.get().current_user.id;
        messenger_store.get().unread_message_count(current_user_id)
    };

    let recent_messages = move || {
        let mut messages = messenger_store.get().messages.clone();
        messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        messages.into_iter().take(4).collect::<Vec<_>>()
    };

    let sender_name_for_message = move |msg: &crate::models::Message| {
        let _app = app_store.get();
        let org = organization_store.get();
        let messenger = messenger_store.get();
        messenger
            .messenger_contacts
            .iter()
            .find(|c| c.id == msg.sender_id)
            .map(|c| c.name.clone())
            .or_else(|| {
                org.organization_users
                    .iter()
                    .find(|u| u.id == msg.sender_id)
                    .map(|u| u.name.clone())
            })
            .unwrap_or_else(|| "Unknown".to_string())
    };

    let on_open_messages = move |_| {
        messenger_store.update(|s| s.set_message_drawer(true));
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
        let mut events: Vec<_> = calendar_store.get().calendar_events.clone();
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
        let mut users: Vec<_> = organization_store.get().organization_users.clone();
        users.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        users.into_iter().take(4).collect::<Vec<_>>()
    };

    // Recent transactions
    let recent_transactions = move || {
        let mut txns: Vec<_> = transaction_store.get().transactions.clone();
        txns.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        txns.into_iter().take(4).collect::<Vec<_>>()
    };

    // Recent notifications
    let recent_notifications = move || {
        let mut notifs: Vec<_> = notification_store.get().notifications.clone();
        notifs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        notifs.into_iter().take(4).collect::<Vec<_>>()
    };

    let notif_icon = |t: &crate::stores::NotificationType| match t {
        crate::stores::NotificationType::Success => "✅",
        crate::stores::NotificationType::Error => "❌",
        crate::stores::NotificationType::Warning => "⚠",
        crate::stores::NotificationType::Info => "ℹ",
    };

    let txn_type_label = |t: &crate::types::TransactionType| match t {
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
    };

    #[derive(Clone, PartialEq)]
    struct ActivityItem {
        icon: &'static str,
        message: String,
        time: chrono::DateTime<chrono::Utc>,
    }

    let recent_activity = Memo::new(move |_| {
        let mut items: Vec<ActivityItem> = Vec::new();
        let app = app_store.get();
        let notifications = notification_store.get();
        let txns = transaction_store.get();
        let msgs = messenger_store.get();
        let cal = calendar_store.get();

        for n in &notifications.notifications {
            let message = if n.message.len() > 40 {
                format!("{}…", &n.message[..40])
            } else {
                n.message.clone()
            };
            items.push(ActivityItem {
                icon: notif_icon(&n.notification_type),
                message,
                time: n.timestamp,
            });
        }

        for t in &txns.transactions {
            let label = txn_type_label(&t.transaction_type);
            let desc = t.description.clone().unwrap_or_else(|| t.to_entity.name.clone());
            let desc_short = if desc.len() > 24 {
                format!("{}…", &desc[..24])
            } else {
                desc
            };
            let message = format!("{} ${:.0} · {}", label, t.amount, desc_short);
            items.push(ActivityItem {
                icon: "💰",
                message,
                time: t.created_at,
            });
        }

        for m in &msgs.messages {
            let sender = sender_name_for_message(&m);
            let preview = if m.content.len() > 30 {
                format!("{}…", &m.content[..30])
            } else {
                m.content.clone()
            };
            let message = format!("{}: {}", sender, preview);
            items.push(ActivityItem {
                icon: "💬",
                message,
                time: m.timestamp,
            });
        }

        for e in &cal.calendar_events {
            items.push(ActivityItem {
                icon: "📅",
                message: e.title.clone(),
                time: e.start,
            });
        }

        for st in &app.service_tasks {
            let message = format!("{} task scheduled", st.task_type.display());
            items.push(ActivityItem {
                icon: "🛠",
                message,
                time: st.start_datetime,
            });
        }

        items.sort_by(|a, b| b.time.cmp(&a.time));
        items.into_iter().take(6).collect::<Vec<_>>()
    });

    // Organization summary
    let org_summary = move || {
        let orgs = organization_store.get();
        let total_orgs = orgs.organizations.len();
        let total_members = orgs.organization_users.len();
        let current_org = orgs
            .current_organization_id
            .and_then(|id| orgs.organizations.iter().find(|o| o.id == id));
        let current_org_name = current_org.map(|o| o.name.clone()).unwrap_or_else(|| "None".to_string());
        let current_org_type = current_org
            .and_then(|o| o.business_type.clone())
            .unwrap_or_else(|| "—".to_string());
        let total_roles: usize = current_org.map(|o| o.roles.len()).unwrap_or(0);
        (total_orgs, total_members, current_org_name, current_org_type, total_roles)
    };

    // Portfolio stats
    let portfolio_stats = move || {
        let store = app_store.get();
        let portfolios = &store.portfolios;
        let total_value: f64 = portfolios.iter().map(|p| p.total_value).sum();
        let total_assets: usize = portfolios.iter().map(|p| p.get_all_assets().len()).sum();
        let total_groups: usize = portfolios.iter().map(|p| p.asset_groups.len()).sum();
        let total_docs: usize = portfolios.iter().map(|p| p.documents.len()).sum();
        (portfolios.len(), total_value, total_assets, total_groups, total_docs)
    };

    // Channel stats
    let channel_stats = move || {
        let store = app_store.get();
        let channels = &store.channels;
        let connected = channels.iter().filter(|c| c.connection_status == ConnectionStatus::Connected).count();
        let disconnected = channels.iter().filter(|c| c.connection_status == ConnectionStatus::Disconnected).count();
        let errors = channels.iter().filter(|c| c.connection_status == ConnectionStatus::Error).count();
        let enabled = channels.iter().filter(|c| c.enabled).count();
        (channels.len(), connected, disconnected, errors, enabled)
    };

    // Service task stats
    let service_task_stats = move || {
        let store = app_store.get();
        let total = store.service_tasks.len();
        let pending = store.service_tasks.iter().filter(|t| t.status != crate::models::ServiceTaskStatus::Done && t.status != crate::models::ServiceTaskStatus::Cancelled).count();
        let completed = store.service_tasks.iter().filter(|t| t.status == crate::models::ServiceTaskStatus::Done).count();
        (total, pending, completed)
    };

    // Bottom section data lists
    let organizations_list = move || {
        let orgs = organization_store.get();
        orgs.organizations
            .iter()
            .map(|o| {
                let member_count = orgs
                    .organization_users
                    .iter()
                    .filter(|u| u.organization_id == Some(o.id))
                    .count();
                (
                    o.name.clone(),
                    o.business_type.clone().unwrap_or_else(|| "—".to_string()),
                    member_count,
                    o.roles.len(),
                )
            })
            .collect::<Vec<_>>()
    };

    let top_portfolios_by_value = move || {
        let mut portfolios: Vec<_> = app_store
            .get()
            .portfolios
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    p.total_value,
                    p.get_all_assets().len(),
                    format!("{:?}", p.status),
                )
            })
            .collect();
        portfolios.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        portfolios.into_iter().take(4).collect::<Vec<_>>()
    };

    let top_assets_by_value = move || {
        let app = app_store.get();
        let mut assets: Vec<_> = Vec::new();
        for p in &app.portfolios {
            for a in p.get_all_assets() {
                assets.push((
                    a.name.clone(),
                    a.current_value,
                    a.asset_type.to_input_string(),
                    p.name.clone(),
                ));
            }
        }
        assets.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        assets.into_iter().take(4).collect::<Vec<_>>()
    };

    let channels_list = move || {
        app_store
            .get()
            .channels
            .iter()
            .map(|c| {
                let status_icon = match c.connection_status {
                    ConnectionStatus::Connected => "🟢",
                    ConnectionStatus::Disconnected => "⚪",
                    ConnectionStatus::Error => "🔴",
                };
                let type_label = match &c.channel_type {
                    crate::models::ChannelType::Airbnb => "Airbnb",
                    crate::models::ChannelType::BookingCom => "Booking.com",
                    crate::models::ChannelType::Expedia => "Expedia",
                    crate::models::ChannelType::Vrbo => "Vrbo",
                    crate::models::ChannelType::LinkedIn => "LinkedIn",
                    crate::models::ChannelType::Test => "Test",
                    crate::models::ChannelType::Other(s) => s.as_str(),
                };
                let sync_time = c
                    .last_sync_at
                    .map(|t| fmt_time(t))
                    .unwrap_or_else(|| "never".to_string());
                (
                    c.name.clone(),
                    status_icon.to_string(),
                    type_label.to_string(),
                    c.enabled,
                    sync_time,
                )
            })
            .collect::<Vec<_>>()
    };

    let upcoming_bookings = move || {
        let now = chrono::Utc::now();
        let mut events: Vec<_> = calendar_store
            .get()
            .calendar_events
            .iter()
            .filter(|e| e.start >= now)
            .cloned()
            .collect();
        events.sort_by(|a, b| a.start.cmp(&b.start));
        events
            .into_iter()
            .take(4)
            .map(|e| (e.title.clone(), e.start, e.all_day))
            .collect::<Vec<_>>()
    };

    let top_transactions_by_amount = move || {
        let mut txns: Vec<_> = transaction_store
            .get()
            .transactions
            .iter()
            .map(|t| (t.transaction_type.clone(), t.amount, t.to_entity.name.clone(), t.created_at))
            .collect();
        txns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        txns.into_iter().take(4).collect::<Vec<_>>()
    };

    let on_open_organization = move |_| {
        app_store.update(|s| s.expand_tab(crate::types::TabType::Organization));
    };

    let on_open_networking = move |_| {
        app_store.update(|s| s.expand_tab(crate::types::TabType::Networking));
    };

    view! {
        <div class="overview-content">
            <div class="overview-greeting">
                "Welcome, "
                <EditableText value=Signal::derive(user_name) on_commit=on_name_commit />
            </div>

            // Organization + Portfolio detail row
            <div class="overview-detail-row">
                <div class="overview-detail-card clickable" on:click=on_open_organization>
                    <div class="overview-detail-header">
                        <span class="overview-detail-icon">"🏢"</span>
                        <span class="overview-detail-title">"Current Organization"</span>
                    </div>
                    <div class="overview-detail-body">
                        <div class="overview-detail-row-item">
                            <span class="overview-detail-key">"Name"</span>
                            <span class="overview-detail-val">{move || org_summary().2}</span>
                        </div>
                        <div class="overview-detail-row-item">
                            <span class="overview-detail-key">"Type"</span>
                            <span class="overview-detail-val">{move || org_summary().3}</span>
                        </div>
                        <div class="overview-detail-row-item">
                            <span class="overview-detail-key">"Members"</span>
                            <span class="overview-detail-val">{move || org_summary().1}</span>
                        </div>
                        <div class="overview-detail-row-item">
                            <span class="overview-detail-key">"Roles"</span>
                            <span class="overview-detail-val">{move || org_summary().4}</span>
                        </div>
                    </div>
                </div>

                <div class="overview-detail-card clickable" on:click=on_open_portfolios>
                    <div class="overview-detail-header">
                        <span class="overview-detail-icon">"📊"</span>
                        <span class="overview-detail-title">"Portfolio Summary"</span>
                    </div>
                    <div class="overview-detail-body">
                        <div class="overview-detail-row-item">
                            <span class="overview-detail-key">"Total Value"</span>
                            <span class="overview-detail-val">{move || format!("${:.2}M", portfolio_stats().1 / 1_000_000.0)}</span>
                        </div>
                        <div class="overview-detail-row-item">
                            <span class="overview-detail-key">"Asset Groups"</span>
                            <span class="overview-detail-val">{move || portfolio_stats().3}</span>
                        </div>
                        <div class="overview-detail-row-item">
                            <span class="overview-detail-key">"Documents"</span>
                            <span class="overview-detail-val">{move || portfolio_stats().4}</span>
                        </div>
                        <div class="overview-detail-row-item">
                            <span class="overview-detail-key">"Service Tasks"</span>
                            <span class="overview-detail-val">{move || format!("{} pending", service_task_stats().1)}</span>
                        </div>
                    </div>
                </div>

                <div class="overview-detail-card clickable" on:click=on_open_networking>
                    <div class="overview-detail-header">
                        <span class="overview-detail-icon">"📡"</span>
                        <span class="overview-detail-title">"Channel Status"</span>
                    </div>
                    <div class="overview-detail-body">
                        <div class="overview-detail-row-item">
                            <span class="overview-detail-key">"Connected"</span>
                            <span class="overview-detail-val overview-detail-val-good">{move || channel_stats().1}</span>
                        </div>
                        <div class="overview-detail-row-item">
                            <span class="overview-detail-key">"Disconnected"</span>
                            <span class="overview-detail-val">{move || channel_stats().2}</span>
                        </div>
                        <div class="overview-detail-row-item">
                            <span class="overview-detail-key">"Errors"</span>
                            <span class="overview-detail-val" class:overview-detail-val-bad={move || channel_stats().3 > 0}>{move || channel_stats().3}</span>
                        </div>
                        <div class="overview-detail-row-item">
                            <span class="overview-detail-key">"Enabled"</span>
                            <span class="overview-detail-val">{move || channel_stats().4}</span>
                        </div>
                    </div>
                </div>
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
                        <span class="overview-square-count">{move || calendar_store.get().calendar_events.len()}</span>
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
                        <span class="overview-square-count">{move || transaction_store.get().transactions.len()}</span>
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
                        <span class="overview-square-count">{move || organization_store.get().organization_users.len()}</span>
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
                        <span class="overview-square-count">{move || notification_store.get().notifications.len()}</span>
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

                // Recent Activity (full width)
                <div class="overview-square overview-square-wide">
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"🔔"</span>
                        <span class="overview-square-label">"Recent Activity"</span>
                        <span class="overview-square-count">{move || recent_activity.get().len()}</span>
                    </div>
                    <div class="overview-square-messages">
                        {move || {
                            let activities = recent_activity.get();
                            if activities.is_empty() {
                                view! { <div class="overview-square-empty">"No recent activity"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {activities.into_iter().map(|a| {
                                            let time = fmt_time(a.time);
                                            view! {
                                                <div class="overview-message-row-compact">
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{format!("{} {}", a.icon, a.message)}</span>
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
            </div>

            // Linked-content continuation
            <div class="overview-square-grid">
                // Organizations (wide rectangle)
                <div class="overview-square overview-square-wide clickable" on:click=on_open_organization>
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"🏢"</span>
                        <span class="overview-square-label">"Organizations"</span>
                        <span class="overview-square-count">{move || organization_store.get().organizations.len()}</span>
                    </div>
                    <div class="overview-square-messages">
                        {move || {
                            let orgs = organizations_list();
                            if orgs.is_empty() {
                                view! { <div class="overview-square-empty">"No organizations"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {orgs.into_iter().map(|(name, biz_type, members, roles)| {
                                            view! {
                                                <div class="overview-message-row-compact">
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{name}</span>
                                                        <span class="overview-message-time">{format!("{} members", members)}</span>
                                                    </div>
                                                    <div class="overview-message-preview">{format!("{} · {} roles", biz_type, roles)}</div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // Top Portfolios
                <div class="overview-square clickable" on:click=on_open_portfolios>
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"📊"</span>
                        <span class="overview-square-label">"Top Portfolios"</span>
                        <span class="overview-square-count">{move || app_store.get().portfolios.len()}</span>
                    </div>
                    <div class="overview-square-messages">
                        {move || {
                            let portfolios = top_portfolios_by_value();
                            if portfolios.is_empty() {
                                view! { <div class="overview-square-empty">"No portfolios"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {portfolios.into_iter().map(|(name, value, asset_count, status)| {
                                            view! {
                                                <div class="overview-message-row-compact">
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{name}</span>
                                                        <span class="overview-message-time">{status}</span>
                                                    </div>
                                                    <div class="overview-message-preview">{format!("${:.2}M · {} assets", value / 1_000_000.0, asset_count)}</div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // Top Assets
                <div class="overview-square clickable" on:click=on_open_portfolios>
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"🏠"</span>
                        <span class="overview-square-label">"Top Assets"</span>
                        <span class="overview-square-count">{move || portfolio_stats().2}</span>
                    </div>
                    <div class="overview-square-messages">
                        {move || {
                            let assets = top_assets_by_value();
                            if assets.is_empty() {
                                view! { <div class="overview-square-empty">"No assets"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {assets.into_iter().map(|(name, value, asset_type, portfolio_name)| {
                                            view! {
                                                <div class="overview-message-row-compact">
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{name}</span>
                                                        <span class="overview-message-time">{asset_type}</span>
                                                    </div>
                                                    <div class="overview-message-preview">{format!("${:.2}M · {}", value / 1_000_000.0, portfolio_name)}</div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // Channels (wide rectangle)
                <div class="overview-square overview-square-wide clickable" on:click=on_open_networking>
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"📡"</span>
                        <span class="overview-square-label">"Channels"</span>
                        <span class="overview-square-count">{move || channel_stats().0}</span>
                    </div>
                    <div class="overview-square-messages">
                        {move || {
                            let channels = channels_list();
                            if channels.is_empty() {
                                view! { <div class="overview-square-empty">"No channels"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {channels.into_iter().map(|(name, status_icon, type_label, enabled, sync_time)| {
                                            view! {
                                                <div class="overview-message-row-compact">
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{status_icon} " " {name}</span>
                                                        <span class="overview-message-time">{if enabled { "enabled" } else { "disabled" }}</span>
                                                    </div>
                                                    <div class="overview-message-preview">{format!("{} · sync {}", type_label, sync_time)}</div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // Upcoming Bookings
                <div class="overview-square clickable" on:click=on_open_bookings>
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"📅"</span>
                        <span class="overview-square-label">"Upcoming Bookings"</span>
                        <span class="overview-square-count">{move || calendar_store.get().calendar_events.len()}</span>
                    </div>
                    <div class="overview-square-messages">
                        {move || {
                            let events = upcoming_bookings();
                            if events.is_empty() {
                                view! { <div class="overview-square-empty">"No upcoming bookings"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {events.into_iter().map(|(title, start, all_day)| {
                                            let time = if all_day { start.format("%d %b").to_string() } else { start.format("%d %b %H:%M").to_string() };
                                            view! {
                                                <div class="overview-message-row-compact">
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{title}</span>
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

                // Top Transactions
                <div class="overview-square clickable" on:click=on_open_transactions>
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"💰"</span>
                        <span class="overview-square-label">"Top Transactions"</span>
                        <span class="overview-square-count">{move || transaction_store.get().transactions.len()}</span>
                    </div>
                    <div class="overview-square-messages">
                        {move || {
                            let txns = top_transactions_by_amount();
                            if txns.is_empty() {
                                view! { <div class="overview-square-empty">"No transactions"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {txns.into_iter().map(|(txn_type, amount, to_name, created_at)| {
                                            let label = txn_type_label(&txn_type);
                                            let time = fmt_time(created_at);
                                            view! {
                                                <div class="overview-message-row-compact">
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{format!("{} · ${:.0}", label, amount)}</span>
                                                        <span class="overview-message-time">{time}</span>
                                                    </div>
                                                    <div class="overview-message-preview">{to_name}</div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}
