use leptos::prelude::*;

#[component]
pub(crate) fn SecuritySettings(
    setup_user: ReadSignal<String>,
    set_setup_user: WriteSignal<String>,
    setup_pass: ReadSignal<String>,
    set_setup_pass: WriteSignal<String>,
    setup_secret: ReadSignal<String>,
    setup_uri: ReadSignal<String>,
    setup_code: ReadSignal<String>,
    set_setup_code: WriteSignal<String>,
    setup_error: ReadSignal<String>,
    setup_success: ReadSignal<String>,
    setup_step: ReadSignal<u8>,
    email_2fa_status: ReadSignal<String>,
    phone_number: ReadSignal<String>,
    set_phone_number: WriteSignal<String>,
    phone_2fa_status: ReadSignal<String>,
    on_enable_totp: Callback<leptos::ev::MouseEvent>,
    on_confirm_totp: Callback<leptos::ev::MouseEvent>,
    on_toggle_email_2fa: Callback<bool>,
    on_toggle_phone_2fa: Callback<bool>,
) -> impl IntoView {
    view! {
        <div class="data-card">
            <div class="card-header">
                <span class="card-title">"Two-Factor Authentication"</span>
            </div>
            <div class="settings-list">
                <div class="list-item" style="flex-direction: column; align-items: stretch; gap: 8px;">
                    <div class="list-item-title">"Account credentials"</div>
                    <input
                        class="form-input"
                        type="text"
                        placeholder="Username"
                        prop:value=move || setup_user.get()
                        on:input=move |ev| set_setup_user.set(event_target_value(&ev))
                    />
                    <input
                        class="form-input"
                        type="password"
                        placeholder="Password"
                        prop:value=move || setup_pass.get()
                        on:input=move |ev| set_setup_pass.set(event_target_value(&ev))
                    />
                    {move || {
                        let step = setup_step.get();
                        if step == 0 {
                            view! {
                                <button class="settings-action-btn" on:click=move |ev| on_enable_totp.run(ev)>"Enable Google Authenticator"</button>
                            }.into_any()
                        } else if step == 1 {
                            let secret = setup_secret.get();
                            let uri = setup_uri.get();
                            view! {
                                <div class="setup-secret">
                                    <div class="setup-label">"Secret:"</div>
                                    <div class="setup-value">{secret}</div>
                                    <div class="setup-label">"URI:"</div>
                                    <div class="setup-value">{uri}</div>
                                    <input
                                        class="form-input"
                                        type="text"
                                        placeholder="Enter 6-digit code from Authenticator"
                                        maxlength="6"
                                        prop:value=move || setup_code.get()
                                        on:input=move |ev| set_setup_code.set(event_target_value(&ev))
                                    />
                                    <button class="settings-action-btn" on:click=move |ev| on_confirm_totp.run(ev)>"Confirm TOTP"</button>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="setup-success">{setup_success.get()}</div>
                            }.into_any()
                        }
                    }}
                    {move || {
                        let err = setup_error.get();
                        if err.is_empty() { ().into_any() } else {
                            view! { <div class="setup-error">{err}</div> }.into_any()
                        }
                    }}
                </div>
                <div class="list-item" style="flex-direction: column; align-items: stretch; gap: 8px;">
                    <div class="list-item-title">"Email 2FA"</div>
                    <div class="list-item-desc">"Send a 6-digit code to your email at sign-in."</div>
                    <div class="setup-row">
                        <button class="settings-action-btn" on:click=move |_| on_toggle_email_2fa.run(true)>"Enable"</button>
                        <button class="settings-action-btn" on:click=move |_| on_toggle_email_2fa.run(false)>"Disable"</button>
                    </div>
                    {move || {
                        let status = email_2fa_status.get();
                        if status.is_empty() { ().into_any() } else {
                            view! { <div class="setup-status">{status}</div> }.into_any()
                        }
                    }}
                </div>
                <div class="list-item" style="flex-direction: column; align-items: stretch; gap: 8px;">
                    <div class="list-item-title">"Phone 2FA"</div>
                    <div class="list-item-desc">"Send a 6-digit code to your phone number at sign-in."</div>
                    <input
                        class="form-input"
                        type="tel"
                        placeholder="Phone number (e.g. +61 400 000 000)"
                        prop:value=move || phone_number.get()
                        on:input=move |ev| set_phone_number.set(event_target_value(&ev))
                    />
                    <div class="setup-row">
                        <button class="settings-action-btn" on:click=move |_| on_toggle_phone_2fa.run(true)>"Enable"</button>
                        <button class="settings-action-btn" on:click=move |_| on_toggle_phone_2fa.run(false)>"Disable"</button>
                    </div>
                    {move || {
                        let status = phone_2fa_status.get();
                        if status.is_empty() { ().into_any() } else {
                            view! { <div class="setup-status">{status}</div> }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
