use crate::models::{ConnectionStatus, Perm};
use crate::stores::{
    use_app_store, use_calendar_store, use_messenger_store, use_notification_store,
    use_organization_store, use_transaction_store, use_ui_store, AppStore, CalendarStore,
    MessengerStore, NotificationStore, OrganizationStore, TransactionStore,
};
use crate::types::OverviewSortMode;
use chrono::{DateTime, Duration, Utc};
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use web_sys::window;

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

fn portfolio_is_visible(
    p: &crate::models::Portfolio,
    user_id: Uuid,
    can_view_all: bool,
    orgs: &OrganizationStore,
) -> bool {
    p.is_visible_to(user_id, can_view_all)
        || p.organization_id.map_or(can_view_all, |oid| {
            orgs.user_has_perm_in_org(oid, user_id, &Perm::ViewPortfolios)
        })
}

fn asset_is_visible(
    a: &crate::models::Asset,
    user_id: Uuid,
    can_view_all: bool,
    orgs: &OrganizationStore,
) -> bool {
    a.is_visible_to(user_id, can_view_all)
        || a.organization_id.map_or(can_view_all, |oid| {
            orgs.user_has_perm_in_org(oid, user_id, &Perm::ViewAssets)
        })
}

fn org_is_visible(
    o: &crate::models::Organization,
    user_id: Uuid,
    can_view_all: bool,
    orgs: &OrganizationStore,
) -> bool {
    o.owner_id == user_id
        || o.members.contains(&user_id)
        || orgs.user_has_perm_in_org(o.id, user_id, &Perm::ViewOrganization)
        || can_view_all
}

fn section_last_changed(
    id: &str,
    app: &AppStore,
    org: &OrganizationStore,
    cal: &CalendarStore,
    messenger: &MessengerStore,
    notif: &NotificationStore,
    txn: &TransactionStore,
    now: DateTime<Utc>,
    user_id: Uuid,
    can_view_all: bool,
) -> Option<DateTime<Utc>> {
    match id {
        "recent-messages" => messenger.messages.iter().map(|m| m.timestamp).max(),
        "recent-bookings" => app.bookings.iter().map(|b| b.updated_at).max(),
        "updated-investments" => app
            .portfolios
            .iter()
            .filter(|p| portfolio_is_visible(*p, user_id, can_view_all, org))
            .map(|p| p.updated_at)
            .max(),
        "recent-transactions" => txn.transactions.iter().map(|t| t.created_at).max(),
        "recent-contacts" => org.organization_users.iter().map(|u| u.updated_at).max(),
        "notifications" => notif.notifications.iter().map(|n| n.timestamp).max(),
        "property-overview" => app
            .portfolios
            .iter()
            .filter(|p| portfolio_is_visible(*p, user_id, can_view_all, org))
            .flat_map(|p| {
                std::iter::once(p.updated_at).chain(p.asset_groups.iter().flat_map(|g| {
                    g.assets
                        .iter()
                        .filter(|a| asset_is_visible(*a, user_id, can_view_all, org))
                        .map(|a| a.updated_at)
                }))
            })
            .max(),
        "recent-activity" => {
            let mut times: Vec<DateTime<Utc>> = Vec::new();
            times.extend(notif.notifications.iter().map(|n| n.timestamp));
            times.extend(txn.transactions.iter().map(|t| t.created_at));
            times.extend(messenger.messages.iter().map(|m| m.timestamp));
            times.extend(cal.calendar_events.iter().map(|e| e.start));
            times.extend(app.service_tasks.iter().map(|st| st.updated_at));
            times.into_iter().max()
        }
        "organizations" => org
            .organizations
            .iter()
            .filter(|o| org_is_visible(*o, user_id, can_view_all, org))
            .map(|o| o.updated_at)
            .max(),
        "top-portfolios" => app
            .portfolios
            .iter()
            .filter(|p| portfolio_is_visible(*p, user_id, can_view_all, org))
            .map(|p| p.updated_at)
            .max(),
        "top-assets" => app
            .portfolios
            .iter()
            .filter(|p| portfolio_is_visible(*p, user_id, can_view_all, org))
            .flat_map(|p| {
                p.asset_groups.iter().flat_map(|g| {
                    g.assets
                        .iter()
                        .filter(|a| asset_is_visible(*a, user_id, can_view_all, org))
                        .map(|a| a.updated_at)
                })
            })
            .max(),
        "channels" => app
            .channels
            .iter()
            .map(|c| c.last_sync_at.unwrap_or(c.updated_at))
            .max(),
        "upcoming-bookings" => cal
            .calendar_events
            .iter()
            .filter(|e| e.start >= now && e.start <= now + Duration::hours(24))
            .map(|e| e.start)
            .max(),
        "top-transactions" => txn.transactions.iter().map(|t| t.created_at).max(),
        "current-organization" => org
            .organizations
            .iter()
            .filter(|o| org_is_visible(*o, user_id, can_view_all, org))
            .map(|o| o.updated_at)
            .max(),
        "portfolio-summary" => app
            .portfolios
            .iter()
            .filter(|p| portfolio_is_visible(*p, user_id, can_view_all, org))
            .map(|p| p.updated_at)
            .max(),
        "channel-status" => app
            .channels
            .iter()
            .map(|c| c.last_sync_at.unwrap_or(c.updated_at))
            .max(),
        _ => None,
    }
}

