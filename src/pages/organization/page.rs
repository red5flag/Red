use crate::models::{OrgRole, Organization, Perm, User};
use crate::pages::organization::{
    organization_forms::{AddOrgForm, OrgContextMenu, RoleContextMenu, RoleEditorModal},
    organization_list::OrganizationList,
};
use crate::stores::{use_app_store, use_organization_store, use_ui_store};
use crate::types::UserRole;
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub fn OrganizationPage() -> impl IntoView {
    let app_store = use_app_store();
    let organization_store = use_organization_store();
    let ui_store = use_ui_store();

    let organizations = Memo::new(move |_| organization_store.get().organizations.clone());

    let (expanded_orgs, set_expanded_orgs) = signal(HashSet::<Uuid>::new());
    let (org_active_tab, set_org_active_tab) =
        signal(std::collections::HashMap::<Uuid, &'static str>::new());
    let (expanded_roles, set_expanded_roles) = signal(HashSet::<(Uuid, Uuid)>::new());
    let (expanded_perm_groups, set_expanded_perm_groups) =
        signal(HashSet::<(Uuid, Uuid, usize)>::new());
    let (expanded_members, set_expanded_members) = signal(HashSet::<(Uuid, Uuid)>::new());
    let (editing_role, set_editing_role) = signal(Option::<(Uuid, Uuid)>::None);
    let (edit_role_name, set_edit_role_name) = signal(String::new());
    let (edit_role_desc, set_edit_role_desc) = signal(String::new());
    let (edit_role_rank, set_edit_role_rank) = signal(50u32);
    let (edit_role_color, set_edit_role_color) = signal(String::new());
    let (edit_role_scope, set_edit_role_scope) =
        signal(crate::models::RoleScope::EntireOrganization);
    let (show_add_org, set_show_add_org) = signal(false);
    let (new_org_name, set_new_org_name) = signal(String::new());
    let (new_org_desc, set_new_org_desc) = signal(String::new());
    let (show_add_member, set_show_add_member) = signal(Option::<Uuid>::None);
    let (member_name, set_member_name) = signal(String::new());
    let (member_email, set_member_email) = signal(String::new());
    let (member_role, set_member_role) = signal(UserRole::Worker);
    let (editing_org, set_editing_org) = signal(Option::<Uuid>::None);
    let (edit_name, set_edit_name) = signal(String::new());
    let (edit_desc, set_edit_desc) = signal(String::new());
    let (edit_color, set_edit_color) = signal(String::new());
    let (context_menu, set_context_menu) = signal(Option::<(i32, i32, Uuid)>::None);
    let (role_context_menu, set_role_context_menu) = signal(Option::<(i32, i32, Uuid, Uuid)>::None);

    let toggle_org = move |oid: Uuid| {
        set_expanded_orgs.update(|s| {
            if !s.remove(&oid) {
                s.insert(oid);
            }
        });
    };
    let get_org_tab = move |oid: Uuid| {
        org_active_tab
            .get()
            .get(&oid)
            .copied()
            .unwrap_or("portfolios")
    };
    let set_org_tab = move |oid: Uuid, tab: &'static str| {
        set_org_active_tab.update(|m| {
            m.insert(oid, tab);
        });
    };
    let toggle_role = move |oid: Uuid, rid: Uuid| {
        set_expanded_roles.update(|s| {
            if !s.remove(&(oid, rid)) {
                s.insert((oid, rid));
            }
        });
    };
    let toggle_perm_group = move |oid: Uuid, rid: Uuid, gi: usize| {
        set_expanded_perm_groups.update(|s| {
            if !s.remove(&(oid, rid, gi)) {
                s.insert((oid, rid, gi));
            }
        });
    };
    let toggle_member = move |oid: Uuid, uid: Uuid| {
        set_expanded_members.update(|s| {
            if !s.remove(&(oid, uid)) {
                s.insert((oid, uid));
            }
        });
    };

    let on_add_org = move |_| {
        let name = new_org_name.get();
        if name.trim().is_empty() {
            return;
        }
        let owner_id = app_store.get().current_user.id;
        let mut org = Organization::new(name, owner_id);
        org.description = if new_org_desc.get().trim().is_empty() {
            None
        } else {
            Some(new_org_desc.get())
        };
        organization_store.update(|s| s.add_organization(org));
        set_new_org_name.set(String::new());
        set_new_org_desc.set(String::new());
        set_show_add_org.set(false);
    };

    let on_delete_org = move |id: Uuid| {
        organization_store.update(|s| {
            s.remove_organization(id);
        });
    };

    let on_start_edit =
        move |id: Uuid, name: String, desc: Option<String>, color: Option<String>| {
            set_edit_name.set(name);
            set_edit_desc.set(desc.unwrap_or_default());
            set_edit_color.set(color.unwrap_or_default());
            set_editing_org.set(Some(id));
        };

    let on_save_edit = move |id: Uuid| {
        let name = edit_name.get();
        if name.trim().is_empty() {
            return;
        }
        let color = edit_color.get();
        organization_store.update(|s| {
            if let Some(org) = s.get_organization_mut(id) {
                org.name = name;
                org.description = if edit_desc.get().trim().is_empty() {
                    None
                } else {
                    Some(edit_desc.get())
                };
                org.settings.color = if color.trim().is_empty() {
                    None
                } else {
                    Some(color)
                };
                org.updated_at = chrono::Utc::now();
            }
        });
        set_editing_org.set(None);
    };

    let on_add_member = move |org_id: Uuid| {
        let name = member_name.get();
        let email = member_email.get();
        if name.trim().is_empty() || email.trim().is_empty() {
            return;
        }
        let mut user = User::new(name, email, member_role.get());
        user.organization_id = Some(org_id);
        organization_store.update(|s| {
            s.add_organization_user(user.clone());
            if let Some(org) = s.get_organization_mut(org_id) {
                org.add_member(user.id);
            }
        });
        set_member_name.set(String::new());
        set_member_email.set(String::new());
        set_member_role.set(UserRole::Worker);
        set_show_add_member.set(None);
    };

    let on_remove_member = move |org_id: Uuid, user_id: Uuid| {
        organization_store.update(|s| {
            s.remove_organization_user(user_id);
            if let Some(org) = s.get_organization_mut(org_id) {
                org.remove_member(user_id);
            }
        });
    };

    let on_update_member_role = move |user_id: Uuid, new_role: UserRole| {
        organization_store.update(|s| {
            let _ = s.update_user_role(user_id, new_role, app_store.get().current_user.id);
        });
    };

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
        set_edit_role_scope.set(crate::models::RoleScope::EntireOrganization);
        set_editing_role.set(Some((oid, Uuid::nil())));
    };

    let on_save_role = move |_| {
        if let Some((oid, rid)) = editing_role.get() {
            let name = edit_role_name.get();
            if name.trim().is_empty() {
                return;
            }
            let desc = edit_role_desc.get();
            let rank = edit_role_rank.get();
            let color = edit_role_color.get();
            let scope = edit_role_scope.get();
            let color_opt = if color.trim().is_empty() {
                None
            } else {
                Some(color)
            };
            if rid == Uuid::nil() {
                let new_role = OrgRole::new(name, rank, desc, vec![]);
                organization_store.update(|s| {
                    let mut r = new_role;
                    r.color = color_opt;
                    r.scope = scope;
                    s.add_role_to_org(oid, r);
                });
            } else {
                organization_store
                    .update(|s| s.update_org_role(oid, rid, name, desc, color_opt, rank, scope));
            }
            set_editing_role.set(None);
        }
    };

    let on_delete_role = move |oid: Uuid, rid: Uuid| {
        organization_store.update(|s| s.delete_org_role(oid, rid));
    };

    let on_duplicate_role = move |oid: Uuid, rid: Uuid| {
        organization_store.update(|s| {
            let _ = s.duplicate_org_role(oid, rid);
        });
    };

    let on_toggle_role_perm = move |oid: Uuid, rid: Uuid, perm: Perm| {
        organization_store.update(|s| s.toggle_role_permission(oid, rid, perm));
    };

    let on_assign_role_member = move |oid: Uuid, rid: Uuid, uid: Uuid| {
        organization_store.update(|s| s.assign_member_to_role(oid, rid, uid));
    };

    let on_remove_role_member = move |oid: Uuid, rid: Uuid, uid: Uuid| {
        organization_store.update(|s| s.remove_member_from_role(oid, rid, uid));
    };

    let blind_add_button = move || {
        let blind = ui_store.get().blind_mode;
        let any_manage = organization_store.get().organizations.iter().any(|o| {
            let role = organization_store.get().current_user_role_in_org(
                o.id,
                app_store.get().current_user.id,
                app_store.get().current_user.role.clone(),
            );
            matches!(
                role,
                UserRole::Owner | UserRole::Director | UserRole::SeniorManager | UserRole::Manager
            )
        });
        if any_manage && blind {
            view! {
                <div class="view-toggle">
                    <button class="view-btn" class:active=show_add_org
                        on:click=move |_| set_show_add_org.update(|v| *v = !*v)>
                        "+ Add Organization"
                    </button>
                </div>
            }
            .into_any()
        } else {
            ().into_any()
        }
    };

    view! {
        <div class="home-screen">
            {blind_add_button}

            <AddOrgForm
                show={show_add_org}
                _set_show={set_show_add_org}
                name={new_org_name}
                set_name={set_new_org_name}
                desc={new_org_desc}
                set_desc={set_new_org_desc}
                on_add={Callback::new(move |_| on_add_org(()))}
            />

            {move || {
                let orgs = organizations.get();
                view! {
                    <OrganizationList
                        app_store={app_store}
                        organization_store={organization_store}
                        ui_store={ui_store}
                        organizations={orgs}
                        editing_org={editing_org}
                        expanded_orgs={expanded_orgs}
                        on_toggle_org={Callback::new(move |v: Uuid| toggle_org(v))}
                        edit_name={edit_name}
                        set_edit_name={set_edit_name}
                        edit_desc={edit_desc}
                        set_edit_desc={set_edit_desc}
                        edit_color={edit_color}
                        set_edit_color={set_edit_color}
                        set_editing_org={set_editing_org}
                        on_start_edit={Callback::new(move |(id, name, desc, color): (Uuid, String, Option<String>, Option<String>)| on_start_edit(id, name, desc, color))}
                        on_save_edit={Callback::new(move |v: Uuid| on_save_edit(v))}
                        on_delete_org={Callback::new(move |v: Uuid| on_delete_org(v))}
                        get_org_tab={Callback::new(move |v: Uuid| get_org_tab(v))}
                        set_org_tab={Callback::new(move |(oid, tab): (Uuid, &'static str)| set_org_tab(oid, tab))}
                        expanded_roles={expanded_roles}
                        on_toggle_role={Callback::new(move |v: (Uuid, Uuid)| toggle_role(v.0, v.1))}
                        on_start_new_role={Callback::new(move |v: Uuid| on_start_new_role(v))}
                        on_start_role_edit={Callback::new(move |(oid, role): (Uuid, OrgRole)| on_start_role_edit(oid, &role))}
                        on_duplicate_role={Callback::new(move |v: (Uuid, Uuid)| on_duplicate_role(v.0, v.1))}
                        on_delete_role={Callback::new(move |v: (Uuid, Uuid)| on_delete_role(v.0, v.1))}
                        expanded_perm_groups={expanded_perm_groups}
                        on_toggle_perm_group={Callback::new(move |v: (Uuid, Uuid, usize)| toggle_perm_group(v.0, v.1, v.2))}
                        on_toggle_role_perm={Callback::new(move |v: (Uuid, Uuid, Perm)| on_toggle_role_perm(v.0, v.1, v.2))}
                        on_assign_role_member={Callback::new(move |v: (Uuid, Uuid, Uuid)| on_assign_role_member(v.0, v.1, v.2))}
                        on_remove_role_member={Callback::new(move |v: (Uuid, Uuid, Uuid)| on_remove_role_member(v.0, v.1, v.2))}
                        expanded_members={expanded_members}
                        on_toggle_member={Callback::new(move |v: (Uuid, Uuid)| toggle_member(v.0, v.1))}
                        show_add_member={show_add_member}
                        set_show_add_member={set_show_add_member}
                        member_name={member_name}
                        set_member_name={set_member_name}
                        member_email={member_email}
                        set_member_email={set_member_email}
                        member_role={member_role}
                        set_member_role={set_member_role}
                        on_add_member={Callback::new(move |v: Uuid| on_add_member(v))}
                        on_remove_member={Callback::new(move |v: (Uuid, Uuid)| on_remove_member(v.0, v.1))}
                        on_update_member_role={Callback::new(move |v: (Uuid, UserRole)| on_update_member_role(v.0, v.1))}
                        on_context_menu={Callback::new(move |v: (i32, i32, Uuid)| set_context_menu.set(Some(v)))}
                        on_role_context_menu={Callback::new(move |v: (i32, i32, Uuid, Uuid)| set_role_context_menu.set(Some(v)))}
                    />
                }
            }}

            <RoleEditorModal
                editing_role={editing_role}
                set_editing_role={set_editing_role}
                edit_role_name={edit_role_name}
                set_edit_role_name={set_edit_role_name}
                edit_role_desc={edit_role_desc}
                set_edit_role_desc={set_edit_role_desc}
                edit_role_rank={edit_role_rank}
                set_edit_role_rank={set_edit_role_rank}
                edit_role_color={edit_role_color}
                set_edit_role_color={set_edit_role_color}
                edit_role_scope={edit_role_scope}
                set_edit_role_scope={set_edit_role_scope}
                on_save_role={Callback::new(move |_| on_save_role(()))}
            />

            <OrgContextMenu
                organization_store={organization_store}
                context_menu={context_menu}
                set_context_menu={set_context_menu}
                set_show_add_org={set_show_add_org}
                set_show_add_member={set_show_add_member}
                set_expanded_orgs={set_expanded_orgs}
                set_org_tab={Callback::new(move |(oid, tab): (Uuid, &'static str)| set_org_tab(oid, tab))}
                on_start_edit={Callback::new(move |(id, name, desc, color): (Uuid, String, Option<String>, Option<String>)| on_start_edit(id, name, desc, color))}
                on_start_new_role={Callback::new(move |v: Uuid| on_start_new_role(v))}
                on_delete_org={Callback::new(move |v: Uuid| on_delete_org(v))}
            />

            <RoleContextMenu
                organization_store={organization_store}
                context_menu={role_context_menu}
                set_context_menu={set_role_context_menu}
                set_expanded_orgs={set_expanded_orgs}
                set_expanded_roles={set_expanded_roles}
                set_org_tab={Callback::new(move |(oid, tab): (Uuid, &'static str)| set_org_tab(oid, tab))}
                on_start_role_edit={Callback::new(move |(oid, role): (Uuid, OrgRole)| on_start_role_edit(oid, &role))}
                on_duplicate_role={Callback::new(move |v: (Uuid, Uuid)| on_duplicate_role(v.0, v.1))}
                on_delete_role={Callback::new(move |v: (Uuid, Uuid)| on_delete_role(v.0, v.1))}
            />
        </div>
    }
}
