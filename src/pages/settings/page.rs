use crate::models::{CalendarEvent, ContactSource, MessengerContact};
use crate::stores::{use_calendar_store, use_messenger_store};
use cfg_if::cfg_if;
use leptos::prelude::*;
use leptos::task::spawn_local;
use uuid::Uuid;

use crate::pages::settings::{
    accessibility_settings, account_settings, data_settings, display_settings,
    notification_settings, preset_selector, security_settings, SettingsTab,
};

#[cfg(feature = "hydrate")]
use crate::api::email::{
    ConfirmTotpRequest, ConfirmTotpResponse, EnableTotpRequest, EnableTotpResponse,
    ToggleEmail2faRequest, ToggleEmail2faResponse, TogglePhone2faRequest, TogglePhone2faResponse,
};
#[cfg(feature = "hydrate")]
use gloo_net::http::Request as GlooRequest;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let calendar_store = use_calendar_store();
    let messenger_store = use_messenger_store();
    let (import_status, set_import_status) = signal(String::new());
    let (active_tab, set_active_tab) = signal(SettingsTab::Appearance);

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
    let (phone_number, set_phone_number) = signal(String::new());
    let (phone_2fa_status, set_phone_2fa_status) = signal(String::new());

    // Developer mode test signals
    let (dev_msg_content, set_dev_msg_content) = signal(String::new());
    let (dev_doc_name, set_dev_doc_name) = signal(String::new());
    let (dev_doc_type, set_dev_doc_type) = signal(String::new());
    let (dev_tx_amount, set_dev_tx_amount) = signal(String::new());
    let (dev_tx_desc, set_dev_tx_desc) = signal(String::new());
    let (dev_cal_title, set_dev_cal_title) = signal(String::new());
    let (dev_cal_days, set_dev_cal_days) = signal(String::new());
    let (dev_portfolio_name, set_dev_portfolio_name) = signal(String::new());
    let (dev_user_name, set_dev_user_name) = signal(String::new());
    let (dev_notif_msg, set_dev_notif_msg) = signal(String::new());
    let (dev_notif_from, set_dev_notif_from) = signal(String::new());

    let import_contacts = Callback::new(move |_: leptos::ev::MouseEvent| {
        let names = vec!["Alice", "Bob", "Carol", "David"];
        messenger_store.update(|s| {
            for name in names {
                let contact = MessengerContact {
                    id: Uuid::new_v4(),
                    name: name.to_string(),
                    source: ContactSource::Imported,
                    phone: Some(format!(
                        "+61 4{:02} {:03} {:03}",
                        s.messages.len() + 1,
                        100,
                        200
                    )),
                    email: Some(format!("{}@example.com", name.to_lowercase())),
                    unread_count: 0,
                };
                s.add_messenger_contact(contact);
            }
        });
        set_import_status.set("Imported 4 contacts into messenger.".to_string());
    });

    let import_channel = Callback::new(move |channel: &'static str| {
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
        calendar_store.update(|s| {
            for ev in events {
                s.add_calendar_event(ev);
            }
        });
        set_import_status.set(format!("Imported {} bookings into calendar.", channel));
    });

    let on_enable_totp = Callback::new(move |_: leptos::ev::MouseEvent| {
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
    });

    let on_confirm_totp = Callback::new(move |_: leptos::ev::MouseEvent| {
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
    });

    let on_toggle_phone_2fa = Callback::new(move |enabled: bool| {
        set_phone_2fa_status.set(String::new());
        let u = setup_user.get();
        let p = setup_pass.get();
        let phone = phone_number.get();
        if u.trim().is_empty() || p.trim().is_empty() {
            set_phone_2fa_status
                .set("Enter username and password in the credentials section above".to_string());
            return;
        }
        if enabled && phone.trim().is_empty() {
            set_phone_2fa_status.set("Enter a phone number to enable phone 2FA".to_string());
            return;
        }
        let set_status = set_phone_2fa_status.clone();
        spawn_local(async move {
            cfg_if! {
                if #[cfg(feature = "hydrate")] {
                    let req = TogglePhone2faRequest { username: u, password: p, enabled, phone_number: phone };
                    let resp = GlooRequest::post("/api/toggle_phone_2fa")
                        .json(&req)
                        .unwrap()
                        .send()
                        .await;
                    match resp {
                        Ok(r) => {
                            if let Ok(v) = r.json::<TogglePhone2faResponse>().await {
                                set_status.set(v.message);
                            } else {
                                set_status.set("Failed to parse server response".to_string());
                            }
                        }
                        Err(e) => set_status.set(format!("Network error: {}", e))
                    }
                } else {
                    let _ = (u, p, phone, enabled, set_status);
                }
            }
        });
    });

    let on_toggle_email_2fa = Callback::new(move |enabled: bool| {
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
    });

    view! {
        <div class="home-screen settings-page">
            <div class="reporting-tabs-outer reporting-tabs-top">
                <div class="reporting-tabs">
                    {[
                        SettingsTab::Appearance,
                        SettingsTab::Account,
                        SettingsTab::Import,
                        SettingsTab::Accessibility,
                        SettingsTab::Storage,
                        SettingsTab::Notifications,
                        SettingsTab::TwoFactorAuth,
                        SettingsTab::Data,
                        SettingsTab::Developer,
                    ].iter().map(|&t| {
                        let label = t.label();
                        view! {
                            <button
                                class="reporting-tab"
                                class:active={move || active_tab.get() == t}
                                on:click=move |_| set_active_tab.set(t)
                            >
                                {label}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            <div
                class="settings-data-wrapper"
                class:settings-hidden={move || !matches!(
                    active_tab.get(),
                    SettingsTab::Import
                        | SettingsTab::Storage
                        | SettingsTab::Data
                        | SettingsTab::Developer
                )}
            >
                <data_settings::DataSettings
                    tab={active_tab}
                    import_status={import_status}
                    set_import_status={set_import_status}
                    import_contacts={import_contacts}
                    import_channel={import_channel}
                    dev_msg_content={dev_msg_content}
                    set_dev_msg_content={set_dev_msg_content}
                    dev_doc_name={dev_doc_name}
                    set_dev_doc_name={set_dev_doc_name}
                    dev_doc_type={dev_doc_type}
                    set_dev_doc_type={set_dev_doc_type}
                    dev_tx_amount={dev_tx_amount}
                    set_dev_tx_amount={set_dev_tx_amount}
                    dev_tx_desc={dev_tx_desc}
                    set_dev_tx_desc={set_dev_tx_desc}
                    dev_cal_title={dev_cal_title}
                    set_dev_cal_title={set_dev_cal_title}
                    dev_cal_days={dev_cal_days}
                    set_dev_cal_days={set_dev_cal_days}
                    dev_portfolio_name={dev_portfolio_name}
                    set_dev_portfolio_name={set_dev_portfolio_name}
                    dev_user_name={dev_user_name}
                    set_dev_user_name={set_dev_user_name}
                    dev_notif_msg={dev_notif_msg}
                    set_dev_notif_msg={set_dev_notif_msg}
                    dev_notif_from={dev_notif_from}
                    set_dev_notif_from={set_dev_notif_from}
                />
            </div>

            {move || match active_tab.get() {
                SettingsTab::Appearance => view! {
                    <display_settings::DisplaySettings />
                    <preset_selector::PresetSelector />
                }.into_any(),
                SettingsTab::Account => view! {
                    <account_settings::AccountSettings />
                }.into_any(),
                SettingsTab::Accessibility => view! {
                    <accessibility_settings::AccessibilitySettings />
                }.into_any(),
                SettingsTab::Notifications => view! {
                    <notification_settings::NotificationSettings />
                }.into_any(),
                SettingsTab::TwoFactorAuth => view! {
                    <security_settings::SecuritySettings
                        setup_user={setup_user}
                        set_setup_user={set_setup_user}
                        setup_pass={setup_pass}
                        set_setup_pass={set_setup_pass}
                        setup_secret={setup_secret}
                        setup_uri={setup_uri}
                        setup_code={setup_code}
                        set_setup_code={set_setup_code}
                        setup_error={setup_error}
                        setup_success={setup_success}
                        setup_step={setup_step}
                        email_2fa_status={email_2fa_status}
                        phone_number={phone_number}
                        set_phone_number={set_phone_number}
                        phone_2fa_status={phone_2fa_status}
                        on_enable_totp={on_enable_totp}
                        on_confirm_totp={on_confirm_totp}
                        on_toggle_email_2fa={on_toggle_email_2fa}
                        on_toggle_phone_2fa={on_toggle_phone_2fa}
                    />
                }.into_any(),
                _ => ().into_any(),
            }}
        </div>
    }
}