fn section_change_count(
    id: &str,
    app: &AppStore,
    org: &OrganizationStore,
    cal: &CalendarStore,
    messenger: &MessengerStore,
    notif: &NotificationStore,
    txn: &TransactionStore,
    now: DateTime<Utc>,
    window: Duration,
    user_id: Uuid,
    can_view_all: bool,
) -> usize {
    let cutoff = now - window;
    match id {
        "recent-messages" => messenger
            .messages
            .iter()
            .filter(|m| m.timestamp >= cutoff)
            .count(),
        "recent-bookings" => app
            .bookings
            .iter()
            .filter(|b| b.updated_at >= cutoff)
            .count(),
        "updated-investments" => app
            .portfolios
            .iter()
            .filter(|p| {
                p.updated_at >= cutoff && portfolio_is_visible(*p, user_id, can_view_all, org)
            })
            .count(),
        "recent-transactions" => txn
            .transactions
            .iter()
            .filter(|t| t.created_at >= cutoff)
            .count(),
        "recent-contacts" => org
            .organization_users
            .iter()
            .filter(|u| u.updated_at >= cutoff)
            .count(),
        "notifications" => notif
            .notifications
            .iter()
            .filter(|n| n.timestamp >= cutoff)
            .count(),
        "property-overview" => {
            let portfolio_count = app
                .portfolios
                .iter()
                .filter(|p| {
                    p.updated_at >= cutoff && portfolio_is_visible(*p, user_id, can_view_all, org)
                })
                .count();
            let asset_count = app
                .portfolios
                .iter()
                .filter(|p| portfolio_is_visible(*p, user_id, can_view_all, org))
                .flat_map(|p| {
                    p.asset_groups.iter().flat_map(|g| {
                        g.assets.iter().filter(|a| {
                            a.updated_at >= cutoff
                                && asset_is_visible(*a, user_id, can_view_all, org)
                        })
                    })
                })
                .count();
            portfolio_count + asset_count
        }
        "recent-activity" => {
            notif
                .notifications
                .iter()
                .filter(|n| n.timestamp >= cutoff)
                .count()
                + txn
                    .transactions
                    .iter()
                    .filter(|t| t.created_at >= cutoff)
                    .count()
                + messenger
                    .messages
                    .iter()
                    .filter(|m| m.timestamp >= cutoff)
                    .count()
                + cal
                    .calendar_events
                    .iter()
                    .filter(|e| e.start >= cutoff && e.start <= now)
                    .count()
                + app
                    .service_tasks
                    .iter()
                    .filter(|st| st.updated_at >= cutoff)
                    .count()
        }
        "organizations" => org
            .organizations
            .iter()
            .filter(|o| o.updated_at >= cutoff && org_is_visible(*o, user_id, can_view_all, org))
            .count(),
        "top-portfolios" => app
            .portfolios
            .iter()
            .filter(|p| {
                p.updated_at >= cutoff && portfolio_is_visible(*p, user_id, can_view_all, org)
            })
            .count(),
        "top-assets" => app
            .portfolios
            .iter()
            .filter(|p| portfolio_is_visible(*p, user_id, can_view_all, org))
            .flat_map(|p| {
                p.asset_groups.iter().flat_map(|g| {
                    g.assets.iter().filter(|a| {
                        a.updated_at >= cutoff && asset_is_visible(*a, user_id, can_view_all, org)
                    })
                })
            })
            .count(),
        "channels" => app
            .channels
            .iter()
            .filter(|c| c.last_sync_at.unwrap_or(c.updated_at) >= cutoff)
            .count(),
        "upcoming-bookings" => cal
            .calendar_events
            .iter()
            .filter(|e| e.start >= now && e.start <= now + window)
            .count(),
        "top-transactions" => txn
            .transactions
            .iter()
            .filter(|t| t.created_at >= cutoff)
            .count(),
        "current-organization" => org
            .organizations
            .iter()
            .filter(|o| o.updated_at >= cutoff && org_is_visible(*o, user_id, can_view_all, org))
            .count(),
        "portfolio-summary" => app
            .portfolios
            .iter()
            .filter(|p| {
                p.updated_at >= cutoff && portfolio_is_visible(*p, user_id, can_view_all, org)
            })
            .count(),
        "channel-status" => app
            .channels
            .iter()
            .filter(|c| c.last_sync_at.unwrap_or(c.updated_at) >= cutoff)
            .count(),
        _ => 0,
    }
}

