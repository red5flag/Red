use leptos::prelude::*;

#[component]
pub(crate) fn TwoFactor(
    fa_mode: ReadSignal<Option<&'static str>>,
    fa_code: ReadSignal<String>,
    set_fa_code: WriteSignal<String>,
    fa_message: ReadSignal<String>,
    pressed_btn: ReadSignal<Option<&'static str>>,
    flash_press: Callback<&'static str>,
    on_verify_2fa: Callback<&'static str>,
) -> impl IntoView {
    view! {
        {move || {
            let mode = fa_mode.get();
            if mode.is_none() { return ().into_any(); }
            let label = if mode == Some("totp") { "Authenticator code" } else if mode == Some("phone") { "SMS code" } else { "Email code" };
            let msg = fa_message.get();
            view! {
                <div class="lp-2fa-panel">
                    <div class="lp-2fa-message">{msg}</div>
                    <div class="lp-field-row">
                        <input
                            type="text"
                            class="lp-field-input"
                            placeholder={label}
                            maxlength="6"
                            prop:value=move || fa_code.get()
                            on:input=move |ev| set_fa_code.set(event_target_value(&ev))
                        />
                    </div>
                    <button class="lp-action-btn lp-login"
                        class:lp-pressed={move || pressed_btn.get() == Some("2fa-verify")}
                        on:click=move |_| { flash_press.run("2fa-verify"); on_verify_2fa.run(mode.unwrap_or("email")); }>
                        "VERIFY"
                    </button>
                </div>
            }.into_any()
        }}
    }
}
