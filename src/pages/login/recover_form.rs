use leptos::prelude::*;

#[component]
pub(crate) fn RecoverForm(
    show_recover: ReadSignal<bool>,
    set_show_recover: WriteSignal<bool>,
    recovery_email: ReadSignal<String>,
    set_recovery_email: WriteSignal<String>,
    recovery_phone: ReadSignal<String>,
    set_recovery_phone: WriteSignal<String>,
    recovery_message: ReadSignal<String>,
    on_recover: Callback<()>,
) -> impl IntoView {
    view! {
        {move || {
            if show_recover.get() {
                view! {
                    <div class="lp-screen lp-recover-screen">
                        <div class="lp-header">
                            <div class="lp-logo">"λ"</div>
                            <div class="lp-title">"RECOVER LOGIN"</div>
                            <button class="lp-close-btn" on:click=move |_| set_show_recover.set(false)>"✕"</button>
                        </div>

                        <div class="lp-saved-row" style="cursor:default;">
                            <div class="lp-saved-label">"Recover by email or phone"</div>
                        </div>

                        // Email
                        <div class="lp-field-row">
                            <div class="lp-visibility-btn" style="cursor:default;opacity:0.5;">"@"</div>
                            <input type="email" class="lp-field-input" placeholder="Email"
                                prop:value={move || recovery_email.get()}
                                on:input=move |ev| set_recovery_email.set(event_target_value(&ev)) />
                        </div>

                        // Phone
                        <div class="lp-field-row">
                            <div class="lp-visibility-btn" style="cursor:default;opacity:0.5;">"☎"</div>
                            <input type="tel" class="lp-field-input" placeholder="Phone number"
                                prop:value={move || recovery_phone.get()}
                                on:input=move |ev| set_recovery_phone.set(event_target_value(&ev)) />
                        </div>

                        // Message
                        {move || {
                            let msg = recovery_message.get();
                            if msg.is_empty() { ().into_any() } else {
                                view! { <div class="lp-recover-message">{msg}</div> }.into_any()
                            }
                        }}

                        // Action row
                        <div class="lp-action-row">
                            <button class="lp-action-btn lp-register"
                                on:click=move |_| set_show_recover.set(false)>
                                "BACK"
                            </button>
                            <button class="lp-action-btn lp-login" on:click=move |_| on_recover.run(())>
                                "SEND CODE"
                            </button>
                        </div>

                        <div class="lp-footer">
                            <button class="lp-footer-btn">"NEWS"</button>
                            <div class="lp-footer-divider" />
                            <button class="lp-footer-btn">"ABOUT"</button>
                            <div class="lp-footer-divider" />
                            <button class="lp-footer-btn">"HELP"</button>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }
        }}
    }
}
