use crate::stores::use_app_store;
use leptos::prelude::*;

#[component]
pub(crate) fn CredentialControls(
    show_profiles: ReadSignal<bool>,
    set_show_profiles: WriteSignal<bool>,
    set_username: WriteSignal<String>,
    set_password: WriteSignal<String>,
    login_store_local: ReadSignal<bool>,
    set_login_store_local: WriteSignal<bool>,
    login_store_cloud: ReadSignal<bool>,
    set_login_store_cloud: WriteSignal<bool>,
    set_profile_2fa: WriteSignal<Option<String>>,
    set_totp_stub_code: WriteSignal<String>,
) -> impl IntoView {
    let app_store = use_app_store();

    view! {
        // ── SAVED PROFILES DROPDOWN ──
        <div class="lp-profiles-dropdown">
            <button class="lp-profiles-toggle" on:click=move |_| set_show_profiles.update(|v| *v = !*v)>
                <span class="lp-profiles-icon">"👤"</span>
                <span class="lp-profiles-label">"Saved Profiles"</span>
                <span class="lp-profiles-arrow">{move || if show_profiles.get() { "▲" } else { "▼" }}</span>
            </button>
            {move || if show_profiles.get() {
                let creds = app_store.get().credentials.credentials.clone();
                let profiles: Vec<_> = creds.iter().collect::<Vec<_>>();
                if profiles.is_empty() {
                    view! {
                        <div class="lp-profiles-menu">
                            <div class="lp-profiles-empty">"No saved profiles"</div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="lp-profiles-menu">
                            {profiles.into_iter().map(|(uname, cred)| {
                                let uname_clone = uname.clone();
                                let display_name = cred.display_name.clone();
                                let _email = cred.email.clone();
                                let validated = cred.validated;
                                let uname_for_click = uname.clone();
                                let remembered_password = cred.remembered_password.clone();
                                view! {
                                    <button class="lp-profile-item"
                                        on:click=move |_| {
                                            set_username.set(uname_for_click.clone());
                                            set_password.set(remembered_password.clone().unwrap_or_default());
                                            set_show_profiles.set(false);
                                            set_totp_stub_code.set(String::new());
                                            set_profile_2fa.set(Some(uname_for_click.clone()));
                                        }>
                                        <div class="lp-profile-item-info">
                                            <div class="lp-profile-item-name">{display_name.clone()}</div>
                                            <div class="lp-profile-item-meta">
                                                <span class="lp-profile-item-user">{uname_clone.clone()}</span>
                                                {if validated {
                                                    view! { <span class="lp-profile-item-badge lp-profile-validated">"✓"</span> }.into_any()
                                                } else {
                                                    view! { <span class="lp-profile-item-badge lp-profile-unvalidated">"⚠"</span> }.into_any()
                                                }}
                                            </div>
                                        </div>
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }
            } else { ().into_any() }}
        </div>

        // ── LOGIN STORAGE OPTION ──
        <div class="lp-storage-row">
            <div class="lp-storage-label">"STORAGE:"</div>
            <label class="lp-storage-option" class:active={move || login_store_local.get()}>
                <input
                    type="checkbox"
                    checked={move || login_store_local.get()}
                    on:change=move |ev| set_login_store_local.set(event_target_checked(&ev))
                />
                "Local"
            </label>
            <label class="lp-storage-option" class:active={move || login_store_cloud.get()}>
                <input
                    type="checkbox"
                    checked={move || login_store_cloud.get()}
                    on:change=move |ev| set_login_store_cloud.set(event_target_checked(&ev))
                />
                "Cloud"
            </label>
        </div>
    }
}
