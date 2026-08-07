use crate::models::{OrgRole, PermGroup};
use crate::pages::portfolios::{clamp_context_menu, read_image_as_data_url};
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
    #[prop(into)] image_url: ReadSignal<Option<String>>,
    #[prop(into)] set_image_url: WriteSignal<Option<String>>,
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

                <label class="org-edit-label">"Logo (optional)"</label>
                {move || image_url.get().map(|url| view! {
                    <img class="pf-header-image" src={url} alt="Organization logo preview" />
                })}
                <input
                    class="login-input apf-file-input"
                    type="file"
                    accept="image/*"
                    on:change=move |ev| read_image_as_data_url(&ev, move |url| set_image_url.set(Some(url))) />

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
pub(crate) fn OrgOptionsPanel(
    org_id: Uuid,
    #[prop(into)] set_actions_open: WriteSignal<bool>,
    #[prop(into)] set_show_add_member: WriteSignal<Option<Uuid>>,
    #[prop(into)] set_expanded_orgs: WriteSignal<HashSet<Uuid>>,
    set_org_tab: Callback<(Uuid, &'static str), ()>,
    on_start_new_role: Callback<Uuid, ()>,
    on_delete_org: Callback<Uuid, ()>,
    on_add_image: Callback<Uuid, ()>,
    on_add_org: Callback<(), ()>,
) -> impl IntoView {
    let (expanded_submenu, set_expanded_submenu) = signal(Option::<&'static str>::None);
    view! {
        <div class="org-options-panel" on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()>
            <button class="context-menu-item org-options-item"
                on:click=move |_| {
                    set_actions_open.set(false);
                    set_expanded_submenu.set(None);
                    on_add_image.run(org_id);
                }>
                "\u{1F5BC} Add Image"
            </button>
            <button class="context-menu-item org-options-item"
                on:click=move |_| set_expanded_submenu.update(|v| *v = if *v == Some("add_member") { None } else { Some("add_member") })>
                {move || if expanded_submenu.get() == Some("add_member") { "\u{25BC} Add Member" } else { "\u{25B6} Add Member" }}
            </button>
            {move || if expanded_submenu.get() == Some("add_member") { view! {
                <div class="org-options-submenu">
                    <button class="context-menu-item org-options-item org-options-sub"
                        on:click=move |_| {
                            set_actions_open.set(false);
                            set_expanded_submenu.set(None);
                            set_show_add_member.set(Some(org_id));
                            set_expanded_orgs.update(|s| { s.insert(org_id); });
                            set_org_tab.run((org_id, "members"));
                        }>
                        "Add member"
                    </button>
                    <button class="context-menu-item org-options-item org-options-sub"
                        on:click=move |_| {
                            set_actions_open.set(false);
                            set_expanded_submenu.set(None);
                            set_expanded_orgs.update(|s| { s.insert(org_id); });
                            set_org_tab.run((org_id, "roles"));
                        }>
                        "Add member to role"
                    </button>
                </div>
            }.into_any() } else { ().into_any() }}
            <button class="context-menu-item org-options-item"
                on:click=move |_| set_expanded_submenu.update(|v| *v = if *v == Some("create_role") { None } else { Some("create_role") })>
                {move || if expanded_submenu.get() == Some("create_role") { "\u{25BC} Create Role" } else { "\u{25B6} Create Role" }}
            </button>
            {move || if expanded_submenu.get() == Some("create_role") { view! {
                <div class="org-options-submenu">
                    <button class="context-menu-item org-options-item org-options-sub"
                        on:click=move |_| {
                            set_actions_open.set(false);
                            set_expanded_submenu.set(None);
                            on_start_new_role.run(org_id);
                            set_expanded_orgs.update(|s| { s.insert(org_id); });
                            set_org_tab.run((org_id, "roles"));
                        }>
                        "Create New Role"
                    </button>
                    <button class="context-menu-item org-options-item org-options-sub"
                        on:click=move |_| {
                            set_actions_open.set(false);
                            set_expanded_submenu.set(None);
                            set_expanded_orgs.update(|s| { s.insert(org_id); });
                            set_org_tab.run((org_id, "roles"));
                        }>
                        "Create role from duplicate"
                    </button>
                </div>
            }.into_any() } else { ().into_any() }}
            <button class="context-menu-item org-options-item"
                on:click=move |_| set_expanded_submenu.update(|v| *v = if *v == Some("add_org") { None } else { Some("add_org") })>
                {move || if expanded_submenu.get() == Some("add_org") { "\u{25BC} Add Organization" } else { "\u{25B6} Add Organization" }}
            </button>
            {move || if expanded_submenu.get() == Some("add_org") { view! {
                <div class="org-options-submenu">
                    <button class="context-menu-item org-options-item org-options-sub"
                        on:click=move |_| {
                            set_actions_open.set(false);
                            set_expanded_submenu.set(None);
                            on_add_org.run(());
                        }>
                        "Add Organization"
                    </button>
                    <button class="context-menu-item org-options-item org-options-sub"
                        on:click=move |_| {
                            set_actions_open.set(false);
                            set_expanded_submenu.set(None);
                            on_add_org.run(());
                        }>
                        "Add new organization"
                    </button>
                    <button class="context-menu-item org-options-item org-options-sub" disabled>
                        "Add Organization from network"
                    </button>
                </div>
            }.into_any() } else { ().into_any() }}
            <button class="context-menu-item org-options-item danger"
                on:click=move |_| {
                    set_actions_open.set(false);
                    set_expanded_submenu.set(None);
                    on_delete_org.run(org_id);
                }>
                "\u{1F5D1} Delete Organization"
            </button>
        </div>
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
            let (x, y) = clamp_context_menu(x, y);
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
