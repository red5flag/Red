use crate::components::rule_engine::RuleEngine;
use crate::models::{OrgRole, Organization, Perm, PermGroup, Permission, RoleScope, User};
use crate::stores::use_app_store;
use crate::types::UserRole;
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

fn permission_label(p: &Permission) -> &'static str {
    match p {
        Permission::ViewOwn => "View own",
        Permission::ViewOrganization => "View organization",
        Permission::ViewAll => "View all",
        Permission::CreateOwn => "Create own",
        Permission::CreateOrganization => "Create organization",
        Permission::EditOwn => "Edit own",
        Permission::EditOrganization => "Edit organization",
        Permission::EditAll => "Edit all",
        Permission::DeleteOwn => "Delete own",
        Permission::DeleteOrganization => "Delete organization",
        Permission::DeleteAll => "Delete all",
        Permission::ManageUsers => "Manage users",
        Permission::ManageRoles => "Manage roles",
        Permission::ManagePayments => "Manage payments",
        Permission::ManageSettings => "Manage settings",
        Permission::ExportData => "Export data",
        Permission::ImportData => "Import data",
        Permission::EditDocuments => "Edit documents",
        Permission::Custom(s) => return Box::leak(format!("Custom: {}", s).into_boxed_str()),
    }
}

fn role_from_str(s: &str) -> UserRole {
    match s {
        "Owner" => UserRole::Owner,
        "Director" => UserRole::Director,
        "SeniorManager" => UserRole::SeniorManager,
        "Manager" => UserRole::Manager,
        "Worker" => UserRole::Worker,
        "DocumentWorker" => UserRole::DocumentWorker,
        "Contractor" => UserRole::Contractor,
        _ => UserRole::Guest,
    }
}

fn role_display(role: &UserRole) -> &'static str {
    match role {
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

fn scope_from_str(s: &str) -> RoleScope {
    match s {
        "EntireOrganization" => RoleScope::EntireOrganization,
        "ReportingOnly" => RoleScope::ReportingOnly,
        "CalendarOnly" => RoleScope::CalendarOnly,
        "TransactionsOnly" => RoleScope::TransactionsOnly,
        "NetworkingOnly" => RoleScope::NetworkingOnly,
        "HistoryOnly" => RoleScope::HistoryOnly,
        _ => RoleScope::EntireOrganization,
    }
}

fn scope_display(s: &RoleScope) -> &'static str {
    s.display()
}

