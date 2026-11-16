use crate::stores::{use_app_store, use_organization_store};
use leptos::prelude::*;

#[component]
pub(crate) fn AccountSettings() -> impl IntoView {
    let app_store = use_app_store();
    let organization_store = use_organization_store();

    view! {
        <div class="data-card">
            <div class="card-header">
                <span class="card-title">"Account Settings"</span>
            </div>
            <div class="settings-list">
                <div class="account-current-user">
                    <div class="account-avatar">{move || app_store.with(|s| s.current_user.icon.clone().unwrap_or_else(|| s.current_user.name.chars().next().unwrap_or('U').to_string()))}</div>
                    <div class="account-user-info">
                        <div class="account-user-name">{move || app_store.with(|s| s.current_user.name.clone())}</div>
                        <div class="account-user-email">{move || app_store.with(|s| s.current_user.email.clone())}</div>
                        <div class="account-user-role">{move || app_store.with(|s| format!("{:?}", s.current_user.role))}</div>
                    </div>
                </div>
                <div class="account-icon-setting">
                    <span class="account-detail-label">"Profile Icon"</span>
                    <input
                        type="text"
                        class="account-icon-input"
                        placeholder="Emoji or icon..."
                        prop:value={move || app_store.with(|s| s.current_user.icon.clone().unwrap_or_default())}
                        on:input=move |ev| {
                            let v = event_target_value(&ev);
                            app_store.update(|s| s.current_user.icon = if v.trim().is_empty() { None } else { Some(v) });
                        }
                    />
                </div>
                <div class="account-details-grid">
                    <div class="account-detail-row">
                        <span class="account-detail-label">"Username"</span>
                        <span class="account-detail-value">{move || app_store.with(|s| s.current_user.name.clone())}</span>
                    </div>
                    <div class="account-detail-row">
                        <span class="account-detail-label">"Email"</span>
                        <span class="account-detail-value">{move || app_store.with(|s| s.current_user.email.clone())}</span>
                    </div>
                    <div class="account-detail-row">
                        <span class="account-detail-label">"Role"</span>
                        <span class="account-detail-value">{move || app_store.with(|s| format!("{:?}", s.current_user.role))}</span>
                    </div>
                    <div class="account-detail-row">
                        <span class="account-detail-label">"Organization"</span>
                        <span class="account-detail-value">{move || {
                            let org_id = app_store.with(|s| s.current_user.organization_id);
                            organization_store.with(|org| {
                                org_id
                                    .and_then(|oid| org.organizations.iter().find(|o| o.id == oid).map(|o| o.name.clone()))
                                    .unwrap_or_else(|| "None".to_string())
                            })
                        }}</span>
                    </div>
                    <div class="account-detail-row">
                        <span class="account-detail-label">"User ID"</span>
                        <span class="account-detail-value">{move || app_store.with(|s| s.current_user.id.to_string()[..8].to_string())}</span>
                    </div>
                    <div class="account-detail-row">
                        <span class="account-detail-label">"Member since"</span>
                        <span class="account-detail-value">{move || app_store.with(|s| s.current_user.created_at.format("%b %d, %Y").to_string())}</span>
                    </div>
                </div>
            </div>
            <div class="card-header" style="border-top: 2px solid var(--border-color); margin-top: 8px;">
                <span class="card-title">"Saved Profiles"</span>
            </div>
            <div class="settings-list">
                {move || {
                    let (creds, current_name) = app_store.with(|s| {
                        (s.credentials.credentials.clone(), s.current_user.name.clone())
                    });
                    if creds.is_empty() {
                        view! {
                            <div class="list-item">
                                <div class="list-item-left">
                                    <div class="list-item-subtitle">"No saved profiles"</div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        creds.values().map(|c| {
                            let is_current = c.display_name == current_name;
                            let validated_badge = if c.validated { "✓ Validated" } else { "⚠ Not validated" };
                            view! {
                                <div class="list-item" class:account-profile-active={is_current}>
                                    <div class="list-item-left">
                                        <div class="list-item-title">{c.display_name.clone()}</div>
                                        <div class="list-item-subtitle">{c.username.clone()} " · " {c.email.clone()}</div>
                                        <div class="list-item-subtitle">{validated_badge}</div>
                                    </div>
                                    <div class="list-item-right">
                                        {if is_current {
                                            view! { <span class="account-current-tag">"Current"</span> }.into_any()
                                        } else {
                                            ().into_any()
                                        }}
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>().into_any()
                    }
                }}
            </div>
        </div>
    }
}
