use leptos::prelude::*;

#[component]
pub(crate) fn LoginForm(
    username: ReadSignal<String>,
    set_username: WriteSignal<String>,
    password: ReadSignal<String>,
    set_password: WriteSignal<String>,
    remember_me: ReadSignal<bool>,
    set_remember_me: WriteSignal<bool>,
    remember_password: ReadSignal<bool>,
    set_remember_password: WriteSignal<bool>,
    remember_30_days: ReadSignal<bool>,
    set_remember_30_days: WriteSignal<bool>,
    show_username: ReadSignal<bool>,
    set_show_username: WriteSignal<bool>,
    show_password: ReadSignal<bool>,
    set_show_password: WriteSignal<bool>,
    profile_2fa: ReadSignal<Option<String>>,
    set_profile_2fa: WriteSignal<Option<String>>,
    totp_stub_code: ReadSignal<String>,
    set_totp_stub_code: WriteSignal<String>,
    pressed_btn: ReadSignal<Option<&'static str>>,
    flash_press: Callback<&'static str>,
    on_login: Callback<()>,
) -> impl IntoView {
    view! {
        // ── USERNAME ROW ──
        <div class="lp-field-row">
            <button class="lp-visibility-btn" tabindex="-1" title="Toggle username visibility"
                class:lp-pressed={move || pressed_btn.get() == Some("vis-user")}
                on:click=move |_| { flash_press.run("vis-user"); set_show_username.update(|v| *v = !*v); }>
                {move || if show_username.get() { "◉" } else { "○" }}
            </button>
            {move || {
                let vis = show_username.get();
                view! {
                    <input
                        type={if vis { "text" } else { "password" }}
                        class="lp-field-input"
                        name="farley-username"
                        autocomplete="off"
                        data-lpignore="true"
                        placeholder="Username"
                        prop:value=move || username.get()
                        on:input=move |ev| set_username.set(event_target_value(&ev))
                        on:keydown=move |ev| { if ev.key() == "Enter" { on_login.run(()); } }
                    />
                }
            }}
        </div>

        // ── PASSWORD ROW ──
        <div class="lp-field-row">
            <button class="lp-visibility-btn" tabindex="-1" title="Toggle password visibility"
                class:lp-pressed={move || pressed_btn.get() == Some("vis-pass")}
                on:click=move |_| { flash_press.run("vis-pass"); set_show_password.update(|v| *v = !*v); }>
                {move || if show_password.get() { "◉" } else { "○" }}
            </button>
            {move || {
                let vis = show_password.get();
                view! {
                    <input
                        type={if vis { "text" } else { "password" }}
                        class="lp-field-input"
                        name="farley-password"
                        autocomplete="off"
                        data-lpignore="true"
                        placeholder="Password"
                        prop:value={move || password.get()}
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                        on:keydown=move |ev| { if ev.key() == "Enter" { on_login.run(()); } }
                    />
                }
            }}
        </div>

        // ── REMEMBER ME ──
        <div class="lp-remember-row">
            <label class="lp-remember-label">
                <input
                    type="checkbox"
                    checked={move || remember_me.get()}
                    on:change=move |ev| set_remember_me.set(event_target_checked(&ev))
                />
                "Remember username for ease-of-entry"
            </label>
        </div>
        <div class="lp-remember-row">
            <label class="lp-remember-label">
                <input
                    type="checkbox"
                    checked={move || remember_password.get()}
                    on:change=move |ev| set_remember_password.set(event_target_checked(&ev))
                />
                "Remember Password"
            </label>
            {move || if remember_password.get() {
                view! {
                    <span class="lp-remember-note" style="font-size:11px;color:var(--text-secondary, #888);">
                        "(password is saved locally so saved profiles can auto-fill it)"
                    </span>
                }.into_any()
            } else { ().into_any() }}
        </div>

        // ── PROFILE 2FA MODAL ──
        {move || profile_2fa.get().map(|profile_name| {
            view! {
                <div class="lp-2fa-modal-overlay" on:click=move |_| set_profile_2fa.set(None)>
                    <div class="lp-2fa-modal" on:click=|ev| ev.stop_propagation()>
                        <div class="lp-2fa-modal-header">
                            <span class="lp-2fa-modal-title">"🔐 2FA Verification"</span>
                            <button class="lp-2fa-modal-close" on:click=move |_| set_profile_2fa.set(None)>"✕"</button>
                        </div>
                        <div class="lp-2fa-modal-body">
                            <div class="lp-2fa-modal-profile">
                                <span class="lp-2fa-modal-profile-icon">"👤"</span>
                                <span class="lp-2fa-modal-profile-name">{profile_name.clone()}</span>
                            </div>
                            <div class="lp-2fa-modal-message">"Enter the 6-digit code from your authenticator app to continue"</div>
                            <input
                                type="text"
                                class="lp-2fa-modal-input"
                                placeholder="000000"
                                maxlength="6"
                                inputmode="numeric"
                                prop:value={move || totp_stub_code.get()}
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    let filtered: String = v.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
                                    set_totp_stub_code.set(filtered);
                                }
                                on:keydown=move |ev| {
                                    if ev.key() == "Enter" {
                                        set_profile_2fa.set(None);
                                        on_login.run(());
                                    }
                                }
                            />
                            <label class="lp-2fa-modal-remember">
                                <input
                                    type="checkbox"
                                    checked={move || remember_30_days.get()}
                                    on:change=move |ev| set_remember_30_days.set(event_target_checked(&ev))
                                />
                                "Remember me for 30 days"
                            </label>
                            <div class="lp-2fa-modal-actions">
                                <button class="lp-action-btn lp-register"
                                    on:click=move |_| set_profile_2fa.set(None)>
                                    "CANCEL"
                                </button>
                                <button class="lp-action-btn lp-login"
                                    on:click=move |_| {
                                        set_profile_2fa.set(None);
                                        on_login.run(());
                                    }>
                                    "VERIFY & LOGIN"
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            }.into_any()
        })}
    }
}
