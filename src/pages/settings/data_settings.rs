use crate::pages::organization::organization_forms::AddOrgForm;
use crate::pages::settings::SettingsTab;
use crate::stores::{
    use_app_store, use_calendar_store, use_messenger_store, use_notification_store,
    use_organization_store, use_transaction_store, use_ui_store,
};
use leptos::prelude::*;

#[component]
pub(crate) fn DataSettings(
    tab: ReadSignal<SettingsTab>,
    import_status: ReadSignal<String>,
    set_import_status: WriteSignal<String>,
    import_contacts: Callback<leptos::ev::MouseEvent>,
    import_channel: Callback<&'static str>,
    dev_msg_content: ReadSignal<String>,
    set_dev_msg_content: WriteSignal<String>,
    dev_doc_name: ReadSignal<String>,
    set_dev_doc_name: WriteSignal<String>,
    dev_doc_type: ReadSignal<String>,
    set_dev_doc_type: WriteSignal<String>,
    dev_tx_amount: ReadSignal<String>,
    set_dev_tx_amount: WriteSignal<String>,
    dev_tx_desc: ReadSignal<String>,
    set_dev_tx_desc: WriteSignal<String>,
    dev_cal_title: ReadSignal<String>,
    set_dev_cal_title: WriteSignal<String>,
    dev_cal_days: ReadSignal<String>,
    set_dev_cal_days: WriteSignal<String>,
    dev_portfolio_name: ReadSignal<String>,
    set_dev_portfolio_name: WriteSignal<String>,
    dev_user_name: ReadSignal<String>,
    set_dev_user_name: WriteSignal<String>,
    dev_notif_msg: ReadSignal<String>,
    set_dev_notif_msg: WriteSignal<String>,
    dev_notif_from: ReadSignal<String>,
    set_dev_notif_from: WriteSignal<String>,
) -> impl IntoView {
    let app_store = use_app_store();
    let organization_store = use_organization_store();
    let calendar_store = use_calendar_store();
    let messenger_store = use_messenger_store();
    let notification_store = use_notification_store();
    let transaction_store = use_transaction_store();
    let ui_store = use_ui_store();

    // Add Organization form state (mirrors Organization page)
    let (settings_show_add_org, set_settings_show_add_org) = signal(false);
    let (settings_new_org_name, set_settings_new_org_name) = signal(String::new());
    let (settings_new_org_image_url, set_settings_new_org_image_url) =
        signal(Option::<String>::None);
    let (settings_new_org_desc, set_settings_new_org_desc) = signal(String::new());
    let (settings_new_org_abn, set_settings_new_org_abn) = signal(String::new());
    let (settings_new_org_lei, set_settings_new_org_lei) = signal(String::new());
    let (settings_new_org_business_type, set_settings_new_org_business_type) =
        signal(String::new());
    let (settings_new_org_business_address, set_settings_new_org_business_address) =
        signal(String::new());
    let (settings_new_org_business_phone, set_settings_new_org_business_phone) =
        signal(String::new());
    let (settings_new_org_business_email, set_settings_new_org_business_email) =
        signal(String::new());

    let on_settings_add_org = move |_| {
        let name = settings_new_org_name.get();
        if name.trim().is_empty() {
            return;
        }
        let owner_id = app_store.get().current_user.id;
        let mut org = crate::models::Organization::new(name, owner_id);
        org.description = if settings_new_org_desc.get().trim().is_empty() {
            None
        } else {
            Some(settings_new_org_desc.get())
        };
        org.abn = if settings_new_org_abn.get().trim().is_empty() {
            None
        } else {
            Some(settings_new_org_abn.get())
        };
        org.lei = if settings_new_org_lei.get().trim().is_empty() {
            None
        } else {
            Some(settings_new_org_lei.get())
        };
        org.business_type = if settings_new_org_business_type.get().trim().is_empty() {
            None
        } else {
            Some(settings_new_org_business_type.get())
        };
        org.business_address = if settings_new_org_business_address.get().trim().is_empty() {
            None
        } else {
            Some(settings_new_org_business_address.get())
        };
        org.business_phone = if settings_new_org_business_phone.get().trim().is_empty() {
            None
        } else {
            Some(settings_new_org_business_phone.get())
        };
        org.business_email = if settings_new_org_business_email.get().trim().is_empty() {
            None
        } else {
            Some(settings_new_org_business_email.get())
        };
        org.image_url = settings_new_org_image_url.get();
        organization_store.update(|s| s.add_organization(org));
        set_settings_new_org_name.set(String::new());
        set_settings_new_org_image_url.set(None);
        set_settings_new_org_desc.set(String::new());
        set_settings_new_org_abn.set(String::new());
        set_settings_new_org_lei.set(String::new());
        set_settings_new_org_business_type.set(String::new());
        set_settings_new_org_business_address.set(String::new());
        set_settings_new_org_business_phone.set(String::new());
        set_settings_new_org_business_email.set(String::new());
        set_settings_show_add_org.set(false);
        set_import_status.set("Organization created.".to_string());
    };

    view! {
        <div class="data-card" class:settings-hidden={move || tab.get() != SettingsTab::Storage}>
            <div class="card-header">
                <span class="card-title">"Storage Options"</span>
            </div>
            <div class="settings-list">
                {move || {
                    let username = app_store.get().current_user.username.clone();
                    if username.is_empty() {
                        view! {
                            <div class="list-item">
                                <div class="list-item-left">
                                    <div class="list-item-title">"Not signed in"</div>
                                    <div class="list-item-desc">"Sign in to manage storage options for your account."</div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        let (store_local, store_cloud) = app_store.get().credentials.get_storage_options(&username);
                        let username_local = username.clone();
                        let username_cloud = username.clone();
                        view! {
                            <div class="list-item">
                                <div class="list-item-left">
                                    <div class="list-item-title">"Local Storage"</div>
                                    <div class="list-item-desc">"Store credentials and data locally on this device."</div>
                                </div>
                                <div class="list-item-right">
                                    <label class="lp-storage-option" class:active={move || store_local}>
                                        <input
                                            type="checkbox"
                                            checked={move || store_local}
                                            on:change={move |ev| {
                                                let checked = event_target_checked(&ev);
                                                app_store.update(|s| {
                                                    s.set_storage_options(&username_local, checked, store_cloud);
                                                });
                                            }}
                                        />
                                        "Local"
                                    </label>
                                </div>
                            </div>
                            <div class="list-item">
                                <div class="list-item-left">
                                    <div class="list-item-title">"Cloud Storage"</div>
                                    <div class="list-item-desc">"Sync credentials and data with the server."</div>
                                </div>
                                <div class="list-item-right">
                                    <label class="lp-storage-option" class:active={move || store_cloud}>
                                        <input
                                            type="checkbox"
                                            checked={move || store_cloud}
                                            on:change={move |ev| {
                                                let checked = event_target_checked(&ev);
                                                app_store.update(|s| {
                                                    s.set_storage_options(&username_cloud, store_local, checked);
                                                });
                                            }}
                                        />
                                        "Cloud"
                                    </label>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>

        <div class="data-card" class:settings-hidden={move || tab.get() != SettingsTab::Developer}>
            <div class="card-header">
                <span class="card-title">"Developer Mode"</span>
            </div>
            <div class="settings-list">
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Enable Developer Mode"</div>
                        <div class="list-item-desc">"Toggle on to access developer tools for testing notifications, creating test members, and other features."</div>
                    </div>
                    <div class="list-item-right">
                        <input type="checkbox" prop:checked={move || app_store.get().developer_mode}
                            on:change=move |ev| {
                                let checked = event_target_checked(&ev);
                                app_store.update(|s| s.developer_mode = checked);
                            }
                        />
                    </div>
                </div>
            </div>
            {move || if app_store.get().developer_mode {
                view! {
                    <div class="dev-test-panel">
                        <div class="dev-test-section">
                            <div class="dev-test-section-title">"Notifications"</div>
                            <div class="dev-test-grid">
                                <input class="dev-input" type="text" placeholder="From user"
                                    prop:value={move || dev_notif_from.get()}
                                    on:input=move |ev| set_dev_notif_from.set(event_target_value(&ev)) />
                                <input class="dev-input" type="text" placeholder="Message"
                                    prop:value={move || dev_notif_msg.get()}
                                    on:input=move |ev| set_dev_notif_msg.set(event_target_value(&ev)) />
                            </div>
                            <div class="dev-test-grid">
                                <button class="settings-action-btn" on:click=move |_| {
                                    let f=dev_notif_from.get(); let m=dev_notif_msg.get();
                                    let dev_mode = app_store.get().developer_mode;
                                    notification_store.update(|s| s.send_test_notification(dev_mode, if f.is_empty(){"Bot"}else{&f}, if m.is_empty(){"Test"}else{&m}, crate::types::TabType::Portfolios));
                                }>"→ Portfolios"</button>
                                <button class="settings-action-btn" on:click=move |_| {
                                    let f=dev_notif_from.get(); let m=dev_notif_msg.get();
                                    let dev_mode = app_store.get().developer_mode;
                                    notification_store.update(|s| s.send_test_notification(dev_mode, if f.is_empty(){"Bot"}else{&f}, if m.is_empty(){"Test"}else{&m}, crate::types::TabType::Transactions));
                                }>"→ Transactions"</button>
                                <button class="settings-action-btn" on:click=move |_| {
                                    let f=dev_notif_from.get(); let m=dev_notif_msg.get();
                                    let dev_mode = app_store.get().developer_mode;
                                    notification_store.update(|s| s.send_test_notification(dev_mode, if f.is_empty(){"Bot"}else{&f}, if m.is_empty(){"Test"}else{&m}, crate::types::TabType::Networking));
                                }>"→ Networking"</button>
                                <button class="settings-action-btn" on:click=move |_| {
                                    let f=dev_notif_from.get(); let m=dev_notif_msg.get();
                                    let dev_mode = app_store.get().developer_mode;
                                    notification_store.update(|s| s.send_test_notification(dev_mode, if f.is_empty(){"Bot"}else{&f}, if m.is_empty(){"Test"}else{&m}, crate::types::TabType::Calendar));
                                }>"→ Calendar"</button>
                                <button class="settings-action-btn" on:click=move |_| {
                                    let f=dev_notif_from.get(); let m=dev_notif_msg.get();
                                    let dev_mode = app_store.get().developer_mode;
                                    notification_store.update(|s| s.send_test_notification(dev_mode, if f.is_empty(){"Bot"}else{&f}, if m.is_empty(){"Test"}else{&m}, crate::types::TabType::Organization));
                                }>"→ Organization"</button>
                                <button class="settings-action-btn dev-btn-danger" on:click=move |_| {
                                    notification_store.update(|s| s.clear_notifications());
                                }>"Clear All"</button>
                            </div>
                        </div>
                        //DEV_INSERT
                        <div class="dev-test-section">
                            <div class="dev-test-section-title">"Calendar"</div>
                            <div class="dev-test-grid">
                                <input class="dev-input" type="text" placeholder="Event title"
                                    prop:value={move || dev_cal_title.get()}
                                    on:input=move |ev| set_dev_cal_title.set(event_target_value(&ev)) />
                                <input class="dev-input" type="text" placeholder="Days from now"
                                    prop:value={move || dev_cal_days.get()}
                                    on:input=move |ev| set_dev_cal_days.set(event_target_value(&ev)) />
                            </div>
                            <div class="dev-test-grid">
                                <button class="settings-action-btn" on:click=move |_| {
                                    let t=dev_cal_title.get(); let d=dev_cal_days.get();
                                    let days: i64 = d.parse().unwrap_or(3);
                                    let dev_mode = app_store.get().developer_mode;
                                    let title = if t.is_empty(){"Meeting with Bot"}else{&t};
                                    let notif = &mut notification_store.get_untracked();
                                    calendar_store.update(|s| {
                                        if dev_mode {
                                            s.dev_test_add_calendar_event(title, days);
                                            notif.add_notification_for(
                                                format!("Event: \"{}\"", title),
                                                crate::stores::NotificationType::Info,
                                                Some(crate::types::TabType::Calendar),
                                                Some("System".into()),
                                            );
                                        }
                                    });
                                }>"Add Event"</button>
                            </div>
                        </div>
                        <div class="dev-test-section">
                            <div class="dev-test-section-title">"Portfolios"</div>
                            <div class="dev-test-grid">
                                <input class="dev-input" type="text" placeholder="Portfolio name"
                                    prop:value={move || dev_portfolio_name.get()}
                                    on:input=move |ev| set_dev_portfolio_name.set(event_target_value(&ev)) />
                            </div>
                            <div class="dev-test-grid">
                                <button class="settings-action-btn" on:click=move |_| {
                                    let n=dev_portfolio_name.get();
                                    app_store.update(|s| { s.dev_test_add_portfolio(if n.is_empty(){"Test Portfolio"}else{&n}, &mut notification_store.get_untracked()); });
                                }>"Create Portfolio"</button>
                            </div>
                        </div>
                        <div class="dev-test-section">
                            <div class="dev-test-section-title">"Organization Users"</div>
                            <div class="dev-test-grid">
                                <input class="dev-input" type="text" placeholder="User name"
                                    prop:value={move || dev_user_name.get()}
                                    on:input=move |ev| set_dev_user_name.set(event_target_value(&ev)) />
                            </div>
                            <div class="dev-test-grid">
                                <button class="settings-action-btn" on:click=move |_| {
                                    let n=dev_user_name.get();
                                    organization_store.update(|s| s.dev_test_add_org_user(if n.is_empty(){"TestUser"}else{&n}, crate::types::UserRole::Manager, &mut notification_store.get_untracked()));
                                }>"Add as Manager"</button>
                                <button class="settings-action-btn" on:click=move |_| {
                                    let n=dev_user_name.get();
                                    organization_store.update(|s| s.dev_test_add_org_user(if n.is_empty(){"TestUser"}else{&n}, crate::types::UserRole::Worker, &mut notification_store.get_untracked()));
                                }>"Add as Worker"</button>
                            </div>
                        </div>
                        <div class="dev-test-section">
                            <div class="dev-test-section-title">"Documents"</div>
                            <div class="dev-test-grid">
                                <input class="dev-input" type="text" placeholder="Document name"
                                    prop:value={move || dev_doc_name.get()}
                                    on:input=move |ev| set_dev_doc_name.set(event_target_value(&ev)) />
                                <input class="dev-input" type="text" placeholder="File type (pdf, docx)"
                                    prop:value={move || dev_doc_type.get()}
                                    on:input=move |ev| set_dev_doc_type.set(event_target_value(&ev)) />
                            </div>
                            <div class="dev-test-grid">
                                <button class="settings-action-btn" on:click=move |_| {
                                    let n=dev_doc_name.get(); let t=dev_doc_type.get();
                                    app_store.update(|s| { s.dev_test_add_document(if n.is_empty(){"Lease Agreement"}else{&n}, if t.is_empty(){"pdf"}else{&t}, &mut notification_store.get_untracked()); });
                                }>"Add Document"</button>
                                <button class="settings-action-btn" on:click=move |_| {
                                    let n=dev_doc_name.get(); let t=dev_doc_type.get();
                                    let dn = if n.is_empty() {"Contract Draft"} else {&n};
                                    app_store.update(|s| {
                                        if let Some(id) = s.dev_test_add_document(dn, if t.is_empty(){"docx"}else{&t}, &mut notification_store.get_untracked()) {
                                            s.dev_test_update_document(id, &format!("{} (updated)", dn), &mut notification_store.get_untracked());
                                        }
                                    });
                                }>"Add + Bot Update"</button>
                            </div>
                        </div>
                        <div class="dev-test-section">
                            <div class="dev-test-section-title">"Transactions"</div>
                            <div class="dev-test-grid">
                                <input class="dev-input" type="text" placeholder="Amount (e.g. 5000)"
                                    prop:value={move || dev_tx_amount.get()}
                                    on:input=move |ev| set_dev_tx_amount.set(event_target_value(&ev)) />
                                <input class="dev-input" type="text" placeholder="Description"
                                    prop:value={move || dev_tx_desc.get()}
                                    on:input=move |ev| set_dev_tx_desc.set(event_target_value(&ev)) />
                            </div>
                            <div class="dev-test-grid">
                                <button class="settings-action-btn" on:click=move |_| {
                                    let a=dev_tx_amount.get(); let d=dev_tx_desc.get();
                                    let amt: f64 = a.parse().unwrap_or(5000.0);
                                    let dev_mode = app_store.get().developer_mode;
                                    let app = app_store.get();
                                    let uid = app.current_user.id;
                                    let name = app.current_user.name.clone();
                                    let notif = &mut notification_store.get_untracked();
                                    let desc = if d.is_empty(){"Test transfer"}else{&d};
                                    transaction_store.update(|s| {
                                        if dev_mode {
                                            let tx = s.dev_test_add_transaction(uid, name, amt, desc);
                                            notif.add_notification_for(
                                                format!("Transaction: ${:.2} - {}", tx.amount, desc),
                                                crate::stores::NotificationType::Info,
                                                Some(crate::types::TabType::Transactions),
                                                Some("Bot".into()),
                                            );
                                        }
                                    });
                                }>"Create Pending"</button>
                                <button class="settings-action-btn" on:click=move |_| {
                                    let dev_mode = app_store.get().developer_mode;
                                    let notif = &mut notification_store.get_untracked();
                                    transaction_store.update(|s| {
                                        if dev_mode {
                                            if let Some(tx) = s.dev_test_approve_last_tx() {
                                                notif.add_notification_for(
                                                    format!("Approved: ${:.2}", tx.amount),
                                                    crate::stores::NotificationType::Success,
                                                    Some(crate::types::TabType::Transactions),
                                                    Some("System".into()),
                                                );
                                            }
                                        }
                                    });
                                }>"Approve Last"</button>
                                <button class="settings-action-btn" on:click=move |_| {
                                    let dev_mode = app_store.get().developer_mode;
                                    let notif = &mut notification_store.get_untracked();
                                    transaction_store.update(|s| {
                                        if dev_mode {
                                            if let Some(tx) = s.dev_test_execute_last_tx() {
                                                notif.add_notification_for(
                                                    format!("Executed: ${:.2}", tx.amount),
                                                    crate::stores::NotificationType::Success,
                                                    Some(crate::types::TabType::Transactions),
                                                    Some("System".into()),
                                                );
                                            }
                                        }
                                    });
                                }>"Execute Last"</button>
                            </div>
                        </div>
                        <div class="dev-test-section">
                            <div class="dev-test-section-title">"Messaging"</div>
                            <div class="dev-test-grid">
                                <input class="dev-input" type="text" placeholder="Message from Bot"
                                    prop:value={move || dev_msg_content.get()}
                                    on:input=move |ev| set_dev_msg_content.set(event_target_value(&ev)) />
                            </div>
                            <div class="dev-test-grid">
                                <button class="settings-action-btn" on:click=move |_| {
                                    let c=dev_msg_content.get();
                                    let dev_mode = app_store.get().developer_mode;
                                    let current_user_id = app_store.get().current_user.id;
                                    let notif = &mut notification_store.get_untracked();
                                    messenger_store.update(|s| {
                                        if dev_mode {
                                            s.dev_test_message_from_bot(current_user_id, if c.is_empty(){"Hey Red, review the report?"}else{&c});
                                            notif.add_notification_for(
                                                format!("New message from Bot: \"{}\"", if c.is_empty(){"Hey Red, review the report?"}else{&c}),
                                                crate::stores::NotificationType::Info,
                                                Some(crate::types::TabType::Networking),
                                                Some("Bot".into()),
                                            );
                                        }
                                    });
                                }>"Receive from Bot"</button>
                                <button class="settings-action-btn" on:click=move |_| {
                                    let dev_mode = app_store.get().developer_mode;
                                    messenger_store.update(|s| {
                                        if dev_mode {
                                            s.dev_test_add_bot_contact();
                                        }
                                    });
                                }>"Add Bot Contact"</button>
                                <button class="settings-action-btn" on:click=move |_| {
                                    messenger_store.update(|s| s.set_message_drawer(true));
                                }>"Open Messenger"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </div>

        <div class="data-card" class:settings-hidden={move || tab.get() != SettingsTab::Import}>
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
                        <button class="settings-action-btn" on:click=move |ev| import_contacts.run(ev)>"Import"</button>
                    </div>
                </div>
            </div>
        </div>

        <div class="data-card" class:settings-hidden={move || tab.get() != SettingsTab::Import}>
            <div class="card-header">
                <span class="card-title">"Import Booking Data"</span>
            </div>
            <div class="settings-list">
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Xero"</div>
                    </div>
                    <div class="list-item-right">
                        <button class="settings-action-btn" on:click=move |_| import_channel.run("xero")>"Import"</button>
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Booking.com"</div>
                    </div>
                    <div class="list-item-right">
                        <button class="settings-action-btn" on:click=move |_| import_channel.run("booking")>"Import"</button>
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Airbnb"</div>
                    </div>
                    <div class="list-item-right">
                        <button class="settings-action-btn" on:click=move |_| import_channel.run("airbnb")>"Import"</button>
                    </div>
                </div>
            </div>
        </div>

        <div class="data-card" class:settings-hidden={move || tab.get() != SettingsTab::Import}>
            <div class="card-header">
                <span class="card-title">"Import Organizations"</span>
            </div>
            <div class="settings-list">
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Organizations"</div>
                        <div class="list-item-desc">"Create a new organization or import from external sources."</div>
                    </div>
                    <div class="list-item-right">
                        <button
                            class="settings-action-btn"
                            on:click=move |_| set_settings_show_add_org.update(|v| *v = !*v)
                        >
                            {move || if settings_show_add_org.get() { "Cancel" } else { "Add / Import" }}
                        </button>
                    </div>
                </div>
                {move || if settings_show_add_org.get() {
                    view! {
                        <div class="list-item org-import-add-form">
                            <AddOrgForm
                                show={settings_show_add_org}
                                _set_show={set_settings_show_add_org}
                                name={settings_new_org_name}
                                set_name={set_settings_new_org_name}
                                desc={settings_new_org_desc}
                                set_desc={set_settings_new_org_desc}
                                image_url={settings_new_org_image_url}
                                set_image_url={set_settings_new_org_image_url}
                                abn={settings_new_org_abn}
                                set_abn={set_settings_new_org_abn}
                                lei={settings_new_org_lei}
                                set_lei={set_settings_new_org_lei}
                                business_type={settings_new_org_business_type}
                                set_business_type={set_settings_new_org_business_type}
                                business_address={settings_new_org_business_address}
                                set_business_address={set_settings_new_org_business_address}
                                business_phone={settings_new_org_business_phone}
                                set_business_phone={set_settings_new_org_business_phone}
                                business_email={settings_new_org_business_email}
                                set_business_email={set_settings_new_org_business_email}
                                on_add={Callback::new(move |_| on_settings_add_org(()))}
                            />
                        </div>
                    }.into_any()
                } else { ().into_any() }}
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Import Organization"</div>
                        <div class="list-item-desc">"Import an existing organization from LinkedIn, ABN Lookup, or other sources."</div>
                    </div>
                    <div class="list-item-right">
                        <button class="settings-action-btn" disabled>"Import"</button>
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"LinkedIn Import"</div>
                        <div class="list-item-desc">"Sync your organization details from LinkedIn."</div>
                    </div>
                    <div class="list-item-right">
                        <button class="settings-action-btn" disabled>"Import"</button>
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Assisted Setup"</div>
                        <div class="list-item-desc">"Let the assistant create an organization profile for you."</div>
                    </div>
                    <div class="list-item-right">
                        <button class="settings-action-btn" disabled>"Setup"</button>
                    </div>
                </div>
            </div>
        </div>

        <div class="data-card" class:settings-hidden={move || tab.get() != SettingsTab::Data}>
            <div class="card-header">
                <span class="card-title">"Data & Reset"</span>
            </div>
            <div class="settings-list">
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Reset Settings"</div>
                        <div class="list-item-desc">"Restore appearance, accessibility, and notification preferences to defaults."</div>
                    </div>
                    <div class="list-item-right">
                        <button
                            class="settings-action-btn settings-action-btn-danger"
                            aria-label="Reset all settings to defaults"
                            on:click=move |_| {
                                ui_store.update(|s| {
                                    s.reset_display_preferences();
                                });
                                app_store.update(|s| {
                                    s.developer_mode = false;
                                });
                                notification_store.update(|s| {
                                    s.email_notifications = true;
                                    s.push_notifications = true;
                                    s.sound_enabled = true;
                                });
                                set_import_status.set("Settings restored to defaults.".to_string());
                            }
                        >"Reset to Defaults"</button>
                    </div>
                </div>
            </div>
        </div>

        {move || if !import_status.get().is_empty() {
            view! {
                <div class="data-card import-status-card" class:settings-hidden={move || !matches!(tab.get(), SettingsTab::Storage | SettingsTab::Import | SettingsTab::Developer | SettingsTab::Data)}>
                    <div class="import-status">{import_status.get()}</div>
                </div>
            }.into_any()
        } else { ().into_any() }}
    }
}
