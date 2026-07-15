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
    show_username: ReadSignal<bool>,
    set_show_username: WriteSignal<bool>,
    show_password: ReadSignal<bool>,
    set_show_password: WriteSignal<bool>,
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
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" {
                                flash_press.run("login");
                                on_login.run(());
                            }
                        }
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
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" {
                                flash_press.run("login");
                                on_login.run(());
                            }
                        }
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

    }
}
