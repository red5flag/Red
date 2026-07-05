use crate::models::EntityNotificationSetting;
use crate::stores::{use_app_store, use_notification_store, use_organization_store, Notification};
use crate::types::{NotificationTrigger, NotificationType, UserRole};
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

use super::NotifTarget;

fn trigger_label(t: &NotificationTrigger) -> String {
    match t {
        NotificationTrigger::PriceChange { percentage } => {
            format!("Price change > {}%", percentage)
        }
        NotificationTrigger::Sale => "Sale".to_string(),
        NotificationTrigger::Auction => "Auction".to_string(),
        NotificationTrigger::Rent => "Rent".to_string(),
        NotificationTrigger::Unrent => "Unrent".to_string(),
        NotificationTrigger::NoSales { days } => format!("No sales for {} days", days),
        NotificationTrigger::Custom(s) => s.clone(),
    }
}

fn trigger_short(t: &NotificationTrigger) -> &'static str {
    match t {
        NotificationTrigger::PriceChange { .. } => "Price Change",
        NotificationTrigger::Sale => "Sale",
        NotificationTrigger::Auction => "Auction",
        NotificationTrigger::Rent => "Rent",
        NotificationTrigger::Unrent => "Unrent",
        NotificationTrigger::NoSales { .. } => "No Sales",
        NotificationTrigger::Custom(_) => "Custom",
    }
}

fn notif_type_label(t: &NotificationType) -> &'static str {
    match t {
        NotificationType::Push => "Push",
        NotificationType::Email => "Email",
        NotificationType::Sms => "SMS",
        NotificationType::InApp => "In-App",
    }
}

