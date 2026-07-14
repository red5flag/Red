use crate::models::{OrgRole, PermGroup};
use crate::stores::OrganizationStore;
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub(crate) fn AddOrgForm(
    #[prop(into)] show: ReadSignal<bool>,
    #[prop(into)] _set_show: WriteSignal<bool>,
    #[prop(into)] name: ReadSignal<String>,
    #[prop(into)] set_name: WriteSignal<String>,
    #[prop(into)] desc: ReadSignal<String>,
    #[prop(into)] set_desc: WriteSignal<String>,
    #[prop(into)] abn: ReadSignal<String>,
    #[prop(into)] set_abn: WriteSignal<String>,
    #[prop(into)] lei: ReadSignal<String>,
    #[prop(into)] set_lei: WriteSignal<String>,
    #[prop(into)] business_type: ReadSignal<String>,
    #[prop(into)] set_business_type: WriteSignal<String>,
    #[prop(into)] business_address: ReadSignal<String>,
    #[prop(into)] set_business_address: WriteSignal<String>,
    #[prop(into)] business_phone: ReadSignal<String>,
    #[prop(into)] set_business_phone: WriteSignal<String>,
    #[prop(into)] business_email: ReadSignal<String>,
    #[prop(into)] set_business_email: WriteSignal<String>,
    on_add: Callback<(), ()>,
) -> impl IntoView {
    view! {
        {move || show.get().then(|| view! {
            <div class="add-form">
                <input class="login-input" type="text" placeholder="Organization name"
                    prop:value=move || name.get()
                    on:input=move |ev| set_name.set(event_target_value(&ev)) />
                <input class="login-input" type="text" placeholder="Description (optional)"
                    prop:value=move || desc.get()
                    on:input=move |ev| set_desc.set(event_target_value(&ev)) />

                <div class="org-business-section">
                    <div class="org-section-title">"Business Details (Optional)"</div>
                    <input class="login-input" type="text" placeholder="ABN (Australian Business Number)"
                        prop:value=move || abn.get()
                        on:input=move |ev| set_abn.set(event_target_value(&ev)) />
                    <input class="login-input" type="text" placeholder="LEI (Legal Entity Identifier)"
                        prop:value=move || lei.get()
                        on:input=move |ev| set_lei.set(event_target_value(&ev)) />
                    <select class="login-input"
                        prop:value=move || business_type.get()
                        on:change=move |ev| set_business_type.set(event_target_value(&ev))>
                        <option value="">"Business Type"</option>
                        <option value="Sole Trader">"Sole Trader"</option>
                        <option value="Company">"Company"</option>
                        <option value="Partnership">"Partnership"</option>
                        <option value="Trust">"Trust"</option>
                        <option value="Non-profit">"Non-profit"</option>
                        <option value="Government">"Government"</option>
                    </select>
                    <input class="login-input" type="text" placeholder="Business Address"
                        prop:value=move || business_address.get()
                        on:input=move |ev| set_business_address.set(event_target_value(&ev)) />
                    <input class="login-input" type="text" placeholder="Business Phone"
                        prop:value=move || business_phone.get()
                        on:input=move |ev| set_business_phone.set(event_target_value(&ev)) />
                    <input class="login-input" type="text" placeholder="Business Email"
                        prop:value=move || business_email.get()
                        on:input=move |ev| set_business_email.set(event_target_value(&ev)) />
                </div>

                <button class="login-btn" on:click=move |_| on_add.run(())>"Create Organization"</button>
            </div>
        })}
    }
}

