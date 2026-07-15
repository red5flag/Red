use crate::api::email::{LoginRequest, SignupRequest, SyncCredentialsRequest};
use crate::stores::{
    create_action, use_app_store, use_notification_store, use_organization_store,
    use_undo_redo_store,
};
use crate::types::ActionType;
use cfg_if::cfg_if;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::time::Duration;

use crate::pages::login::{
    auth_messages, credential_controls, login_form, register_form, two_factor,
};

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
    let notification_store = use_notification_store();
    let organization_store = use_organization_store();
    let undo_store = use_undo_redo_store();

    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(String::new());
    let (show_signup, set_show_signup) = signal(false);
    let (remember_me, set_remember_me) = signal(true);
    let (remember_password, set_remember_password) = signal(false);
    let (show_profiles, set_show_profiles) = signal(false);

    let (pending_username, set_pending_username) = signal(String::new());
    let (pending_password, set_pending_password) = signal(String::new());
    let (fa_code, set_fa_code) = signal(String::new());
    let (fa_mode, set_fa_mode) = signal::<Option<&'static str>>(None);
    let (fa_message, set_fa_message) = signal(String::new());

    let (su_username, set_su_username) = signal(String::new());
    let (su_password, set_su_password) = signal(String::new());
    let (su_confirm, set_su_confirm) = signal(String::new());
    let (su_email, set_su_email) = signal(String::new());
    let (su_error, set_su_error) = signal(String::new());
    let (su_success, set_su_success) = signal(String::new());

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

    let finish_login = move |username: String, name: String, email: String, role: String| {
        set_error.set(String::new());
        app_store.update(|store| {
            store.login_server_validated(
                &username,
                &name,
                &email,
                &mut notification_store.get_untracked(),
                &mut organization_store.get_untracked(),
            );
        });
        notification_store.update(|store| {
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

    let initial_username = load_last_username();
    if !initial_username.is_empty() {
        set_username.set(initial_username.clone());
    }

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

        let password_matches = app_store.get().check_password(&u, &p);

        if password_matches {
            let mut result: Option<Result<(String, String), String>> = None;
            app_store.update(|store| {
                result = Some(store.login_with_credentials(
                    &u,
                    &p,
                    &mut notification_store.get_untracked(),
                ));
            });

            if let Some(Ok((name, role_str))) = result {
                set_error.set(String::new());
                notification_store.update(|store| {
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
                app_store.update(|s| {
                    s.save_password_to_credentials(&u, &p);
                });
                let (_, login_cloud) = app_store.get().credentials.get_storage_options(&u);
                if login_cloud {
                    sync_credentials_to_cloud(u.clone());
                }
                return;
            } else if let Some(Err(e)) = result {
                if !e.to_lowercase().contains("not validated") {
                    set_error.set(e);
                    set_login_pressed.set(false);
                    return;
                }
            }
        }

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
        let _remember_password_clone = remember_password;
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
                                        app_store.update(|store| {
                                            store.upsert_credential_from_login(&u_clone, &p_clone, &name, &email, true, true, false);
                                        });
                                        app_store.update(|s| {
                                            s.save_password_to_credentials(&u_clone, &p_clone);
                                        });
                                        finish_login(u_clone.clone(), name, email, "Owner".to_string());
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
                        set_pending_username_clone, set_pending_password_clone, set_fa_code_clone);
                }
            }
        });
    };

    let on_select_profile = Callback::new(move |()| {
        let u = username.get();
        let p = password.get();
        if u.trim().is_empty() || p.trim().is_empty() {
            set_error.set("Saved profile is missing username or password".to_string());
            return;
        }
        let mut result: Option<Result<(String, String), String>> = None;
        app_store.update(|store| {
            result = Some(store.login_saved_profile(
                &u,
                &p,
                &mut notification_store.get_untracked(),
                &mut organization_store.get_untracked(),
            ));
        });
        match result {
            Some(Ok((name, role_str))) => {
                set_error.set(String::new());
                notification_store.update(|store| {
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
            }
            Some(Err(e)) => {
                set_error.set(e);
            }
            None => {}
        }
    });

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
                                        app_store.update(|store| {
                                            store.upsert_credential_from_login(&u, &p, &name, &email, true, true, false);
                                        });
                                        finish_login_clone(u.clone(), name, email, "Owner".to_string());
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
                                    app_store_for_signup.update(|store| {
                                        let _ = store.register_user(&u, &p, &email_for_msg, true, false);
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
    let (commits, _set_commits) = signal(Vec::<(String, String)>::new());
    let (commits_loading, set_commits_loading) = signal(false);
    let (commits_fetched, set_commits_fetched) = signal(false);

    let fetch_commits = move |_| {
        if commits_fetched.get() {
            return;
        }
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
                                    _set_commits.set(parsed);
                                }
                            }
                        }
                    }
                    set_commits_loading.set(false);
                });
            }
        }
    };

    let on_toggle_changelog = Callback::new(move |_: leptos::ev::MouseEvent| {
        set_changelog_open.update(|v| {
            *v = !*v;
            if *v {
                fetch_commits(());
            }
        });
    });

    let (show_username, set_show_username) = signal(true);
    let (show_password, set_show_password) = signal(false);
    let (su_show_pass, set_su_show_pass) = signal(false);
    let (su_show_confirm, set_su_show_confirm) = signal(false);

    let (pressed_btn, set_pressed_btn) = signal(Option::<&'static str>::None);
    let flash_press = Callback::new(move |key: &'static str| {
        set_pressed_btn.set(Some(key));
        set_timeout(
            move || {
                if pressed_btn.get() == Some(key) {
                    set_pressed_btn.set(None);
                }
            },
            Duration::from_millis(500),
        );
    });

    let on_login_cb = Callback::new(move |()| on_login());
    let on_verify_2fa_cb = Callback::new(move |mode: &'static str| on_verify_2fa(mode));
    let on_signup_cb = Callback::new(on_signup);

    view! {
        <div class="lp-screen">
            <div class="lp-header">
                <a
                    class="lp-logo-link"
                    href="WEBSITE_URL_HERE"
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label="Open Red website"
                >
                    <div class="lp-logo">"λ"</div>
                </a>
                <div class="lp-title">"LOGIN"</div>
                <div class="lp-version">"0.01"</div>
            </div>

            <credential_controls::CredentialControls
                show_profiles={show_profiles}
                set_show_profiles={set_show_profiles}
                set_username={set_username}
                set_password={set_password}
                on_select_profile={on_select_profile}
            />

            <login_form::LoginForm
                username={username}
                set_username={set_username}
                password={password}
                set_password={set_password}
                remember_me={remember_me}
                set_remember_me={set_remember_me}
                remember_password={remember_password}
                set_remember_password={set_remember_password}
                show_username={show_username}
                set_show_username={set_show_username}
                show_password={show_password}
                set_show_password={set_show_password}
                pressed_btn={pressed_btn}
                flash_press={flash_press}
                on_login={on_login_cb}
            />

            <two_factor::TwoFactor
                fa_mode={fa_mode}
                fa_code={fa_code}
                set_fa_code={set_fa_code}
                fa_message={fa_message}
                pressed_btn={pressed_btn}
                flash_press={flash_press}
                on_verify_2fa={on_verify_2fa_cb}
            />

            // ── REGISTER / LOGIN BUTTONS ──
            <div class="lp-action-row">
                <button class="lp-action-btn lp-register"
                    class:lp-pressed={move || pressed_btn.get() == Some("register")}
                    on:click=move |_| { flash_press.run("register"); set_show_signup.set(true); }>
                    "REGISTER"
                </button>
                <button class="lp-action-btn lp-login"
                    class:lp-pressed={move || pressed_btn.get() == Some("login")}
                    class:lp-login-grey={move || login_pressed.get()}
                    on:click=move |_| {
                        flash_press.run("login");
                        on_login_cb.run(());
                    }>
                    {move || if login_pressed.get() { "LOGGING IN…" } else { "LOGIN" }}
                </button>
            </div>

            // ── OAUTH / 2FA ROW ──
            <div class="lp-oauth-row">
                <button class="lp-oauth-btn"
                    class:lp-pressed={move || pressed_btn.get() == Some("oauth-gmail")}
                    on:click=move |_| flash_press.run("oauth-gmail")>"GMAIL"</button>
                <div class="lp-oauth-divider" />
                <button class="lp-oauth-btn"
                    class:lp-pressed={move || pressed_btn.get() == Some("oauth-linkedin")}
                    on:click=move |_| flash_press.run("oauth-linkedin")>"LINKEDIN"</button>
                <div class="lp-oauth-divider" />
                <button class="lp-oauth-btn"
                    class:lp-pressed={move || pressed_btn.get() == Some("oauth-2fa")}
                    on:click=move |_| flash_press.run("oauth-2fa")>"2FA"</button>
            </div>

            <auth_messages::AuthMessages
                error={error}
                changelog_open={changelog_open}
                commits={commits}
                commits_loading={commits_loading}
                on_toggle_changelog={on_toggle_changelog}
            />

            // ── FOOTER ──
            <div class="lp-footer">
                <button class="lp-footer-btn"
                    class:lp-pressed={move || pressed_btn.get() == Some("footer-news")}
                    on:click=move |_| flash_press.run("footer-news")>"NEWS"</button>
                <div class="lp-footer-divider" />
                <button class="lp-footer-btn"
                    class:lp-pressed={move || pressed_btn.get() == Some("footer-about")}
                    on:click=move |_| flash_press.run("footer-about")>"ABOUT"</button>
                <div class="lp-footer-divider" />
                <button class="lp-footer-btn"
                    class:lp-pressed={move || pressed_btn.get() == Some("footer-help")}
                    on:click=move |_| flash_press.run("footer-help")>"HELP"</button>
            </div>

        </div>

        // ── REGISTER SCREEN (lp-style) ──
        <register_form::RegisterForm
            show_signup={show_signup}
            set_show_signup={set_show_signup}
            set_su_username={set_su_username}
            set_su_password={set_su_password}
            set_su_confirm={set_su_confirm}
            set_su_email={set_su_email}
            su_error={su_error}
            su_success={su_success}
            su_show_pass={su_show_pass}
            set_su_show_pass={set_su_show_pass}
            su_show_confirm={su_show_confirm}
            set_su_show_confirm={set_su_show_confirm}
            on_signup={on_signup_cb}
        />
    }
}
