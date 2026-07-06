use crate::models::{OrgRole, Organization, Perm};
use crate::pages::organization::organization_card::OrganizationCard;
use crate::stores::{AppStore, OrganizationStore};
use crate::types::UserRole;
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub(crate) fn OrganizationList(
    app_store: RwSignal<AppStore>,
    organization_store: RwSignal<OrganizationStore>,
    ui_store: RwSignal<crate::stores::UiStore>,
    #[prop(into)] organizations: Vec<Organization>,
    #[prop(into)] editing_org: ReadSignal<Option<Uuid>>,
    #[prop(into)] expanded_orgs: ReadSignal<HashSet<Uuid>>,
    on_toggle_org: Callback<Uuid, ()>,
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
    let orgs = organizations.clone();
    let orgs_memo = Memo::new(move |_| orgs.clone());
    view! {
        {if orgs_memo.get().is_empty() {
            view! {
                <div class="data-card">
                    <div class="empty-state"><div class="empty-text">"No organizations yet."</div></div>
                </div>
            }.into_any()
        } else {
            view! {
                <div class="pf-accordion">
                <For
                    each=move || orgs_memo.get()
                    key=|org| org.id
                    children=move |org| {
                        let oid = org.id;
                        let can = organization_store.get().current_user_role_in_org(oid, app_store.get().current_user.id, app_store.get().current_user.role.clone());
                        let can_manage = matches!(
                            can,
                            UserRole::Owner | UserRole::Director | UserRole::SeniorManager | UserRole::Manager
                        );
                        let is_exp = move || expanded_orgs.get().contains(&oid);
                        let is_editing = editing_org.get() == Some(oid);
                        view! {
                            <OrganizationCard
                                app_store={app_store}
                                organization_store={organization_store}
                                ui_store={ui_store}
                                org={org}
                                can_manage={can_manage}
                                is_expanded={Signal::derive(is_exp)}
                                on_toggle_org={on_toggle_org}
                                is_editing={is_editing}
                                edit_name={edit_name}
                                set_edit_name={set_edit_name}
                                edit_desc={edit_desc}
                                set_edit_desc={set_edit_desc}
                                edit_color={edit_color}
                                set_edit_color={set_edit_color}
                                set_editing_org={set_editing_org}
                                on_start_edit={on_start_edit}
                                on_save_edit={on_save_edit}
                                on_delete_org={on_delete_org}
                                get_org_tab={get_org_tab}
                                set_org_tab={set_org_tab}
                                expanded_roles={expanded_roles}
                                on_toggle_role={on_toggle_role}
                                on_start_new_role={on_start_new_role}
                                on_start_role_edit={on_start_role_edit}
                                on_duplicate_role={on_duplicate_role}
                                on_delete_role={on_delete_role}
                                expanded_perm_groups={expanded_perm_groups}
                                on_toggle_perm_group={on_toggle_perm_group}
                                on_toggle_role_perm={on_toggle_role_perm}
                                on_assign_role_member={on_assign_role_member}
                                on_remove_role_member={on_remove_role_member}
                                expanded_members={expanded_members}
                                on_toggle_member={on_toggle_member}
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
                                on_context_menu={on_context_menu}
                                on_role_context_menu={on_role_context_menu}
                            />
                        }
                    }
                />
                </div>
            }.into_any()
        }}
    }
}
