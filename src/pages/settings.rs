use crate::models::{CalendarEvent, ContactSource, MessengerContact};
use crate::stores::use_app_store;
use cfg_if::cfg_if;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[cfg(feature = "hydrate")]
use gloo_net::http::Request as GlooRequest;
#[cfg(feature = "hydrate")]
use crate::api::email::{EnableTotpRequest, EnableTotpResponse, ConfirmTotpRequest, ConfirmTotpResponse, ToggleEmail2faRequest, ToggleEmail2faResponse};
use uuid::Uuid;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let app_store = use_app_store();
    let (import_status, set_import_status) = signal(String::new());

    // 2FA setup signals
    let (setup_user, set_setup_user) = signal(String::new());
    let (setup_pass, set_setup_pass) = signal(String::new());
    let (setup_secret, set_setup_secret) = signal(String::new());
    let (setup_uri, set_setup_uri) = signal(String::new());
    let (setup_code, set_setup_code) = signal(String::new());
    let (setup_error, set_setup_error) = signal(String::new());
    let (setup_success, set_setup_success) = signal(String::new());
    let (setup_step, set_setup_step) = signal(0u8); // 0: credentials, 1: confirm, 2: success
    let (email_2fa_status, set_email_2fa_status) = signal(String::new());

    let import_contacts = move |_| {
        let names = vec!["Alice", "Bob", "Carol", "David"];
        app_store.update(|s| {
            for name in names {
                let contact = MessengerContact {
                    id: Uuid::new_v4(),
                    name: name.to_string(),
                    source: ContactSource::Imported,
                    phone: Some(format!("+61 4{:02} {:03} {:03}", s.messages.len() + 1, 100, 200)),
                    email: Some(format!("{}@example.com", name.to_lowercase())),
                    unread_count: 0,
                };
                s.add_messenger_contact(contact);
            }
        });
        set_import_status.set("Imported 4 contacts into messenger.".to_string());
    };

    let import_channel = move |channel: &'static str| {
        let _ = channel;
        let mut events = Vec::new();
        use chrono::Duration;
        let now = chrono::Utc::now();
        let titles = match channel {
            "xero" => vec!["Q1 reconciliation", "Invoice run"],
            "booking" => vec!["Guest check-in", "Guest check-out"],
            "airbnb" => vec!["Airbnb reservation", "Cleaning schedule"],
            _ => vec!["Imported event"],
        };
        for (i, title) in titles.iter().enumerate() {
            let start = now + Duration::days(i as i64 + 1);
            let end = start + Duration::hours(2);
            let mut ev = CalendarEvent::new(title.to_string(), start, end);
            ev.source = Some(channel.to_string());
            events.push(ev);
        }
        app_store.update(|s| {
            for ev in events {
                s.add_calendar_event(ev);
            }
        });
        set_import_status.set(format!("Imported {} bookings into calendar.", channel));
    };

    let on_enable_totp = move |_| {
        set_setup_error.set(String::new());
        set_setup_success.set(String::new());
        let u = setup_user.get();
        let p = setup_pass.get();
        if u.trim().is_empty() || p.trim().is_empty() {
            set_setup_error.set("Username and password are required".to_string());
            return;
        }
        let set_secret = set_setup_secret.clone();
        let set_uri = set_setup_uri.clone();
        let set_step = set_setup_step.clone();
        let set_err = set_setup_error.clone();
        spawn_local(async move {
            cfg_if! {
                if #[cfg(feature = "hydrate")] {
                    let req = EnableTotpRequest { username: u, password: p };
                    let resp = GlooRequest::post("/api/enable_totp")
                        .json(&req)
                        .unwrap()
                        .send()
                        .await;
                    match resp {
                        Ok(r) => {
                            if let Ok(v) = r.json::<EnableTotpResponse>().await {
                                if v.success {
                                    if let (Some(secret), Some(uri)) = (v.secret, v.uri) {
                                        set_secret.set(secret);
                                        set_uri.set(uri);
                                        set_step.set(1);
                                    }
                                } else {
                                    set_err.set(v.message);
                                }
                            } else {
                                set_err.set("Failed to parse server response".to_string());
                            }
                        }
                        Err(e) => set_err.set(format!("Network error: {}", e))
                    }
                } else {
                    let _ = (u, p, set_secret, set_uri, set_step, set_err);
                }
            }
        });
    };

    let on_confirm_totp = move |_| {
        set_setup_error.set(String::new());
        set_setup_success.set(String::new());
        let u = setup_user.get();
        let secret = setup_secret.get();
        let code = setup_code.get();
        if code.trim().is_empty() {
            set_setup_error.set("Please enter the verification code".to_string());
            return;
        }
        let set_step = set_setup_step.clone();
        let set_err = set_setup_error.clone();
        let set_succ = set_setup_success.clone();
        spawn_local(async move {
            cfg_if! {
                if #[cfg(feature = "hydrate")] {
                    let req = ConfirmTotpRequest { username: u, secret, code };
                    let resp = GlooRequest::post("/api/confirm_totp")
                        .json(&req)
                        .unwrap()
                        .send()
                        .await;
                    match resp {
                        Ok(r) => {
                            if let Ok(v) = r.json::<ConfirmTotpResponse>().await {
                                if v.success {
                                    set_succ.set(v.message);
                                    set_step.set(2);
                                } else {
                                    set_err.set(v.message);
                                }
                            } else {
                                set_err.set("Failed to parse server response".to_string());
                            }
                        }
                        Err(e) => set_err.set(format!("Network error: {}", e))
                    }
                } else {
                    let _ = (u, secret, code, set_step, set_err, set_succ);
                }
            }
        });
    };

    let on_toggle_email_2fa = move |enabled: bool| {
        set_email_2fa_status.set(String::new());
        let u = setup_user.get();
        let p = setup_pass.get();
        if u.trim().is_empty() || p.trim().is_empty() {
            set_email_2fa_status.set("Enter username and password above".to_string());
            return;
        }
        let set_status = set_email_2fa_status.clone();
        spawn_local(async move {
            cfg_if! {
                if #[cfg(feature = "hydrate")] {
                    let req = ToggleEmail2faRequest { username: u, password: p, enabled };
                    let resp = GlooRequest::post("/api/toggle_email_2fa")
                        .json(&req)
                        .unwrap()
                        .send()
                        .await;
                    match resp {
                        Ok(r) => {
                            if let Ok(v) = r.json::<ToggleEmail2faResponse>().await {
                                set_status.set(v.message);
                            } else {
                                set_status.set("Failed to parse server response".to_string());
                            }
                        }
                        Err(e) => set_status.set(format!("Network error: {}", e))
                    }
                } else {
                    let _ = (u, p, enabled, set_status);
                }
            }
        });
    };

    view! {
        <div class="home-screen">
            <div class="welcome-header">
                <h1>"Settings"</h1>
                <p>"Customize your experience"</p>
            </div>
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Appearance"</span>
                </div>
                <div class="settings-list">
                    <div class="list-item">
                        <div class="list-item-left">
                            <div class="list-item-title">"Theme"</div>
                        </div>
                        <div class="list-item-right">
                            <select
                                class="form-select"
                                prop:value={move || app_store.get().theme.as_str().to_string()}
                                on:change=move |ev| {
                                    let value = event_target_value(&ev);
                                    let theme = match value.as_str() {
                                        "light" => crate::types::Theme::Light,
                                        "dark" => crate::types::Theme::Dark,
                                        "high-contrast" => crate::types::Theme::HighContrast,
                                        "deuteranopia" => crate::types::Theme::Deuteranopia,
                                        "protanopia" => crate::types::Theme::Protanopia,
                                        "tritanopia" => crate::types::Theme::Tritanopia,
                                        "deuteranomaly" => crate::types::Theme::Deuteranomaly,
                                        "protanomaly" => crate::types::Theme::Protanomaly,
                                        "tritanomaly" => crate::types::Theme::Tritanomaly,
                                        "achromatopsia" => crate::types::Theme::Achromatopsia,
                                        "achromatomaly" => crate::types::Theme::Achromatomaly,
                                        "low-vision" => crate::types::Theme::LowVision,
                                        _ => crate::types::Theme::Light,
                                    };
                                    app_store.update(|s| s.set_theme(theme));
                                }
                            >
                                <option value="light">"Light"</option>
                                <option value="dark">"Dark"</option>
                                <option value="high-contrast">"High Contrast"</option>
                                <optgroup label="Color blindness">
                                    <option value="deuteranopia">"Deuteranopia (green-blind)"</option>
                                    <option value="protanopia">"Protanopia (red-blind)"</option>
                                    <option value="tritanopia">"Tritanopia (blue-blind)"</option>
                                    <option value="deuteranomaly">"Deuteranomaly (green-weak)"</option>
                                    <option value="protanomaly">"Protanomaly (red-weak)"</option>
                                    <option value="tritanomaly">"Tritanomaly (blue-weak)"</option>
                                    <option value="achromatopsia">"Achromatopsia (no colour)"</option>
                                    <option value="achromatomaly">"Achromatomaly (reduced colour)"</option>
                                </optgroup>
                                <option value="low-vision">"Low Vision / Legally Blind"</option>
                            </select>
                        </div>
                    </div>
                </div>
            </div>
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Notifications"</span>
                </div>
                <div class="settings-list">
                    <div class="list-item">
                        <div class="list-item-left">
                            <div class="list-item-title">"Email Notifications"</div>
                        </div>
                        <div class="list-item-right">
                            <input type="checkbox" checked />
                        </div>
                    </div>
                    <div class="list-item">
                        <div class="list-item-left">
                            <div class="list-item-title">"Push Notifications"</div>
                        </div>
                        <div class="list-item-right">
                            <input type="checkbox" checked />
                        </div>
                    </div>
                </div>
            </div>
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Import Contacts"</span>
                </div>
                <div class="settings-list">
                    <div class="list-item">
                        <div class="list-item-left">
                            <div class="list-item-title">"Contacts"</div>
                            <div class="list-item-desc">"Import phone numbers, WhatsApp chats, etc. into the messenger."</div>
                        </div>
                        <div class="list-item-right">
                            <button class="settings-action-btn" on:click=import_contacts>"Import"</button>
                        </div>
                    </div>
                </div>
            </div>
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Import Booking Data"</span>
                </div>
                <div class="settings-list">
                    <div class="list-item">
                        <div class="list-item-left">
                            <div class="list-item-title">"Xero"</div>
                        </div>
                        <div class="list-item-right">
                            <button class="settings-action-btn" on:click=move |_| import_channel("xero")>"Import"</button>
                        </div>
                    </div>
                    <div class="list-item">
                        <div class="list-item-left">
                            <div class="list-item-title">"Booking.com"</div>
                        </div>
                        <div class="list-item-right">
                            <button class="settings-action-btn" on:click=move |_| import_channel("booking")>"Import"</button>
                        </div>
                    </div>
                    <div class="list-item">
                        <div class="list-item-left">
                            <div class="list-item-title">"Airbnb"</div>
                        </div>
                        <div class="list-item-right">
                            <button class="settings-action-btn" on:click=move |_| import_channel("airbnb")>"Import"</button>
                        </div>
                    </div>
                </div>
            </div>
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
                                    <button class="settings-action-btn" on:click=on_enable_totp>"Enable Google Authenticator"</button>
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
                                        <button class="settings-action-btn" on:click=on_confirm_totp>"Confirm TOTP"</button>
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
                            <button class="settings-action-btn" on:click=move |_| on_toggle_email_2fa(true)>"Enable"</button>
                            <button class="settings-action-btn" on:click=move |_| on_toggle_email_2fa(false)>"Disable"</button>
                        </div>
                        {move || {
                            let status = email_2fa_status.get();
                            if status.is_empty() { ().into_any() } else {
                                view! { <div class="setup-status">{status}</div> }.into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
            {move || if !import_status.get().is_empty() {
                view! {
                    <div class="data-card import-status-card">
                        <div class="import-status">{import_status.get()}</div>
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </div>
    }
}
