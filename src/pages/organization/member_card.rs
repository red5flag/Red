use crate::models::User;
use crate::pages::organization::{permission_label, role_display};
use crate::types::UserRole;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub(crate) fn MemberCard(
    #[prop(into)] org_id: Uuid,
    user: User,
    #[prop(into)] can_edit: bool,
    #[prop(into)] is_expanded: Signal<bool>,
    on_toggle: Callback<(Uuid, Uuid), ()>,
    on_update_role: Callback<(Uuid, UserRole), ()>,
    on_remove: Callback<(Uuid, Uuid), ()>,
    #[prop(into)] is_role_dropdown_open: Signal<bool>,
    #[prop(into)] on_toggle_role_dropdown: Callback<Uuid, ()>,
    #[prop(into)] role_names: Vec<String>,
    #[prop(into)] accessible_portfolios: Vec<String>,
) -> impl IntoView {
    let uid = user.id;
    let uid_del = user.id;
    let uid_role = user.id;
    let user_role = user.role.clone();
    let user_name = user.name.clone();
    let user_email = user.email.clone();
    let user_perms = user.permissions.clone();
    let user_role_display = role_display(&user.role);
    let user_address = user.address.clone();
    let user_department = user.department.clone();
    let user_phone = user.phone.clone();
    let user_assignments = user.assignments.clone();
    let user_active = user.is_active;
    let utint = format!(
        "background: rgba(255,255,255,{:.1});",
        (uid.as_u128() as f64 * 0.000000000000000000000000000004).min(0.25)
    );

    view! {
        <div class="org-member-card" class:expanded=is_expanded style={utint}>
            <div class="org-member-header"
                on:click=move |_| on_toggle.run((org_id, uid))>
                <span class="org-member-arrow">
                    {move || if is_expanded.get() { "\u{25B2}" } else { "\u{25BC}" }}
                </span>
                <div class="org-member-avatar-sm">
                    {user_name.chars().next().unwrap_or('?').to_uppercase().to_string()}
                </div>
                <div class="org-member-info">
                    <div class="org-member-name">{user_name.clone()}</div>
                    <div class="org-member-meta">
                        {format!("{} \u{00B7} {} \u{00B7} {} portfolio{}",
                            user_role_display,
                            user_email,
                            accessible_portfolios.len(),
                            if accessible_portfolios.len() == 1 { "" } else { "s" })}
                    </div>
                </div>
                {if can_edit {
                    view! {
                        <div class="org-member-actions" on:click=|ev| ev.stop_propagation()>
                            <div class="org-role-dropdown-wrap">
                                <button
                                    class="org-role-dropdown-toggle"
                                    title={user_role_display.to_string()}
                                    aria-label={format!("Current role: {}. Click to change.", user_role_display)}
                                    on:click=move |_| on_toggle_role_dropdown.run(uid)>
                                    {user_role_display}
                                </button>
                                {move || if is_role_dropdown_open.get() { view! {
                                    <div class="org-role-dropdown" on:click=|ev| ev.stop_propagation()>
                                        {[UserRole::Owner, UserRole::Director, UserRole::SeniorManager, UserRole::Manager, UserRole::Worker, UserRole::DocumentWorker, UserRole::Contractor, UserRole::Guest]
                                            .into_iter()
                                            .map(|role| {
                                                let r_click = role.clone();
                                                let label = role_display(&role);
                                                view! {
                                                    <button
                                                        class="org-role-dropdown-option"
                                                        class:org-role-dropdown-active={user_role == role}
                                                        on:click=move |_| {
                                                            on_update_role.run((uid_role, r_click));
                                                            on_toggle_role_dropdown.run(uid);
                                                        }
                                                    >
                                                        {label}
                                                    </button>
                                                }
                                            }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any() } else { ().into_any() }}
                            </div>
                            <button class="pf-action-btn org-member-remove"
                                aria-label={format!("Remove {} from organization", user_name)}
                                on:click=move |_| on_remove.run((org_id, uid_del))>
                                "\u{2715}"
                            </button>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="pf-list-actions">
                            <span class="org-role-badge">{user_role_display}</span>
                        </div>
                    }.into_any()
                }}
            </div>

            // Member expanded content
            <div class="org-member-content" class:hidden={move || !is_expanded.get()}>
                // Location & contact info
                <div class="org-member-location">
                    <span class="org-member-section-label">"Location & Contact:"</span>
                    <div class="org-member-detail-grid">
                        {user_address.as_ref().map(|a| view! {
                            <div class="org-member-detail-row">
                                <span class="org-member-detail-key">"Address"</span>
                                <span class="org-member-detail-val">{a.clone()}</span>
                            </div>
                        })}
                        {user_department.as_ref().map(|d| view! {
                            <div class="org-member-detail-row">
                                <span class="org-member-detail-key">"Department"</span>
                                <span class="org-member-detail-val">{d.clone()}</span>
                            </div>
                        })}
                        {user_phone.as_ref().map(|p| view! {
                            <div class="org-member-detail-row">
                                <span class="org-member-detail-key">"Phone"</span>
                                <span class="org-member-detail-val">{p.clone()}</span>
                            </div>
                        })}
                        <div class="org-member-detail-row">
                            <span class="org-member-detail-key">"Status"</span>
                            <span class="org-member-detail-val">{if user_active { "Active" } else { "Inactive" }}</span>
                        </div>
                    </div>
                </div>

                // Discord-style roles
                {if !role_names.is_empty() {
                    view! {
                        <div class="org-member-roles">
                            <span class="org-member-section-label">"Roles:"</span>
                            {role_names.iter().map(|rn| view! {
                                <span class="org-role-badge">{rn.clone()}</span>
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                // Accessible portfolios
                {if !accessible_portfolios.is_empty() {
                    view! {
                        <div class="org-member-portfolios">
                            <span class="org-member-section-label">"Portfolio access:"</span>
                            {accessible_portfolios.iter().map(|pn| view! {
                                <span class="org-portfolio-badge">{pn.clone()}</span>
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                // Permissions list
                {if !user_perms.is_empty() {
                    view! {
                        <div class="org-member-permissions">
                            <span class="org-member-section-label">{format!("Permissions ({}):", user_perms.len())}</span>
                            <div class="org-member-perm-list">
                                {user_perms.iter().map(|p| view! {
                                    <span class="org-perm-badge">{permission_label(p)}</span>
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                // Assignments
                {if !user_assignments.is_empty() {
                    view! {
                        <div class="org-member-assignments">
                            <span class="org-member-section-label">{format!("Assignments ({}):", user_assignments.len())}</span>
                            <div class="org-member-assignment-list">
                                {user_assignments.iter().map(|a| view! {
                                    <div class="org-member-assignment-row">
                                        <span class="org-member-assignment-name">{a.target_name.clone()}</span>
                                        <span class="org-member-assignment-type">{a.target_type.clone()}</span>
                                    </div>
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_any()
                } else { ().into_any() }}
            </div>
        </div>
    }
}