#[component]
pub fn OrganizationPage() -> impl IntoView {
    let app_store = use_app_store();

    let organizations = Memo::new(move |_| app_store.get().organizations.clone());

    let can_manage_in_org = move |org_id: Uuid| {
        let role = app_store.get().current_user_role_in_org(org_id);
        matches!(role, UserRole::Owner | UserRole::Director | UserRole::SeniorManager | UserRole::Manager)
    };

    // Expanded top-level org rows
    let (expanded_orgs, set_expanded_orgs) = signal(HashSet::<Uuid>::new());
    let toggle_org = move |oid: Uuid| {
        set_expanded_orgs.update(|s| { if !s.remove(&oid) { s.insert(oid); } });
    };

    // Active sub-tab per org: (org_id, "portfolios"|"roles"|"members")
    let (org_active_tab, set_org_active_tab) = signal(std::collections::HashMap::<Uuid, &'static str>::new());
    let get_org_tab = move |oid: Uuid| org_active_tab.get().get(&oid).copied().unwrap_or("portfolios");
    let set_org_tab = move |oid: Uuid, tab: &'static str| {
        set_org_active_tab.update(|m| { m.insert(oid, tab); });
    };

    // Expanded role cards: (org_id, role_id)
    let (expanded_roles, set_expanded_roles) = signal(HashSet::<(Uuid, Uuid)>::new());
    let toggle_role = move |oid: Uuid, rid: Uuid| {
        set_expanded_roles.update(|s| { if !s.remove(&(oid, rid)) { s.insert((oid, rid)); } });
    };

    // Expanded permission groups within a role: (org_id, role_id, group_index)
    let (expanded_perm_groups, set_expanded_perm_groups) = signal(HashSet::<(Uuid, Uuid, usize)>::new());
    let toggle_perm_group = move |oid: Uuid, rid: Uuid, gi: usize| {
        set_expanded_perm_groups.update(|s| { if !s.remove(&(oid, rid, gi)) { s.insert((oid, rid, gi)); } });
    };

    // Expanded member rows: (org_id, user_id)
    let (expanded_members, set_expanded_members) = signal(HashSet::<(Uuid, Uuid)>::new());
    let toggle_member = move |oid: Uuid, uid: Uuid| {
        set_expanded_members.update(|s| { if !s.remove(&(oid, uid)) { s.insert((oid, uid)); } });
    };

    // Role editor modal: (org_id, role_id) — None means closed, Some with new Uuid::nil() means create new
    let (editing_role, set_editing_role) = signal(Option::<(Uuid, Uuid)>::None);
    let (edit_role_name, set_edit_role_name) = signal(String::new());
    let (edit_role_desc, set_edit_role_desc) = signal(String::new());
    let (edit_role_rank, set_edit_role_rank) = signal(50u32);
    let (edit_role_color, set_edit_role_color) = signal(String::new());
    let (edit_role_scope, set_edit_role_scope) = signal(RoleScope::EntireOrganization);

    // Add org form
    let (show_add_org, set_show_add_org) = signal(false);
    let (new_org_name, set_new_org_name) = signal(String::new());
    let (new_org_desc, set_new_org_desc) = signal(String::new());

    // Add member form
    let (show_add_member, set_show_add_member) = signal(Option::<Uuid>::None);
    let (member_name, set_member_name) = signal(String::new());
    let (member_email, set_member_email) = signal(String::new());
    let (member_role, set_member_role) = signal(UserRole::Worker);

    // Edit org inline
    let (editing_org, set_editing_org) = signal(Option::<Uuid>::None);
    let (edit_name, set_edit_name) = signal(String::new());
    let (edit_desc, set_edit_desc) = signal(String::new());
    let (edit_color, set_edit_color) = signal(String::new());

    // Context menu for organizations: (x, y, org_id)
    let (context_menu, set_context_menu) = signal(Option::<(i32, i32, Uuid)>::None);
    // Context menu for roles: (x, y, org_id, role_id)
    let (role_context_menu, set_role_context_menu) = signal(Option::<(i32, i32, Uuid, Uuid)>::None);

    let on_add_org = move |_| {
        let name = new_org_name.get();
        if name.trim().is_empty() { return; }
        let owner_id = app_store.get().current_user.id;
        let mut org = Organization::new(name, owner_id);
        org.description = if new_org_desc.get().trim().is_empty() { None } else { Some(new_org_desc.get()) };
        app_store.update(|s| s.add_organization(org));
        set_new_org_name.set(String::new());
        set_new_org_desc.set(String::new());
        set_show_add_org.set(false);
    };

    let on_delete_org = move |id: Uuid| {
        app_store.update(|s| { s.remove_organization(id); });
    };

    let on_start_edit = move |id: Uuid, name: String, desc: Option<String>, color: Option<String>| {
        set_edit_name.set(name);
        set_edit_desc.set(desc.unwrap_or_default());
        set_edit_color.set(color.unwrap_or_default());
        set_editing_org.set(Some(id));
    };

    let on_save_edit = move |id: Uuid| {
        let name = edit_name.get();
        if name.trim().is_empty() { return; }
        let color = edit_color.get();
        app_store.update(|s| {
            if let Some(org) = s.get_organization_mut(id) {
                org.name = name;
                org.description = if edit_desc.get().trim().is_empty() { None } else { Some(edit_desc.get()) };
                org.settings.color = if color.trim().is_empty() { None } else { Some(color) };
                org.updated_at = chrono::Utc::now();
            }
        });
        set_editing_org.set(None);
    };

    let on_add_member = move |org_id: Uuid| {
        let name = member_name.get();
        let email = member_email.get();
        if name.trim().is_empty() || email.trim().is_empty() { return; }
        let mut user = User::new(name, email, member_role.get());
        user.organization_id = Some(org_id);
        app_store.update(|s| {
            s.add_organization_user(user.clone());
            if let Some(org) = s.get_organization_mut(org_id) { org.add_member(user.id); }
        });
        set_member_name.set(String::new());
        set_member_email.set(String::new());
        set_member_role.set(UserRole::Worker);
        set_show_add_member.set(None);
    };

    let on_remove_member = move |org_id: Uuid, user_id: Uuid| {
        app_store.update(|s| {
            s.remove_organization_user(user_id);
            if let Some(org) = s.get_organization_mut(org_id) { org.remove_member(user_id); }
        });
    };

    let on_update_member_role = move |user_id: Uuid, new_role: UserRole| {
        app_store.update(|s| { let _ = s.update_user_role(user_id, new_role); });
    };

    let close_context_menu = move || set_context_menu.set(None);
    let close_role_context_menu = move || set_role_context_menu.set(None);

    let on_start_role_edit = move |oid: Uuid, role: &OrgRole| {
        set_edit_role_name.set(role.name.clone());
        set_edit_role_desc.set(role.description.clone());
        set_edit_role_rank.set(role.rank);
        set_edit_role_color.set(role.color.clone().unwrap_or_default());
        set_edit_role_scope.set(role.scope.clone());
        set_editing_role.set(Some((oid, role.id)));
    };

    let on_start_new_role = move |oid: Uuid| {
        set_edit_role_name.set(String::new());
        set_edit_role_desc.set(String::new());
        set_edit_role_rank.set(50);
        set_edit_role_color.set(String::new());
        set_edit_role_scope.set(RoleScope::EntireOrganization);
        set_editing_role.set(Some((oid, Uuid::nil())));
    };

    let on_save_role = move |_| {
        if let Some((oid, rid)) = editing_role.get() {
            let name = edit_role_name.get();
            if name.trim().is_empty() { return; }
            let desc = edit_role_desc.get();
            let rank = edit_role_rank.get();
            let color = edit_role_color.get();
            let scope = edit_role_scope.get();
            let color_opt = if color.trim().is_empty() { None } else { Some(color) };
            if rid == Uuid::nil() {
                let new_role = OrgRole::new(name, rank, desc, vec![]);
                app_store.update(|s| {
                    let mut r = new_role;
                    r.color = color_opt;
                    r.scope = scope;
                    s.add_role_to_org(oid, r);
                });
            } else {
                app_store.update(|s| s.update_org_role(oid, rid, name, desc, color_opt, rank, scope));
            }
            set_editing_role.set(None);
        }
    };

    let on_delete_role = move |oid: Uuid, rid: Uuid| {
        app_store.update(|s| s.delete_org_role(oid, rid));
    };

    let on_duplicate_role = move |oid: Uuid, rid: Uuid| {
        app_store.update(|s| { let _ = s.duplicate_org_role(oid, rid); });
    };

    let on_toggle_role_perm = move |oid: Uuid, rid: Uuid, perm: Perm| {
        app_store.update(|s| s.toggle_role_permission(oid, rid, perm));
    };

    let on_assign_role_member = move |oid: Uuid, rid: Uuid, uid: Uuid| {
        app_store.update(|s| s.assign_member_to_role(oid, rid, uid));
    };

    let on_remove_role_member = move |oid: Uuid, rid: Uuid, uid: Uuid| {
        app_store.update(|s| s.remove_member_from_role(oid, rid, uid));
    };

    view! {
        <div class="home-screen">

            // Add Organization button (blind mode)
            {move || {
                let blind = app_store.get().blind_mode;
                let any_manage = app_store.get().organizations.iter().any(|o| can_manage_in_org(o.id));
                if any_manage && blind {
                    view! {
                        <div class="view-toggle">
                            <button class="view-btn" class:active=show_add_org
                                on:click=move |_| set_show_add_org.update(|v| *v = !*v)>
                                "+ Add Organization"
                            </button>
                        </div>
                    }.into_any()
                } else { ().into_any() }
            }}

            {move || show_add_org.get().then(|| view! {
                <div class="add-form">
                    <input class="login-input" type="text" placeholder="Organization name"
                        on:input=move |ev| set_new_org_name.set(event_target_value(&ev)) />
                    <input class="login-input" type="text" placeholder="Description (optional)"
                        on:input=move |ev| set_new_org_desc.set(event_target_value(&ev)) />
                    <button class="login-btn" on:click=on_add_org>"Create Organization"</button>
                </div>
            })}

            // Organization list
            {move || {
                let orgs = organizations.get();
                let editing = editing_org.get();
                if orgs.is_empty() {
                    view! {
                        <div class="data-card">
                            <div class="empty-state"><div class="empty-text">"No organizations yet."</div></div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="pf-accordion">
                        {orgs.into_iter().map(|org| {
                            let oid = org.id;
                            let can = can_manage_in_org(oid);
                            let blind = app_store.get().blind_mode;
                            let is_editing = editing == Some(oid);
                            let is_exp = move || expanded_orgs.get().contains(&oid);

                            let member_count = org.members.len();
                            let portfolio_count = app_store.get().portfolios.iter()
                                .filter(|p| p.organization_id == Some(oid)).count();
                            let role_count = org.roles.len();
                            let owner_name = app_store.get().organization_users.iter()
                                .find(|u| u.id == org.owner_id)
                                .map(|u| u.name.clone())
                                .unwrap_or_else(|| "Unknown".to_string());
                            let locked_doc_count: usize = app_store.get().portfolios.iter()
                                .filter(|p| p.organization_id == Some(oid))
                                .map(|p| p.documents.len())
                                .sum();

                            let org_name = org.name.clone();
                            let org_desc = org.description.clone();
                            let org_color = org.settings.color.clone();
                            let blind_btn_name = org.name.clone();
                            let blind_btn_desc = org.description.clone();
                            let blind_btn_color = org.settings.color.clone();
                            let color_style = org_color.as_ref()
                                .map(|c| format!("border-left: 4px solid {};", c))
                                .unwrap_or_default();

                            view! {
                                <div class="asset-group" class:expanded=is_exp
                                >
                                    // Header
                                    <div class="asset-group-header" style={color_style}
                                        on:click=move |_| { if !is_editing { toggle_org(oid); } }
                                        on:dblclick=move |ev: leptos::ev::MouseEvent| {
                                            if can && !is_editing {
                                                ev.stop_propagation();
                                                on_start_edit(oid, org_name.clone(), org_desc.clone(), org_color.clone());
                                            }
                                        }
                                    >
                                        <span class="asset-group-arrow">
                                            {move || if is_exp() { "\u{25B2}" } else { "\u{25BC}" }}
                                        </span>
                                        <div class="asset-group-icon">"\u{1F3E2}"</div>
                                        <div class="asset-group-info-wrap" on:click=|ev| ev.stop_propagation()>
                                            {move || if is_editing {
                                                view! {
                                                    <div class="asset-group-edit-form">
                                                        <input class="pf-edit-input" placeholder="Organization name"
                                                            prop:value=move || edit_name.get()
                                                            on:input=move |ev| set_edit_name.set(event_target_value(&ev)) />
                                                        <input class="pf-edit-input" placeholder="Description"
                                                            prop:value=move || edit_desc.get()
                                                            on:input=move |ev| set_edit_desc.set(event_target_value(&ev)) />
                                                        <div class="org-color-row">
                                                            <span class="org-color-label">"Accent"</span>
                                                            <input class="org-color-input" type="color"
                                                                prop:value=move || edit_color.get()
                                                                on:input=move |ev| set_edit_color.set(event_target_value(&ev)) />
                                                        </div>
                                                        <div style="display:flex;gap:6px;margin-top:4px;">
                                                            <button class="login-btn" style="flex:1;" on:click=move |_| on_save_edit(oid)>"Save"</button>
                                                            <button class="view-btn" style="flex:1;" on:click=move |_| set_editing_org.set(None)>"Cancel"</button>
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div
                                                        on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                                                            if can && !blind {
                                                                ev.prevent_default();
                                                                ev.stop_propagation();
                                                                set_context_menu.set(Some((ev.client_x(), ev.client_y(), oid)));
                                                            }
                                                        }
                                                    >
                                                        <div class="asset-group-name">{org.name.clone()}</div>
                                                        {org.description.as_ref().map(|d| view! {
                                                            <div class="asset-group-desc">{d.clone()}</div>
                                                        })}
                                                        <div class="asset-group-count">
                                                            {format!("{} portfolios \u{00B7} {} members \u{00B7} {} roles", portfolio_count, member_count, role_count)}
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            }}
                                        </div>
                                        {if blind && can && !is_editing {
                                            view! {
                                                <div class="pf-list-actions" on:click=|ev| ev.stop_propagation()>
                                                    <button class="pf-action-btn"
                                                        on:click=move |_| on_start_edit(oid, blind_btn_name.clone(), blind_btn_desc.clone(), blind_btn_color.clone())>
                                                        "\u{270E}"
                                                    </button>
                                                    <button class="pf-action-btn"
                                                        on:click=move |_| on_delete_org(oid)>
                                                        "\u{1F5D1}"
                                                    </button>
                                                </div>
                                            }.into_any()
                                        } else { ().into_any() }}
                                    </div>

                                    // Expanded content
                                    <div class="asset-group-content" class:hidden={move || !is_exp()}>

                                        // ── Organization Overview ────────────────────────
                                        <div class="org-overview">
                                            <div class="org-overview-row">
                                                <span class="org-overview-label">"Owner"</span>
                                                <span class="org-overview-value">{owner_name.clone()}</span>
                                            </div>
                                            <div class="org-overview-row">
                                                <span class="org-overview-label">"Portfolios"</span>
                                                <span class="org-overview-value">{portfolio_count.to_string()}</span>
                                            </div>
                                            <div class="org-overview-row">
                                                <span class="org-overview-label">"Members"</span>
                                                <span class="org-overview-value">{member_count.to_string()}</span>
                                            </div>
                                            <div class="org-overview-row">
                                                <span class="org-overview-label">"Roles"</span>
                                                <span class="org-overview-value">{role_count.to_string()}</span>
                                            </div>
                                            <div class="org-overview-row">
                                                <span class="org-overview-label">"Documents"</span>
                                                <span class="org-overview-value">{locked_doc_count.to_string()}</span>
                                            </div>
                                        </div>

                                        // ── Sub-tab bar ──────────────────────────────────
                                        <div class="org-sub-tabs">
                                            <button class="org-sub-tab" class:active={move || get_org_tab(oid) == "portfolios"}
                                                on:click=move |_| set_org_tab(oid, "portfolios")>
                                                <span>"Portfolios ("</span>
                                                {move || app_store.get().portfolios.iter()
                                                    .filter(|p| p.organization_id == Some(oid)).count().to_string()}
                                                <span>")"</span>
                                            </button>
                                            <button class="org-sub-tab" class:active={move || get_org_tab(oid) == "roles"}
                                                on:click=move |_| set_org_tab(oid, "roles")>
                                                {format!("Roles ({})", role_count)}
                                            </button>
                                            <button class="org-sub-tab" class:active={move || get_org_tab(oid) == "members"}
                                                on:click=move |_| set_org_tab(oid, "members")>
                                                {format!("Members ({})", member_count)}
                                            </button>
                                        </div>

                                        // ── 1. Portfolios tab ────────────────────────────
                                        <div class="org-sub-tab-content" class:hidden={move || get_org_tab(oid) != "portfolios"}>
                                            {move || {
                                                let ps: Vec<_> = app_store.get().portfolios.iter()
                                                    .filter(|p| p.organization_id == Some(oid))
                                                    .cloned().collect();
                                                if ps.is_empty() {
                                                    view! { <div class="empty-state"><div class="empty-text">"No portfolios."</div></div> }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="asset-list">
                                                        {ps.into_iter().enumerate().map(|(idx, p)| {
                                                            let total = p.total_value;
                                                            let direct_count = p.assets.len();
                                                            let group_count = p.asset_groups.len();
                                                            let asset_count = direct_count + p.asset_groups.iter().map(|g| g.assets.len()).sum::<usize>();
                                                            let doc_count = p.documents.len();
                                                            let assigned_count = p.assigned_users.len();
                                                            let tint = format!("background: rgba(255,255,255,{:.1});", (idx as f64 * 0.07).min(0.4));
                                                            view! {
                                                                <div class="asset-item org-portfolio-card" style={tint}>
                                                                    <div class="asset-icon">"\u{1F4C1}"</div>
                                                                    <div class="asset-info">
                                                                        <div class="asset-name">{p.name.clone()}</div>
                                                                        {p.description.as_ref().map(|d| view! { <div class="asset-desc">{d.clone()}</div> })}
                                                                        <div class="asset-subtext">
                                                                            {format!("{} assets \u{00B7} {} direct \u{00B7} {} groups \u{00B7} {} docs \u{00B7} {} members",
                                                                                asset_count, direct_count, group_count, doc_count, assigned_count)}
                                                                        </div>
                                                                    </div>
                                                                    <div class="asset-value" style="color:var(--success);">
                                                                        {format!("${:.0}", total)}
                                                                    </div>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                        </div>
                                                    }.into_any()
                                                }
                                            }}
                                        </div>

                                        // ── 2. Roles tab ─────────────────────────────────
                                        <div class="org-sub-tab-content" class:hidden={move || get_org_tab(oid) != "roles"}>
                                            <div class="org-sub-tab-header">
                                                <span class="org-sub-tab-title">"Roles"</span>
                                                {if can {
                                                    view! {
                                                        <button class="add-btn-small"
                                                            on:click=move |_| on_start_new_role(oid)>
                                                            "+ Role"
                                                        </button>
                                                    }.into_any()
                                                } else { ().into_any() }}
                                            </div>
                                            {move || {
                                                let roles = app_store.get().organizations.iter()
                                                    .find(|o| o.id == oid)
                                                    .map(|o| o.roles.clone())
                                                    .unwrap_or_default();
                                                let mut sorted_roles = roles.clone();
                                                sorted_roles.sort_by(|a, b| b.rank.cmp(&a.rank));

                                                view! {
                                                    <div class="org-role-list">
                                                    {sorted_roles.into_iter().enumerate().map(|(ridx, role)| {
                                                        let rid = role.id;
                                                        let role_exp = move || expanded_roles.get().contains(&(oid, rid));
                                                        let rtint = format!("background: rgba(255,255,255,{:.1});", (ridx as f64 * 0.04).min(0.3));
                                                        let role_color_style = role.color.as_ref()
                                                            .map(|c| format!("border-left: 4px solid {};", c))
                                                            .unwrap_or_default();
                                                        let role_summary = role.summary();
                                                        let role_name_for_edit = role.name.clone();
                                                        let role_desc_for_edit = role.description.clone();
                                                        let role_color_for_edit = role.color.clone();
                                                        let role_rank_for_edit = role.rank;
                                                        let role_scope_for_edit = role.scope.clone();
                                                        let role_perms = role.permissions.clone();
                                                        let role_members_ids = role.member_ids.clone();
                                                        let is_system = role.is_system;
                                                        let role_name_for_dup = role.name.clone();
                                                        // Clones for the Edit button closure (which moves captures)
                                                        let edit_name = role_name_for_edit.clone();
                                                        let edit_desc = role_desc_for_edit.clone();
                                                        let edit_color = role_color_for_edit.clone();
                                                        let edit_scope = role_scope_for_edit.clone();
                                                        let edit_perms = role_perms.clone();
                                                        let edit_members = role_members_ids.clone();
                                                        let edit_name_dup = role_name_for_edit.clone();

                                                        view! {
                                                            <div class="org-role-card" class:expanded=role_exp style={format!("{}{}", rtint, role_color_style)}
                                                                role="region"
                                                                aria-label={role_summary.clone()}>
                                                                // Role header
                                                                <div class="org-role-header"
                                                                    on:click=move |_| toggle_role(oid, rid)>
                                                                    <span class="org-role-arrow">
                                                                        {move || if role_exp() { "\u{25B2}" } else { "\u{25BC}" }}
                                                                    </span>
                                                                    <div class="org-role-info"
                                                                        on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                                                                            if can {
                                                                                ev.prevent_default();
                                                                                ev.stop_propagation();
                                                                                set_role_context_menu.set(Some((ev.client_x(), ev.client_y(), oid, rid)));
                                                                            }
                                                                        }
                                                                    >
                                                                        <div class="org-role-name">{role_name_for_dup.clone()}</div>
                                                                        <div class="org-role-meta">
                                                                            {format!("Rank {} \u{00B7} {} \u{00B7} {} members",
                                                                                role_rank_for_edit,
                                                                                scope_display(&role_scope_for_edit),
                                                                                role_members_ids.len())}
                                                                        </div>
                                                                    </div>
                                                                    {if can {
                                                                        view! {
                                                                            <div class="org-role-actions" on:click=|ev| ev.stop_propagation()>
                                                                                <button class="org-role-btn"
                                                                                    aria-label={format!("Edit {} role", edit_name)}
                                                                                    on:click=move |_| {
                                                                                        on_start_role_edit(oid, &OrgRole {
                                                                                            id: rid,
                                                                                            name: edit_name.clone(),
                                                                                            rank: role_rank_for_edit,
                                                                                            color: edit_color.clone(),
                                                                                            description: edit_desc.clone(),
                                                                                            scope: edit_scope.clone(),
                                                                                            permissions: edit_perms.clone(),
                                                                                            member_ids: edit_members.clone(),
                                                                                            documents: Vec::new(),
                                                                                            is_system,
                                                                                        });
                                                                                    }>
                                                                                    "Edit"
                                                                                </button>
                                                                                <button class="org-role-btn"
                                                                                    aria-label={format!("Duplicate {} role", edit_name_dup)}
                                                                                    on:click=move |_| on_duplicate_role(oid, rid)>
                                                                                    "Duplicate"
                                                                                </button>
                                                                                {if !is_system {
                                                                                    view! {
                                                                                        <button class="org-role-btn org-role-btn-danger"
                                                                                            aria-label={format!("Delete {} role", role_name_for_edit)}
                                                                                            on:click=move |_| on_delete_role(oid, rid)>
                                                                                            "Delete"
                                                                                        </button>
                                                                                    }.into_any()
                                                                                } else { ().into_any() }}
                                                                            </div>
                                                                        }.into_any()
                                                                    } else { ().into_any() }}
                                                                </div>

                                                                // Role expanded content
                                                                <div class="org-role-content" class:hidden={move || !role_exp()}>
                                                                    // Plain-English summary
                                                                    <div class="org-role-summary">
                                                                        <div class="org-role-summary-label">"Summary"</div>
                                                                        <div class="org-role-summary-text">{role_desc_for_edit.clone()}</div>
                                                                    </div>

                                                                    // Permission groups
                                                                    <div class="org-perm-groups">
                                                                        {PermGroup::all().iter().enumerate().map(|(gi, group)| {
                                                                            let group_exp = move || expanded_perm_groups.get().contains(&(oid, rid, gi));
                                                                            let group_label = group.label();
                                                                            let group_perms = Perm::for_group(group);
                                                                            let current_perms = role_perms.clone();

                                                                            view! {
                                                                                <div class="org-perm-group" class:expanded=group_exp>
                                                                                    <div class="org-perm-group-header"
                                                                                        on:click=move |_| toggle_perm_group(oid, rid, gi)>
                                                                                        <span class="asset-section-arrow">
                                                                                            {move || if group_exp() { "\u{25BC}" } else { "\u{25B6}" }}
                                                                                        </span>
                                                                                        <span class="org-perm-group-label">{group_label}</span>
                                                                                        <span class="org-perm-group-count">
                                                                                            {format!("{}/{}",
                                                                                                current_perms.iter().filter(|p| p.group() == *group).count(),
                                                                                                group_perms.len())}
                                                                                        </span>
                                                                                    </div>
                                                                                    <div class="org-perm-group-body" class:hidden={move || !group_exp()}>
                                                                                        {group_perms.into_iter().map(|perm| {
                                                                                            let has = current_perms.contains(&perm);
                                                                                            let label = perm.label();
                                                                                            let p2 = perm.clone();
                                                                                            view! {
                                                                                                <label class="org-perm-toggle">
                                                                                                    <input type="checkbox" checked={has}
                                                                                                        disabled={!can}
                                                                                                        on:change=move |_| {
                                                                                                            if can {
                                                                                                                on_toggle_role_perm(oid, rid, p2.clone());
                                                                                                            }
                                                                                                        } />
                                                                                                    <span class="org-perm-toggle-label">{label}</span>
                                                                                                </label>
                                                                                            }
                                                                                        }).collect::<Vec<_>>()}
                                                                                    </div>
                                                                                </div>
                                                                            }
                                                                        }).collect::<Vec<_>>()}
                                                                    </div>

                                                                    // Role members
                                                                    <div class="org-role-members-section">
                                                                        <div class="org-role-members-label">
                                                                            {format!("Members ({}):", role_members_ids.len())}
                                                                        </div>
                                                                        {if role_members_ids.is_empty() {
                                                                            view! { <div class="empty-state"><div class="empty-text">"No members assigned to this role."</div></div> }.into_any()
                                                                        } else {
                                                                            let all_users = app_store.get().organization_users.clone();
                                                                            let role_users: Vec<User> = all_users.iter()
                                                                                .filter(|u| role_members_ids.contains(&u.id))
                                                                                .cloned().collect();
                                                                            view! {
                                                                                <div class="org-role-members-list">
                                                                                {role_users.into_iter().map(|user| {
                                                                                    let uid = user.id;
                                                                                    let uname = user.name.clone();
                                                                                    view! {
                                                                                        <div class="org-role-member-chip">
                                                                                            <span class="org-member-avatar-sm">
                                                                                                {user.name.chars().next().unwrap_or('?').to_uppercase().to_string()}
                                                                                            </span>
                                                                                            <span>{uname}</span>
                                                                                            {if can {
                                                                                                view! {
                                                                                                    <button class="org-role-btn org-role-btn-sm org-role-btn-danger"
                                                                                                        aria-label={format!("Remove {} from role", user.name)}
                                                                                                        on:click=move |_| on_remove_role_member(oid, rid, uid)>
                                                                                                        "\u{2715}"
                                                                                                    </button>
                                                                                                }.into_any()
                                                                                            } else { ().into_any() }}
                                                                                        </div>
                                                                                    }
                                                                                }).collect::<Vec<_>>()}
                                                                                </div>
                                                                            }.into_any()
                                                                        }}

                                                                        // Add member to role
                                                                        {if can {
                                                                            let org_users = app_store.get().organization_users.iter()
                                                                                .filter(|u| app_store.get().organizations.iter()
                                                                                    .find(|o| o.id == oid)
                                                                                    .map(|o| o.members.contains(&u.id))
                                                                                    .unwrap_or(false))
                                                                                .filter(|u| !role_members_ids.contains(&u.id))
                                                                                .cloned().collect::<Vec<_>>();
                                                                            view! {
                                                                                <div class="org-role-add-member">
                                                                                    <select class="login-input org-role-member-select"
                                                                                        on:change=move |ev| {
                                                                                            let v = event_target_value(&ev);
                                                                                            if let Ok(uid) = Uuid::parse_str(&v) {
                                                                                                on_assign_role_member(oid, rid, uid);
                                                                                            }
                                                                                        }>
                                                                                        <option value="">"+ Assign member to role"</option>
                                                                                        {org_users.into_iter().map(|u| view! {
                                                                                            <option value={u.id.to_string()}>{u.name.clone()}</option>
                                                                                        }).collect::<Vec<_>>()}
                                                                                    </select>
                                                                                </div>
                                                                            }.into_any()
                                                                        } else { ().into_any() }}
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                    </div>
                                                }.into_any()
                                            }}

                                            // Role Rules & Notifications section
                                            {move || if get_org_tab(oid) == "roles" {
                                                view! {
                                                    <div class="org-rule-engine-wrap">
                                                        <RuleEngine org_id={oid} />
                                                    </div>
                                                }.into_any()
                                            } else { ().into_any() }}
                                        </div>

                                        // ── 3. Members tab ───────────────────────────────
                                        <div class="org-sub-tab-content" class:hidden={move || get_org_tab(oid) != "members"}>
                                            <div class="org-sub-tab-header">
                                                <span class="org-sub-tab-title">"Members"</span>
                                                {if can {
                                                    view! {
                                                        <button class="add-btn-small"
                                                            on:click=move |_| {
                                                                set_show_add_member.set(Some(oid));
                                                            }>
                                                            "+ Member"
                                                        </button>
                                                    }.into_any()
                                                } else { ().into_any() }}
                                            </div>

                                            // Add member inline form
                                            {move || show_add_member.get().filter(|&gp| gp == oid).map(|_| view! {
                                                <div class="add-form" style="margin:0;border-radius:0;border-left:none;border-right:none;">
                                                    <input class="login-input" type="text" placeholder="Name"
                                                        on:input=move |ev| set_member_name.set(event_target_value(&ev)) />
                                                    <input class="login-input" type="email" placeholder="Email"
                                                        on:input=move |ev| set_member_email.set(event_target_value(&ev)) />
                                                    <select class="login-input"
                                                        on:change=move |ev| set_member_role.set(role_from_str(&event_target_value(&ev)))>
                                                        <option value="Owner">"Owner"</option>
                                                        <option value="Director">"Director"</option>
                                                        <option value="SeniorManager">"Senior Manager"</option>
                                                        <option value="Manager">"Manager"</option>
                                                        <option value="Worker" selected=true>"Worker"</option>
                                                        <option value="DocumentWorker">"Document Worker"</option>
                                                        <option value="Contractor">"Contractor"</option>
                                                        <option value="Guest">"Guest"</option>
                                                    </select>
                                                    <div style="display:flex;gap:6px;">
                                                        <button class="login-btn" style="flex:1;" on:click=move |_| on_add_member(oid)>"Add"</button>
                                                        <button class="view-btn" style="flex:1;" on:click=move |_| set_show_add_member.set(None)>"Cancel"</button>
                                                    </div>
                                                </div>
                                            })}

                                            {move || (get_org_tab(oid) == "members").then(|| {
                                                let all_members: Vec<User> = app_store.get().organization_users.iter()
                                                    .filter(|u| {
                                                        app_store.get().organizations.iter()
                                                            .find(|o| o.id == oid)
                                                            .map(|o| o.members.contains(&u.id))
                                                            .unwrap_or(false)
                                                    })
                                                    .cloned().collect();

                                                let org_roles = app_store.get().organizations.iter()
                                                    .find(|o| o.id == oid)
                                                    .map(|o| o.roles.clone())
                                                    .unwrap_or_default();

                                                if all_members.is_empty() {
                                                    view! { <div class="empty-state"><div class="empty-text">"No members."</div></div> }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="org-member-list">
                                                        {all_members.into_iter().enumerate().map(|(uidx, user)| {
                                                            let uid = user.id;
                                                            let uid_del = user.id;
                                                            let uid_role = user.id;
                                                            let user_exp = move || expanded_members.get().contains(&(oid, uid));
                                                            let utint = format!("background: rgba(255,255,255,{:.1});", (uidx as f64 * 0.04).min(0.25));
                                                            let user_role = user.role.clone();
                                                            let user_name = user.name.clone();
                                                            let user_email = user.email.clone();
                                                            let user_perms = user.permissions.clone();
                                                            let user_role_display = role_display(&user.role);
                                                            let user_address = user.address.clone();
                                                            let user_department = user.department.clone();
                                                            let user_phone = user.phone.clone();
                                                            let user_assignments = user.assignments.clone();
                                                            let user_active = user.is_active;

                                                            // Find roles this user is assigned to
                                                            let user_role_names: Vec<String> = org_roles.iter()
                                                                .filter(|r| r.member_ids.contains(&uid))
                                                                .map(|r| r.name.clone())
                                                                .collect();

                                                            // Portfolios accessible
                                                            let accessible_portfolios = app_store.get().portfolios.iter()
                                                                .filter(|p| p.organization_id == Some(oid) && p.assigned_users.contains(&uid))
                                                                .map(|p| p.name.clone())
                                                                .collect::<Vec<_>>();

                                                            view! {
                                                                <div class="org-member-card" class:expanded=user_exp style={utint}>
                                                                    <div class="org-member-header"
                                                                        on:click=move |_| toggle_member(oid, uid)>
                                                                        <span class="org-member-arrow">
                                                                            {move || if user_exp() { "\u{25B2}" } else { "\u{25BC}" }}
                                                                        </span>
                                                                        <div class="org-member-avatar-sm">
                                                                            {user_name.chars().next().unwrap_or('?').to_uppercase().to_string()}
                                                                        </div>
                                                                        <div class="org-member-info">
                                                                            <div class="org-member-name">{user_name.clone()}</div>
                                                                            <div class="org-member-meta">
                                                                                {format!("{} \u{00B7} {} \u{00B7} {} portfolio{}",
                                                                                    user_role_display,
                                                                                    user_email,
                                                                                    accessible_portfolios.len(),
                                                                                    if accessible_portfolios.len() == 1 { "" } else { "s" })}
                                                                            </div>
                                                                        </div>
                                                                        {if can {
                                                                            view! {
                                                                                <div class="org-member-actions" on:click=|ev| ev.stop_propagation()>
                                                                                    <select class="org-role-select"
                                                                                        on:change=move |ev| {
                                                                                            on_update_member_role(uid_role, role_from_str(&event_target_value(&ev)));
                                                                                        }>
                                                                                        <option value="Owner" selected={user_role == UserRole::Owner}>"Owner"</option>
                                                                                        <option value="Director" selected={user_role == UserRole::Director}>"Director"</option>
                                                                                        <option value="SeniorManager" selected={user_role == UserRole::SeniorManager}>"Sr Mgr"</option>
                                                                                        <option value="Manager" selected={user_role == UserRole::Manager}>"Manager"</option>
                                                                                        <option value="Worker" selected={user_role == UserRole::Worker}>"Worker"</option>
                                                                                        <option value="DocumentWorker" selected={user_role == UserRole::DocumentWorker}>"Doc Worker"</option>
                                                                                        <option value="Contractor" selected={user_role == UserRole::Contractor}>"Contractor"</option>
                                                                                        <option value="Guest" selected={user_role == UserRole::Guest}>"Guest"</option>
                                                                                    </select>
                                                                                    <button class="pf-action-btn"
                                                                                        aria-label={format!("Remove {} from organization", user_name)}
                                                                                        on:click=move |_| on_remove_member(oid, uid_del)>
                                                                                        "\u{2715}"
                                                                                    </button>
                                                                                </div>
                                                                            }.into_any()
                                                                        } else {
                                                                            view! {
                                                                                <div class="pf-list-actions">
                                                                                    <span class="org-role-badge">{user_role_display}</span>
                                                                                </div>
                                                                            }.into_any()
                                                                        }}
                                                                    </div>

                                                                    // Member expanded content
                                                                    <div class="org-member-content" class:hidden={move || !user_exp()}>
                                                                        // Location & contact info
                                                                        <div class="org-member-location">
                                                                            <span class="org-member-section-label">"Location & Contact:"</span>
                                                                            <div class="org-member-detail-grid">
                                                                                {user_address.as_ref().map(|a| view! {
                                                                                    <div class="org-member-detail-row">
                                                                                        <span class="org-member-detail-key">"Address"</span>
                                                                                        <span class="org-member-detail-val">{a.clone()}</span>
                                                                                    </div>
                                                                                })}
                                                                                {user_department.as_ref().map(|d| view! {
                                                                                    <div class="org-member-detail-row">
                                                                                        <span class="org-member-detail-key">"Department"</span>
                                                                                        <span class="org-member-detail-val">{d.clone()}</span>
                                                                                    </div>
                                                                                })}
                                                                                {user_phone.as_ref().map(|p| view! {
                                                                                    <div class="org-member-detail-row">
                                                                                        <span class="org-member-detail-key">"Phone"</span>
                                                                                        <span class="org-member-detail-val">{p.clone()}</span>
                                                                                    </div>
                                                                                })}
                                                                                <div class="org-member-detail-row">
                                                                                    <span class="org-member-detail-key">"Status"</span>
                                                                                    <span class="org-member-detail-val">{if user_active { "Active" } else { "Inactive" }}</span>
                                                                                </div>
                                                                            </div>
                                                                        </div>

                                                                        // Discord-style roles
                                                                        {if !user_role_names.is_empty() {
                                                                            view! {
                                                                                <div class="org-member-roles">
                                                                                    <span class="org-member-section-label">"Roles:"</span>
                                                                                    {user_role_names.iter().map(|rn| view! {
                                                                                        <span class="org-role-badge">{rn.clone()}</span>
                                                                                    }).collect::<Vec<_>>()}
                                                                                </div>
                                                                            }.into_any()
                                                                        } else { ().into_any() }}

                                                                        // Accessible portfolios
                                                                        {if !accessible_portfolios.is_empty() {
                                                                            view! {
                                                                                <div class="org-member-portfolios">
                                                                                    <span class="org-member-section-label">"Portfolio access:"</span>
                                                                                    {accessible_portfolios.iter().map(|pn| view! {
                                                                                        <span class="org-portfolio-badge">{pn.clone()}</span>
                                                                                    }).collect::<Vec<_>>()}
                                                                                </div>
                                                                            }.into_any()
                                                                        } else { ().into_any() }}

                                                                        // Permissions list
                                                                        {if !user_perms.is_empty() {
                                                                            view! {
                                                                                <div class="org-member-permissions">
                                                                                    <span class="org-member-section-label">{format!("Permissions ({}):", user_perms.len())}</span>
                                                                                    <div class="org-member-perm-list">
                                                                                        {user_perms.iter().map(|p| view! {
                                                                                            <span class="org-perm-badge">{permission_label(p)}</span>
                                                                                        }).collect::<Vec<_>>()}
                                                                                    </div>
                                                                                </div>
                                                                            }.into_any()
                                                                        } else { ().into_any() }}

                                                                        // Assignments
                                                                        {if !user_assignments.is_empty() {
                                                                            view! {
                                                                                <div class="org-member-assignments">
                                                                                    <span class="org-member-section-label">{format!("Assignments ({}):", user_assignments.len())}</span>
                                                                                    <div class="org-member-assignment-list">
                                                                                        {user_assignments.iter().map(|a| view! {
                                                                                            <div class="org-member-assignment-row">
                                                                                                <span class="org-member-assignment-name">{a.target_name.clone()}</span>
                                                                                                <span class="org-member-assignment-type">{a.target_type.clone()}</span>
                                                                                            </div>
                                                                                        }).collect::<Vec<_>>()}
                                                                                    </div>
                                                                                </div>
                                                                            }.into_any()
                                                                        } else { ().into_any() }}
                                                                    </div>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                        </div>
                                                    }.into_any()
                                                }
                                            })}
                                        </div>

                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }
            }}

            // Role editor modal
            {move || editing_role.get().map(|(oid, rid)| {
                let is_new = rid == Uuid::nil();
                let title = if is_new { "Create New Role" } else { "Edit Role" };
                view! {
                    <div class="modal-overlay" on:click=move |_| set_editing_role.set(None)>
                        <div class="modal" on:click=|ev| ev.stop_propagation()>
                            <div class="modal-header">
                                <span class="modal-title">{title}</span>
                                <button class="modal-close" on:click=move |_| set_editing_role.set(None)>"\u{00D7}"</button>
                            </div>
                            <div class="modal-body">
                                <div class="org-role-edit-form">
                                    <label class="org-edit-label">"Role name"</label>
                                    <input class="login-input" type="text" placeholder="Role name"
                                        prop:value=move || edit_role_name.get()
                                        on:input=move |ev| set_edit_role_name.set(event_target_value(&ev)) />

                                    <label class="org-edit-label">"Description"</label>
                                    <textarea class="login-input org-role-edit-desc" placeholder="Plain-English description of what this role can do"
                                        on:input=move |ev| set_edit_role_desc.set(event_target_value(&ev))>
                                        {move || edit_role_desc.get()}
                                    </textarea>

                                    <label class="org-edit-label">"Rank (higher = more authority)"</label>
                                    <input class="login-input" type="number" min="0" max="100"
                                        prop:value=move || edit_role_rank.get().to_string()
                                        on:input=move |ev| {
                                            if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                                                set_edit_role_rank.set(v);
                                            }
                                        } />

                                    <label class="org-edit-label">"Accent color"</label>
                                    <input class="org-color-input" type="color"
                                        prop:value=move || edit_role_color.get()
                                        on:input=move |ev| set_edit_role_color.set(event_target_value(&ev)) />

                                    <label class="org-edit-label">"Scope"</label>
                                    <select class="login-input"
                                        on:change=move |ev| set_edit_role_scope.set(scope_from_str(&event_target_value(&ev)))>
                                        <option value="EntireOrganization" selected=true>"Entire organization"</option>
                                        <option value="ReportingOnly">"Reporting only"</option>
                                        <option value="CalendarOnly">"Calendar only"</option>
                                        <option value="TransactionsOnly">"Transactions only"</option>
                                        <option value="NetworkingOnly">"Networking only"</option>
                                        <option value="HistoryOnly">"History/audit only"</option>
                                    </select>

                                    <div style="display:flex;gap:8px;margin-top:12px;">
                                        <button class="login-btn" style="flex:1;" on:click=on_save_role>"Save Role"</button>
                                        <button class="view-btn" style="flex:1;" on:click=move |_| set_editing_role.set(None)>"Cancel"</button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any()
            })}

            // Context menu for organizations
            {move || context_menu.get().map(|(x, y, id)| {
                let org = app_store.get().organizations.iter().find(|o| o.id == id).cloned();
                org.map(|o| {
                    let name = o.name.clone();
                    let desc = o.description.clone();
                    let color = o.settings.color.clone();
                    view! {
                        <div class="context-menu-overlay" on:click=move |_| close_context_menu()>
                            <div class="context-menu" style={format!("left: {}px; top: {}px;", x, y)}>
                                <button class="context-menu-item"
                                    on:click=move |_| {
                                        close_context_menu();
                                        on_start_edit(id, name.clone(), desc.clone(), color.clone());
                                    }>"\u{270E} Edit Organization"</button>
                                <button class="context-menu-item"
                                    on:click=move |_| {
                                        close_context_menu();
                                        set_show_add_member.set(Some(id));
                                        set_expanded_orgs.update(|s| { s.insert(id); });
                                        set_org_tab(id, "members");
                                    }>
                                    "+ Add Member"
                                </button>
                                <button class="context-menu-item"
                                    on:click=move |_| {
                                        close_context_menu();
                                        on_start_new_role(id);
                                        set_expanded_orgs.update(|s| { s.insert(id); });
                                        set_org_tab(id, "roles");
                                    }>
                                    "+ Create Role"
                                </button>
                                <button class="context-menu-item"
                                    on:click=move |_| {
                                        close_context_menu();
                                        set_show_add_org.set(true);
                                    }>
                                    "+ Add Organization"
                                </button>
                                <button class="context-menu-item danger"
                                    on:click=move |_| {
                                        close_context_menu();
                                        on_delete_org(id);
                                    }>
                                    "\u{1F5D1} Delete Organization"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                }).unwrap_or(().into_any())
            })}

            // Context menu for roles
            {move || role_context_menu.get().map(|(x, y, oid, rid)| {
                let role = app_store.get().organizations.iter()
                    .find(|o| o.id == oid)
                    .and_then(|o| o.roles.iter().find(|r| r.id == rid))
                    .cloned();
                role.map(|r| {
                    let r_name = r.name.clone();
                    let r_desc = r.description.clone();
                    let r_color = r.color.clone();
                    let r_rank = r.rank;
                    let r_scope = r.scope.clone();
                    let r_perms = r.permissions.clone();
                    let r_members = r.member_ids.clone();
                    let r_is_system = r.is_system;
                    view! {
                        <div class="context-menu-overlay" on:click=move |_| close_role_context_menu()>
                            <div class="context-menu" style={format!("left: {}px; top: {}px;", x, y)}>
                                <button class="context-menu-item"
                                    on:click=move |_| {
                                        close_role_context_menu();
                                        on_start_role_edit(oid, &OrgRole {
                                            id: rid,
                                            name: r_name.clone(),
                                            rank: r_rank,
                                            color: r_color.clone(),
                                            description: r_desc.clone(),
                                            scope: r_scope.clone(),
                                            permissions: r_perms.clone(),
                                            member_ids: r_members.clone(),
                                            documents: Vec::new(),
                                            is_system: r_is_system,
                                        });
                                    }>"\u{270E} Edit Role"</button>
                                <button class="context-menu-item"
                                    on:click=move |_| {
                                        close_role_context_menu();
                                        on_duplicate_role(oid, rid);
                                    }>"\u{1F4CB} Duplicate Role"</button>
                                <button class="context-menu-item"
                                    on:click=move |_| {
                                        close_role_context_menu();
                                        set_expanded_orgs.update(|s| { s.insert(oid); });
                                        set_org_tab(oid, "roles");
                                        set_expanded_roles.update(|s| { s.insert((oid, rid)); });
                                    }>"\u{25BC} Expand Role"</button>
                                {if !r_is_system {
                                    view! {
                                        <button class="context-menu-item danger"
                                            on:click=move |_| {
                                                close_role_context_menu();
                                                on_delete_role(oid, rid);
                                            }>
                                            "\u{1F5D1} Delete Role"
                                        </button>
                                    }.into_any()
                                } else { ().into_any() }}
                            </div>
                        </div>
                    }.into_any()
                }).unwrap_or(().into_any())
            })}

        </div>
    }
}
