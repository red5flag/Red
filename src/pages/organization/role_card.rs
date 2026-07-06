use crate::models::{OrgRole, Perm, User};
use crate::pages::organization::{scope_display, PermissionGroups};
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub(crate) fn RoleCard(
    #[prop(into)] org_id: Uuid,
    role: OrgRole,
    #[prop(into)] can_edit: bool,
    #[prop(into)] is_expanded: Signal<bool>,
    on_toggle: Callback<(Uuid, Uuid), ()>,
    on_start_edit: Callback<(Uuid, OrgRole), ()>,
    on_duplicate: Callback<(Uuid, Uuid), ()>,
    on_delete: Callback<(Uuid, Uuid), ()>,
    #[prop(into)] expanded_perm_groups: ReadSignal<HashSet<(Uuid, Uuid, usize)>>,
    on_toggle_perm_group: Callback<(Uuid, Uuid, usize), ()>,
    on_toggle_perm: Callback<(Uuid, Uuid, Perm), ()>,
    on_assign_member: Callback<(Uuid, Uuid, Uuid), ()>,
    on_remove_member: Callback<(Uuid, Uuid, Uuid), ()>,
    #[prop(into)] available_users: Vec<User>,
    on_context_menu: Callback<(i32, i32, Uuid, Uuid), ()>,
) -> impl IntoView {
    let rid = role.id;
    let role_summary = role.summary();
    let role_name = role.name.clone();
    let role_desc = role.description.clone();
    let role_scope = role.scope.clone();
    let role_rank = role.rank;
    let role_color = role.color.clone();
    let role_perms = role.permissions.clone();
    let role_members = role.member_ids.clone();
    let is_system = role.is_system;
    let role_name_for_dup = role.name.clone();
    let role_name_for_delete = role.name.clone();

    let role_color_style = role_color
        .as_ref()
        .map(|c| format!("border-left: 4px solid {};", c))
        .unwrap_or_default();

    let edit_role = OrgRole {
        id: rid,
        name: role_name.clone(),
        rank: role_rank,
        color: role_color.clone(),
        description: role_desc.clone(),
        scope: role_scope.clone(),
        permissions: role_perms.clone(),
        member_ids: role_members.clone(),
        documents: Vec::new(),
        is_system,
    };

    view! {
        <div class="org-role-card" class:expanded=is_expanded style={role_color_style}
            role="region"
            aria-label={role_summary.clone()}>
            // Role header
            <div class="org-role-header"
                on:click=move |_| on_toggle.run((org_id, rid))
                on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                    ev.prevent_default();
                    ev.stop_propagation();
                    on_context_menu.run((ev.client_x(), ev.client_y(), org_id, rid));
                }>
                <span class="org-role-arrow">
                    {move || if is_expanded.get() { "\u{25B2}" } else { "\u{25BC}" }}
                </span>
                <div class="org-role-info">
                    <div class="org-role-name">{role_name_for_dup.clone()}</div>
                    <div class="org-role-meta">
                        {format!("Rank {} \u{00B7} {} \u{00B7} {} members",
                            role_rank,
                            scope_display(&role_scope),
                            role_members.len())}
                    </div>
                </div>
                {if can_edit {
                    view! {
                        <div class="org-role-actions" on:click=|ev| ev.stop_propagation()>
                            <button class="org-role-btn"
                                aria-label={format!("Edit {} role", role_name)}
                                on:click=move |_| {
                                    on_start_edit.run((org_id, edit_role.clone()));
                                }>
                                "Edit"
                            </button>
                            <button class="org-role-btn"
                                aria-label={format!("Duplicate {} role", role_name_for_dup)}
                                on:click=move |_| on_duplicate.run((org_id, rid))>
                                "Duplicate"
                            </button>
                            {if !is_system {
                                view! {
                                    <button class="org-role-btn org-role-btn-danger"
                                        aria-label={format!("Delete {} role", role_name_for_delete)}
                                        on:click=move |_| on_delete.run((org_id, rid))>
                                        "Delete"
                                    </button>
                                }.into_any()
                            } else { ().into_any() }}
                        </div>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            // Role expanded content
            <div class="org-role-content" class:hidden={move || !is_expanded.get()}>
                // Plain-English summary
                <div class="org-role-summary">
                    <div class="org-role-summary-label">"Summary"</div>
                    <div class="org-role-summary-text">{role_desc.clone()}</div>
                </div>

                // Permission groups
                <PermissionGroups
                    org_id=org_id
                    role_id=rid
                    current_permissions=role_perms.clone()
                    can_edit=can_edit
                    expanded_groups=expanded_perm_groups
                    on_toggle_group=on_toggle_perm_group
                    on_toggle_perm=on_toggle_perm
                />

                // Role members
                <div class="org-role-members-section">
                    <div class="org-role-members-label">
                        {format!("Members ({}):", role_members.len())}
                    </div>
                    {if role_members.is_empty() {
                        view! { <div class="empty-state"><div class="empty-text">"No members assigned to this role."</div></div> }.into_any()
                    } else {
                        let role_users: Vec<User> = available_users.iter()
                            .filter(|u| role_members.contains(&u.id))
                            .cloned().collect();
                        let role_users_for = role_users.clone();
                        let role_users_memo = Memo::new(move |_| role_users_for.clone());
                        view! {
                            <div class="org-role-members-list">
                            <For
                                each=move || role_users_memo.get()
                                key=|user| user.id
                                children=move |user| {
                                    let uid = user.id;
                                    let uname = user.name.clone();
                                    view! {
                                        <div class="org-role-member-chip">
                                            <span class="org-member-avatar-sm">
                                                {user.name.chars().next().unwrap_or('?').to_uppercase().to_string()}
                                            </span>
                                            <span>{uname}</span>
                                            {if can_edit {
                                                view! {
                                                    <button class="org-role-btn org-role-btn-sm org-role-btn-danger"
                                                        aria-label={format!("Remove {} from role", user.name)}
                                                        on:click=move |_| on_remove_member.run((org_id, rid, uid))>
                                                        "\u{2715}"
                                                    </button>
                                                }.into_any()
                                            } else { ().into_any() }}
                                        </div>
                                    }
                                }
                            />
                            </div>
                        }.into_any()
                    }}

                    // Add member to role
                    {if can_edit {
                        let unassigned: Vec<User> = available_users.into_iter()
                            .filter(|u| !role_members.contains(&u.id))
                            .collect();
                        let unassigned_for = unassigned.clone();
                        let unassigned_memo = Memo::new(move |_| unassigned_for.clone());
                        view! {
                            <div class="org-role-add-member">
                                <select class="login-input org-role-member-select"
                                    on:change=move |ev| {
                                        let v = event_target_value(&ev);
                                        if let Ok(uid) = Uuid::parse_str(&v) {
                                            on_assign_member.run((org_id, rid, uid));
                                        }
                                    }>
                                    <option value="">"+ Assign member to role"</option>
                                    <For
                                        each=move || unassigned_memo.get()
                                        key=|u| u.id
                                        children=move |u| view! {
                                            <option value={u.id.to_string()}>{u.name.clone()}</option>
                                        }
                                    />
                                </select>
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
            </div>
        </div>
    }
}
