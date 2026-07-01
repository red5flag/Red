use crate::models::{default_permissions_for_role, Payment, PaymentSettings, PaymentStatus, Permission, User, UserActivity, UserAssignment};
use crate::stores::use_app_store;
use crate::types::{PaymentInterval, PaymentMethod, TabType, UserRole};
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn NetworkingPage() -> impl IntoView {
    let app_store = use_app_store();

    // Users come from app store, fall back to default mock users if empty, sorted by role
    let users = Memo::new(move |_| {
        let store = app_store.get();
        let store_users = store.organization_users.clone();
        let current_org = store.current_organization_id.or(store.current_user.organization_id);
        let mut users: Vec<_> = if store_users.is_empty() {
            default_mock_users()
        } else {
            store_users
        }
        .into_iter()
        .filter(|u| current_org.is_none() || u.organization_id == current_org)
        .collect();
        users.sort_by(|a, b| b.role.level().cmp(&a.role.level()));
        users
    });

    let _on_update_role = move |id: Uuid, role: UserRole| {
        app_store.update(|s| {
            let _ = s.update_user_role(id, role);
        });
    };

    let _on_remove_user = move |id: Uuid| {
        app_store.update(|s| {
            s.remove_organization_user(id);
        });
    };

    let _on_toggle_permission = move |id: Uuid, permission: Permission| {
        app_store.update(|s| {
            s.toggle_user_permission(id, permission);
        });
    };

    let (edit_mode, set_edit_mode) = signal(false);
    let (_people_open, _set_people_open) = signal(false);
    let (active_tab, set_active_tab) = signal("organizations");

    // Mock transactions
    let transactions = Memo::new(move |_| {
        vec![
            Payment {
                id: Uuid::new_v4(),
                from_user_id: Uuid::new_v4(),
                to_user_id: Uuid::new_v4(),
                amount: 5000.0,
                currency: crate::types::Currency::USD,
                payment_method: PaymentMethod::BankTransfer,
                description: Some("Monthly salary payment".to_string()),
                related_asset_id: None,
                related_portfolio_id: None,
                status: PaymentStatus::Completed,
                scheduled_date: None,
                executed_date: Some(Utc::now()),
                created_at: Utc::now(),
                is_recurring: true,
                recurrence_rule: Some("monthly".to_string()),
            },
            Payment {
                id: Uuid::new_v4(),
                from_user_id: Uuid::new_v4(),
                to_user_id: Uuid::new_v4(),
                amount: 2500.0,
                currency: crate::types::Currency::USD,
                payment_method: PaymentMethod::BankTransfer,
                description: Some("Asset performance bonus".to_string()),
                related_asset_id: Some(Uuid::new_v4()),
                related_portfolio_id: None,
                status: PaymentStatus::Pending,
                scheduled_date: Some(Utc::now()),
                executed_date: None,
                created_at: Utc::now(),
                is_recurring: false,
                recurrence_rule: None,
            },
        ]
    });

    let render_tab_content = move || {
        let tab = active_tab.get();
        if tab == "organizations" {
            render_organizations(users).into_any()
        } else if tab == "members" {
            render_members(users).into_any()
        } else if tab == "activity" {
            render_activity().into_any()
        } else {
            render_payments(transactions).into_any()
        }
    };

    view! {
        <div class="home-screen">
            // Quick tabs
            <div class="net-quick-tabs">
                <button
                    class="net-quick-tab"
                    class:active={move || active_tab.get() == "organizations"}
                    on:click=move |_| set_active_tab.set("organizations")
                >
                    "Organizations"
                </button>
                <button
                    class="net-quick-tab"
                    class:active={move || active_tab.get() == "members"}
                    on:click=move |_| set_active_tab.set("members")
                >
                    {move || format!("Members {}", users.get().len())}
                </button>
                <button
                    class="net-quick-tab"
                    class:active={move || active_tab.get() == "activity"}
                    on:click=move |_| set_active_tab.set("activity")
                >
                    "Activity"
                </button>
                <button
                    class="net-quick-tab"
                    class:active={move || active_tab.get() == "payments"}
                    on:click=move |_| set_active_tab.set("payments")
                >
                    "Payments"
                </button>
            </div>

            // Organization Stats
            <div class="org-metrics-bar">
                <div class="org-metric">
                    <div class="org-metric-value">{move || app_store.get().organizations.len()}</div>
                    <div class="org-metric-label">"Organizations"</div>
                </div>
                <div class="org-metric">
                    <div class="org-metric-value">{move || users.get().len()}</div>
                    <div class="org-metric-label">"Members"</div>
                </div>
                <div class="org-metric">
                    <div class="org-metric-value">
                        {move || {
                            transactions.get()
                                .iter()
                                .filter(|t| t.status == PaymentStatus::Pending)
                                .count()
                        }}
                    </div>
                    <div class="org-metric-label">"Pending"</div>
                </div>
                <div class="org-metric">
                    <div class="org-metric-value">
                        {move || {
                            let total: f64 = transactions.get()
                                .iter()
                                .filter(|t| t.status == PaymentStatus::Completed)
                                .map(|t| t.amount)
                                .sum();
                            format!("${:.0}K", total / 1000.0)
                        }}
                    </div>
                    <div class="org-metric-label">"Payouts"</div>
                </div>
            </div>

            // Edit action bar
            <div class="net-action-bar">
                <button
                    class="net-action-btn"
                    class:active={move || edit_mode.get()}
                    on:click=move |_| set_edit_mode.update(|v| *v = !*v)
                >
                    "Edit"
                </button>
                <button
                    class="net-action-btn"
                    on:click=move |_| app_store.update(|s| s.toggle_networking_add_member())
                >
                    "Add"
                </button>
                <button
                    class="net-action-btn"
                    on:click=move |_| {
                        if let Some(first) = users.get().first() {
                            app_store.update(|s| { let _ = s.remove_organization_user(first.id); });
                        }
                    }
                >
                    "Remove"
                </button>
                <button
                    class="net-action-btn"
                    on:click=move |_| app_store.update(|s| s.open_search())
                >
                    "Search"
                </button>
            </div>

            // Tab content
            {render_tab_content}
        </div>
    }
}

