use crate::components::rule_engine::RuleEngine;
use crate::models::{OrgRole, Perm, User};
use crate::pages::organization::RoleCard;
use crate::stores::use_organization_store;
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub(crate) fn RolesSection(
    #[prop(into)] org_id: Uuid,
    #[prop(into)] can_edit: bool,
    #[prop(into)] roles: Vec<OrgRole>,
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
    #[prop(into)] available_users: Vec<User>,
    on_role_context_menu: Callback<(i32, i32, Uuid, Uuid), ()>,
    #[prop(into)] on_empty_add_role: Callback<(), ()>,
) -> impl IntoView {
    let organization_store = use_organization_store();

    let mut sorted_roles = roles.clone();
    sorted_roles.sort_by(|a, b| b.rank.cmp(&a.rank));

    let sorted = sorted_roles.clone();
    let indexed_roles = Memo::new(move |_| sorted.iter().cloned().enumerate().collect::<Vec<_>>());
    let (dragging_role, set_dragging_role) = signal(Option::<Uuid>::None);

    view! {
        <div class="org-sub-tab-header">
            <span class="org-sub-tab-title">"Roles"</span>
            {if can_edit {
                view! {
                    <button class="add-btn-small"
                        on:click=move |_| on_start_new_role.run(org_id)>
                        "+ Role"
                    </button>
                }.into_any()
            } else { ().into_any() }}
        </div>
        {if indexed_roles.get().is_empty() {
            view! {
                <div class="empty-state org-section-empty"
                    on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                        if can_edit {
                            ev.prevent_default();
                            ev.stop_propagation();
                            on_empty_add_role.run(());
                        }
                    }>
                    <div class="empty-text">"No roles."</div>
                    {if can_edit {
                        view! {
                            <div class="org-section-empty-actions">
                                <button class="add-btn-small" on:click=move |_| on_empty_add_role.run(())>
                                    "+ Role"
                                </button>
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
            }.into_any()
        } else {
            view! {
                <div class="org-role-list">
                <For
                    each=move || indexed_roles.get()
                    key=|(_, role)| role.id
                    children=move |(ridx, role)| {
                let rid = role.id;
                let is_exp = move || expanded_roles.get().contains(&(org_id, rid));
                let rtint = format!("background: rgba(255,255,255,{:.1});", (ridx as f64 * 0.04).min(0.3));
                view! {
                    <div
                        style={rtint}
                        class="org-role-card-wrapper"
                        class:dragging={move || dragging_role.get() == Some(rid)}
                        draggable={can_edit}
                        on:dragstart=move |_| set_dragging_role.set(Some(rid))
                        on:dragover=move |ev| {
                            ev.prevent_default();
                        }
                        on:drop=move |ev| {
                            ev.prevent_default();
                            if let Some(dragged) = dragging_role.get() {
                                set_dragging_role.set(None);
                                if dragged != rid {
                                    organization_store.update(|s| s.drag_role(org_id, dragged, rid));
                                }
                            }
                        }
                    >
                        <RoleCard
                            org_id=org_id
                            role={role}
                            can_edit=can_edit
                            is_expanded={Signal::derive(is_exp)}
                            on_toggle=on_toggle_role
                            on_start_edit={Callback::new(move |v| on_start_role_edit.run(v))}
                            on_duplicate={Callback::new(move |v| on_duplicate_role.run(v))}
                            on_delete={Callback::new(move |v| on_delete_role.run(v))}
                            expanded_perm_groups=expanded_perm_groups
                            on_toggle_perm_group=on_toggle_perm_group
                            on_toggle_perm={Callback::new(move |v| on_toggle_role_perm.run(v))}
                            on_assign_member={Callback::new(move |v| on_assign_role_member.run(v))}
                            on_remove_member={Callback::new(move |v| on_remove_role_member.run(v))}
                            available_users=available_users.clone()
                            on_context_menu={Callback::new(move |v| on_role_context_menu.run(v))}
                        />
                    </div>
                }
            }
        />
                </div>
            }.into_any()
        }}
        <div class="org-rule-engine-wrap">
            <RuleEngine org_id={org_id} />
        </div>
    }
}
