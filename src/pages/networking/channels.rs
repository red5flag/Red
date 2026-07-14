use crate::models::{Channel as AssetChannel, ChannelType};
use crate::stores::use_app_store;
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

pub(crate) fn render_channels() -> impl IntoView {
    let app_store = use_app_store();
    let channels = Memo::new(move |_| app_store.get().channels.clone());

    let (new_name, set_new_name) = signal(String::new());

    let create_test = Callback::new(move |_| {
        let name = {
            let n = new_name.get();
            if n.trim().is_empty() {
                "Test Channel".to_string()
            } else {
                n
            }
        };
        app_store.update(|s| s.add_channel(AssetChannel::new_test_channel(name, None, None)));
        set_new_name.set(String::new());
    });

    let connect = Callback::new(move |id: Uuid| {
        app_store.update(|s| {
            if let Some(c) = s.get_channel_mut(id) {
                c.connect();
            }
        });
    });

    let disconnect = Callback::new(move |id: Uuid| {
        app_store.update(|s| {
            if let Some(c) = s.get_channel_mut(id) {
                c.disconnect();
            }
        });
    });

    let check = Callback::new(move |id: Uuid| {
        app_store.update(|s| {
            if let Some(c) = s.get_channel_mut(id) {
                c.check_connection();
            }
        });
    });

    let sync = Callback::new(move |id: Uuid| {
        app_store.update(|s| {
            if let Some(c) = s.get_channel_mut(id) {
                c.last_sync_at = Some(Utc::now());
                c.last_sync_status = Some("Test channel sync completed (local)".to_string());
                c.sync_errors.clear();
            }
        });
    });

    let remove = Callback::new(move |id: Uuid| {
        app_store.update(|s| {
            s.remove_channel(id);
        });
    });

    let external_channels = [
        (
            "Airbnb",
            "External short-term rental integration (deferred)",
        ),
        ("Booking.com", "External hotel/OTA integration (deferred)"),
        ("Expedia", "External travel integration (deferred)"),
        ("Vrbo", "External vacation rental integration (deferred)"),
        (
            "LinkedIn",
            "External professional network integration (deferred)",
        ),
    ];

    view! {
        <div class="net-tab-content">
            <div class="net-section-header">
                <span class="net-section-title">"Asset Channels"</span>
                <span class="net-section-subtitle">"Test Channel is the only active local integration. Real-world APIs are disabled in this stage."</span>
            </div>

            <div class="net-channels-create">
                <input
                    type="text"
                    placeholder="New Test Channel name"
                    prop:value={move || new_name.get()}
                    on:input=move |ev| set_new_name.set(event_target_value(&ev))
                />
                <button class="net-btn net-btn-primary" on:click=move |_| create_test.run(())>"Create Test Channel"</button>
            </div>

            <div class="net-channels-grid">
                {move || {
                    let list = channels.get();
                    if list.is_empty() {
                        view! { <div class="net-channels-empty">"No channels configured yet. Create a Test Channel to begin."</div> }.into_any()
                    } else {
                        view! {
                            <div>
                                {list.into_iter().map(|ch| {
                                    let id = ch.id;
                                    let is_test = ch.is_test();
                                    let status = format!("{:?}", ch.connection_status);
                                    let type_label = match ch.channel_type {
                                        ChannelType::Test => "Test Channel",
                                        ChannelType::Airbnb => "Airbnb",
                                        ChannelType::BookingCom => "Booking.com",
                                        ChannelType::Expedia => "Expedia",
                                        ChannelType::Vrbo => "Vrbo",
                                        ChannelType::LinkedIn => "LinkedIn",
                                        ChannelType::Other(_) => "Other",
                                    };
                                    let linked = ch.linked_asset_id.map(|_| "Linked to asset").unwrap_or("Unlinked");
                                    let last = ch.last_sync_status.clone().unwrap_or_else(|| "Never synced".to_string());
                                    let rate = ch.nightly_rate_override.map(|r| format!("${:.2}", r)).unwrap_or_else(|| "—".to_string());
                                    let min = ch.minimum_nights.map(|n| n.to_string()).unwrap_or_else(|| "—".to_string());
                                    let max = ch.maximum_nights.map(|n| n.to_string()).unwrap_or_else(|| "—".to_string());
                                    let comm = ch.commission_percent.map(|p| format!("{:.1}%", p)).unwrap_or_else(|| "—".to_string());
                                    let errors = if ch.sync_errors.is_empty() {
                                        "No errors".to_string()
                                    } else {
                                        format!("{} errors", ch.sync_errors.len())
                                    };
                                    view! {
                                        <div class="net-channel-card">
                                            <div class="net-channel-icon">{if is_test { "🧪" } else { "📡" }}</div>
                                            <div class="net-channel-name">{ch.name.clone()}</div>
                                            <div class="net-channel-type">{type_label}</div>
                                            <div class="net-channel-meta">{linked}" | Status: "{status}</div>
                                            <div class="net-channel-meta">{"Rate: "}{rate}{" | Min/Max: "}{min}{" / "}{max}{" | Commission: "}{comm}</div>
                                            <div class="net-channel-last">{last}" | "{errors}</div>
                                            <div class="net-channel-actions">
                                                <button class="net-btn" on:click=move |_| connect.run(id)>"Connect"</button>
                                                <button class="net-btn" on:click=move |_| disconnect.run(id)>"Disconnect"</button>
                                                <button class="net-btn" on:click=move |_| check.run(id)>"Check"</button>
                                                <button class="net-btn" on:click=move |_| sync.run(id)>"Sync"</button>
                                                <button class="net-btn net-btn-danger" on:click=move |_| remove.run(id)>"Remove"</button>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            <div class="net-external-channels">
                <div class="net-section-header">
                    <span class="net-section-title">"External Integrations"</span>
                    <span class="net-section-subtitle">"Available integrations are shown for planning but are not connected in Stage 1."</span>
                </div>
                <div class="net-channels-grid">
                    {external_channels.into_iter().map(|(name, desc)| view! {
                        <div class="net-channel-card net-channel-disabled">
                            <div class="net-channel-icon">"🔒"</div>
                            <div class="net-channel-name">{name}</div>
                            <div class="net-channel-type">{desc}</div>
                            <span class="net-rel-status net-status-pending">"Deferred"</span>
                        </div>
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}
