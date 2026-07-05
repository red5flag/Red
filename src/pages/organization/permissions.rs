use crate::models::{Perm, PermGroup};
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub(crate) fn PermissionGroups(
    #[prop(into)] org_id: Uuid,
    #[prop(into)] role_id: Uuid,
    #[prop(into)] current_permissions: Vec<Perm>,
    #[prop(into)] can_edit: bool,
    #[prop(into)] expanded_groups: ReadSignal<HashSet<(Uuid, Uuid, usize)>>,
    on_toggle_group: Callback<(Uuid, Uuid, usize), ()>,
    on_toggle_perm: Callback<(Uuid, Uuid, Perm), ()>,
) -> impl IntoView {
    view! {
        <div class="org-perm-groups">
            {PermGroup::all().iter().enumerate().map(|(gi, group)| {
                let group_label = group.label();
                let group_perms = Perm::for_group(group);
                let current_perms = current_permissions.clone();
                let group_exp = move || expanded_groups.get().contains(&(org_id, role_id, gi));

                view! {
                    <div class="org-perm-group" class:expanded=group_exp>
                        <div class="org-perm-group-header"
                            on:click=move |_| on_toggle_group.run((org_id, role_id, gi))>
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
                                            disabled={!can_edit}
                                            on:change=move |_| {
                                                if can_edit {
                                                    on_toggle_perm.run((org_id, role_id, p2.clone()));
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
    }
}
