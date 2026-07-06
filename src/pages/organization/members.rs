use crate::models::{OrgRole, User};
use crate::pages::organization::{member_card::MemberCard, role_display, role_from_str};
use crate::types::UserRole;
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub(crate) fn MembersSection(
    #[prop(into)] org_id: Uuid,
    #[prop(into)] can_edit: bool,
    #[prop(into)] members: Vec<User>,
    #[prop(into)] roles: Vec<OrgRole>,
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
    #[prop(into)] expanded_members: ReadSignal<HashSet<(Uuid, Uuid)>>,
    on_toggle_member: Callback<(Uuid, Uuid), ()>,
    #[prop(into)] portfolios: Vec<crate::models::Portfolio>,
) -> impl IntoView {
    let mems = members.clone();
    let indexed_mems = Memo::new(move |_| mems.iter().cloned().enumerate().collect::<Vec<_>>());
    view! {
        <div class="org-sub-tab-header">
            <span class="org-sub-tab-title">"Members"</span>
            {if can_edit {
                view! {
                    <button class="add-btn-small"
                        on:click=move |_| {
                            set_show_add_member.set(Some(org_id));
                        }>
                        "+ Member"
                    </button>
                }.into_any()
            } else { ().into_any() }}
        </div>

        // Add member inline form
        {move || show_add_member.get().filter(|&gp| gp == org_id).map(|_| view! {
            <div class="add-form" style="margin:0;border-radius:0;border-left:none;border-right:none;">
                <input class="login-input" type="text" placeholder="Name"
                    prop:value=move || member_name.get()
                    on:input=move |ev| set_member_name.set(event_target_value(&ev)) />
                <input class="login-input" type="email" placeholder="Email"
                    prop:value=move || member_email.get()
                    on:input=move |ev| set_member_email.set(event_target_value(&ev)) />
                <select class="login-input"
                    prop:value=move || role_display(&member_role.get())
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
                    <button class="login-btn" style="flex:1;" on:click=move |_| on_add_member.run(org_id)>"Add"</button>
                    <button class="view-btn" style="flex:1;" on:click=move |_| set_show_add_member.set(None)>"Cancel"</button>
                </div>
            </div>
        })}

        {if indexed_mems.get().is_empty() {
            view! { <div class="empty-state"><div class="empty-text">"No members."</div></div> }.into_any()
        } else {
            view! {
                <div class="org-member-list">
                <For
                    each=move || indexed_mems.get()
                    key=|(_, user)| user.id
                    children=move |(uidx, user)| {
                        let uid = user.id;
                        let is_exp = move || expanded_members.get().contains(&(org_id, uid));
                        let user_role_names: Vec<String> = roles.iter()
                            .filter(|r| r.member_ids.contains(&uid))
                            .map(|r| r.name.clone())
                            .collect();
                        let accessible_portfolios = portfolios.iter()
                            .filter(|p| p.assigned_users.contains(&uid))
                            .map(|p| p.name.clone())
                            .collect::<Vec<_>>();
                        let utint = format!("background: rgba(255,255,255,{:.1});", (uidx as f64 * 0.04).min(0.25));
                        view! {
                            <div style={utint}>
                                <MemberCard
                                    org_id=org_id
                                    user={user}
                                    can_edit=can_edit
                                    is_expanded={Signal::derive(is_exp)}
                                    on_toggle={Callback::new(move |v: (Uuid, Uuid)| on_toggle_member.run(v))}
                                    on_update_role={Callback::new(move |(id, role): (Uuid, UserRole)| on_update_member_role.run((id, role)))}
                                    on_remove={Callback::new(move |v: (Uuid, Uuid)| on_remove_member.run(v))}
                                    role_names=user_role_names
                                    accessible_portfolios=accessible_portfolios
                                />
                            </div>
                        }
                    }
                />
                </div>
            }.into_any()
        }}
    }
}