#[component]
pub(crate) fn RoleEditorModal(
    #[prop(into)] editing_role: ReadSignal<Option<(Uuid, Uuid)>>,
    #[prop(into)] set_editing_role: WriteSignal<Option<(Uuid, Uuid)>>,
    #[prop(into)] edit_role_name: ReadSignal<String>,
    #[prop(into)] set_edit_role_name: WriteSignal<String>,
    #[prop(into)] edit_role_desc: ReadSignal<String>,
    #[prop(into)] set_edit_role_desc: WriteSignal<String>,
    #[prop(into)] edit_role_rank: ReadSignal<u32>,
    #[prop(into)] set_edit_role_rank: WriteSignal<u32>,
    #[prop(into)] edit_role_color: ReadSignal<String>,
    #[prop(into)] set_edit_role_color: WriteSignal<String>,
    #[prop(into)] edit_role_scope: ReadSignal<crate::models::RoleScope>,
    #[prop(into)] set_edit_role_scope: WriteSignal<crate::models::RoleScope>,
    on_save_role: Callback<(), ()>,
) -> impl IntoView {
    view! {
        {move || editing_role.get().map(|(_oid, rid)| {
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

                                <label class="org-edit-label">"Rank (drag roles to reorder; edit as a secondary control)"</label>
                                <input class="login-input" type="number" min="0" max="10000"
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
                                <div class="org-scope-matrix">
                                    <div class="org-scope-matrix-header">
                                        <span class="org-scope-matrix-label">"Area"</span>
                                        <span class="org-scope-matrix-label">"View"</span>
                                        <span class="org-scope-matrix-label">"Edit"</span>
                                    </div>
                                    {PermGroup::all().iter().map(|group| {
                                        let group = *group;
                                        let group_label = group.label();
                                        view! {
                                            <div class="org-scope-matrix-row">
                                                <span class="org-scope-matrix-name">{group_label}</span>
                                                <label class="org-scope-matrix-cell">
                                                    <input
                                                        type="checkbox"
                                                        checked={move || edit_role_scope.get().view(group)}
                                                        on:change=move |_| set_edit_role_scope.update(|s| s.toggle_view(group))
                                                    />
                                                </label>
                                                <label class="org-scope-matrix-cell">
                                                    <input
                                                        type="checkbox"
                                                        checked={move || edit_role_scope.get().edit(group)}
                                                        on:change=move |_| set_edit_role_scope.update(|s| s.toggle_edit(group))
                                                    />
                                                </label>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>

                                <div style="display:flex;gap:8px;margin-top:12px;">
                                    <button class="login-btn" style="flex:1;" on:click=move |_| on_save_role.run(())>"Save Role"</button>
                                    <button class="view-btn" style="flex:1;" on:click=move |_| set_editing_role.set(None)>"Cancel"</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            }.into_any()
        })}
    }
}

#[component]
pub(crate) fn OrgContextMenu(
    organization_store: RwSignal<OrganizationStore>,
    #[prop(into)] context_menu: ReadSignal<Option<(i32, i32, Uuid)>>,
    #[prop(into)] set_context_menu: WriteSignal<Option<(i32, i32, Uuid)>>,
    #[prop(into)] set_show_add_org: WriteSignal<bool>,
    #[prop(into)] set_show_add_member: WriteSignal<Option<Uuid>>,
    #[prop(into)] set_expanded_orgs: WriteSignal<HashSet<Uuid>>,
    set_org_tab: Callback<(Uuid, &'static str), ()>,
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
        ),
        (),
    >,
    on_start_new_role: Callback<Uuid, ()>,
    on_delete_org: Callback<Uuid, ()>,
) -> impl IntoView {
    view! {
        {move || context_menu.get().map(|(x, y, id)| {
            let org = organization_store.get().organizations.iter().find(|o| o.id == id).cloned();
            let name = org.as_ref().map(|o| o.name.clone()).unwrap_or_default();
            let desc = org.as_ref().and_then(|o| o.description.clone());
            let color = org.as_ref().and_then(|o| o.settings.color.clone());
            let abn = org.as_ref().and_then(|o| o.abn.clone());
            let lei = org.as_ref().and_then(|o| o.lei.clone());
            let business_type = org.as_ref().and_then(|o| o.business_type.clone());
            let business_address = org.as_ref().and_then(|o| o.business_address.clone());
            let business_phone = org.as_ref().and_then(|o| o.business_phone.clone());
            let business_email = org.as_ref().and_then(|o| o.business_email.clone());
            let name_for_edit = name.clone();
            view! {
                <div class="context-menu-overlay" on:click=move |_| set_context_menu.set(None)>
                    <div class="context-menu org-context-menu" style={format!("left: {}px; top: {}px;", x, y)}>
                        <button class="context-menu-item org-context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                set_expanded_orgs.update(|s| { s.insert(id); });
                            }>
                            "\u{25BC} Open Organization"
                        </button>
                        <button class="context-menu-item org-context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                on_start_edit.run((id, name_for_edit.clone(), desc.clone(), color.clone(), abn.clone(), lei.clone(), business_type.clone(), business_address.clone(), business_phone.clone(), business_email.clone()));
                            }>"\u{270E} Edit Organization"</button>
                        <button class="context-menu-item org-context-menu-item" disabled>
                            "\u{2795} Assign Portfolio (coming soon)"
                        </button>
                        <button class="context-menu-item org-context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                set_show_add_member.set(Some(id));
                                set_expanded_orgs.update(|s| { s.insert(id); });
                                set_org_tab.run((id, "members"));
                            }>
                            "\u{2795} Add Member"
                        </button>
                        <button class="context-menu-item org-context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                on_start_new_role.run(id);
                                set_expanded_orgs.update(|s| { s.insert(id); });
                                set_org_tab.run((id, "roles"));
                            }>
                            "\u{2795} Create Role"
                        </button>
                        <button class="context-menu-item org-context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                set_show_add_org.set(true);
                            }>
                            "\u{2795} Add Organization"
                        </button>
                        <button class="context-menu-item org-context-menu-item danger"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                on_delete_org.run(id);
                            }>
                            "\u{1F5D1} Delete Organization"
                        </button>
                    </div>
                </div>
            }.into_any()
        })}
    }
}

