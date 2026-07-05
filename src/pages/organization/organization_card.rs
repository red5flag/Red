use crate::models::{OrgRole, Organization, Perm, User};
use crate::pages::organization::{
    members::MembersSection, organization_summary::OrganizationSummary,
    portfolio_access::PortfolioAccessList, roles::RolesSection,
};
use crate::stores::{AppStore, OrganizationStore};
use crate::types::UserRole;
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub(crate) fn OrganizationCard(
    app_store: RwSignal<AppStore>,
    organization_store: RwSignal<OrganizationStore>,
    ui_store: RwSignal<crate::stores::UiStore>,
    org: Organization,
    #[prop(into)] can_manage: bool,
    #[prop(into)] is_expanded: Signal<bool>,
    on_toggle_org: Callback<Uuid, ()>,
    #[prop(into)] is_editing: bool,
    #[prop(into)] edit_name: ReadSignal<String>,
    #[prop(into)] set_edit_name: WriteSignal<String>,
    #[prop(into)] edit_desc: ReadSignal<String>,
    #[prop(into)] set_edit_desc: WriteSignal<String>,
    #[prop(into)] edit_color: ReadSignal<String>,
    #[prop(into)] set_edit_color: WriteSignal<String>,
    #[prop(into)] set_editing_org: WriteSignal<Option<Uuid>>,
    on_start_edit: Callback<(Uuid, String, Option<String>, Option<String>), ()>,
    on_save_edit: Callback<Uuid, ()>,
    on_delete_org: Callback<Uuid, ()>,
    #[prop(into)] get_org_tab: Callback<Uuid, &'static str>,
    #[prop(into)] set_org_tab: Callback<(Uuid, &'static str), ()>,
    #[prop(into)] expanded_roles: ReadSignal<HashSet<(Uuid, Uuid)>>,
    on_toggle_role: Callback<(Uuid, Uuid), ()>,
    on_start_new_role: Callback<Uuid, ()>,
    on_start_role_edit: Callback<(Uuid, OrgRole), ()>,
    on_duplicate_role: Callback<(Uuid, Uuid), ()>,
    on_delete_role: Callback<(Uuid, Uuid), ()>,
    #[prop(into)] expanded_perm_groups: ReadSignal<HashSet<(Uuid, Uuid, usize)>>,
    on_toggle_perm_group: Callback<(Uuid, Uuid, usize), ()>,
    on_toggle_role_perm: Callback<(Uuid, Uuid, Perm), ()>,
    on_assign_role_member: Callback<(Uuid, Uuid, Uuid), ()>,
    on_remove_role_member: Callback<(Uuid, Uuid, Uuid), ()>,
    #[prop(into)] expanded_members: ReadSignal<HashSet<(Uuid, Uuid)>>,
    on_toggle_member: Callback<(Uuid, Uuid), ()>,
    #[prop(into)] show_add_member: ReadSignal<Option<Uuid>>,
    #[prop(into)] set_show_add_member: WriteSignal<Option<Uuid>>,
    #[prop(into)] member_name: ReadSignal<String>,
    #[prop(into)] set_member_name: WriteSignal<String>,
    #[prop(into)] member_email: ReadSignal<String>,
    #[prop(into)] set_member_email: WriteSignal<String>,
    #[prop(into)] member_role: ReadSignal<UserRole>,
    #[prop(into)] set_member_role: WriteSignal<UserRole>,
    on_add_member: Callback<Uuid, ()>,
    on_remove_member: Callback<(Uuid, Uuid), ()>,
    on_update_member_role: Callback<(Uuid, UserRole), ()>,
    on_context_menu: Callback<(i32, i32, Uuid), ()>,
    on_role_context_menu: Callback<(i32, i32, Uuid, Uuid), ()>,
) -> impl IntoView {
    let role_context_menu = on_role_context_menu;
    let oid = org.id;
    let member_count = org.members.len();
    let portfolio_count = app_store
        .get()
        .portfolios
        .iter()
        .filter(|p| p.organization_id == Some(oid))
        .count();
    let role_count = org.roles.len();
    let owner_name = organization_store
        .get()
        .organization_users
        .iter()
        .find(|u| u.id == org.owner_id)
        .map(|u| u.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let document_count: usize = app_store
        .get()
        .portfolios
        .iter()
        .filter(|p| p.organization_id == Some(oid))
        .map(|p| p.documents.len())
        .sum();

    let org_name = org.name.clone();
    let org_desc = org.description.clone();
    let org_color = org.settings.color.clone();
    let color_style = org_color
        .as_ref()
        .map(|c| format!("border-left: 4px solid {};", c))
        .unwrap_or_default();
    let blind_btn_name = org.name.clone();
    let blind_btn_desc = org.description.clone();
    let blind_btn_color = org.settings.color.clone();
    let blind = ui_store.get().blind_mode;

    let org_portfolios: Vec<_> = app_store
        .get()
        .portfolios
        .iter()
        .filter(|p| p.organization_id == Some(oid))
        .cloned()
        .collect();
    let org_roles = org.roles.clone();
    let org_members: Vec<User> = organization_store
        .get()
        .organization_users
        .iter()
        .filter(|u| org.members.contains(&u.id))
        .cloned()
        .collect();
    let all_org_users: Vec<User> = organization_store
        .get()
        .organization_users
        .iter()
        .filter(|u| org.members.contains(&u.id))
        .cloned()
        .collect();

    let org_tab = move || get_org_tab.run(oid);

    view! {
        <div class="asset-group" class:expanded=is_expanded>
            // Header
            <div class="asset-group-header" style={color_style}
                on:click=move |_| { if !is_editing { on_toggle_org.run(oid); } }
                on:dblclick=move |ev: leptos::ev::MouseEvent| {
                    if can_manage && !is_editing {
                        ev.stop_propagation();
                        on_start_edit.run((oid, org_name.clone(), org_desc.clone(), org_color.clone()));
                    }
                }
            >
                <span class="asset-group-arrow">
                    {move || if is_expanded.get() { "\u{25B2}" } else { "\u{25BC}" }}
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
                                    <button class="login-btn" style="flex:1;" on:click=move |_| on_save_edit.run(oid)>"Save"</button>
                                    <button class="view-btn" style="flex:1;" on:click=move |_| set_editing_org.set(None)>"Cancel"</button>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div
                                on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                                    if can_manage && !blind {
                                        ev.prevent_default();
                                        ev.stop_propagation();
                                        on_context_menu.run((ev.client_x(), ev.client_y(), oid));
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
                {if blind && can_manage && !is_editing {
                    view! {
                        <div class="pf-list-actions" on:click=|ev| ev.stop_propagation()>
                            <button class="pf-action-btn"
                                on:click=move |_| on_start_edit.run((oid, blind_btn_name.clone(), blind_btn_desc.clone(), blind_btn_color.clone()))>
                                "\u{270E}"
                            </button>
                            <button class="pf-action-btn"
                                on:click=move |_| on_delete_org.run(oid)>
                                "\u{1F5D1}"
                            </button>
                        </div>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            // Expanded content
            <div class="asset-group-content" class:hidden={move || !is_expanded.get()}>
                // Organization Overview
                <OrganizationSummary
                    owner_name={owner_name.clone()}
                    portfolio_count={portfolio_count}
                    member_count={member_count}
                    role_count={role_count}
                    document_count={document_count}
                />

                // Sub-tab bar
                <div class="org-sub-tabs">
                    <button class="org-sub-tab" class:active={move || org_tab() == "portfolios"}
                        on:click=move |_| set_org_tab.run((oid, "portfolios"))>
                        <span>"Portfolios ("</span>
                        {move || app_store.get().portfolios.iter()
                            .filter(|p| p.organization_id == Some(oid)).count().to_string()}
                        <span>")"</span>
                    </button>
                    <button class="org-sub-tab" class:active={move || org_tab() == "roles"}
                        on:click=move |_| set_org_tab.run((oid, "roles"))>
                        {format!("Roles ({})", role_count)}
                    </button>
                    <button class="org-sub-tab" class:active={move || org_tab() == "members"}
                        on:click=move |_| set_org_tab.run((oid, "members"))>
                        {format!("Members ({})", member_count)}
                    </button>
                </div>

                // Portfolios tab
                <div class="org-sub-tab-content" class:hidden={move || org_tab() != "portfolios"}>
                    <PortfolioAccessList portfolios={org_portfolios.clone()} />
                </div>

                // Roles tab
                <div class="org-sub-tab-content" class:hidden={move || org_tab() != "roles"}>
                    <RolesSection
                        org_id=oid
                        can_edit={can_manage}
                        roles={org_roles.clone()}
                        expanded_roles=expanded_roles
                        on_toggle_role={on_toggle_role}
                        on_start_new_role={on_start_new_role}
                        on_start_role_edit={on_start_role_edit}
                        on_duplicate_role={on_duplicate_role}
                        on_delete_role={on_delete_role}
                        expanded_perm_groups=expanded_perm_groups
                        on_toggle_perm_group={on_toggle_perm_group}
                        on_toggle_role_perm={on_toggle_role_perm}
                        on_assign_role_member={on_assign_role_member}
                        on_remove_role_member={on_remove_role_member}
                        available_users={all_org_users.clone()}
                        on_role_context_menu={role_context_menu}
                    />
                </div>

                // Members tab
                <div class="org-sub-tab-content" class:hidden={move || org_tab() != "members"}>
                    <MembersSection
                        org_id=oid
                        can_edit={can_manage}
                        members={org_members.clone()}
                        roles={org_roles.clone()}
                        show_add_member={show_add_member}
                        set_show_add_member={set_show_add_member}
                        member_name={member_name}
                        set_member_name={set_member_name}
                        member_email={member_email}
                        set_member_email={set_member_email}
                        member_role={member_role}
                        set_member_role={set_member_role}
                        on_add_member={on_add_member}
                        on_remove_member={on_remove_member}
                        on_update_member_role={on_update_member_role}
                        expanded_members={expanded_members}
                        on_toggle_member={on_toggle_member}
                        portfolios={org_portfolios.clone()}
                    />
                </div>
            </div>
        </div>
    }
}
