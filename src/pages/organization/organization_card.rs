use crate::models::{OrgRole, Organization, Perm, User};
use crate::pages::organization::{
    members::MembersSection, organization_summary::OrganizationSummary,
    portfolio_access::PortfolioAccessList, roles::RolesSection,
};
use crate::pages::portfolios::read_image_as_data_url;
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
    #[prop(into)] edit_abn: ReadSignal<String>,
    #[prop(into)] set_edit_abn: WriteSignal<String>,
    #[prop(into)] edit_lei: ReadSignal<String>,
    #[prop(into)] set_edit_lei: WriteSignal<String>,
    #[prop(into)] edit_business_type: ReadSignal<String>,
    #[prop(into)] set_edit_business_type: WriteSignal<String>,
    #[prop(into)] edit_business_address: ReadSignal<String>,
    #[prop(into)] set_edit_business_address: WriteSignal<String>,
    #[prop(into)] edit_business_phone: ReadSignal<String>,
    #[prop(into)] set_edit_business_phone: WriteSignal<String>,
    #[prop(into)] edit_business_email: ReadSignal<String>,
    #[prop(into)] set_edit_business_email: WriteSignal<String>,
    #[prop(into)] set_editing_org: WriteSignal<Option<Uuid>>,
    on_start_edit: Callback<
        (
            Uuid,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
        (),
    >,
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
    #[prop(into)] edit_image_url: ReadSignal<Option<String>>,
    #[prop(into)] set_edit_image_url: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (section_menu, set_section_menu) = signal(Option::<(i32, i32, &'static str, Uuid)>::None);
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
    let org_image_url = org.image_url.clone();
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
    let org_abn = org.abn.clone();
    let org_lei = org.lei.clone();
    let org_business_type = org.business_type.clone();
    let org_business_address = org.business_address.clone();
    let org_business_phone = org.business_phone.clone();
    let org_business_email = org.business_email.clone();

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

    let on_create_test_member = Callback::new(move |(oid, role): (Uuid, UserRole)| {
        if !app_store.get().developer_mode {
            return;
        }
        organization_store.update(|s| {
            let index = s.organization_users.len() + 1;
            let name = format!("Test User {}", index);
            let email = format!("test{}@farley.test", index);
            let mut user = User::new(name, email, role);
            user.organization_id = Some(oid);
            let uid = user.id;
            s.add_organization_user(user);
            if let Some(org) = s.get_organization_mut(oid) {
                org.add_member(uid);
            }
        });
    });

    let org_tab = move || get_org_tab.run(oid);

    view! {
        <div class="asset-group" class:expanded=is_expanded>
            // Header
            <div class="asset-group-header" style={color_style}
                on:click=move |_| { if !is_editing { on_toggle_org.run(oid); } }
                on:dblclick=move |ev: leptos::ev::MouseEvent| {
                    if can_manage && !is_editing {
                        ev.stop_propagation();
                        on_start_edit.run((oid, org_name.clone(), org.image_url.clone(), org_desc.clone(), org_color.clone(), org.abn.clone(), org.lei.clone(), org.business_type.clone(), org.business_address.clone(), org.business_phone.clone(), org.business_email.clone()));
                    }
                }
            >
                <span class="asset-group-arrow">
                    {move || if is_expanded.get() { "\u{25B2}" } else { "\u{25BC}" }}
                </span>
                <div class="asset-group-icon">
                    {if let Some(ref url) = org_image_url {
                        view! { <img class="pf-header-image" src={url.clone()} alt={format!("{} logo", org_name)} /> }.into_any()
                    } else {
                        view! { <span>"\u{1F3E2}"</span> }.into_any()
                    }}
                </div>
                <div class="asset-group-info-wrap">
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
                                    <span class="org-color-label">"Logo"</span>
                                    {move || edit_image_url.get().map(|url| view! {
                                        <img class="pf-header-image" src={url} alt="Organization logo preview" />
                                    })}
                                    <input class="login-input apf-file-input" type="file" accept="image/*"
                                        on:change=move |ev| read_image_as_data_url(&ev, move |url| set_edit_image_url.set(Some(url))) />
                                </div>
                                <div class="org-color-row">
                                    <span class="org-color-label">"Accent"</span>
                                    <input class="org-color-input" type="color"
                                        prop:value=move || edit_color.get()
                                        on:input=move |ev| set_edit_color.set(event_target_value(&ev)) />
                                </div>
                                <div class="org-business-section">
                                    <div class="org-section-title">"Business Details (Optional)"</div>
                                    <input class="pf-edit-input" placeholder="ABN (Australian Business Number)"
                                        prop:value=move || edit_abn.get()
                                        on:input=move |ev| set_edit_abn.set(event_target_value(&ev)) />
                                    <input class="pf-edit-input" placeholder="LEI (Legal Entity Identifier)"
                                        prop:value=move || edit_lei.get()
                                        on:input=move |ev| set_edit_lei.set(event_target_value(&ev)) />
                                    <select class="pf-edit-input"
                                        prop:value=move || edit_business_type.get()
                                        on:change=move |ev| set_edit_business_type.set(event_target_value(&ev))>
                                        <option value="">"Business Type"</option>
                                        <option value="Sole Trader">"Sole Trader"</option>
                                        <option value="Company">"Company"</option>
                                        <option value="Partnership">"Partnership"</option>
                                        <option value="Trust">"Trust"</option>
                                        <option value="Non-profit">"Non-profit"</option>
                                        <option value="Government">"Government"</option>
                                    </select>
                                    <input class="pf-edit-input" placeholder="Business Address"
                                        prop:value=move || edit_business_address.get()
                                        on:input=move |ev| set_edit_business_address.set(event_target_value(&ev)) />
                                    <input class="pf-edit-input" placeholder="Business Phone"
                                        prop:value=move || edit_business_phone.get()
                                        on:input=move |ev| set_edit_business_phone.set(event_target_value(&ev)) />
                                    <input class="pf-edit-input" placeholder="Business Email"
                                        prop:value=move || edit_business_email.get()
                                        on:input=move |ev| set_edit_business_email.set(event_target_value(&ev)) />
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
                {if can_manage && !is_editing {
                    view! {
                        <div class="pf-list-actions org-card-actions" on:click=|ev| ev.stop_propagation()>
                            <button class="pf-action-btn org-context-trigger"
                                aria-label="Open organization actions"
                                aria-haspopup="menu"
                                aria-expanded={move || section_menu.get().is_some()}
                                on:click=move |ev: leptos::ev::MouseEvent| {
                                    ev.prevent_default();
                                    ev.stop_propagation();
                                    on_context_menu.run((ev.client_x(), ev.client_y(), oid));
                                }>
                                "\u{22EE}"
                            </button>
                            {if blind {
                                view! {
                                    <button class="pf-action-btn"
                                        aria-label="Edit organization"
                                        on:click=move |_| on_start_edit.run((oid, blind_btn_name.clone(), org_image_url.clone(), blind_btn_desc.clone(), blind_btn_color.clone(), org_abn.clone(), org_lei.clone(), org_business_type.clone(), org_business_address.clone(), org_business_phone.clone(), org_business_email.clone()))>
                                        "\u{270E}"
                                    </button>
                                    <button class="pf-action-btn"
                                        aria-label="Delete organization"
                                        on:click=move |_| on_delete_org.run(oid)>
                                        "\u{1F5D1}"
                                    </button>
                                }.into_any()
                            } else { ().into_any() }}
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
                        on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                            ev.prevent_default();
                            ev.stop_propagation();
                            set_section_menu.set(Some((ev.client_x(), ev.client_y(), "portfolios", oid)));
                        }
                        on:click=move |_| set_org_tab.run((oid, "portfolios"))>
                        <span>"Portfolios ("</span>
                        {move || app_store.get().portfolios.iter()
                            .filter(|p| p.organization_id == Some(oid)).count().to_string()}
                        <span>")"</span>
                        {if can_manage {
                            view! {
                                <span class="org-section-actions"
                                    role="button"
                                    aria-label="Open portfolio section actions"
                                    aria-haspopup="menu"
                                    aria-expanded={move || section_menu.get().map(|(_, _, s, _)| s == "portfolios").unwrap_or(false)}
                                    on:click=move |ev: leptos::ev::MouseEvent| {
                                        ev.prevent_default();
                                        ev.stop_propagation();
                                        set_section_menu.set(Some((ev.client_x(), ev.client_y(), "portfolios", oid)));
                                    }>
                                    "\u{22EE}"
                                </span>
                            }.into_any()
                        } else { ().into_any() }}
                    </button>
                    <button class="org-sub-tab" class:active={move || org_tab() == "roles"}
                        on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                            ev.prevent_default();
                            ev.stop_propagation();
                            set_section_menu.set(Some((ev.client_x(), ev.client_y(), "roles", oid)));
                        }
                        on:click=move |_| set_org_tab.run((oid, "roles"))>
                        {format!("Roles ({})", role_count)}
                        {if can_manage {
                            view! {
                                <span class="org-section-actions"
                                    role="button"
                                    aria-label="Open role section actions"
                                    aria-haspopup="menu"
                                    aria-expanded={move || section_menu.get().map(|(_, _, s, _)| s == "roles").unwrap_or(false)}
                                    on:click=move |ev: leptos::ev::MouseEvent| {
                                        ev.prevent_default();
                                        ev.stop_propagation();
                                        set_section_menu.set(Some((ev.client_x(), ev.client_y(), "roles", oid)));
                                    }>
                                    "\u{22EE}"
                                </span>
                            }.into_any()
                        } else { ().into_any() }}
                    </button>
                    <button class="org-sub-tab" class:active={move || org_tab() == "members"}
                        on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                            ev.prevent_default();
                            ev.stop_propagation();
                            set_section_menu.set(Some((ev.client_x(), ev.client_y(), "members", oid)));
                        }
                        on:click=move |_| set_org_tab.run((oid, "members"))>
                        {format!("Members ({})", member_count)}
                        {if can_manage {
                            view! {
                                <span class="org-section-actions"
                                    role="button"
                                    aria-label="Open member section actions"
                                    aria-haspopup="menu"
                                    aria-expanded={move || section_menu.get().map(|(_, _, s, _)| s == "members").unwrap_or(false)}
                                    on:click=move |ev: leptos::ev::MouseEvent| {
                                        ev.prevent_default();
                                        ev.stop_propagation();
                                        set_section_menu.set(Some((ev.client_x(), ev.client_y(), "members", oid)));
                                    }>
                                    "\u{22EE}"
                                </span>
                            }.into_any()
                        } else { ().into_any() }}
                    </button>
                </div>

                // Portfolios tab
                <div class="org-sub-tab-content" class:hidden={move || org_tab() != "portfolios"}>
                    <PortfolioAccessList
                        org_id=oid
                        app_store={app_store}
                        can_edit={can_manage}
                    />
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
                        on_empty_add_role={Callback::new(move |_| on_start_new_role.run(oid))}
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
                        on_empty_add_member={Callback::new(move |_| {
                            set_show_add_member.set(Some(oid));
                        })}
                        app_store={app_store}
                        on_create_test_member={on_create_test_member}
                    />
                </div>
            </div>

            // Section header context menu
            {move || section_menu.get().map(|(x, y, section, menu_oid)| {
                view! {
                    <div class="context-menu-overlay org-context-overlay" on:click=move |_| set_section_menu.set(None)>
                        <div class="context-menu org-context-menu" style={format!("left: {}px; top: {}px;", x, y)}>
                            {match section {
                                "portfolios" => view! {
                                    <>
                                        <button class="context-menu-item org-context-menu-item" disabled>
                                            "\u{2795} Assign Portfolio (coming soon)"
                                        </button>
                                        <button class="context-menu-item org-context-menu-item"
                                            on:click=move |_| {
                                                set_section_menu.set(None);
                                                set_org_tab.run((menu_oid, "portfolios"));
                                            }>
                                            "\u{1F4C1} View Portfolios"
                                        </button>
                                    </>
                                }.into_any(),
                                "roles" => view! {
                                    <>
                                        <button class="context-menu-item org-context-menu-item"
                                            on:click=move |_| {
                                                set_section_menu.set(None);
                                                on_start_new_role.run(menu_oid);
                                            }>
                                            "\u{2795} Create Role"
                                        </button>
                                        <button class="context-menu-item org-context-menu-item"
                                            on:click=move |_| {
                                                set_section_menu.set(None);
                                                set_org_tab.run((menu_oid, "roles"));
                                            }>
                                            "\u{1F465} View Roles"
                                        </button>
                                    </>
                                }.into_any(),
                                "members" => view! {
                                    <>
                                        <button class="context-menu-item org-context-menu-item"
                                            on:click=move |_| {
                                                set_section_menu.set(None);
                                                set_show_add_member.set(Some(menu_oid));
                                            }>
                                            "\u{2795} Add Member"
                                        </button>
                                        <button class="context-menu-item org-context-menu-item"
                                            on:click=move |_| {
                                                set_section_menu.set(None);
                                                set_org_tab.run((menu_oid, "members"));
                                            }>
                                            "\u{1F465} View Members"
                                        </button>
                                    </>
                                }.into_any(),
                                _ => ().into_any(),
                            }}
                        </div>
                    </div>
                }.into_any()
            })}
        </div>
    }
}
