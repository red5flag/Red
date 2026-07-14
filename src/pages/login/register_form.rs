use leptos::prelude::*;

#[component]
pub(crate) fn RegisterForm(
    show_signup: ReadSignal<bool>,
    set_show_signup: WriteSignal<bool>,
    set_su_username: WriteSignal<String>,
    set_su_password: WriteSignal<String>,
    set_su_confirm: WriteSignal<String>,
    set_su_email: WriteSignal<String>,
    su_error: ReadSignal<String>,
    su_success: ReadSignal<String>,
    su_show_pass: ReadSignal<bool>,
    set_su_show_pass: WriteSignal<bool>,
    su_show_confirm: ReadSignal<bool>,
    set_su_show_confirm: WriteSignal<bool>,
    on_signup: Callback<leptos::ev::MouseEvent>,
) -> impl IntoView {
    view! {
        {move || {
            if show_signup.get() {
                view! {
                    <div class="lp-screen lp-register-screen">
                        <div class="lp-header">
                            <div class="lp-logo">"λ"</div>
                            <div class="lp-title">"REGISTER"</div>
                            <button class="lp-close-btn" on:click=move |_| set_show_signup.set(false)>"✕"</button>
                        </div>

                        <div class="lp-saved-row" style="cursor:default;">
                            <div class="lp-saved-label">"Create a new account"</div>
                        </div>

                        // Username
                        <div class="lp-field-row">
                            <div class="lp-visibility-btn" style="cursor:default;opacity:0.5;">"◉"</div>
                            <input type="text" class="lp-field-input" placeholder="Username"
                                on:input=move |ev| set_su_username.set(event_target_value(&ev)) />
                        </div>

                        // Email
                        <div class="lp-field-row">
                            <div class="lp-visibility-btn" style="cursor:default;opacity:0.5;">"@"</div>
                            <input type="email" class="lp-field-input" placeholder="Email"
                                on:input=move |ev| set_su_email.set(event_target_value(&ev)) />
                        </div>

                        // Password
                        <div class="lp-field-row">
                            <button class="lp-visibility-btn" tabindex="-1" on:click=move |_| set_su_show_pass.update(|v| *v = !*v)>
                                {move || if su_show_pass.get() { "◉" } else { "○" }}
                            </button>
                            {move || {
                                let vis = su_show_pass.get();
                                view! {
                                    <input type={if vis { "text" } else { "password" }}
                                        class="lp-field-input" placeholder="Password"
                                        on:input=move |ev| set_su_password.set(event_target_value(&ev)) />
                                }
                            }}
                        </div>

                        // Confirm password
                        <div class="lp-field-row">
                            <button class="lp-visibility-btn" tabindex="-1" on:click=move |_| set_su_show_confirm.update(|v| *v = !*v)>
                                {move || if su_show_confirm.get() { "◉" } else { "○" }}
                            </button>
                            {move || {
                                let vis = su_show_confirm.get();
                                view! {
                                    <input type={if vis { "text" } else { "password" }}
                                        class="lp-field-input" placeholder="Confirm Password"
                                        on:input=move |ev| set_su_confirm.set(event_target_value(&ev)) />
                                }
                            }}
                        </div>

                        // Error / success
                        {move || {
                            let err = su_error.get();
                            if err.is_empty() { ().into_any() } else {
                                view! { <div class="lp-error">{err}</div> }.into_any()
                            }
                        }}
                        {move || {
                            let s = su_success.get();
                            if s.is_empty() { ().into_any() } else {
                                view! { <div class="lp-success">{s}</div> }.into_any()
                            }
                        }}

                        // Action row
                        <div class="lp-action-row">
                            <button class="lp-action-btn lp-register"
                                on:click=move |_| set_show_signup.set(false)>
                                "BACK"
                            </button>
                            <button class="lp-action-btn lp-login" on:click=move |ev| on_signup.run(ev)>
                                "CREATE"
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