fn role_label(r: &UserRole) -> &'static str {
    match r {
        UserRole::Owner => "Owner",
        UserRole::Director => "Director",
        UserRole::SeniorManager => "Senior Manager",
        UserRole::Manager => "Manager",
        UserRole::Worker => "Worker",
        UserRole::DocumentWorker => "Document Worker",
        UserRole::Contractor => "Contractor",
        UserRole::Guest => "Guest",
    }
}
/// Quick notification settings popover for a portfolio or group.
/// Opens when the 🔔 badge is clicked.
#[component]
pub(crate) fn NotificationQuickSettings(
    target: NotifTarget,
    entity_name: String,
    on_close: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    let app_store = use_app_store();
    let organization_store = use_organization_store();
    let on_close = std::sync::Arc::new(on_close);
    let on_close2 = on_close.clone();

    let can_manage_recipients = app_store.get().can_manage_notification_recipients();
    let org_users = organization_store.get().organization_users.clone();

    let target_for_settings = target.clone();
    let settings = Memo::new(move |_| match &target_for_settings {
        NotifTarget::Portfolio(pid) => app_store.get().portfolio_notification_settings(*pid),
        NotifTarget::Group(pid, gid) => app_store.get().group_notification_settings(*pid, *gid),
    });

    let (selected_trigger, set_selected_trigger) = signal(NotificationTrigger::Sale);
    let (new_condition, set_new_condition) = signal(String::new());

    let target_for_add = target.clone();
    let add_setting = move |_| {
        let mut setting = EntityNotificationSetting::new(selected_trigger.get());
        if !new_condition.get().trim().is_empty() {
            setting.condition = Some(new_condition.get().trim().to_string());
        }
        match &target_for_add {
            NotifTarget::Portfolio(pid) => {
                app_store.update(|s| s.add_portfolio_notification_setting(*pid, setting));
            }
            NotifTarget::Group(pid, gid) => {
                app_store.update(|s| s.add_group_notification_setting(*pid, *gid, setting));
            }
        }
        set_new_condition.set(String::new());
    };

    let target_for_recipients_section = target.clone();
    let org_users_for_recipients = org_users.clone();

    let all_triggers = vec![
        NotificationTrigger::Sale,
        NotificationTrigger::Auction,
        NotificationTrigger::Rent,
        NotificationTrigger::Unrent,
        NotificationTrigger::PriceChange { percentage: 10.0 },
        NotificationTrigger::NoSales { days: 30 },
        NotificationTrigger::Custom("Document Added".to_string()),
        NotificationTrigger::Custom("Document Updated".to_string()),
    ];

    let all_notif_types = vec![
        NotificationType::InApp,
        NotificationType::Push,
        NotificationType::Email,
        NotificationType::Sms,
    ];

    let all_roles = vec![
        UserRole::Owner,
        UserRole::Director,
        UserRole::SeniorManager,
        UserRole::Manager,
        UserRole::Worker,
        UserRole::DocumentWorker,
    ];

    view! {
        <div class="notif-qs-overlay" on:click=move |_| on_close2()>
            <div class="notif-qs-popover" on:click=|ev| ev.stop_propagation()>
                <div class="notif-qs-header">
                    <span class="notif-qs-title">"🔔 Notification Settings"</span>
                    <span class="notif-qs-entity">{entity_name.clone()}</span>
                    <button class="notif-qs-close" aria-label={format!("Close notification settings for {}", entity_name)} on:click=move |_| on_close()>"✕"</button>
                </div>

                // Existing settings
                <div class="notif-qs-section">
                    <div class="notif-qs-section-label">"Current Rules"</div>
                    {move || {
                        let items = settings.get();
                        if items.is_empty() {
                            view! {
                                <div class="notif-qs-empty">"No notification rules yet. Add one below."</div>
                            }.into_any()
                        } else {
                            items.into_iter().map(|s| {
                                let sid = s.id;
                                let sid_toggle = sid;
                                let sid_remove = sid;
                                let s_enabled = s.enabled;
                                let s_label = trigger_label(&s.trigger);
                                let s_types = s.notification_types.clone();
                                let s_recipients = s.recipients.clone();
                                let s_roles = s.recipient_roles.clone();
                                let s_condition = s.condition.clone();
                                let target_toggle = target.clone();
                                let target_remove = target.clone();

                                let all_nt = all_notif_types.clone();
                                let target_for_nt = target.clone();
                                let sid_for_nt = sid;

                                view! {
                                    <div class="notif-qs-rule" class:disabled={!s_enabled}>
                                        <div class="notif-qs-rule-top">
                                            <label class="notif-qs-toggle">
                                                <input type="checkbox" checked=s_enabled
                                                    on:change=move |_| {
                                                        match &target_toggle {
                                                            NotifTarget::Portfolio(pid) => app_store.update(|s| s.toggle_portfolio_notification_setting(*pid, sid_toggle)),
                                                            NotifTarget::Group(pid, gid) => app_store.update(|s| s.toggle_group_notification_setting(*pid, *gid, sid_toggle)),
                                                        }
                                                    } />
                                                <span class="notif-qs-rule-name">{s_label.clone()}</span>
                                            </label>
                                            <button class="notif-qs-rule-remove"
                                                aria-label={format!("Remove {} rule", s_label)}
                                                on:click=move |_| {
                                                    match &target_remove {
                                                        NotifTarget::Portfolio(pid) => app_store.update(|s| s.remove_portfolio_notification_setting(*pid, sid_remove)),
                                                        NotifTarget::Group(pid, gid) => app_store.update(|s| s.remove_group_notification_setting(*pid, *gid, sid_remove)),
                                                    }
                                                }>"🗑"</button>
                                        </div>
                                        // Notification type badges
                                        <div class="notif-qs-rule-types">
                                            {all_nt.iter().map(|nt| {
                                                let nt_label = notif_type_label(nt);
                                                let is_on = s_types.contains(nt);
                                                let target_nt = target_for_nt.clone();
                                                let sid_nt = sid_for_nt;
                                                let nt_clone = nt.clone();
                                                view! {
                                                    <button class="notif-qs-type-chip"
                                                        class:active=is_on
                                                        on:click=move |_| {
                                                            match &target_nt {
                                                                NotifTarget::Portfolio(pid) => app_store.update(|s| {
                                                                    if let Some(p) = s.get_portfolio_mut(*pid) {
                                                                        if let Some(st) = p.notification_settings.iter_mut().find(|st| st.id == sid_nt) {
                                                                            if st.notification_types.contains(&nt_clone) {
                                                                                st.notification_types.retain(|t| t != &nt_clone);
                                                                            } else {
                                                                                st.notification_types.push(nt_clone.clone());
                                                                            }
                                                                        }
                                                                    }
                                                                }),
                                                                NotifTarget::Group(pid, gid) => app_store.update(|s| {
                                                                    if let Some(p) = s.get_portfolio_mut(*pid) {
                                                                        if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == *gid) {
                                                                            if let Some(st) = g.notification_settings.iter_mut().find(|st| st.id == sid_nt) {
                                                                                if st.notification_types.contains(&nt_clone) {
                                                                                    st.notification_types.retain(|t| t != &nt_clone);
                                                                                } else {
                                                                                    st.notification_types.push(nt_clone.clone());
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }),
                                                            }
                                                        }>
                                                        {nt_label}
                                                    </button>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                        // Recipients (if can manage)
                                        {if can_manage_recipients {
                                            let recipient_names: Vec<String> = s_recipients.iter().filter_map(|uid| {
                                                org_users.iter().find(|u| u.id == *uid).map(|u| u.name.clone())
                                            }).collect();
                                            let role_names: Vec<&'static str> = s_roles.iter().map(role_label).collect();
                                            let info = if recipient_names.is_empty() && role_names.is_empty() {
                                                "Just me".to_string()
                                            } else {
                                                let mut parts = Vec::new();
                                                if !recipient_names.is_empty() {
                                                    parts.push(format!("Users: {}", recipient_names.join(", ")));
                                                }
                                                if !role_names.is_empty() {
                                                    parts.push(format!("Roles: {}", role_names.join(", ")));
                                                }
                                                parts.join(" · ")
                                            };
                                            view! {
                                                <div class="notif-qs-rule-recipients">{info}</div>
                                            }.into_any()
                                        } else { ().into_any() }}
                                        // Condition
                                        {s_condition.map(|c| view! {
                                            <div class="notif-qs-rule-condition">"Condition: " {c}</div>
                                        })}
                                    </div>
                                }
                            }).collect::<Vec<_>>().into_any()
                        }
                    }}
                </div>

                // Add new rule
                <div class="notif-qs-section">
                    <div class="notif-qs-section-label">"Add Notification Rule"</div>
                    <div class="notif-qs-add-row">
                        <select class="notif-qs-select"
                            on:change=move |ev| {
                                let v = event_target_value(&ev);
                                let t = match v.as_str() {
                                    "Sale" => NotificationTrigger::Sale,
                                    "Auction" => NotificationTrigger::Auction,
                                    "Rent" => NotificationTrigger::Rent,
                                    "Unrent" => NotificationTrigger::Unrent,
                                    "PriceChange" => NotificationTrigger::PriceChange { percentage: 10.0 },
                                    "NoSales" => NotificationTrigger::NoSales { days: 30 },
                                    "DocumentAdded" => NotificationTrigger::Custom("Document Added".to_string()),
                                    "DocumentUpdated" => NotificationTrigger::Custom("Document Updated".to_string()),
                                    _ => NotificationTrigger::Sale,
                                };
                                set_selected_trigger.set(t);
                            }>
                            {all_triggers.iter().map(|t| {
                                let val = match t {
                                    NotificationTrigger::Sale => "Sale",
                                    NotificationTrigger::Auction => "Auction",
                                    NotificationTrigger::Rent => "Rent",
                                    NotificationTrigger::Unrent => "Unrent",
                                    NotificationTrigger::PriceChange { .. } => "PriceChange",
                                    NotificationTrigger::NoSales { .. } => "NoSales",
                                    NotificationTrigger::Custom(s) if s == "Document Added" => "DocumentAdded",
                                    NotificationTrigger::Custom(s) if s == "Document Updated" => "DocumentUpdated",
                                    _ => "Custom",
                                };
                                view! {
                                    <option value={val}>{trigger_short(t)}</option>
                                }
                            }).collect::<Vec<_>>()}
                        </select>
                        <input class="notif-qs-input" type="text" placeholder="Condition (optional, e.g. 'Only PDF docs')"
                            aria-label="Condition (optional)"
                            prop:value=move || new_condition.get()
                            on:input=move |ev| set_new_condition.set(event_target_value(&ev)) />
                        <button class="notif-qs-add-btn" aria-label={format!("Add notification rule for {}", entity_name)} on:click=add_setting>"+ Add"</button>
                    </div>
                </div>

                // Recipient configuration (role-gated)
                {can_manage_recipients.then(|| {
                    let target_for_recipients = target_for_recipients_section.clone();
                    let users_for_select = org_users_for_recipients.clone();
                    view! {
                        <div class="notif-qs-section">
                            <div class="notif-qs-section-label">"Recipient Roles (applies to most recent rule)"</div>
                            <div class="notif-qs-roles-row">
                                {all_roles.iter().map(|r| {
                                    let r_label = role_label(r);
                                    let r_clone = r.clone();
                                    let target_r = target_for_recipients.clone();
                                    view! {
                                        <button class="notif-qs-role-chip"
                                            on:click=move |_| {
                                                match &target_r {
                                                    NotifTarget::Portfolio(pid) => app_store.update(|s| {
                                                        if let Some(p) = s.get_portfolio_mut(*pid) {
                                                            if let Some(last) = p.notification_settings.last_mut() {
                                                                if last.recipient_roles.contains(&r_clone) {
                                                                    last.recipient_roles.retain(|r| r != &r_clone);
                                                                } else {
                                                                    last.recipient_roles.push(r_clone.clone());
                                                                }
                                                            }
                                                        }
                                                    }),
                                                    NotifTarget::Group(pid, gid) => app_store.update(|s| {
                                                        if let Some(p) = s.get_portfolio_mut(*pid) {
                                                            if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == *gid) {
                                                                if let Some(last) = g.notification_settings.last_mut() {
                                                                    if last.recipient_roles.contains(&r_clone) {
                                                                        last.recipient_roles.retain(|r| r != &r_clone);
                                                                    } else {
                                                                        last.recipient_roles.push(r_clone.clone());
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }),
                                                }
                                            }>
                                            {r_label}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            <div class="notif-qs-section-label" style="margin-top: 8px;">"Recipient Users (applies to most recent rule)"</div>
                            <div class="notif-qs-users-row">
                                {users_for_select.iter().map(|u| {
                                    let uname = u.name.clone();
                                    let uid = u.id;
                                    let target_u = target_for_recipients.clone();
                                    view! {
                                        <button class="notif-qs-user-chip"
                                            on:click=move |_| {
                                                match &target_u {
                                                    NotifTarget::Portfolio(pid) => app_store.update(|s| {
                                                        if let Some(p) = s.get_portfolio_mut(*pid) {
                                                            if let Some(last) = p.notification_settings.last_mut() {
                                                                if last.recipients.contains(&uid) {
                                                                    last.recipients.retain(|id| id != &uid);
                                                                } else {
                                                                    last.recipients.push(uid);
                                                                }
                                                            }
                                                        }
                                                    }),
                                                    NotifTarget::Group(pid, gid) => app_store.update(|s| {
                                                        if let Some(p) = s.get_portfolio_mut(*pid) {
                                                            if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == *gid) {
                                                                if let Some(last) = g.notification_settings.last_mut() {
                                                                    if last.recipients.contains(&uid) {
                                                                        last.recipients.retain(|id| id != &uid);
                                                                    } else {
                                                                        last.recipients.push(uid);
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }),
                                                }
                                            }>
                                            {uname}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
/// Read-only notification content view for a portfolio or group.
/// Left-clicking the bell badge opens this; right-clicking opens the settings editor.
#[component]
pub(crate) fn NotificationContentView(
    target: NotifTarget,
    entity_name: String,
    on_close: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    let app_store = use_app_store();
    let notification_store = use_notification_store();
    let on_close = std::sync::Arc::new(on_close);
    let on_close2 = on_close.clone();

    let entity_notifs = Memo::new(move |_| {
        let app = app_store.get();
        let notifications = notification_store.get().notifications;
        let notifs: Vec<Notification> = match &target {
            NotifTarget::Portfolio(pid) => {
                let doc_ids: HashSet<Uuid> = app
                    .portfolios
                    .iter()
                    .find(|p| p.id == *pid)
                    .map(|p| {
                        let mut ids = p.documents.iter().map(|d| d.id).collect::<HashSet<_>>();
                        for g in &p.asset_groups {
                            ids.extend(g.documents.iter().map(|d| d.id));
                            for a in &g.assets {
                                ids.extend(a.documents.iter().map(|d| d.id));
                            }
                        }
                        for a in &p.assets {
                            ids.extend(a.documents.iter().map(|d| d.id));
                        }
                        ids
                    })
                    .unwrap_or_default();
                notifications
                    .iter()
                    .filter(|n| {
                        n.linked_portfolio_id == Some(*pid)
                            || n.linked_doc_id
                                .map(|did| doc_ids.contains(&did))
                                .unwrap_or(false)
                    })
                    .cloned()
                    .collect()
            }
            NotifTarget::Group(pid, gid) => {
                let doc_ids: HashSet<Uuid> = app
                    .portfolios
                    .iter()
                    .find(|p| p.id == *pid)
                    .and_then(|p| p.asset_groups.iter().find(|g| g.id == *gid))
                    .map(|g| {
                        let mut ids = g.documents.iter().map(|d| d.id).collect::<HashSet<_>>();
                        for a in &g.assets {
                            ids.extend(a.documents.iter().map(|d| d.id));
                        }
                        ids
                    })
                    .unwrap_or_default();
                notifications
                    .iter()
                    .filter(|n| {
                        n.linked_group_id == Some(*gid)
                            || n.linked_doc_id
                                .map(|did| doc_ids.contains(&did))
                                .unwrap_or(false)
                    })
                    .cloned()
                    .collect()
            }
        };
        let mut notifs = notifs;
        notifs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        notifs
    });

    let on_close_overlay = on_close2.clone();
    let on_close_btn = on_close2.clone();
    view! {
        <div class="notif-qs-overlay" on:click=move |_| on_close_overlay()>
            <div class="notif-qs-popover" on:click=|ev| ev.stop_propagation()>
                <div class="notif-qs-header">
                    <span class="notif-qs-title">"🔔 Notifications"</span>
                    <span class="notif-qs-entity">{entity_name.clone()}</span>
                    <button class="notif-qs-close" aria-label={format!("Close notifications for {}", entity_name)} on:click=move |_| on_close_btn()>"✕"</button>
                </div>
                <div class="notif-qs-body">
                    {move || {
                        let notifs = entity_notifs.get();
                        if notifs.is_empty() {
                            view! {
                                <div class="notif-qs-empty">"No notifications for this entity."</div>
                            }.into_any()
                        } else {
                            notifs.into_iter().map(|n| {
                                let nid = n.id;
                                let n_for_nav = n.clone();
                                let msg = n.message.clone();
                                let from = n.from_user.clone().unwrap_or_else(|| "System".to_string());
                                let time = format!("{}", n.timestamp.format("%b %d, %H:%M"));
                                let preview = n.content_preview.clone();
                                let has_doc = n.linked_doc_id.is_some();
                                let on_close = on_close.clone();
                                view! {
                                    <div class="notif-content-item">
                                        <div class="notif-content-msg">{msg}</div>
                                        <div class="notif-content-meta">
                                            <span>{from}</span>
                                            <span>{time}</span>
                                        </div>
                                        {preview.map(|p| view! {
                                            <div class="notif-content-preview">
                                                <pre>{p}</pre>
                                            </div>
                                        }.into_any())}
                                        <div class="notif-content-actions">
                                            {if has_doc {
                                                view! {
                                                    <button class="notif-content-open-btn" on:click=move |_| {
                                                        app_store.update(|s| s.navigate_to_notification(&n_for_nav));
                                                        notification_store.update(|s| s.close_drawer());
                                                        on_close();
                                                    }>"Open"</button>
                                                }.into_any()
                                            } else { ().into_any() }}
                                            <button class="notif-content-dismiss-btn" on:click=move |_| {
                                                notification_store.update(|s| s.remove_notification(nid));
                                            }>"Dismiss"</button>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>().into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