#[component]
pub(crate) fn RoleContextMenu(
    organization_store: RwSignal<OrganizationStore>,
    #[prop(into)] context_menu: ReadSignal<Option<(i32, i32, Uuid, Uuid)>>,
    #[prop(into)] set_context_menu: WriteSignal<Option<(i32, i32, Uuid, Uuid)>>,
    #[prop(into)] set_expanded_orgs: WriteSignal<HashSet<Uuid>>,
    #[prop(into)] set_expanded_roles: WriteSignal<HashSet<(Uuid, Uuid)>>,
    set_org_tab: Callback<(Uuid, &'static str), ()>,
    on_start_role_edit: Callback<(Uuid, OrgRole), ()>,
    on_duplicate_role: Callback<(Uuid, Uuid), ()>,
    on_delete_role: Callback<(Uuid, Uuid), ()>,
) -> impl IntoView {
    view! {
        {move || context_menu.get().map(|(x, y, oid, rid)| {
            let role = organization_store.get().organizations.iter()
                .find(|o| o.id == oid)
                .and_then(|o| o.roles.iter().find(|r| r.id == rid))
                .cloned();
            let role_for_edit = role.clone();
            let is_system = role.as_ref().map(|r| r.is_system).unwrap_or(false);
            view! {
                <div class="context-menu-overlay" on:click=move |_| set_context_menu.set(None)>
                    <div class="context-menu" style={format!("left: {}px; top: {}px;", x, y)}>
                        <button class="context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                if let Some(r) = role_for_edit.clone() {
                                    on_start_role_edit.run((oid, r));
                                }
                            }>"\u{270E} Edit Role"</button>
                        <button class="context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                on_duplicate_role.run((oid, rid));
                            }>"\u{1F4CB} Duplicate Role"</button>
                        <button class="context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                set_expanded_orgs.update(|s| { s.insert(oid); });
                                set_org_tab.run((oid, "roles"));
                                set_expanded_roles.update(|s| { s.insert((oid, rid)); });
                            }>"\u{25BC} Expand Role"</button>
                        {if !is_system {
                            view! {
                                <button class="context-menu-item danger"
                                    on:click=move |_| {
                                        set_context_menu.set(None);
                                        on_delete_role.run((oid, rid));
                                    }>
                                    "\u{1F5D1} Delete Role"
                                </button>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                </div>
            }.into_any()
        })}
    }
}