#[component]
fn OverviewSection(
    id: &'static str,
    #[prop(optional)] wide: bool,
    #[prop(optional)] on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    section_order: Memo<HashMap<String, usize>>,
    children: Children,
) -> impl IntoView {
    let ui_store = use_ui_store();
    view! {
        <div
            class="overview-square"
            class:overview-square-wide=wide
            class:clickable={on_click.is_some()}
            class:overview-section-dragging={move || ui_store.get().overview_dragging_id.as_deref() == Some(id)}
            class:overview-section-drag-over={move || ui_store.get().overview_drag_over_id.as_deref() == Some(id)}
            data-section-id=id
            style:order={move || section_order.with(|m| m.get(id).copied().unwrap_or(0).to_string())}
            draggable={move || if ui_store.get().overview_sort_mode == OverviewSortMode::Selected { "true" } else { "false" }}
            on:click={move |_: leptos::ev::MouseEvent| { if let Some(cb) = on_click.as_ref() { cb(); } }}
            on:dragstart={move |_: leptos::ev::DragEvent| {
                if ui_store.get().overview_sort_mode != OverviewSortMode::Selected { return; }
                ui_store.update(|s| s.set_overview_dragging_id(Some(id.to_string())));
            }}
            on:dragover={move |ev: leptos::ev::DragEvent| {
                if ui_store.get().overview_sort_mode != OverviewSortMode::Selected { return; }
                ev.prevent_default();
                ui_store.update(|s| s.set_overview_drag_over_id(Some(id.to_string())));
            }}
            on:dragleave={move |_| { ui_store.update(|s| s.set_overview_drag_over_id(None)); }}
            on:drop={move |ev: leptos::ev::DragEvent| {
                ev.prevent_default();
                let from = ui_store.get().overview_dragging_id.clone();
                let to = id.to_string();
                if let Some(from) = from {
                    if from != to {
                        ui_store.update(|s| s.reorder_overview_section(&from, &to));
                    }
                }
                ui_store.update(|s| s.clear_overview_drag());
            }}
            on:dragend={move |_| { ui_store.update(|s| s.clear_overview_drag()); }}
            on:touchstart={move |ev: leptos::ev::TouchEvent| {
                if ui_store.get().overview_sort_mode != OverviewSortMode::Selected { return; }
                if let Some(t) = ev.touches().get(0) {
                    let start_id = id.to_string();
                    let x = t.client_x();
                    let y = t.client_y();
                    ui_store.update(|s| s.set_overview_touch_anchor(Some(start_id.clone()), Some((x, y))));
                    spawn_local(async move {
                        TimeoutFuture::new(500).await;
                        if ui_store.get().overview_touch_long_id.as_ref() == Some(&start_id) && ui_store.get().overview_dragging_id.is_none() {
                            ui_store.update(|s| s.set_overview_dragging_id(Some(start_id)));
                        }
                    });
                }
            }}
            on:touchmove={move |ev: leptos::ev::TouchEvent| {
                if ui_store.get().overview_sort_mode != OverviewSortMode::Selected { return; }
                let mut cancel = false;
                if let (Some(t), Some((ax, ay))) = (ev.touches().get(0), ui_store.get().overview_touch_anchor) {
                    let dx = (t.client_x() - ax).abs();
                    let dy = (t.client_y() - ay).abs();
                    if dx > 10 || dy > 10 { cancel = true; }
                }
                if cancel {
                    ui_store.update(|s| { s.set_overview_touch_anchor(None, None); s.set_overview_drag_over_id(None); });
                }
                if ui_store.get().overview_dragging_id.is_some() {
                    if let Some(t) = ev.touches().get(0) {
                        if let Some(doc) = window().and_then(|w| w.document()) {
                            if let Some(elem) = doc.element_from_point(t.client_x() as f32, t.client_y() as f32) {
                                if let Ok(Some(target)) = elem.closest(".overview-square") {
                                    if let Some(target_id) = target.get_attribute("data-section-id") {
                                        if target_id != id { ui_store.update(|s| s.set_overview_drag_over_id(Some(target_id))); }
                                    }
                                }
                            }
                        }
                    }
                }
            }}
            on:touchend={move |ev: leptos::ev::TouchEvent| {
                if ui_store.get().overview_sort_mode != OverviewSortMode::Selected { return; }
                let was_dragging = ui_store.get().overview_dragging_id.is_some();
                if was_dragging { ev.prevent_default(); }
                let from = ui_store.get().overview_dragging_id.clone();
                let to = ui_store.get().overview_drag_over_id.clone();
                if let (Some(from), Some(to)) = (from, to) {
                    if from != to { ui_store.update(|s| s.reorder_overview_section(&from, &to)); }
                }
                ui_store.update(|s| s.clear_overview_drag());
            }}
        >
            {children()}
        </div>
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
    let ui_store = use_ui_store();

    let current_user_id = app_store.get().current_user.id;
    let can_view_all = app_store.get().current_user.can_view_all();

    let portfolio_visible = move |p: &crate::models::Portfolio| -> bool {
        p.is_visible_to(current_user_id, can_view_all)
            || p.organization_id.map_or(can_view_all, |oid| {
                organization_store.get().user_has_perm_in_org(
                    oid,
                    current_user_id,
                    &Perm::ViewPortfolios,
                )
            })
    };
    let asset_visible = move |a: &crate::models::Asset| -> bool {
        a.is_visible_to(current_user_id, can_view_all)
            || a.organization_id.map_or(can_view_all, |oid| {
                organization_store.get().user_has_perm_in_org(
                    oid,
                    current_user_id,
                    &Perm::ViewAssets,
                )
            })
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

    let on_open_messages: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        messenger_store.update(|s| s.set_message_drawer(true));
    });

    let on_open_bookings: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        app_store.update(|s| s.expand_tab(crate::types::TabType::Calendar));
    });

    let on_open_transactions: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        app_store.update(|s| s.expand_tab(crate::types::TabType::Transactions));
    });

    let on_open_portfolios: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        app_store.update(|s| s.expand_tab(crate::types::TabType::Portfolios));
    });

    // Recent bookings = upcoming/recent calendar events
    let recent_bookings = move || {
        let mut events: Vec<_> = calendar_store.get().calendar_events.clone();
        events.sort_by(|a, b| b.start.cmp(&a.start));
        events.into_iter().take(4).collect::<Vec<_>>()
    };

    // Updated investments = recently updated portfolios
    let recent_investments = move || {
        let mut portfolios: Vec<_> = app_store
            .get()
            .portfolios
            .iter()
            .filter(|p| portfolio_visible(*p))
            .cloned()
            .collect();
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
            let desc = t
                .description
                .clone()
                .unwrap_or_else(|| t.to_entity.name.clone());
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
        let current_org_name = current_org
            .map(|o| o.name.clone())
            .unwrap_or_else(|| "None".to_string());
        let current_org_type = current_org
            .and_then(|o| o.business_type.clone())
            .unwrap_or_else(|| "—".to_string());
        let total_roles: usize = current_org.map(|o| o.roles.len()).unwrap_or(0);
        (
            total_orgs,
            total_members,
            current_org_name,
            current_org_type,
            total_roles,
        )
    };

    // Portfolio stats
    let portfolio_stats = move || {
        let store = app_store.get();
        let portfolios: Vec<_> = store
            .portfolios
            .iter()
            .filter(|p| portfolio_visible(*p))
            .collect();
        let total_value: f64 = portfolios.iter().map(|p| p.total_value).sum();
        let total_assets: usize = portfolios.iter().map(|p| p.get_all_assets().len()).sum();
        let total_groups: usize = portfolios.iter().map(|p| p.asset_groups.len()).sum();
        let total_docs: usize = portfolios.iter().map(|p| p.documents.len()).sum();
        (
            portfolios.len(),
            total_value,
            total_assets,
            total_groups,
            total_docs,
        )
    };

    // Channel stats
    let channel_stats = move || {
        let store = app_store.get();
        let channels = &store.channels;
        let connected = channels
            .iter()
            .filter(|c| c.connection_status == ConnectionStatus::Connected)
            .count();
        let disconnected = channels
            .iter()
            .filter(|c| c.connection_status == ConnectionStatus::Disconnected)
            .count();
        let errors = channels
            .iter()
            .filter(|c| c.connection_status == ConnectionStatus::Error)
            .count();
        let enabled = channels.iter().filter(|c| c.enabled).count();
        (channels.len(), connected, disconnected, errors, enabled)
    };

    // Service task stats
    let service_task_stats = move || {
        let store = app_store.get();
        let total = store.service_tasks.len();
        let pending = store
            .service_tasks
            .iter()
            .filter(|t| {
                t.status != crate::models::ServiceTaskStatus::Done
                    && t.status != crate::models::ServiceTaskStatus::Cancelled
            })
            .count();
        let completed = store
            .service_tasks
            .iter()
            .filter(|t| t.status == crate::models::ServiceTaskStatus::Done)
            .count();
        (total, pending, completed)
    };

    // Bottom section data lists
    let organizations_list = move || {
        let orgs = organization_store.get();
        orgs.organizations
            .iter()
            .filter(|o| org_is_visible(*o, current_user_id, can_view_all, &orgs))
            .map(|o| {
                let member_count = organization_store
                    .get()
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
            .filter(|p| portfolio_visible(*p))
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
            if !portfolio_visible(p) {
                continue;
            }
            for a in p.get_all_assets() {
                if !asset_visible(a) {
                    continue;
                }
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
            .map(|t| {
                (
                    t.transaction_type.clone(),
                    t.amount,
                    t.to_entity.name.clone(),
                    t.created_at,
                )
            })
            .collect();
        txns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        txns.into_iter().take(4).collect::<Vec<_>>()
    };

    let on_open_organization: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        app_store.update(|s| s.expand_tab(crate::types::TabType::Organization));
    });

    let on_open_networking: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        app_store.update(|s| s.expand_tab(crate::types::TabType::Networking));
    });

    // Section ordering for Selected/Recent/Trending tabs
    let sorted_section_order = Memo::new(move |_| {
        use crate::stores::OVERVIEW_DEFAULT_ORDER;
        let mode = ui_store.get().overview_sort_mode;
        let mut ids: Vec<String> = OVERVIEW_DEFAULT_ORDER
            .iter()
            .map(|&s| s.to_string())
            .collect();
        match mode {
            OverviewSortMode::Selected => {
                let selected = ui_store.get().overview_selected_order;
                if !selected.is_empty() {
                    let mut ordered = selected.clone();
                    for id in ids {
                        if !ordered.contains(&id) {
                            ordered.push(id);
                        }
                    }
                    ids = ordered;
                }
            }
            OverviewSortMode::Recent => {
                let app = app_store.get();
                let org = organization_store.get();
                let cal = calendar_store.get();
                let messenger = messenger_store.get();
                let notif = notification_store.get();
                let txn = transaction_store.get();
                let now = Utc::now();
                ids.sort_by(|a, b| {
                    let a_time = section_last_changed(
                        a,
                        &app,
                        &org,
                        &cal,
                        &messenger,
                        &notif,
                        &txn,
                        now,
                        current_user_id,
                        can_view_all,
                    );
                    let b_time = section_last_changed(
                        b,
                        &app,
                        &org,
                        &cal,
                        &messenger,
                        &notif,
                        &txn,
                        now,
                        current_user_id,
                        can_view_all,
                    );
                    b_time.cmp(&a_time)
                });
            }
            OverviewSortMode::Trending => {
                let app = app_store.get();
                let org = organization_store.get();
                let cal = calendar_store.get();
                let messenger = messenger_store.get();
                let notif = notification_store.get();
                let txn = transaction_store.get();
                let now = Utc::now();
                let window = Duration::hours(24);
                ids.sort_by(|a, b| {
                    let a_count = section_change_count(
                        a,
                        &app,
                        &org,
                        &cal,
                        &messenger,
                        &notif,
                        &txn,
                        now,
                        window,
                        current_user_id,
                        can_view_all,
                    );
                    let b_count = section_change_count(
                        b,
                        &app,
                        &org,
                        &cal,
                        &messenger,
                        &notif,
                        &txn,
                        now,
                        window,
                        current_user_id,
                        can_view_all,
                    );
                    b_count.cmp(&a_count)
                });
            }
        }
        ids.into_iter()
            .enumerate()
            .map(|(i, id)| (id, i))
            .collect::<HashMap<String, usize>>()
    });

    // Cloned click handlers for detail cards (move closures consume Arcs)
    let on_open_organization_detail = on_open_organization.clone();
    let on_open_portfolios_detail = on_open_portfolios.clone();
    let on_open_networking_detail = on_open_networking.clone();

    view! {
        <div class="overview-content">
            <div class="overview-controls-bar">
                <button
                    class="overview-sort-tab"
                    class:active={move || ui_store.get().overview_sort_mode == OverviewSortMode::Selected}
                    on:click=move |_| ui_store.update(|s| s.set_overview_sort_mode(OverviewSortMode::Selected))
                    title="Selected order; drag and drop boxes"
                >"Selected"</button>
                <button
                    class="overview-sort-tab"
                    class:active={move || ui_store.get().overview_sort_mode == OverviewSortMode::Recent}
                    on:click=move |_| ui_store.update(|s| s.set_overview_sort_mode(OverviewSortMode::Recent))
                    title="Recently updated"
                >"Recent"</button>
                <button
                    class="overview-sort-tab"
                    class:active={move || ui_store.get().overview_sort_mode == OverviewSortMode::Trending}
                    on:click=move |_| ui_store.update(|s| s.set_overview_sort_mode(OverviewSortMode::Trending))
                    title="Most interaction recently"
                >"Trending"</button>
            </div>
            <div class="overview-square-grid">
                // Summary cards
                <OverviewSection id="current-organization" section_order=sorted_section_order wide=true on_click={on_open_organization_detail.clone()}>
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
                </OverviewSection>

                <OverviewSection id="portfolio-summary" section_order=sorted_section_order wide=true on_click={on_open_portfolios_detail.clone()}>
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
                </OverviewSection>

                <OverviewSection id="channel-status" section_order=sorted_section_order wide=true on_click={on_open_networking_detail.clone()}>
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
                </OverviewSection>

                // Recent Messages
                <OverviewSection id="recent-messages" section_order=sorted_section_order on_click={on_open_messages.clone()}>
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
                </OverviewSection>

                // Recent Bookings
                <OverviewSection id="recent-bookings" section_order=sorted_section_order on_click={on_open_bookings.clone()}>
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
                </OverviewSection>

                // Updated Investments
                <OverviewSection id="updated-investments" section_order=sorted_section_order>
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
                </OverviewSection>

                // Recent Transactions
                <OverviewSection id="recent-transactions" section_order=sorted_section_order on_click={on_open_transactions.clone()}>
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
                </OverviewSection>

                // Recent Contacts
                <OverviewSection id="recent-contacts" section_order=sorted_section_order>
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
                </OverviewSection>

                // Notifications
                <OverviewSection id="notifications" section_order=sorted_section_order>
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
                </OverviewSection>

                // Property Overview (full width)
                <OverviewSection id="property-overview" section_order=sorted_section_order wide=true on_click={on_open_portfolios.clone()}>
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
                </OverviewSection>

                // Recent Activity (full width)
                <OverviewSection id="recent-activity" section_order=sorted_section_order wide=true>
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
                </OverviewSection>
            </div>

            // Linked-content continuation
            <div class="overview-square-grid">
                // Organizations (wide rectangle)
                <OverviewSection id="organizations" section_order=sorted_section_order wide=true on_click={on_open_organization.clone()}>
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
                </OverviewSection>

                // Top Portfolios
                <OverviewSection id="top-portfolios" section_order=sorted_section_order on_click={on_open_portfolios.clone()}>
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
                </OverviewSection>

                // Top Assets
                <OverviewSection id="top-assets" section_order=sorted_section_order on_click={on_open_portfolios.clone()}>
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
                </OverviewSection>

                // Channels (wide rectangle)
                <OverviewSection id="channels" section_order=sorted_section_order wide=true on_click={on_open_networking.clone()}>
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
                </OverviewSection>

                // Upcoming Bookings
                <OverviewSection id="upcoming-bookings" section_order=sorted_section_order on_click={on_open_bookings.clone()}>
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
                </OverviewSection>

                // Top Transactions
                <OverviewSection id="top-transactions" section_order=sorted_section_order on_click={on_open_transactions.clone()}>
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
                </OverviewSection>
            </div>
        </div>
    }
}