fn render_organizations(_users: Memo<Vec<User>>) -> impl IntoView {
    let app_store = use_app_store();
    let (editing_org, set_editing_org) = signal(Option::<Uuid>::None);
    let (edit_name, set_edit_name) = signal(String::new());

    let save_org_name = move |id: Uuid| {
        let name = edit_name.get();
        if name.trim().is_empty() { return; }
        app_store.update(|s| {
            if let Some(org) = s.get_organization_mut(id) {
                org.name = name;
                org.updated_at = chrono::Utc::now();
            }
        });
        set_editing_org.set(None);
    };

    view! {
        <div class="net-tab-content">
            {move || {
                let store = app_store.get();
                let orgs = store.organizations.clone();
                let all_users = if store.organization_users.is_empty() {
                    default_mock_users()
                } else {
                    store.organization_users.clone()
                };
                if orgs.is_empty() {
                    view! {
                        <div class="data-card">
                            <div class="empty-state">
                                <div class="empty-text">"No organizations yet"</div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    orgs.into_iter().map(|org| {
                        let oid = org.id;
                        let is_editing = editing_org.get() == Some(oid);
                        let mut org_users: Vec<User> = all_users.iter()
                            .filter(|u| u.organization_id == Some(org.id))
                            .cloned()
                            .collect();
                        org_users.sort_by(|a, b| b.role.level().cmp(&a.role.level()));
                        let org_color_style = org.settings.color.as_ref().map(|c| format!("border-left: 6px solid {};", c)).unwrap_or_default();
                        view! {
                            <div class="data-card">
                                <div class="card-header" style={org_color_style}>
                                    {if is_editing {
                                        view! {
                                            <input class="pf-edit-input" type="text" prop:value=edit_name
                                                on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                                on:blur=move |_| save_org_name(oid)
                                                on:keydown=move |ev| { if ev.key() == "Enter" { save_org_name(oid); } }
                                            />
                                        }.into_any()
                                    } else {
                                        let n = org.name.clone();
                                        view! {
                                            <span class="card-title" on:dblclick=move |_| {
                                                set_edit_name.set(n.clone());
                                                set_editing_org.set(Some(oid));
                                            }>{org.name.clone()}</span>
                                        }.into_any()
                                    }}
                                    <span class="stat-value">{format!("{} members", org_users.len())}</span>
                                </div>
                                <div class="net-org-members">
                                    {org_users.into_iter().map(|user| view! {
                                        <UserCard user={user} />
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>().into_any()
                }
            }}
        </div>
    }
}

fn render_members(users: Memo<Vec<User>>) -> impl IntoView {
    let app_store = use_app_store();
    let (filter_by, set_filter_by) = signal("role");
    let (expanded_id, set_expanded_id) = signal(Option::<Uuid>::None);

    view! {
        <div class="net-tab-content">
            <div class="net-filter-bar">
                <span class="net-filter-label">"Filter by:"</span>
                <button
                    class="net-filter-btn"
                    class:active={move || filter_by.get() == "role"}
                    on:click=move |_| set_filter_by.set("role")
                >
                    "Role"
                </button>
                <button
                    class="net-filter-btn"
                    class:active={move || filter_by.get() == "user"}
                    on:click=move |_| set_filter_by.set("user")
                >
                    "User"
                </button>
                <button
                    class="net-filter-btn net-filter-add"
                    style="margin-left: auto;"
                    on:click=move |_| app_store.update(|s| s.expand_tab(TabType::NetworkingAddMember))
                >
                    "+ Add User"
                </button>
            </div>
            <div class="net-members-table">
                {move || {
                    let filter = filter_by.get();
                    let mut items = users.get();
                    if filter == "role" {
                        items.sort_by(|a, b| b.role.level().cmp(&a.role.level()));
                    } else {
                        items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                    }
                    items.into_iter().map(|user| {
                        let uid = user.id;
                        let is_expanded = expanded_id.get() == Some(uid);
                        let on_toggle = move || {
                            set_expanded_id.update(|v| {
                                if *v == Some(uid) { *v = None; }
                                else { *v = Some(uid); }
                            });
                        };
                        view! {
                            <MemberRow user={user} filter_by={filter} expanded={is_expanded} on_toggle={on_toggle} />
                        }
                    }).collect::<Vec<_>>().into_any()
                }}
            </div>
        </div>
    }
}

#[component]
fn MemberRow(
    user: User,
    filter_by: &'static str,
    expanded: bool,
    on_toggle: impl Fn() + 'static,
) -> impl IntoView {
    let app_store = use_app_store();
    let (is_editing, set_is_editing) = signal(false);
    let (edit_name, set_edit_name) = signal(user.name.clone());
    let user_id = user.id;

    let save_name = move || {
        let name = edit_name.get();
        if name.trim().is_empty() { return; }
        app_store.update(|s| {
            let _ = s.update_user_name(user_id, name);
        });
        set_is_editing.set(false);
    };

    let user_name = user.name.clone();
    let initials: String = user_name
        .split_whitespace()
        .filter_map(|s| s.chars().next())
        .collect::<String>()
        .to_uppercase();
    let role = format!("{:?}", user.role);
    let assignment = user.assignments.first().map(|a| a.target_name.clone()).unwrap_or_else(|| "—".to_string());
    let availability = if user.is_active { "AVA" } else { "OFF" };
    let location = user.address.clone().unwrap_or_else(|| "—".to_string());
    let account = user.payment_settings.account_details.clone();

    view! {
        <div class="net-member-row" class:expanded={expanded}>
            <div class="net-member-header" on:click=move |_| on_toggle()>
                <span class="net-member-arrow">{if expanded { "▼" } else { "▶" }}</span>
                {if is_editing.get() {
                    view! {
                        <input class="pf-edit-input" type="text" prop:value=edit_name
                            on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                            on:blur=move |_| save_name()
                            on:keydown=move |ev| { if ev.key() == "Enter" { save_name(); } }
                        />
                    }.into_any()
                } else if filter_by == "role" {
                    let title_name = user_name.clone();
                    view! {
                        <span class="net-member-avatar">{initials}</span>
                        <span class="net-member-title" on:dblclick=move |_| {
                            set_edit_name.set(title_name.clone());
                            set_is_editing.set(true);
                        }>{user_name.clone()}</span>
                        <span class="net-member-role">{role.clone()}</span>
                    }.into_any()
                } else {
                    let filter_name = user_name.clone();
                    view! {
                        <span class="net-member-name" on:dblclick=move |_| {
                            set_edit_name.set(filter_name.clone());
                            set_is_editing.set(true);
                        }>{user_name.clone()}</span>
                        <span class="net-member-role-port">{format!("{}/{}", role, assignment)}</span>
                    }.into_any()
                }}
                <button class="net-member-add" on:click=|ev| ev.stop_propagation()>"+"</button>
            </div>
            {if expanded {
                view! {
                    <div class="net-member-detail">
                        <div class="net-member-detail-row">
                            <span class="net-member-detail-label">"NAME"</span>
                            {if is_editing.get() {
                                view! {
                                    <input class="pf-edit-input" type="text" prop:value=edit_name
                                        on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                        on:blur=move |_| save_name()
                                        on:keydown=move |ev| { if ev.key() == "Enter" { save_name(); } }
                                    />
                                }.into_any()
                            } else {
                                let detail_name = user_name.clone();
                                view! {
                                    <span class="net-member-detail-value" on:dblclick=move |_| {
                                        set_edit_name.set(detail_name.clone());
                                        set_is_editing.set(true);
                                    }>{user_name.clone()}</span>
                                }.into_any()
                            }}
                        </div>
                        <div class="net-member-detail-row">
                            <span class="net-member-detail-label">"ROLE & AVAILABILITY"</span>
                            <span class="net-member-detail-value">{format!("{} ({})", role, availability)}</span>
                        </div>
                        <div class="net-member-detail-row">
                            <span class="net-member-detail-label">"LOCATION"</span>
                            <span class="net-member-detail-value">{location}</span>
                        </div>
                        <div class="net-member-detail-row">
                            <span class="net-member-detail-label">"ACCOUNTS"</span>
                            <span class="net-member-detail-value">{account}</span>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </div>
    }
}

fn render_activity() -> impl IntoView {
    let app_store = use_app_store();
    view! {
        <div class="net-tab-content">
            {move || {
                let store = app_store.get();
                let all_users = if store.organization_users.is_empty() {
                    default_mock_users()
                } else {
                    store.organization_users.clone()
                };
                let activities: Vec<_> = all_users.iter()
                    .flat_map(|u| u.activity_log.iter().map(move |a| (u.clone(), a.clone())))
                    .collect();
                if activities.is_empty() {
                    view! {
                        <div class="data-card">
                            <div class="empty-state">
                                <div class="empty-text">"No activity yet"</div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    activities.into_iter().map(|(user, activity)| {
                        view! {
                            <div class="list-item">
                                <div class="list-item-left">
                                    <div class="list-item-title">{format!("{} {}", user.name, activity.action)}</div>
                                    <div class="list-item-subtitle">{format!("{}: {} - {}", activity.target_type, activity.target_name, activity.reason.as_deref().unwrap_or(""))}</div>
                                </div>
                                <div class="list-item-right">
                                    <div class="stat-value">{activity.timestamp.format("%Y-%m-%d").to_string()}</div>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>().into_any()
                }
            }}
        </div>
    }
}

fn render_payments(transactions: Memo<Vec<Payment>>) -> impl IntoView {
    view! {
        <div class="net-tab-content">
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Recent Payments"</span>
                </div>
                <div class="data-list">
                    {move || {
                        transactions.get().into_iter().map(|payment| {
                            let status_class = match payment.status {
                                PaymentStatus::Completed => "positive",
                                PaymentStatus::Pending => "",
                                PaymentStatus::Failed => "negative",
                                _ => "",
                            };
                            let status_icon = match payment.status {
                                PaymentStatus::Completed => "✓",
                                PaymentStatus::Pending => "⏳",
                                PaymentStatus::Scheduled => "📅",
                                PaymentStatus::Processing => "⚙️",
                                PaymentStatus::Failed => "✗",
                                PaymentStatus::Cancelled => "⊘",
                            };
                            view! {
                                <div class="list-item">
                                    <div class="list-item-left">
                                        <div class="list-item-title">
                                            {format!("{} {}", status_icon,
                                                payment.description.as_deref().unwrap_or("Payment")
                                            )}
                                        </div>
                                        <div class="list-item-subtitle">
                                            {format!("{:?} - {:?}",
                                                payment.payment_method,
                                                payment.status
                                            )}
                                        </div>
                                    </div>
                                    <div class="list-item-right">
                                        <div class={format!("list-item-value {}", status_class)}>
                                            {format!("${:.0}", payment.amount)}
                                        </div>
                                        {payment.is_recurring.then(|| {
                                            view! {
                                                <div style="font-size: 10px; color: var(--text-secondary);">
                                                    "🔄 Recurring"
                                                </div>
                                            }
                                        })}
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn UserCard(user: User) -> impl IntoView {
    let avatar = user.avatar_url.clone().unwrap_or_else(|| {
        format!("https://api.dicebear.com/7.x/avataaars/svg?seed={}", user.id)
    });
    let role_label = format!("{:?}", user.role);
    let username = user.username.clone().unwrap_or_else(|| user.name.clone());
    let assignment = user.assignments.first().cloned();

    view! {
        <div class="net-user-card">
            <img class="net-user-avatar" src={avatar} alt={user.name.clone()} />
            <div class="net-user-info">
                <div class="net-user-name">{user.name.clone()}</div>
                <div class="net-user-meta">{format!("@{} • {}", username, user.email)}</div>
                <div class="net-user-role">{role_label}</div>
                {if let Some(a) = assignment {
                    view! {
                        <div class="net-user-assignment">
                            {format!("Assigned to {}: {}", a.target_type, a.target_name)}
                            {a.duration_days.map(|d| format!(" • {} days", d)).unwrap_or_default()}
                            {a.reason.map(|r| format!(" • {}", r)).unwrap_or_default()}
                        </div>
                    }.into_any()
                } else { ().into_any() }}
            </div>
        </div>
    }
}

#[component]
pub fn AddTeamMemberPage() -> impl IntoView {
    let app_store = use_app_store();

    let (search_query, set_search_query) = signal(String::new());
    let (new_name, set_new_name) = signal(String::new());
    let (new_username, set_new_username) = signal(String::new());
    let (new_email, set_new_email) = signal(String::new());
    let (new_role, set_new_role) = signal(UserRole::Worker);

    let add_user = move |name: String, email: String, username: Option<String>, role: UserRole| {
        let name = name.trim().to_string();
        let email = email.trim().to_string();
        if name.is_empty() || email.is_empty() { return; }
        let username = username.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        let avatar = format!("https://api.dicebear.com/7.x/avataaars/svg?seed={}", username.as_ref().unwrap_or(&name));
        let mut user = User::new(name, email, role);
        user.username = username;
        user.avatar_url = Some(avatar);
        let store = app_store.get();
        user.organization_id = store.current_organization_id.or(store.current_user.organization_id);
        drop(store);
        app_store.update(|s| s.add_organization_user(user));
    };

    let on_add_user = move |_| {
        let username = new_username.get().trim().to_string();
        add_user(new_name.get(), new_email.get(), Some(username), new_role.get());
        set_new_name.set(String::new());
        set_new_username.set(String::new());
        set_new_email.set(String::new());
        set_new_role.set(UserRole::Worker);
    };

    let on_add_found = move |name: String, email: String, username: Option<String>| {
        add_user(name, email, username, new_role.get());
    };

    let search_results = Memo::new(move |_| {
        let query = search_query.get().trim().to_lowercase();
        if query.len() < 2 {
            return Vec::<User>::new();
        }
        let store = app_store.get();
        let mut results: Vec<User> = Vec::new();
        let current_org = store.current_organization_id.or(store.current_user.organization_id);
        let existing_ids: std::collections::HashSet<Uuid> = store.organization_users.iter().map(|u| u.id).collect();

        // Local users from credential store
        for cred in store.credentials.credentials.values() {
            let name = cred.display_name.to_lowercase();
            let email = cred.email.to_lowercase();
            let username = cred.username.to_lowercase();
            if name.contains(&query) || email.contains(&query) || username.contains(&query) {
                let mut user = User::new(cred.display_name.clone(), cred.email.clone(), UserRole::Guest);
                user.username = Some(cred.username.clone());
                user.avatar_url = Some(format!("https://api.dicebear.com/7.x/avataaars/svg?seed={}", cred.username));
                user.organization_id = current_org;
                if !existing_ids.contains(&user.id) {
                    results.push(user);
                }
            }
        }

        // Server/online users already known to the app
        for user in store.organization_users.iter() {
            let name = user.name.to_lowercase();
            let email = user.email.to_lowercase();
            let username = user.username.clone().unwrap_or_default().to_lowercase();
            if name.contains(&query) || email.contains(&query) || username.contains(&query) {
                if !results.iter().any(|u| u.email == user.email) {
                    results.push(user.clone());
                }
            }
        }

        // Mock server users representing people available on the server but not yet in the org
        let server_pool = vec![
            User::new("Alice Chen".to_string(), "alice@company.com".to_string(), UserRole::Manager),
            User::new("Bob Martinez".to_string(), "bob@company.com".to_string(), UserRole::Worker),
            User::new("Carol White".to_string(), "carol@company.com".to_string(), UserRole::Director),
            User::new("David Kim".to_string(), "david@company.com".to_string(), UserRole::Contractor),
        ];
        for mut user in server_pool {
            let name = user.name.to_lowercase();
            let email = user.email.to_lowercase();
            if name.contains(&query) || email.contains(&query) {
                user.username = Some(format!("{}", user.id.to_string().split_at(8).0));
                user.avatar_url = Some(format!("https://api.dicebear.com/7.x/avataaars/svg?seed={}", user.name));
                user.organization_id = current_org;
                if !existing_ids.contains(&user.id) && !results.iter().any(|u| u.email == user.email) {
                    results.push(user);
                }
            }
        }

        results
    });

    view! {
        <div class="home-screen">
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Find Team Member"</span>
                </div>
                <div class="form-group">
                    <label class="form-label">"Search"</label>
                    <input
                        class="form-input"
                        type="text"
                        placeholder="Search by name, email, or username"
                        prop:value={move || search_query.get()}
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />
                </div>
                {move || {
                    let results = search_results.get();
                    if results.is_empty() {
                        if search_query.get().trim().len() >= 2 {
                            view! { <div class="list-item"><div class="list-item-left"><div class="list-item-subtitle">"No matching users found"</div></div></div> }.into_any()
                        } else {
                            ().into_any()
                        }
                    } else {
                        view! {
                            <div>
                                <div class="net-filter-label">"Results from local + server"</div>
                                {results.into_iter().map(|u| {
                                    let name = u.name.clone();
                                    let email = u.email.clone();
                                    let username = u.username.clone();
                                    let role = format!("{:?}", u.role);
                                    view! {
                                        <div class="list-item">
                                            <div class="list-item-left">
                                                <div class="list-item-title">{name.clone()}</div>
                                                <div class="list-item-subtitle">{format!("{} • {}", email.clone(), role)}</div>
                                            </div>
                                            <div class="list-item-right">
                                                <button class="net-action-btn" on:click=move |_| on_add_found(name.clone(), email.clone(), username.clone())>"Add"</button>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Add Manually"</span>
                </div>
                <div class="form-group">
                    <label class="form-label">"Name"</label>
                    <input
                        class="form-input"
                        type="text"
                        placeholder="Full name"
                        prop:value=new_name
                        on:input=move |ev| set_new_name.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label class="form-label">"Username"</label>
                    <input
                        class="form-input"
                        type="text"
                        placeholder="Username"
                        prop:value=new_username
                        on:input=move |ev| set_new_username.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label class="form-label">"Email"</label>
                    <input
                        class="form-input"
                        type="email"
                        placeholder="Email address"
                        prop:value=new_email
                        on:input=move |ev| set_new_email.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label class="form-label">"Role"</label>
                    <select
                        class="form-select"
                        prop:value={move || format!("{:?}", new_role.get())}
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_new_role.set(match value.as_str() {
                                "Owner" => UserRole::Owner,
                                "Director" => UserRole::Director,
                                "SeniorManager" => UserRole::SeniorManager,
                                "Manager" => UserRole::Manager,
                                "Worker" => UserRole::Worker,
                                "DocumentWorker" => UserRole::DocumentWorker,
                                "Contractor" => UserRole::Contractor,
                                _ => UserRole::Guest,
                            });
                        }
                    >
                        <option value="Owner">"Owner"</option>
                        <option value="Director">"Director"</option>
                        <option value="SeniorManager">"Senior Manager"</option>
                        <option value="Manager">"Manager"</option>
                        <option value="Worker">"Worker"</option>
                        <option value="DocumentWorker">"Document Worker"</option>
                        <option value="Contractor">"Contractor"</option>
                        <option value="Guest">"Guest"</option>
                    </select>
                </div>
                <button class="card-btn" on:click=on_add_user>"Add Member"</button>
            </div>
        </div>
    }
}

fn default_mock_users() -> Vec<User> {
    let org_id = Uuid::new_v4();
    vec![
        User {
            id: Uuid::new_v4(),
            name: "John Smith".to_string(),
            username: Some("jsmith".to_string()),
            email: "john@company.com".to_string(),
            role: UserRole::Owner,
            organization_id: Some(org_id),
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=John".to_string()),
            assignments: vec![UserAssignment {
                target_type: "Portfolio".to_string(),
                target_id: Uuid::new_v4(),
                target_name: "Downtown Properties".to_string(),
                assigned_at: Utc::now(),
                duration_days: Some(365),
                reason: Some("Property oversight".to_string()),
            }],
            activity_log: vec![UserActivity {
                action: "Created".to_string(),
                target_type: "Asset".to_string(),
                target_name: "Downtown Office".to_string(),
                timestamp: Utc::now(),
                reason: Some("New acquisition".to_string()),
            }],
            department: Some("Executive".to_string()),
            phone: Some("+1-555-0100".to_string()),
            address: Some("123 Main St".to_string()),
            hire_date: Some(Utc::now()),
            base_salary: Some(200000.0),
            payment_settings: PaymentSettings {
                payment_method: PaymentMethod::BankTransfer,
                account_details: "****1234".to_string(),
                payment_interval: PaymentInterval::Monthly,
                currency: crate::types::Currency::USD,
                automatic_payout: true,
                payout_threshold: None,
            },
            notification_preferences: vec![],
            permissions: default_permissions_for_role(&UserRole::Owner),
            documents: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: Some(Utc::now()),
            is_active: true,
        },
        User {
            id: Uuid::new_v4(),
            name: "Sarah Johnson".to_string(),
            username: Some("sjohnson".to_string()),
            email: "sarah@company.com".to_string(),
            role: UserRole::Manager,
            organization_id: Some(org_id),
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=Sarah".to_string()),
            assignments: vec![UserAssignment {
                target_type: "Asset Group".to_string(),
                target_id: Uuid::new_v4(),
                target_name: "Fleet Vehicles".to_string(),
                assigned_at: Utc::now(),
                duration_days: Some(180),
                reason: Some("Fleet coordinator".to_string()),
            }],
            activity_log: vec![UserActivity {
                action: "Modified".to_string(),
                target_type: "Asset".to_string(),
                target_name: "Fleet Van #3".to_string(),
                timestamp: Utc::now(),
                reason: Some("Value update".to_string()),
            }],
            department: Some("Operations".to_string()),
            phone: Some("+1-555-0101".to_string()),
            address: None,
            hire_date: Some(Utc::now()),
            base_salary: Some(120000.0),
            payment_settings: PaymentSettings {
                payment_method: PaymentMethod::DirectDeposit,
                account_details: "****5678".to_string(),
                payment_interval: PaymentInterval::BiWeekly,
                currency: crate::types::Currency::USD,
                automatic_payout: true,
                payout_threshold: None,
            },
            notification_preferences: vec![],
            permissions: default_permissions_for_role(&UserRole::Manager),
            documents: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: Some(Utc::now()),
            is_active: true,
        },
        User {
            id: Uuid::new_v4(),
            name: "Mike Williams".to_string(),
            username: Some("mwilliams".to_string()),
            email: "mike@company.com".to_string(),
            role: UserRole::Worker,
            organization_id: Some(org_id),
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=Mike".to_string()),
            assignments: vec![UserAssignment {
                target_type: "Asset".to_string(),
                target_id: Uuid::new_v4(),
                target_name: "Warehouse A".to_string(),
                assigned_at: Utc::now(),
                duration_days: Some(90),
                reason: Some("Maintenance rotation".to_string()),
            }],
            activity_log: vec![UserActivity {
                action: "Updated".to_string(),
                target_type: "Task".to_string(),
                target_name: "Roof repair".to_string(),
                timestamp: Utc::now(),
                reason: Some("Routine maintenance".to_string()),
            }],
            department: Some("Field Operations".to_string()),
            phone: Some("+1-555-0102".to_string()),
            address: None,
            hire_date: Some(Utc::now()),
            base_salary: Some(65000.0),
            payment_settings: PaymentSettings {
                payment_method: PaymentMethod::BankTransfer,
                account_details: "****9012".to_string(),
                payment_interval: PaymentInterval::Weekly,
                currency: crate::types::Currency::USD,
                automatic_payout: true,
                payout_threshold: None,
            },
            notification_preferences: vec![],
            permissions: default_permissions_for_role(&UserRole::Worker),
            documents: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: Some(Utc::now()),
            is_active: true,
        },
        User {
            id: Uuid::new_v4(),
            name: "Guest User".to_string(),
            username: Some("guest".to_string()),
            email: "guest@company.com".to_string(),
            role: UserRole::Guest,
            organization_id: Some(org_id),
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=Guest".to_string()),
            assignments: vec![],
            activity_log: vec![],
            department: Some("External".to_string()),
            phone: None,
            address: None,
            hire_date: None,
            base_salary: None,
            payment_settings: PaymentSettings::default(),
            notification_preferences: vec![],
            permissions: default_permissions_for_role(&UserRole::Guest),
            documents: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
            is_active: true,
        },
    ]
}
