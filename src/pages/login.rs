use cfg_if::cfg_if;
use crate::api::email::{SignupRequest, LoginRequest, SyncCredentialsRequest};
use crate::stores::{create_action, use_app_store, use_undo_redo_store};
use crate::types::ActionType;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::time::Duration;

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use crate::api::email::{SignupResponse, LoginResponse,
            VerifyTotpRequest, VerifyEmail2faRequest, VerifyPhone2faRequest};
        use web_sys::window;
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let app_store = use_app_store();
    let undo_store = use_undo_redo_store();

    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(String::new());
    let (show_signup, set_show_signup) = signal(false);
    let (remember_me, set_remember_me) = signal(true);
    let (remember_password, set_remember_password) = signal(false);
    let (remember_30_days, set_remember_30_days) = signal(false);
    let (login_store_local, set_login_store_local) = signal(true);
    let (login_store_cloud, set_login_store_cloud) = signal(false);

    // Saved profiles dropdown
    let (show_profiles, set_show_profiles) = signal(false);
    let (totp_stub_code, set_totp_stub_code) = signal(String::new());
    let (profile_2fa, set_profile_2fa) = signal(Option::<String>::None);

    // 2FA flow signals
    let (pending_username, set_pending_username) = signal(String::new());
    let (pending_password, set_pending_password) = signal(String::new());
    let (fa_code, set_fa_code) = signal(String::new());
    let (fa_mode, set_fa_mode) = signal::<Option<&'static str>>(None);
    let (fa_message, set_fa_message) = signal(String::new());

    // Signup form signals
    let (su_username, set_su_username) = signal(String::new());
    let (su_password, set_su_password) = signal(String::new());
    let (su_confirm, set_su_confirm) = signal(String::new());
    let (su_email, set_su_email) = signal(String::new());
    let (su_store_local, set_su_store_local) = signal(true);
    let (su_store_cloud, set_su_store_cloud) = signal(false);
    let (su_error, set_su_error) = signal(String::new());
    let (su_success, set_su_success) = signal(String::new());

    // Save username for ease-of-entry (hydrate only)
    let save_last_username = move |u: String| {
        #[cfg(feature = "hydrate")]
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("farley_last_username", &u);
            }
        }
        let _ = u;
    };

    let load_last_username = move || {
        #[cfg(feature = "hydrate")]
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(u)) = storage.get_item("farley_last_username") {
                    return u;
                }
            }
        }
        String::new()
    };

    let sync_credentials_to_cloud = move |username: String| {
        let creds = app_store.get().credentials.clone();
        spawn_local(async move {
            let _req = SyncCredentialsRequest {
                username: username.clone(),
                credentials: creds,
            };
            cfg_if! {
                if #[cfg(feature = "hydrate")] {
                    let _ = gloo_net::http::Request::post("/api/credentials/sync")
                        .json(&_req)
                        .unwrap()
                        .send()
                        .await;
                }
            }
        });
    };

    let finish_login = move |name: String, email: String, role: String| {
        set_error.set(String::new());
        app_store.update(|store| {
            store.login_server_validated(&name, &email);
            store.add_notification(
                format!("Welcome, {}!", name),
                crate::stores::NotificationType::Success,
            );
        });
        let user_id = app_store.get().current_user.id;
        let org_id = app_store.get().current_user.organization_id;
        undo_store.update(|undo| {
            undo.record_action(create_action(
                ActionType::Login,
                "Auth",
                &format!("User '{}' logged in", name),
                user_id,
                &name,
                &role,
                org_id,
                None,
            ));
        });
    };

    // Load saved username on mount
    let initial_username = load_last_username();
    if !initial_username.is_empty() {
        set_username.set(initial_username.clone());
    }

    // Try loading profiles from cloud if no local profiles exist (hydrate only)
    cfg_if! {
        if #[cfg(feature = "hydrate")] {
            {
                let app_store = app_store.clone();
                let initial_username = initial_username.clone();
                spawn_local(async move {
                    if app_store.get().credentials.credentials.is_empty() && !initial_username.is_empty() {
                        let resp = gloo_net::http::Request::get("/api/credentials/sync")
                            .query([("username", &initial_username)])
                            .send()
                            .await;
                        if let Ok(r) = resp {
                            if let Ok(load_resp) = r.json::<crate::api::email::LoadCredentialsResponse>().await {
                                if load_resp.success {
                                    if let Some(creds) = load_resp.credentials {
                                        app_store.update(|store| {
                                            for (username, cred) in creds.credentials {
                                                store.credentials.credentials.entry(username).or_insert(cred);
                                            }
                                            store.credentials.save_to_local_storage();
                                        });
                                    }
                                }
                            }
                        }
                    }
                });
            }
        }
    }

    let (login_pressed, set_login_pressed) = signal(false);
    let on_login = move || {
        set_login_pressed.set(true);
        let u = username.get();
        let p = password.get();
        if u.trim().is_empty() || p.trim().is_empty() {
            set_error.set("Username and password are required".to_string());
            set_login_pressed.set(false);
            return;
        }

        if remember_me.get() {
            save_last_username(u.clone());
        }

        // Save 30-day remember token
        if remember_30_days.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let expiry = chrono::Utc::now().timestamp() + 30 * 24 * 60 * 60;
                    let token = format!("{}:{}", u, expiry);
                    let _ = storage.set_item("farley_remember_30", &token);
                }
            }
            let _ = &u;
        }

        // Check if password matches locally
        let password_matches = app_store.get().check_password(&u, &p);

        if password_matches {
            // Credentials correct — try local login
            let mut result: Option<Result<(String, String), String>> = None;
            app_store.update(|store| {
                result = Some(store.login_with_credentials(&u, &p));
            });

            if let Some(Ok((name, role_str))) = result {
                set_error.set(String::new()); // clear any previous error
                app_store.update(|store| {
                    store.add_notification(
                        format!("Welcome, {}!", name),
                        crate::stores::NotificationType::Success,
                    );
                });
                let user_id = app_store.get().current_user.id;
                let org_id = app_store.get().current_user.organization_id;
                undo_store.update(|undo| {
                    undo.record_action(create_action(
                        ActionType::Login,
                        "Auth",
                        &format!("User '{}' logged in", name),
                        user_id,
                        &name,
                        &role_str,
                        org_id,
                        None,
                    ));
                });
                let local = login_store_local.get();
                let cloud = login_store_cloud.get();
                app_store.update(|s| {
                    s.set_storage_options(&u, local, cloud);
                });
                if remember_password.get() {
                    app_store.update(|s| {
                        s.save_password_to_credentials(&u, &p);
                    });
                }
                if cloud {
                    sync_credentials_to_cloud(u.clone());
                }
                return;
            } else if let Some(Err(e)) = result {
                if !e.to_lowercase().contains("not validated") {
                    set_error.set(e);
                    set_login_pressed.set(false);
                    return;
                }
                // Not validated locally — fall through to server check
            }
        }

        // Local validation failed or password didn't match — try server
        let u_clone = u.clone();
        let p_clone = p.clone();
        let _app_store_clone = app_store;
        let set_error_clone = set_error;
        let set_login_pressed_clone = set_login_pressed;
        let set_fa_mode_clone = set_fa_mode;
        let set_fa_message_clone = set_fa_message;
        let set_pending_username_clone = set_pending_username;
        let set_pending_password_clone = set_pending_password;
        let set_fa_code_clone = set_fa_code;
        let remember_password_clone = remember_password;
        spawn_local(async move {
            let req = LoginRequest {
                username: u_clone.clone(),
                password: p_clone.clone(),
            };
            cfg_if! {
                if #[cfg(feature = "hydrate")] {
                    let resp = gloo_net::http::Request::post("/api/login")
                        .json(&req)
                        .unwrap()
                        .send()
                        .await;
                    match resp {
                        Ok(r) => {
                            if let Ok(login_resp) = r.json::<LoginResponse>().await {
                                if login_resp.success {
                                    if let (Some(name), Some(email)) = (login_resp.display_name, login_resp.email) {
                                        let local = login_store_local.get();
                                        let cloud = login_store_cloud.get();
                                        app_store.update(|store| {
                                            store.upsert_credential_from_login(&u_clone, &p_clone, &name, &email, true, local, cloud);
                                        });
                                        if remember_password_clone.get() {
                                            app_store.update(|s| {
                                                s.save_password_to_credentials(&u_clone, &p_clone);
                                            });
                                        }
                                        finish_login(name, email, "Owner".to_string());
                                        if cloud {
                                            sync_credentials_to_cloud(u_clone.clone());
                                        }
                                    }
                                } else if login_resp.requires_totp || login_resp.requires_email_2fa || login_resp.requires_phone_2fa {
                                    set_pending_username_clone.set(u_clone.clone());
                                    set_pending_password_clone.set(p_clone.clone());
                                    set_fa_code_clone.set(String::new());
                                    if login_resp.requires_totp {
                                        set_fa_mode_clone.set(Some("totp"));
                                        set_fa_message_clone.set("Enter the 6-digit code from Google Authenticator".to_string());
                                    } else if login_resp.requires_phone_2fa {
                                        set_fa_mode_clone.set(Some("phone"));
                                        set_fa_message_clone.set("Enter the 6-digit code sent to your phone via SMS".to_string());
                                    } else {
                                        set_fa_mode_clone.set(Some("email"));
                                        set_fa_message_clone.set("Enter the 6-digit code sent to your email".to_string());
                                    }
                                    set_error_clone.set(String::new());
                                    set_login_pressed_clone.set(false);
                                } else {
                                    set_error_clone.set(login_resp.message);
                                    set_login_pressed_clone.set(false);
                                }
                            } else {
                                set_error_clone.set("Failed to parse server response".to_string());
                                set_login_pressed_clone.set(false);
                            }
                        }
                        Err(e) => {
                            set_error_clone.set(format!("Network error: {}", e));
                            set_login_pressed_clone.set(false);
                        }
                    }
                } else {
                    let _ = (req, _app_store_clone, set_error_clone, set_login_pressed_clone, set_fa_mode_clone, set_fa_message_clone,
                        set_pending_username_clone, set_pending_password_clone, set_fa_code_clone, remember_password_clone);
                }
            }
        });
    };

    let on_verify_2fa = move |mode: &'static str| {
        let u = pending_username.get();
        let p = pending_password.get();
        let code = fa_code.get();
        if code.trim().is_empty() {
            set_fa_message.set(format!("Please enter the {} code", mode));
            return;
        }
        let finish_login_clone = finish_login.clone();
        let set_fa_mode_clone = set_fa_mode.clone();
        let set_fa_message_clone = set_fa_message.clone();
        spawn_local(async move {
            cfg_if! {
                if #[cfg(feature = "hydrate")] {
                    let resp = if mode == "totp" {
                        let req = VerifyTotpRequest { username: u.clone(), code: code.clone() };
                        gloo_net::http::Request::post("/api/verify_totp").json(&req).unwrap().send().await
                    } else if mode == "phone" {
                        let req = VerifyPhone2faRequest { username: u.clone(), code: code.clone() };
                        gloo_net::http::Request::post("/api/verify_phone_2fa").json(&req).unwrap().send().await
                    } else {
                        let req = VerifyEmail2faRequest { username: u.clone(), code: code.clone() };
                        gloo_net::http::Request::post("/api/verify_email_2fa").json(&req).unwrap().send().await
                    };
                    match resp {
                        Ok(r) => {
                            // All verify responses share the same shape; parse generically
                            let body = r.text().await.unwrap_or_default();
                            #[derive(serde::Deserialize)]
                            struct GenericVerifyResponse {
                                success: bool,
                                message: String,
                                display_name: Option<String>,
                                email: Option<String>,
                            }
                            if let Ok(v) = serde_json::from_str::<GenericVerifyResponse>(&body) {
                                if v.success {
                                    if let (Some(name), Some(email)) = (v.display_name, v.email) {
                                        let local = login_store_local.get();
                                        let cloud = login_store_cloud.get();
                                        app_store.update(|store| {
                                            store.upsert_credential_from_login(&u, &p, &name, &email, true, local, cloud);
                                        });
                                        finish_login_clone(name, email, "Owner".to_string());
                                        if cloud {
                                            sync_credentials_to_cloud(u.clone());
                                        }
                                        set_fa_mode_clone.set(None);
                                    }
                                } else {
                                    set_fa_message_clone.set(v.message);
                                }
                            } else {
                                set_fa_message_clone.set("Failed to parse server response".to_string());
                            }
                        }
                        Err(e) => set_fa_message_clone.set(format!("Network error: {}", e)),
                    }
                } else {
                    let _ = (u, p, code, mode, finish_login_clone, set_fa_mode_clone, set_fa_message_clone);
                }
            }
        });
    };

    let on_signup = move |_| {
        set_su_error.set(String::new());
        set_su_success.set(String::new());

        let u = su_username.get();
        let p = su_password.get();
        let c = su_confirm.get();
        let e = su_email.get();

        if u.trim().is_empty() {
            set_su_error.set("Username is required".to_string());
            return;
        }
        if p.len() < 3 {
            set_su_error.set("Password must be at least 3 characters".to_string());
            return;
        }
        if p != c {
            set_su_error.set("Passwords do not match".to_string());
            return;
        }
        if !e.contains('@') {
            set_su_error.set("A valid email is required".to_string());
            return;
        }

        let set_err = set_su_error.clone();
        let set_succ = set_su_success.clone();
        let set_u = set_su_username.clone();
        let set_p = set_su_password.clone();
        let set_c = set_su_confirm.clone();
        let set_e = set_su_email.clone();
        let email_for_msg = e.clone();
        let app_store_for_signup = app_store;

        spawn_local(async move {
            let req = SignupRequest {
                username: u.clone(),
                password: p.clone(),
                email: e,
            };
            cfg_if! {
                if #[cfg(feature = "hydrate")] {
                    let resp = gloo_net::http::Request::post("/api/signup")
                        .json(&req)
                        .unwrap()
                        .send()
                        .await;

                    match resp {
                        Ok(r) => {
                            if let Ok(signup_resp) = r.json::<SignupResponse>().await {
                                if signup_resp.success {
                                    // Also save credentials with chosen storage options
                                    let su_local = su_store_local.get();
                                    let su_cloud = su_store_cloud.get();
                                    app_store_for_signup.update(|store| {
                                        let _ = store.register_user(&u, &p, &email_for_msg, su_local, su_cloud);
                                    });
                                    set_succ.set(format!(
                                        "Account created! A validation email has been sent to {}. Check /emailvalid to validate and then sign in.",
                                        email_for_msg
                                    ));
                                    set_u.set(String::new());
                                    set_p.set(String::new());
                                    set_c.set(String::new());
                                    set_e.set(String::new());
                                } else {
                                    set_err.set(signup_resp.message);
                                }
                            } else {
                                set_err.set("Failed to parse server response".to_string());
                            }
                        }
                        Err(err) => {
                            set_err.set(format!("Network error: {}", err));
                        }
                    }
                } else {
                    let _ = (req, set_err, set_succ, set_u, set_p, set_c, set_e, email_for_msg, app_store_for_signup, p, u);
                }
            }
        });
    };

    let (changelog_open, set_changelog_open) = signal(false);
    #[allow(unused_variables)]
    let (commits, set_commits) = signal(Vec::<(String, String)>::new());
    let (commits_loading, set_commits_loading) = signal(false);
    let (commits_fetched, set_commits_fetched) = signal(false);

    let fetch_commits = move |_| {
        if commits_fetched.get() { return; }
        set_commits_fetched.set(true);
        set_commits_loading.set(true);
        cfg_if! {
            if #[cfg(feature = "hydrate")] {
                spawn_local(async move {
                    let resp = gloo_net::http::Request::get("https://api.github.com/repos/red5flag/Carly/commits?per_page=10")
                        .header("Accept", "application/vnd.github+json")
                        .header("User-Agent", "Carly-App")
                        .send()
                        .await;
                    if let Ok(r) = resp {
                        if let Ok(text) = r.text().await {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                if let Some(arr) = json.as_array() {
                                    let parsed: Vec<(String, String)> = arr.iter().filter_map(|c| {
                                        let sha = c.get("sha")?.as_str()?.chars().take(7).collect::<String>();
                                        let msg = c.get("commit")?.get("message")?.as_str()?;
                                        let first_line = msg.lines().next().unwrap_or(msg).to_string();
                                        Some((sha, first_line))
                                    }).collect();
                                    set_commits.set(parsed);
                                }
                            }
                        }
                    }
                    set_commits_loading.set(false);
                });
            }
        }
    };

    let on_toggle_changelog = move |_| {
        set_changelog_open.update(|v| {
            *v = !*v;
            if *v { fetch_commits(()); }
        });
    };
    let (show_username, set_show_username) = signal(true);
    let (show_password, set_show_password) = signal(false);
    let (su_show_pass, set_su_show_pass) = signal(false);
    let (su_show_confirm, set_su_show_confirm) = signal(false);

    // Button press flash effect: stores a unique key per press to retrigger animation
    let (pressed_btn, set_pressed_btn) = signal(Option::<&'static str>::None);
    let flash_press = move |key: &'static str| {
        set_pressed_btn.set(Some(key));
        set_timeout(move || {
            if pressed_btn.get() == Some(key) {
                set_pressed_btn.set(None);
            }
        }, Duration::from_millis(500));
    };

    view! {
        <div class="lp-screen">

            // ── RED HEADER ──
            <div class="lp-header">
                <div class="lp-logo">"λ"</div>
                <div class="lp-title">"LOGIN"</div>
                <div class="lp-version">"0.01"</div>
            </div>

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
                                    let email = cred.email.clone();
                                    let validated = cred.validated;
                                    let uname_for_click = uname.clone();
                                    view! {
                                        <button class="lp-profile-item"
                                            on:click=move |_| {
                                                set_username.set(uname_for_click.clone());
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

            // ── USERNAME ROW ──
            <div class="lp-field-row">
                <button class="lp-visibility-btn" tabindex="-1" title="Toggle username visibility"
                    class:lp-pressed={move || pressed_btn.get() == Some("vis-user")}
                    on:click=move |_| { flash_press("vis-user"); set_show_username.update(|v| *v = !*v); }>
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
                            on:keydown=move |ev| { if ev.key() == "Enter" { on_login(); } }
                        />
                    }
                }}
            </div>

            // ── PASSWORD ROW ──
            <div class="lp-field-row">
                <button class="lp-visibility-btn" tabindex="-1" title="Toggle password visibility"
                    class:lp-pressed={move || pressed_btn.get() == Some("vis-pass")}
                    on:click=move |_| { flash_press("vis-pass"); set_show_password.update(|v| *v = !*v); }>
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
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            on:keydown=move |ev| { if ev.key() == "Enter" { on_login(); } }
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
                            "(2FA will be required for saved passwords)"
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
                                            on_login();
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
                                            on_login();
                                        }>
                                        "VERIFY & LOGIN"
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any()
            })}

            // ── ERROR ──
            {move || {
                let err = error.get();
                if err.is_empty() { ().into_any() } else {
                    view! { <div class="lp-error">{err}</div> }.into_any()
                }
            }}

            // ── 2FA CODE ENTRY ──
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
                            on:click=move |_| { flash_press("2fa-verify"); on_verify_2fa(mode.unwrap_or("email")); }>
                            "VERIFY"
                        </button>
                    </div>
                }.into_any()
            }}

            // ── REGISTER / LOGIN BUTTONS ──
            <div class="lp-action-row">
                <button class="lp-action-btn lp-register"
                    class:lp-pressed={move || pressed_btn.get() == Some("register")}
                    on:click=move |_| { flash_press("register"); set_show_signup.set(true); }>
                    "REGISTER"
                </button>
                <button class="lp-action-btn lp-login"
                    class:lp-pressed={move || pressed_btn.get() == Some("login")}
                    class:lp-login-grey={move || login_pressed.get()}
                    on:click=move |_| {
                        flash_press("login");
                        on_login();
                    }>
                    {move || if login_pressed.get() { "LOGGING IN…" } else { "LOGIN" }}
                </button>
            </div>

            // ── OAUTH / 2FA ROW ──
            <div class="lp-oauth-row">
                <button class="lp-oauth-btn"
                    class:lp-pressed={move || pressed_btn.get() == Some("oauth-gmail")}
                    on:click=move |_| flash_press("oauth-gmail")>"GMAIL"</button>
                <div class="lp-oauth-divider" />
                <button class="lp-oauth-btn"
                    class:lp-pressed={move || pressed_btn.get() == Some("oauth-linkedin")}
                    on:click=move |_| flash_press("oauth-linkedin")>"LINKEDIN"</button>
                <div class="lp-oauth-divider" />
                <button class="lp-oauth-btn"
                    class:lp-pressed={move || pressed_btn.get() == Some("oauth-2fa")}
                    on:click=move |_| flash_press("oauth-2fa")>"2FA"</button>
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

            // ── CHANGELOG ACCORDION ──
            <div class="lp-changelog">
                <div class="lp-changelog-header">
                    <a
                        class="lp-changelog-link"
                        href="https://github.com/red5flag/Carly"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        "CHANGELOG"
                    </a>
                    <span
                        class="lp-changelog-toggle"
                        on:click=on_toggle_changelog
                    >
                        {move || if changelog_open.get() { "▲" } else { "▼" }}
                    </span>
                </div>
                {move || if changelog_open.get() {
                    view! {
                        <div class="lp-changelog-body">
                            <div class="lp-changelog-summary">
                                {move || if commits_loading.get() {
                                    "Loading recent commits from GitHub…"
                                } else {
                                    "Latest commits from red5flag/Carly on GitHub."
                                }}
                            </div>
                            <ul class="lp-changelog-list">
                                {move || commits.get().iter().map(|(sha, msg)| {
                                    view! {
                                        <li>
                                            <a
                                                href={format!("https://github.com/red5flag/Carly/commit/{}", sha)}
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                class="lp-changelog-commit"
                                            >
                                                <span class="lp-commit-sha">{sha.clone()}</span>
                                                <span class="lp-commit-msg">{msg.clone()}</span>
                                            </a>
                                        </li>
                                    }
                                }).collect::<Vec<_>>()}
                            </ul>
                            {move || if commits.get().is_empty() && !commits_loading.get() {
                                view! {
                                    <div class="lp-changelog-empty">
                                        "Unable to load commits. "
                                        <a
                                            href="https://github.com/red5flag/Carly/commits/main"
                                            target="_blank"
                                            rel="noopener noreferrer"
                                        >"View on GitHub →"</a>
                                    </div>
                                }.into_any()
                            } else { ().into_any() }}
                        </div>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            // ── FOOTER ──
            <div class="lp-footer">
                <button class="lp-footer-btn"
                    class:lp-pressed={move || pressed_btn.get() == Some("footer-news")}
                    on:click=move |_| flash_press("footer-news")>"NEWS"</button>
                <div class="lp-footer-divider" />
                <button class="lp-footer-btn"
                    class:lp-pressed={move || pressed_btn.get() == Some("footer-about")}
                    on:click=move |_| flash_press("footer-about")>"ABOUT"</button>
                <div class="lp-footer-divider" />
                <button class="lp-footer-btn"
                    class:lp-pressed={move || pressed_btn.get() == Some("footer-help")}
                    on:click=move |_| flash_press("footer-help")>"HELP"</button>
            </div>

        </div>

        // ── REGISTER SCREEN (lp-style) ──
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

                        // Storage option
                        <div class="lp-storage-row">
                            <div class="lp-storage-label">"STORAGE:"</div>
                            <label class="lp-storage-option" class:active={move || su_store_local.get()}>
                                <input
                                    type="checkbox"
                                    checked={move || su_store_local.get()}
                                    on:change=move |ev| set_su_store_local.set(event_target_checked(&ev))
                                />
                                "Local"
                            </label>
                            <label class="lp-storage-option" class:active={move || su_store_cloud.get()}>
                                <input
                                    type="checkbox"
                                    checked={move || su_store_cloud.get()}
                                    on:change=move |ev| set_su_store_cloud.set(event_target_checked(&ev))
                                />
                                "Cloud"
                            </label>
                        </div>

                        // Action row
                        <div class="lp-action-row">
                            <button class="lp-action-btn lp-register"
                                on:click=move |_| set_show_signup.set(false)>
                                "BACK"
                            </button>
                            <button class="lp-action-btn lp-login" on:click=on_signup>
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
