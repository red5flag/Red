use crate::models::{
    Asset, Booking, BookingSource, BookingStatus, Channel, ChannelType, ConditionStatus,
    ConnectionStatus, ContactSource, Message, MessageEmbed, MessengerContact, SyncDirection,
    ebay_site,
};
#[cfg(feature = "hydrate")]
use crate::models::{MessageStatus, MessengerState};
#[cfg(feature = "hydrate")]
use crate::server::{save_messenger_state, send_secure_message};
use crate::stores::{use_app_store, use_calendar_store, use_messenger_store, AppStore, MessengerStore};
use chrono::{Duration, NaiveDate, Utc};
use leptos::prelude::*;
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use super::phantom_edit_scope;
use uuid::Uuid;
use wasm_bindgen::JsCast;

/// Registry of marketplaces an asset can be crosslisted to (FLUF-style).
/// Add a destination by appending one entry here (plus its `ChannelType`
/// variant in `models/channel.rs`) — every UI surface reads from this table.
#[derive(Clone)]
struct MarketplaceDef {
    /// Stable key — also the `ChannelType` variant name used for matching.
    key: &'static str,
    label: &'static str,
    icon: &'static str,
    channel_type: Option<ChannelType>,
    hint: &'static str,
}

static MARKETPLACES: &[MarketplaceDef] = &[
    MarketplaceDef {
        key: "Red",
        label: "Red Marketplace",
        icon: "🏪",
        // Internal marketplace — no external channel type or connect flow.
        channel_type: None,
        hint: "List on the Red Marketplace — visible to your network in the Marketplace tab.",
    },
    MarketplaceDef {
        key: "EBay",
        label: "eBay",
        icon: "🛒",
        channel_type: Some(ChannelType::EBay),
        hint: "List on eBay with auto-filled title, price, photos and details.",
    },
];

fn marketplace_def(key: &str) -> Option<&'static MarketplaceDef> {
    MARKETPLACES.iter().find(|d| d.key == key)
}

/// Display label for a marketplace key ("EBay" → "eBay").
pub(crate) fn marketplace_display_label(key: &str) -> &'static str {
    marketplace_def(key).map(|d| d.label).unwrap_or(match key {
        "EBay" => "eBay",
        "Amazon" => "Amazon",
        "Etsy" => "Etsy",
        "Kogan" => "Kogan",
        _ => "Marketplace",
    })
}

/// Ensure a messenger contact + opening thread exist for a published
/// listing. The opening message embeds the listing itself so the item is
/// the reference at the start of the conversation. Persists the messenger
/// directory and seed message immediately (the drawer may be closed).
/// Returns the thread's contact id.
fn ensure_marketplace_thread(
    messenger_store: RwSignal<MessengerStore>,
    current_user_id: Uuid,
    asset_id: Uuid,
    mp_key: &str,
    listing: &MarketplaceListing,
) -> Uuid {
    let tid = listing.thread_id.unwrap_or_else(Uuid::new_v4);
    let label = marketplace_display_label(mp_key);
    let contact_name = format!("{} · {}", label, listing.title);
    let embed = MessageEmbed::Listing {
        asset_id,
        marketplace: mp_key.to_string(),
    };
    let mut seed_msg: Option<Message> = None;
    messenger_store.update(|ms| {
        if let Some(c) = ms.messenger_contacts.iter_mut().find(|c| c.id == tid) {
            c.name = contact_name.clone();
        } else {
            ms.messenger_contacts.push(MessengerContact {
                id: tid,
                name: contact_name.clone(),
                source: ContactSource::Marketplace,
                phone: None,
                email: None,
                icon: Some("🏷️".to_string()),
                members: Vec::new(),
                unread_count: 0,
            });
        }
        let has_msgs = ms.messages.iter().any(|m| {
            (m.sender_id == tid && m.recipient_id == current_user_id)
                || (m.sender_id == current_user_id && m.recipient_id == tid)
        });
        if !has_msgs {
            // receive_* also seals into the local store under ssr/tauri;
            // hydrate persistence goes through send_secure_message below.
            let msg = ms.receive_message_with_embeds(
                tid,
                current_user_id,
                format!("Listing published — {}", listing.title),
                vec![embed],
            );
            seed_msg = Some(msg);
        }
    });
    #[cfg(feature = "hydrate")]
    {
        let state = MessengerState {
            contacts: messenger_store.get_untracked().messenger_contacts.clone(),
            requests: messenger_store.get_untracked().messenger_requests.clone(),
        };
        leptos::task::spawn_local(async move {
            let _ = save_messenger_state(state).await;
            if let Some(m) = seed_msg {
                let mid = m.id;
                if send_secure_message(m).await.is_ok() {
                    messenger_store.update(|s| s.set_message_status(mid, MessageStatus::Sent));
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = seed_msg;
    }
    tid
}

fn marketplace_channel_type(key: &str) -> Option<ChannelType> {
    // Red Marketplace is internal — it links via a generic channel type.
    if key == "Red" {
        return Some(ChannelType::Other("Red Marketplace".to_string()));
    }
    marketplace_def(key).and_then(|d| d.channel_type.clone())
}

/// Marketplace-appropriate condition label derived from the asset's
/// `ConditionStatus` (New / Excellent … Unsafe).
fn marketplace_condition(cs: &ConditionStatus) -> &'static str {
    match cs {
        ConditionStatus::New => "New",
        ConditionStatus::Excellent | ConditionStatus::Good | ConditionStatus::Fair => "Used",
        ConditionStatus::Poor | ConditionStatus::Damaged | ConditionStatus::Unsafe => "For parts",
    }
}

#[component]
pub(crate) fn AssetChannelsSection(
    asset_id: Uuid,
    asset_name: String,
    portfolio_id: Option<Uuid>,
    can_edit: bool,
) -> impl IntoView {
    let app_store = use_app_store();
    let calendar_store = use_calendar_store();
    let (asset_name_signal, _) = signal(asset_name);

    let channels_for_asset = Memo::new(move |_| {
        app_store
            .get()
            .channels_for_asset(asset_id)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
    });

    let bookings_for_asset = Memo::new(move |_| {
        app_store
            .get()
            .bookings_for_asset(asset_id)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
    });

    let (show_booking_form, set_show_booking_form) = signal(false);
    let (guest_name, set_guest_name) = signal(String::new());
    let (start_date, set_start_date) = signal(String::new());
    let (end_date, set_end_date) = signal(String::new());
    let (cost_per_night, set_cost_per_night) = signal(String::from("100"));
    let (conflict_msg, set_conflict_msg) = signal(Option::<String>::None);
    let (form_error, set_form_error) = signal(Option::<String>::None);
    let (editing_channel, set_editing_channel) = signal(Option::<Uuid>::None);
    let on_close_channel_window = Callback::new(move |_| set_editing_channel.set(None));

    let link_test_channel = Callback::new(move |_| {
        let name = format!("Test Channel - {}", asset_name_signal.get_untracked());
        let channel = Channel::new_test_channel(name, Some(asset_id), portfolio_id);
        app_store.update(|s| s.add_channel(channel));
    });

    let create_test_booking = Callback::new(move |_| {
        let guest = guest_name.get();
        let start_str = start_date.get();
        let end_str = end_date.get();
        let cost = cost_per_night.get();

        if guest.trim().is_empty() || start_str.trim().is_empty() || end_str.trim().is_empty() {
            set_form_error.set(Some("All fields are required".to_string()));
            return;
        }

        let start_naive = match NaiveDate::parse_from_str(&start_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                set_form_error.set(Some("Invalid start date (YYYY-MM-DD)".to_string()));
                return;
            }
        };
        let end_naive = match NaiveDate::parse_from_str(&end_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                set_form_error.set(Some("Invalid end date (YYYY-MM-DD)".to_string()));
                return;
            }
        };
        let start = start_naive.and_hms_opt(15, 0, 0).unwrap().and_utc();
        let end = end_naive.and_hms_opt(10, 0, 0).unwrap().and_utc();

        if end <= start {
            set_form_error.set(Some("End date must be after start date".to_string()));
            return;
        }

        let cost_val = match cost.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                set_form_error.set(Some("Invalid nightly rate".to_string()));
                return;
            }
        };

        let channel_id = app_store
            .get()
            .channels_for_asset(asset_id)
            .first()
            .map(|c| c.id);

        // Conflict detection
        let overlaps = app_store
            .get()
            .overlapping_bookings(asset_id, start, end, None)
            .len();
        if overlaps > 0 {
            set_conflict_msg.set(Some(
                "Warning: this booking overlaps an existing reservation.".to_string(),
            ));
        } else {
            set_conflict_msg.set(None);
        }

        let mut booking = Booking::new(
            asset_id,
            channel_id,
            BookingSource::TestChannel,
            guest,
            start,
            end,
            cost_val,
        );
        booking.status = BookingStatus::Confirmed;

        let asset_name_clone = asset_name_signal.get_untracked();
        let channel_name = channel_id
            .and_then(|cid| app_store.with(|s| s.get_channel(cid).map(|c| c.name.clone())));

        let booking_clone = booking.clone();
        app_store.update(|s| s.add_booking(booking));
        calendar_store.update(|s| {
            s.sync_booking_event(
                &booking_clone,
                &asset_name_clone,
                channel_name.as_deref(),
                portfolio_id,
            );
        });

        set_guest_name.set(String::new());
        set_start_date.set(String::new());
        set_end_date.set(String::new());
        set_show_booking_form.set(false);
        set_form_error.set(None);
    });

    let add_cleaning_for_booking = Callback::new(move |booking_id: Uuid| {
        app_store.update(|s| {
            if let Some(task) = s.add_cleaning_task_for_booking(booking_id, 4) {
                let task_clone = task.clone();
                let asset_name_clone = asset_name_signal.get_untracked();
                calendar_store.update(|cs| {
                    cs.sync_service_task_event(&task_clone, &asset_name_clone, portfolio_id);
                });
            }
        });
    });

    let cancel_booking = Callback::new(move |booking_id: Uuid| {
        app_store.update(|s| {
            if let Some(b) = s.get_booking_mut(booking_id) {
                b.mark_cancelled("Manual");
                let b = b.clone();
                let asset_name_clone = asset_name_signal.get_untracked();
                let channel_name = b
                    .channel_id
                    .and_then(|cid| s.get_channel(cid))
                    .map(|c| c.name.clone());
                calendar_store.update(|cs| {
                    cs.sync_booking_event(
                        &b,
                        &asset_name_clone,
                        channel_name.as_deref(),
                        portfolio_id,
                    );
                });
            }
        });
    });

    let connect_channel = Callback::new(move |channel_id: Uuid| {
        app_store.update(|s| {
            if let Some(c) = s.get_channel_mut(channel_id) {
                c.connect();
            }
        });
    });

    let disconnect_channel = Callback::new(move |channel_id: Uuid| {
        app_store.update(|s| {
            if let Some(c) = s.get_channel_mut(channel_id) {
                c.disconnect();
            }
        });
    });

    let check_channel = Callback::new(move |channel_id: Uuid| {
        app_store.update(|s| {
            if let Some(c) = s.get_channel_mut(channel_id) {
                c.check_connection();
            }
        });
    });

    let sync_channel = Callback::new(move |channel_id: Uuid| {
        app_store.update(|s| {
            if let Some(c) = s.get_channel_mut(channel_id) {
                c.last_sync_at = Some(Utc::now());
                c.last_sync_status = Some("Test channel sync completed (local)".to_string());
                c.sync_errors.clear();
            }
        });
    });

    let simulate_change = Callback::new(move |booking_id: Uuid| {
        app_store.update(|s| {
            let start = Some(Utc::now() + Duration::days(2));
            let end = Some(Utc::now() + Duration::days(5));
            if let Some(b) = s.simulate_booking_change(booking_id, start, end, Some(120.0), None) {
                let b = b.clone();
                let asset_name_clone = asset_name_signal.get_untracked();
                let channel_name = b
                    .channel_id
                    .and_then(|cid| s.get_channel(cid))
                    .map(|c| c.name.clone());
                calendar_store.update(|cs| {
                    cs.sync_booking_event(
                        &b,
                        &asset_name_clone,
                        channel_name.as_deref(),
                        portfolio_id,
                    );
                });
            }
        });
    });

    view! {
        <div class="ai-channels-section">
            <div class="ai-channels-header">
                <span class="ai-channels-title">"Channels"</span>
                {move || if can_edit {
                    view! {
                        <button class="pf-small-btn" on:click=move |_| link_test_channel.run(())>"Link Test Channel"</button>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            {move || {
                let channels = channels_for_asset.get();
                if channels.is_empty() {
                    view! { <div class="ai-channels-empty">"No channels linked. Link a Test Channel to try booking flow."</div> }.into_any()
                } else {
                    view! {
                        <div class="ai-channels-list">
                            {channels.into_iter().map(|c| {
                                let status = format!("{:?}", c.connection_status);
                                let rate = c.nightly_rate_override.map(|r| format!("${:.2}", r)).unwrap_or_else(|| "—".to_string());
                                let min = c.minimum_nights.map(|n| n.to_string()).unwrap_or_else(|| "—".to_string());
                                let max = c.maximum_nights.map(|n| n.to_string()).unwrap_or_else(|| "—".to_string());
                                let comm = c.commission_percent.map(|p| format!("{:.1}%", p)).unwrap_or_else(|| "—".to_string());
                                let last = c.last_sync_status.clone().unwrap_or_else(|| "Never synced".to_string());
                                let errors = if c.sync_errors.is_empty() {
                                    "No errors".to_string()
                                } else {
                                    format!("{} errors", c.sync_errors.len())
                                };
                                let cid = c.id;
                                let is_test = c.is_test();
                                view! {
                                    <div class="ai-channel-card">
                                        <div class="ai-channel-name">
                                            {c.name.clone()}
                                            {if is_test { view! { <span class="ai-channel-badge">"TEST"</span> }.into_any() } else { ().into_any() }}
                                        </div>
                                        <div class="ai-channel-meta">
                                            <span>"Status: "{status}</span>
                                            <span>"Rate: "{rate}</span>
                                            <span>"Min/Max nights: "{min}" / "{max}</span>
                                            <span>"Commission: "{comm}</span>
                                            <span>"Last sync: "{last}</span>
                                            <span>"Errors: "{errors}</span>
                                        </div>
                                        {if can_edit { view! {
                                            <div class="ai-channel-actions">
                                                <button class="pf-small-btn" on:click=move |_| connect_channel.run(cid)>"Connect"</button>
                                                <button class="pf-small-btn" on:click=move |_| disconnect_channel.run(cid)>"Disconnect"</button>
                                                <button class="pf-small-btn" on:click=move |_| check_channel.run(cid)>"Check"</button>
                                                <button class="pf-small-btn" on:click=move |_| sync_channel.run(cid)>"Sync"</button>
                                                <button class="pf-small-btn" on:click=move |_| set_editing_channel.set(Some(cid))>"Edit"</button>
                                            </div>
                                        }.into_any() } else { ().into_any() }}
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }
            }}

            {move || if can_edit { view! {
                <div class="ai-booking-controls">
                    <button class="pf-small-btn" on:click=move |_| set_show_booking_form.set(!show_booking_form.get())>
                        {move || if show_booking_form.get() { "Cancel" } else { "Create Test Booking" }}
                    </button>
                </div>
            }.into_any() } else { ().into_any() }}

            {move || if show_booking_form.get() { view! {
                <div class="ai-booking-form">
                    {move || form_error.get().map(|m| view! { <div class="ai-form-error">{m}</div> }.into_any()).unwrap_or_else(|| ().into_any())}
                    {move || conflict_msg.get().map(|m| view! { <div class="ai-form-warning">{m}</div> }.into_any()).unwrap_or_else(|| ().into_any())}
                    <label>"Guest name"
                        <input type="text" prop:value={move || guest_name.get()} on:input=move |ev| set_guest_name.set(event_target_value(&ev)) />
                    </label>
                    <label>"Start (YYYY-MM-DD)"
                        <input type="text" prop:value={move || start_date.get()} on:input=move |ev| set_start_date.set(event_target_value(&ev)) />
                    </label>
                    <label>"End (YYYY-MM-DD)"
                        <input type="text" prop:value={move || end_date.get()} on:input=move |ev| set_end_date.set(event_target_value(&ev)) />
                    </label>
                    <label>"Nightly rate"
                        <input type="text" prop:value={move || cost_per_night.get()} on:input=move |ev| set_cost_per_night.set(event_target_value(&ev)) />
                    </label>
                    <button class="pf-small-btn" on:click=move |_| create_test_booking.run(())>"Save Test Booking"</button>
                </div>
            }.into_any() } else { ().into_any() }}

            <div class="ai-bookings-list">
                {move || {
                    let bookings = bookings_for_asset.get();
                    if bookings.is_empty() {
                        view! { <div class="ai-bookings-empty">"No bookings yet."</div> }.into_any()
                    } else {
                        view! {
                            <div>
                                {bookings.into_iter().map(|b| {
                                    let bid = b.id;
                                    let status = format!("{:?}", b.status);
                                    let source = b.channel_label();
                                    let nights = b.nights;
                                    let total = format!("${:.2}", b.total);
                                    let start = b.start_datetime.format("%d %b %Y").to_string();
                                    let end = b.end_datetime.format("%d %b %Y").to_string();
                                    view! {
                                        <div class="ai-booking-row">
                                            <div class="ai-booking-guest">{b.guest_name.clone()}</div>
                                            <div class="ai-booking-meta">
                                                <span>{start}" → "{end}</span>
                                                <span>{nights}" nights"</span>
                                                <span>"Total: "{total}</span>
                                                <span>"Source: "{source}</span>
                                                <span>"Status: "{status}</span>
                                            </div>
                                            {if can_edit { view! {
                                                <div class="ai-booking-actions">
                                                    <button class="pf-small-btn" on:click=move |_| add_cleaning_for_booking.run(bid)>"Add Cleaning"</button>
                                                    <button class="pf-small-btn" on:click=move |_| simulate_change.run(bid)>"Simulate Change"</button>
                                                    <button class="pf-small-btn" on:click=move |_| cancel_booking.run(bid)>"Cancel"</button>
                                                </div>
                                            }.into_any() } else { ().into_any() }}
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            {move || if let Some(cid) = editing_channel.get() {
                view! {
                    <ChannelManagementWindow channel_id={cid} on_close={on_close_channel_window} />
                }.into_any()
            } else { ().into_any() }}
        </div>
    }
}

/// Asset Linking Controls — bulk channel management with search, filters,
/// pagination, and selection. Designed mobile-first for dozens of channels.
/// Role-ready: respects `can_link` permission flag independently from booking controls.
#[component]
pub(crate) fn AssetLinkingControls(
    asset_id: Uuid,
    asset_name: String,
    portfolio_id: Option<Uuid>,
    can_link: bool,
) -> impl IntoView {
    let app_store = use_app_store();
    let (asset_name_signal, _) = signal(asset_name);

    let channels_for_asset = Memo::new(move |_| {
        app_store
            .get()
            .channels_for_asset(asset_id)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
    });

    let (editing_channel, set_editing_channel) = signal(Option::<Uuid>::None);
    let on_close_channel_window = Callback::new(move |_| set_editing_channel.set(None));

    let (show_add_form, set_show_add_form) = signal(false);
    let form_ref = NodeRef::<leptos::html::Div>::new();
    let (new_channel_name, set_new_channel_name) = signal(String::new());
    let (new_channel_type, set_new_channel_type) = signal("Test".to_string());
    let (new_channel_rate, set_new_channel_rate) = signal(String::new());
    let (add_error, set_add_error) = signal(Option::<String>::None);

    // Bulk / search / pagination state
    let (search, set_search) = signal(String::new());
    let (status_filter, set_status_filter) = signal("All".to_string());
    let (type_filter, set_type_filter) = signal("All".to_string());
    let (sort_by, set_sort_by) = signal("name".to_string());
    let (page, set_page) = signal(0usize);
    let per_page = 25usize;
    let (selected, set_selected) = signal(HashSet::<Uuid>::new());

    let channel_type_label = |t: &ChannelType| -> &'static str {
        match t {
            ChannelType::Test => "Test",
            ChannelType::Airbnb => "Airbnb",
            ChannelType::BookingCom => "Booking.com",
            ChannelType::Expedia => "Expedia",
            ChannelType::Vrbo => "Vrbo",
            ChannelType::LinkedIn => "LinkedIn",
            ChannelType::EBay => "eBay",
            ChannelType::Gumtree => "Gumtree",
            ChannelType::FacebookMarketplace => "Facebook Marketplace",
            ChannelType::Amazon => "Amazon",
            ChannelType::Etsy => "Etsy",
            ChannelType::Kogan => "Kogan",
            ChannelType::Other(_) => "Other",
        }
    };

    let status_icon = |s: &ConnectionStatus| -> &'static str {
        match s {
            ConnectionStatus::Connected => "🟢",
            ConnectionStatus::Disconnected => "⚪",
            ConnectionStatus::Error => "🔴",
        }
    };

    let filtered_channels = Memo::new(move |_| {
        let mut channels = channels_for_asset.get();
        let q = search.get().trim().to_lowercase();
        if !q.is_empty() {
            channels.retain(|c| c.name.to_lowercase().contains(&q));
        }
        match status_filter.get().as_str() {
            "Connected" => {
                channels.retain(|c| matches!(c.connection_status, ConnectionStatus::Connected))
            }
            "Disconnected" => {
                channels.retain(|c| matches!(c.connection_status, ConnectionStatus::Disconnected))
            }
            "Error" => channels.retain(|c| matches!(c.connection_status, ConnectionStatus::Error)),
            _ => {}
        }
        match type_filter.get().as_str() {
            "Test" => channels.retain(|c| matches!(c.channel_type, ChannelType::Test)),
            "Airbnb" => channels.retain(|c| matches!(c.channel_type, ChannelType::Airbnb)),
            "Booking.com" => channels.retain(|c| matches!(c.channel_type, ChannelType::BookingCom)),
            "Expedia" => channels.retain(|c| matches!(c.channel_type, ChannelType::Expedia)),
            "Vrbo" => channels.retain(|c| matches!(c.channel_type, ChannelType::Vrbo)),
            "LinkedIn" => channels.retain(|c| matches!(c.channel_type, ChannelType::LinkedIn)),
            "EBay" => channels.retain(|c| matches!(c.channel_type, ChannelType::EBay)),
            "Gumtree" => channels.retain(|c| matches!(c.channel_type, ChannelType::Gumtree)),
            "FacebookMarketplace" => {
                channels.retain(|c| matches!(c.channel_type, ChannelType::FacebookMarketplace))
            }
            "Amazon" => channels.retain(|c| matches!(c.channel_type, ChannelType::Amazon)),
            "Etsy" => channels.retain(|c| matches!(c.channel_type, ChannelType::Etsy)),
            "Kogan" => channels.retain(|c| matches!(c.channel_type, ChannelType::Kogan)),
            "Other" => channels.retain(|c| matches!(c.channel_type, ChannelType::Other(_))),
            _ => {}
        }
        match sort_by.get().as_str() {
            "status" => channels.sort_by(|a, b| {
                format!("{:?}", a.connection_status).cmp(&format!("{:?}", b.connection_status))
            }),
            "type" => channels.sort_by(|a, b| {
                channel_type_label(&a.channel_type).cmp(channel_type_label(&b.channel_type))
            }),
            "last_sync" => channels.sort_by(|a, b| a.last_sync_at.cmp(&b.last_sync_at).reverse()),
            _ => channels.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        }
        channels
    });

    let total_pages = Memo::new(move |_| {
        let count = filtered_channels.get().len();
        ((count + per_page - 1) / per_page).max(1)
    });

    let paginated_channels = Memo::new(move |_| {
        let all = filtered_channels.get();
        let total = total_pages.get();
        let page = page.get().min(total.saturating_sub(1));
        let start = page * per_page;
        all.into_iter()
            .skip(start)
            .take(per_page)
            .collect::<Vec<_>>()
    });

    // Keep the current page inside the available range.
    Effect::new(move |_| {
        let total = total_pages.get();
        let current = page.get();
        if current + 1 > total && total > 0 {
            set_page.set(total - 1);
        }
    });

    let add_channel = Callback::new(move |_| {
        set_add_error.set(None);
        let mut name = new_channel_name.get_untracked().trim().to_string();
        if name.is_empty() {
            name = format!("Test Channel - {}", asset_name_signal.get_untracked());
        }
        let rate = if new_channel_rate.get_untracked().trim().is_empty() {
            None
        } else {
            match new_channel_rate.get_untracked().trim().parse::<f64>() {
                Ok(v) if v >= 0.0 => Some(v),
                _ => {
                    set_add_error.set(Some("Invalid nightly rate".to_string()));
                    return;
                }
            }
        };
        let channel_type = match new_channel_type.get_untracked().as_str() {
            "Airbnb" => ChannelType::Airbnb,
            "BookingCom" => ChannelType::BookingCom,
            "Expedia" => ChannelType::Expedia,
            "Vrbo" => ChannelType::Vrbo,
            "LinkedIn" => ChannelType::LinkedIn,
            "EBay" => ChannelType::EBay,
            "Gumtree" => ChannelType::Gumtree,
            "FacebookMarketplace" => ChannelType::FacebookMarketplace,
            "Amazon" => ChannelType::Amazon,
            "Etsy" => ChannelType::Etsy,
            "Kogan" => ChannelType::Kogan,
            _ => ChannelType::Test,
        };
        let mut channel = Channel::new_test_channel(name, Some(asset_id), portfolio_id);
        channel.channel_type = channel_type;
        channel.nightly_rate_override = rate;
        let cid = channel.id;
        app_store.update(|s| {
            s.add_channel(channel);
            if let Some(pid) = portfolio_id {
                if let Some(p) = s.get_portfolio_mut(pid) {
                    if let Some(a) = p.assets.iter_mut().find(|a| a.id == asset_id) {
                        a.channel_ids.push(cid);
                    } else {
                        for g in p.asset_groups.iter_mut() {
                            if let Some(a) = g.assets.iter_mut().find(|a| a.id == asset_id) {
                                a.channel_ids.push(cid);
                                break;
                            }
                        }
                    }
                }
            } else {
                for p in s.portfolios.iter_mut() {
                    if let Some(a) = p.assets.iter_mut().find(|a| a.id == asset_id) {
                        a.channel_ids.push(cid);
                        break;
                    }
                    for g in p.asset_groups.iter_mut() {
                        if let Some(a) = g.assets.iter_mut().find(|a| a.id == asset_id) {
                            a.channel_ids.push(cid);
                            break;
                        }
                    }
                }
            }
        });
        set_new_channel_name.set(String::new());
        set_new_channel_rate.set(String::new());
        set_show_add_form.set(false);
    });

    let toggle_selected = Callback::new(move |id: Uuid| {
        set_selected.update(|s| {
            if s.contains(&id) {
                s.remove(&id);
            } else {
                s.insert(id);
            }
        });
    });

    let select_all = Callback::new(move |_| {
        let ids: HashSet<Uuid> = filtered_channels.get().iter().map(|c| c.id).collect();
        set_selected.set(ids);
    });

    let clear_selection = Callback::new(move |_| {
        set_selected.set(HashSet::new());
    });

    let bulk_connect = Callback::new(move |_| {
        let ids: Vec<_> = selected.get().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        app_store.update(|s| {
            for id in ids {
                if let Some(c) = s.get_channel_mut(id) {
                    c.connect();
                }
            }
        });
    });

    let bulk_disconnect = Callback::new(move |_| {
        let ids: Vec<_> = selected.get().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        app_store.update(|s| {
            for id in ids {
                if let Some(c) = s.get_channel_mut(id) {
                    c.disconnect();
                }
            }
        });
    });

    let bulk_sync = Callback::new(move |_| {
        let ids: Vec<_> = selected.get().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        app_store.update(|s| {
            for id in ids {
                if let Some(c) = s.get_channel_mut(id) {
                    c.last_sync_at = Some(Utc::now());
                    c.last_sync_status = Some("Test channel sync completed (local)".to_string());
                    c.sync_errors.clear();
                }
            }
        });
    });

    let bulk_delete = Callback::new(move |_| {
        let ids: Vec<_> = selected.get().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        app_store.update(|s| {
            for &id in &ids {
                s.remove_channel(id);
            }
        });
        set_selected.update(|s| {
            for id in &ids {
                s.remove(id);
            }
        });
    });

    // True when the asset already has a channel of the given marketplace type.
    let marketplace_linked = move |key: &str| -> bool {
        let Some(ty) = marketplace_channel_type(key) else {
            return false;
        };
        channels_for_asset
            .get()
            .iter()
            .any(|c| c.channel_type == ty)
    };

    // One-click marketplace crosslisting: creates a channel of the marketplace
    // type and links it to this asset, like picking destinations in FLUF.
    let link_marketplace = Callback::new(move |key: &'static str| {
        let Some(channel_type) = marketplace_channel_type(key) else {
            return;
        };
        if channels_for_asset
            .get_untracked()
            .iter()
            .any(|c| c.channel_type == channel_type)
        {
            return;
        }
        let label = marketplace_def(key).map(|d| d.label).unwrap_or(key);
        let name = format!("{} — {}", label, asset_name_signal.get_untracked());
        let mut channel = Channel::new_test_channel(name, Some(asset_id), portfolio_id);
        channel.channel_type = channel_type;
        let cid = channel.id;
        app_store.update(|s| {
            s.add_channel(channel);
            if let Some(pid) = portfolio_id {
                if let Some(p) = s.get_portfolio_mut(pid) {
                    if let Some(a) = p.assets.iter_mut().find(|a| a.id == asset_id) {
                        a.channel_ids.push(cid);
                    } else {
                        for g in p.asset_groups.iter_mut() {
                            if let Some(a) = g.assets.iter_mut().find(|a| a.id == asset_id) {
                                a.channel_ids.push(cid);
                                break;
                            }
                        }
                    }
                }
            } else {
                for p in s.portfolios.iter_mut() {
                    if let Some(a) = p.assets.iter_mut().find(|a| a.id == asset_id) {
                        a.channel_ids.push(cid);
                        break;
                    }
                    for g in p.asset_groups.iter_mut() {
                        if let Some(a) = g.assets.iter_mut().find(|a| a.id == asset_id) {
                            a.channel_ids.push(cid);
                            break;
                        }
                    }
                }
            }
        });
    });

    let channel_stats = Memo::new(move |_| {
        let now = Utc::now();
        let window_end = now + Duration::days(365);
        let channels = channels_for_asset.get();
        let bookings: Vec<_> = app_store
            .get()
            .bookings_for_asset(asset_id)
            .into_iter()
            .cloned()
            .collect();

        let price_per_day = if channels.is_empty() {
            "—".to_string()
        } else {
            let rates: Vec<_> = channels
                .iter()
                .filter_map(|c| c.nightly_rate_override)
                .collect();
            if !rates.is_empty() {
                format!("${:.2}", rates.iter().sum::<f64>() / rates.len() as f64)
            } else {
                bookings
                    .iter()
                    .filter(|b| !matches!(b.status, BookingStatus::Cancelled))
                    .map(|b| b.cost_per_night)
                    .last()
                    .map(|r| format!("${:.2}", r))
                    .unwrap_or_else(|| "—".to_string())
            }
        };

        let booked_days = bookings
            .iter()
            .filter(|b| !matches!(b.status, BookingStatus::Cancelled))
            .map(|b| {
                let start = b.start_datetime.max(now);
                let end = b.end_datetime.min(window_end);
                if end > start {
                    (end.date_naive() - start.date_naive()).num_days() as u32
                } else {
                    0
                }
            })
            .sum::<u32>();

        let unbooked_days = 365u32.saturating_sub(booked_days);
        (price_per_day, booked_days, unbooked_days, channels.len())
    });

    let close_link_form_on_outside = move |ev: leptos::ev::MouseEvent| {
        if !show_add_form.get_untracked() {
            return;
        }
        if let Some(form) = form_ref.get() {
            let outside = ev.target().map_or(true, |t| {
                t.dyn_ref::<web_sys::Node>()
                    .map_or(true, |n| !form.contains(Some(n)))
            });
            if outside {
                set_show_add_form.set(false);
            }
        }
    };

    view! {
        <div class="ai-channel-management ai-linking-controls" on:click=close_link_form_on_outside>
            <div class="ai-channel-management-header">
                <span class="ai-channel-management-title">"Linking Controls"</span>
                {move || if can_link {
                    view! {
                        <button class="pf-small-btn" on:click=move |ev| {
                            ev.stop_propagation();
                            set_show_add_form.update(|v| *v = !*v);
                            if show_add_form.get_untracked() {
                                set_new_channel_name.set(format!("Test Channel - {}", asset_name_signal.get_untracked()));
                            } else {
                                set_new_channel_name.set(String::new());
                            }
                        }>
                            {move || if show_add_form.get() { "Close" } else { "Add Channel" }}
                        </button>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            {move || {
                let (price, booked, unbooked, count) = channel_stats.get();
                view! {
                    <div class="ai-channel-stats">
                        <div class="ai-channel-stat">
                            <span class="ai-channel-stat-label">"Price / day"</span>
                            <span class="ai-channel-stat-value">{price}</span>
                        </div>
                        <div class="ai-channel-stat">
                            <span class="ai-channel-stat-label">"Booked days"</span>
                            <span class="ai-channel-stat-value">{booked.to_string()}</span>
                        </div>
                        <div class="ai-channel-stat">
                            <span class="ai-channel-stat-label">"Unbooked days"</span>
                            <span class="ai-channel-stat-value">{unbooked.to_string()}</span>
                        </div>
                        <div class="ai-channel-stat">
                            <span class="ai-channel-stat-label">"Channels"</span>
                            <span class="ai-channel-stat-value">{count.to_string()}</span>
                        </div>
                    </div>
                }.into_any()
            }}

            {move || if can_link { view! {
                <div class="ai-marketplace-bar">
                    <span class="ai-marketplace-title">"Marketplaces"</span>
                    <div class="ai-marketplace-buttons">
                        {MARKETPLACES.iter().map(|def| {
                            let key = def.key;
                            let label = def.label;
                            view! {
                                <button type="button"
                                    class="ai-marketplace-btn"
                                    class:ai-marketplace-btn-linked={move || marketplace_linked(key)}
                                    disabled={move || marketplace_linked(key)}
                                    title={move || {
                                        if marketplace_linked(key) {
                                            format!("Already listed on {}", label)
                                        } else {
                                            format!("List {} on {}", asset_name_signal.get(), label)
                                        }
                                    }}
                                    on:click=move |_| link_marketplace.run(key)>
                                    {move || {
                                        if marketplace_linked(key) {
                                            format!("✓ {}", label)
                                        } else {
                                            format!("+ {} {}", def.icon, label)
                                        }
                                    }}
                                </button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            }.into_any() } else { ().into_any() }}

            {move || if show_add_form.get() && can_link { view! {
                <div class="ai-channel-management-form" node_ref=form_ref on:click=move |ev| ev.stop_propagation()>
                    {move || add_error.get().map(|m| view! { <div class="ai-form-error">{m}</div> }.into_any()).unwrap_or_else(|| ().into_any())}
                    <label>"Name"
                        <input type="text" prop:value={move || new_channel_name.get()} on:input=move |ev| set_new_channel_name.set(event_target_value(&ev)) style="width: 120px" />
                    </label>
                    <label>"Type"
                        <select prop:value={move || new_channel_type.get()} on:change=move |ev| set_new_channel_type.set(event_target_value(&ev))>
                            <option value="Test">"Test"</option>
                            <option value="Airbnb">"Airbnb"</option>
                            <option value="BookingCom">"Booking.com"</option>
                            <option value="Expedia">"Expedia"</option>
                            <option value="Vrbo">"Vrbo"</option>
                            <option value="LinkedIn">"LinkedIn"</option>
                            <option value="EBay">"eBay"</option>
                        </select>
                    </label>
                    <label>"Rate"
                        <input type="text" prop:value={move || new_channel_rate.get()} on:input=move |ev| set_new_channel_rate.set(event_target_value(&ev)) placeholder="0" style="width: 60px" />
                    </label>
                    <button class="pf-small-btn" on:click=move |_| add_channel.run(())>"Add"</button>
                </div>
            }.into_any() } else { ().into_any() }}

            {move || {
                let channels = channels_for_asset.get();
                if channels.is_empty() {
                    view! { <div class="ai-channel-management-empty">"No channels linked."</div> }.into_any()
                } else {
                    view! {
                        <div class="ai-channel-toolbar">
                            <input class="ai-channel-search" type="text" placeholder="Search channels..."
                                prop:value={move || search.get()}
                                on:input=move |ev| {
                                    set_search.set(event_target_value(&ev));
                                    set_page.set(0);
                                }
                            />
                            <select class="ai-channel-filter" prop:value={move || status_filter.get()}
                                on:change=move |ev| {
                                    set_status_filter.set(event_target_value(&ev));
                                    set_page.set(0);
                                }>
                                <option value="All">"All statuses"</option>
                                <option value="Connected">"Connected"</option>
                                <option value="Disconnected">"Disconnected"</option>
                                <option value="Error">"Error"</option>
                            </select>
                            <select class="ai-channel-filter" prop:value={move || type_filter.get()}
                                on:change=move |ev| {
                                    set_type_filter.set(event_target_value(&ev));
                                    set_page.set(0);
                                }>
                                <option value="All">"All types"</option>
                                <option value="Test">"Test"</option>
                                <option value="Airbnb">"Airbnb"</option>
                                <option value="Booking.com">"Booking.com"</option>
                                <option value="Expedia">"Expedia"</option>
                                <option value="Vrbo">"Vrbo"</option>
                                <option value="LinkedIn">"LinkedIn"</option>
                                <option value="EBay">"eBay"</option>
                                <option value="Other">"Other"</option>
                            </select>
                            <select class="ai-channel-filter" prop:value={move || sort_by.get()}
                                on:change=move |ev| set_sort_by.set(event_target_value(&ev))>
                                <option value="name">"Name"</option>
                                <option value="status">"Status"</option>
                                <option value="type">"Type"</option>
                                <option value="last_sync">"Last sync"</option>
                            </select>
                        </div>
                    }.into_any()
                }
            }}

            {move || {
                let total = filtered_channels.get().len();
                view! {
                    <div class="ai-channel-list-meta">
                        <span>{total.to_string()}" channels found"</span>
                        {move || if can_link && total > 0 { view! {
                            <div class="ai-channel-selection-actions">
                                <button class="pf-small-btn" on:click=move |_| select_all.run(())>"Select all"</button>
                                <button class="pf-small-btn" on:click=move |_| clear_selection.run(())>"Clear"</button>
                            </div>
                        }.into_any() } else { ().into_any() }}
                    </div>
                }.into_any()
            }}

            {move || {
                let channels = paginated_channels.get();
                if channels.is_empty() {
                    view! { <div class="ai-channel-management-empty">"No channels match the current filters."</div> }.into_any()
                } else {
                    view! {
                        <div class="ai-channel-list">
                            {channels.into_iter().map(|c| {
                                let cid = c.id;
                                let label = channel_type_label(&c.channel_type).to_string();
                                let status = format!("{:?}", c.connection_status);
                                let icon = status_icon(&c.connection_status);
                                let rate = c.nightly_rate_override.map(|r| format!("${:.2}", r)).unwrap_or_else(|| "—".to_string());
                                let last = c.last_sync_status.clone().unwrap_or_else(|| "Never synced".to_string());
                                let is_selected = move || selected.get().contains(&cid);
                                view! {
                                    <div class="ai-channel-row"
                                        class:ai-channel-row-selected={is_selected}
                                        on:click=move |_| set_editing_channel.set(Some(cid))
                                    >
                                        {if can_link { view! {
                                            <input type="checkbox" class="ai-channel-checkbox"
                                                checked={is_selected}
                                                on:click=move |ev| {
                                                    ev.stop_propagation();
                                                    toggle_selected.run(cid);
                                                }
                                            />
                                        }.into_any() } else { ().into_any() }}
                                        <span class="ai-channel-status-icon">{icon}</span>
                                        <span class="ai-channel-row-name">{c.name.clone()}</span>
                                        <span class="ai-channel-row-badge ai-channel-type-badge">{label}</span>
                                        <span class="ai-channel-row-badge ai-channel-status-badge">{status}</span>
                                        <span class="ai-channel-row-rate">{rate}</span>
                                        <span class="ai-channel-row-last">{last}</span>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }
            }}

            {move || {
                let total = total_pages.get();
                let current = page.get() + 1;
                if total > 1 { view! {
                    <div class="ai-channel-pagination">
                        <button class="pf-small-btn" on:click=move |_| set_page.set(0) disabled={move || page.get() == 0}>"First"</button>
                        <button class="pf-small-btn" on:click=move |_| set_page.set(page.get().saturating_sub(1)) disabled={move || page.get() == 0}>"Prev"</button>
                        <span class="ai-channel-page-info">{current.to_string()}" / "{total.to_string()}</span>
                        <button class="pf-small-btn" on:click=move |_| set_page.set((page.get() + 1).min(total - 1)) disabled={move || page.get() + 1 >= total}>"Next"</button>
                        <button class="pf-small-btn" on:click=move |_| set_page.set(total - 1) disabled={move || page.get() + 1 >= total}>"Last"</button>
                    </div>
                }.into_any() } else { ().into_any() }
            }}

            {move || {
                let count = selected.get().len();
                if can_link && count > 0 { view! {
                    <div class="ai-channel-bulk-bar">
                        <span class="ai-channel-bulk-count">{count.to_string()}" selected"</span>
                        <div class="ai-channel-bulk-actions">
                            <button class="pf-small-btn" on:click=move |_| bulk_connect.run(())>"Connect"</button>
                            <button class="pf-small-btn" on:click=move |_| bulk_disconnect.run(())>"Disconnect"</button>
                            <button class="pf-small-btn" on:click=move |_| bulk_sync.run(())>"Sync"</button>
                            <button class="pf-small-btn" on:click=move |_| bulk_delete.run(())>"Delete"</button>
                        </div>
                        <button class="pf-small-btn" on:click=move |_| clear_selection.run(())>"Clear"</button>
                    </div>
                }.into_any() } else { ().into_any() }
            }}

            {move || if let Some(cid) = editing_channel.get() {
                view! {
                    <ChannelManagementWindow channel_id={cid} on_close={on_close_channel_window} />
                }.into_any()
            } else { ().into_any() }}
        </div>
    }
}

/// Asset Booking Controls — booking management panel (create/list/cancel bookings).
/// Role-ready: respects `can_book` permission flag independently from linking controls.
#[component]
pub(crate) fn AssetBookingControls(
    asset_id: Uuid,
    asset_name: String,
    portfolio_id: Option<Uuid>,
    can_book: bool,
) -> impl IntoView {
    let app_store = use_app_store();
    let calendar_store = use_calendar_store();
    let (asset_name_signal, _) = signal(asset_name);

    let bookings_for_asset = Memo::new(move |_| {
        app_store
            .get()
            .bookings_for_asset(asset_id)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
    });

    let (show_booking_form, set_show_booking_form) = signal(false);
    let booking_form_ref = NodeRef::<leptos::html::Div>::new();
    let (guest_name, set_guest_name) = signal(String::new());
    let (start_date, set_start_date) = signal(String::new());
    let (end_date, set_end_date) = signal(String::new());
    let (cost_per_night, set_cost_per_night) = signal(String::from("100"));
    let (conflict_msg, set_conflict_msg) = signal(Option::<String>::None);
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let create_booking = Callback::new(move |_| {
        let guest = guest_name.get();
        let start_str = start_date.get();
        let end_str = end_date.get();
        let cost = cost_per_night.get();

        if guest.trim().is_empty() || start_str.trim().is_empty() || end_str.trim().is_empty() {
            set_form_error.set(Some("All fields are required".to_string()));
            return;
        }

        let start_naive = match NaiveDate::parse_from_str(&start_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                set_form_error.set(Some("Invalid start date (YYYY-MM-DD)".to_string()));
                return;
            }
        };
        let end_naive = match NaiveDate::parse_from_str(&end_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                set_form_error.set(Some("Invalid end date (YYYY-MM-DD)".to_string()));
                return;
            }
        };
        let start = start_naive.and_hms_opt(15, 0, 0).unwrap().and_utc();
        let end = end_naive.and_hms_opt(10, 0, 0).unwrap().and_utc();

        if end <= start {
            set_form_error.set(Some("End date must be after start date".to_string()));
            return;
        }

        let cost_val = match cost.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                set_form_error.set(Some("Invalid nightly rate".to_string()));
                return;
            }
        };

        let channel_id = app_store
            .get()
            .channels_for_asset(asset_id)
            .first()
            .map(|c| c.id);

        let overlaps = app_store
            .get()
            .overlapping_bookings(asset_id, start, end, None)
            .len();
        if overlaps > 0 {
            set_conflict_msg.set(Some(
                "Warning: this booking overlaps an existing reservation.".to_string(),
            ));
        } else {
            set_conflict_msg.set(None);
        }

        let mut booking = Booking::new(
            asset_id,
            channel_id,
            BookingSource::Manual,
            guest,
            start,
            end,
            cost_val,
        );
        booking.status = BookingStatus::Confirmed;

        let booking_clone = booking.clone();
        let channel_name = channel_id
            .and_then(|cid| app_store.with(|s| s.get_channel(cid).map(|c| c.name.clone())));
        let asset_name_clone = asset_name_signal.get_untracked();
        app_store.update(|s| s.add_booking(booking));
        calendar_store.update(|s| {
            s.sync_booking_event(
                &booking_clone,
                &asset_name_clone,
                channel_name.as_deref(),
                portfolio_id,
            );
        });

        set_guest_name.set(String::new());
        set_start_date.set(String::new());
        set_end_date.set(String::new());
        set_show_booking_form.set(false);
        set_form_error.set(None);
    });

    let add_cleaning_for_booking = Callback::new(move |booking_id: Uuid| {
        app_store.update(|s| {
            if let Some(task) = s.add_cleaning_task_for_booking(booking_id, 4) {
                let task_clone = task.clone();
                let asset_name_clone = asset_name_signal.get_untracked();
                calendar_store.update(|cs| {
                    cs.sync_service_task_event(&task_clone, &asset_name_clone, portfolio_id);
                });
            }
        });
    });

    let cancel_booking = Callback::new(move |booking_id: Uuid| {
        app_store.update(|s| {
            if let Some(b) = s.get_booking_mut(booking_id) {
                b.mark_cancelled("Manual");
                let b = b.clone();
                let asset_name_clone = asset_name_signal.get_untracked();
                let channel_name = b
                    .channel_id
                    .and_then(|cid| s.get_channel(cid))
                    .map(|c| c.name.clone());
                calendar_store.update(|cs| {
                    cs.sync_booking_event(
                        &b,
                        &asset_name_clone,
                        channel_name.as_deref(),
                        portfolio_id,
                    );
                });
            }
        });
    });

    let simulate_change = Callback::new(move |booking_id: Uuid| {
        app_store.update(|s| {
            let start = Some(Utc::now() + Duration::days(2));
            let end = Some(Utc::now() + Duration::days(5));
            if let Some(b) = s.simulate_booking_change(booking_id, start, end, Some(120.0), None) {
                let b = b.clone();
                let asset_name_clone = asset_name_signal.get_untracked();
                let channel_name = b
                    .channel_id
                    .and_then(|cid| s.get_channel(cid))
                    .map(|c| c.name.clone());
                calendar_store.update(|cs| {
                    cs.sync_booking_event(
                        &b,
                        &asset_name_clone,
                        channel_name.as_deref(),
                        portfolio_id,
                    );
                });
            }
        });
    });

    let close_booking_form_on_outside = move |ev: leptos::ev::MouseEvent| {
        if !show_booking_form.get_untracked() {
            return;
        }
        if let Some(form) = booking_form_ref.get() {
            let outside = ev.target().map_or(true, |t| {
                t.dyn_ref::<web_sys::Node>()
                    .map_or(true, |n| !form.contains(Some(n)))
            });
            if outside {
                set_show_booking_form.set(false);
            }
        }
    };

    view! {
        <div class="ai-channel-management ai-booking-controls" on:click=close_booking_form_on_outside>
            <div class="ai-channel-management-header">
                <span class="ai-channel-management-title">"Booking Controls"</span>
                {move || if can_book { view! {
                    <button class="pf-small-btn" on:click=move |ev| {
                        ev.stop_propagation();
                        set_show_booking_form.update(|v| *v = !*v);
                    }>
                        {move || if show_booking_form.get() { "Close" } else { "+ Add Booking" }}
                    </button>
                }.into_any() } else { ().into_any() }}
            </div>

            {move || if show_booking_form.get() && can_book { view! {
                <div class="ai-channel-management-form" node_ref=booking_form_ref on:click=move |ev| ev.stop_propagation()>
                    {move || form_error.get().map(|m| view! { <div class="ai-form-error">{m}</div> }.into_any()).unwrap_or_else(|| ().into_any())}
                    {move || conflict_msg.get().map(|m| view! { <div class="ai-form-warning">{m}</div> }.into_any()).unwrap_or_else(|| ().into_any())}
                    <label>"Guest"
                        <input type="text" prop:value={move || guest_name.get()} on:input=move |ev| set_guest_name.set(event_target_value(&ev)) />
                    </label>
                    <label>"Start"
                        <input type="text" prop:value={move || start_date.get()} on:input=move |ev| set_start_date.set(event_target_value(&ev)) placeholder="YYYY-MM-DD" />
                    </label>
                    <label>"End"
                        <input type="text" prop:value={move || end_date.get()} on:input=move |ev| set_end_date.set(event_target_value(&ev)) placeholder="YYYY-MM-DD" />
                    </label>
                    <label>"Rate / night"
                        <input type="text" prop:value={move || cost_per_night.get()} on:input=move |ev| set_cost_per_night.set(event_target_value(&ev)) />
                    </label>
                    <button class="pf-small-btn" on:click=move |_| create_booking.run(())>"Save to Calendar"</button>
                </div>
            }.into_any() } else { ().into_any() }}

            <div class="ai-bookings-list">
                {move || {
                    let bookings = bookings_for_asset.get();
                    if bookings.is_empty() {
                        view! { <div class="ai-bookings-empty">"No bookings yet."</div> }.into_any()
                    } else {
                        view! {
                            <div>
                                {bookings.into_iter().map(|b| {
                                    let bid = b.id;
                                    let status = format!("{:?}", b.status);
                                    let source = b.channel_label();
                                    let nights = b.nights;
                                    let total = format!("${:.2}", b.total);
                                    let start = b.start_datetime.format("%d %b %Y").to_string();
                                    let end = b.end_datetime.format("%d %b %Y").to_string();
                                    view! {
                                        <div class="ai-booking-row">
                                            <div class="ai-booking-guest">{b.guest_name.clone()}</div>
                                            <div class="ai-booking-meta">
                                                <span>{start}" → "{end}</span>
                                                <span>{nights}" nights"</span>
                                                <span>"Total: "{total}</span>
                                                <span>"Source: "{source}</span>
                                                <span>"Status: "{status}</span>
                                            </div>
                                            {if can_book { view! {
                                                <div class="ai-booking-actions">
                                                    <button class="pf-small-btn" on:click=move |_| add_cleaning_for_booking.run(bid)>"Add Cleaning"</button>
                                                    <button class="pf-small-btn" on:click=move |_| simulate_change.run(bid)>"Simulate Change"</button>
                                                    <button class="pf-small-btn" on:click=move |_| cancel_booking.run(bid)>"Cancel"</button>
                                                </div>
                                            }.into_any() } else { ().into_any() }}
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

/// Add Channel Modal — creates a new channel and links it to an asset.
#[component]
pub(crate) fn AddChannelModal(
    asset_id: Uuid,
    asset_name: String,
    portfolio_id: Option<Uuid>,
    on_close: Callback<()>,
) -> impl IntoView {
    let app_store = use_app_store();
    let (name, set_name) = signal(format!("Test Channel - {}", asset_name));
    let (channel_type, set_channel_type) = signal("Test".to_string());
    let (rate, set_rate) = signal(String::new());
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let save = Callback::new(move |_| {
        set_form_error.set(None);
        let name = name.get().trim().to_string();
        if name.is_empty() {
            set_form_error.set(Some("Channel name is required".to_string()));
            return;
        }
        let rate = if rate.get().trim().is_empty() {
            None
        } else {
            match rate.get().trim().parse::<f64>() {
                Ok(v) if v >= 0.0 => Some(v),
                _ => {
                    set_form_error.set(Some("Invalid nightly rate".to_string()));
                    return;
                }
            }
        };
        let channel_type = match channel_type.get().as_str() {
            "Airbnb" => ChannelType::Airbnb,
            "BookingCom" => ChannelType::BookingCom,
            "Expedia" => ChannelType::Expedia,
            "Vrbo" => ChannelType::Vrbo,
            "LinkedIn" => ChannelType::LinkedIn,
            "EBay" => ChannelType::EBay,
            "Gumtree" => ChannelType::Gumtree,
            "FacebookMarketplace" => ChannelType::FacebookMarketplace,
            "Amazon" => ChannelType::Amazon,
            "Etsy" => ChannelType::Etsy,
            "Kogan" => ChannelType::Kogan,
            _ => ChannelType::Test,
        };
        let mut channel = Channel::new_test_channel(name, Some(asset_id), portfolio_id);
        channel.channel_type = channel_type;
        channel.nightly_rate_override = rate;
        let cid = channel.id;
        app_store.update(|s| {
            s.add_channel(channel);
            if let Some(pid) = portfolio_id {
                if let Some(p) = s.get_portfolio_mut(pid) {
                    if let Some(a) = p.assets.iter_mut().find(|a| a.id == asset_id) {
                        a.channel_ids.push(cid);
                    } else {
                        for g in p.asset_groups.iter_mut() {
                            if let Some(a) = g.assets.iter_mut().find(|a| a.id == asset_id) {
                                a.channel_ids.push(cid);
                                break;
                            }
                        }
                    }
                }
            } else {
                for p in s.portfolios.iter_mut() {
                    if let Some(a) = p.assets.iter_mut().find(|a| a.id == asset_id) {
                        a.channel_ids.push(cid);
                        break;
                    }
                    for g in p.asset_groups.iter_mut() {
                        if let Some(a) = g.assets.iter_mut().find(|a| a.id == asset_id) {
                            a.channel_ids.push(cid);
                            break;
                        }
                    }
                }
            }
        });
        on_close.run(());
    });

    view! {
        <div class="doc-modal-overlay" on:click=move |_| on_close.run(())>
            <div class="doc-modal channel-management-window" on:click=|ev| ev.stop_propagation()>
                <div class="doc-modal-header">
                    <span class="doc-modal-title">"Add Channel"</span>
                    <button class="doc-modal-close" aria-label="Close add channel" on:click=move |_| on_close.run(())>"✕"</button>
                </div>
                <div class="doc-modal-body">
                    <div class="channel-management-form">
                        {move || form_error.get().map(|m| view! { <div class="ai-form-error">{m}</div> }.into_any()).unwrap_or_else(|| ().into_any())}
                        <label>"Channel name"
                            <input type="text" prop:value={move || name.get()} on:input=move |ev| set_name.set(event_target_value(&ev)) />
                        </label>
                        <label>"Channel type"
                            <select prop:value={move || channel_type.get()} on:change=move |ev| set_channel_type.set(event_target_value(&ev))>
                                <option value="Test">"Test Channel"</option>
                                <option value="Airbnb">"Airbnb"</option>
                                <option value="BookingCom">"Booking.com"</option>
                                <option value="Expedia">"Expedia"</option>
                                <option value="Vrbo">"Vrbo"</option>
                                <option value="LinkedIn">"LinkedIn"</option>
                                <option value="EBay">"eBay"</option>
                            </select>
                        </label>
                        <label>"Nightly rate (optional)"
                            <input type="text" prop:value={move || rate.get()} on:input=move |ev| set_rate.set(event_target_value(&ev)) placeholder="e.g. 100" />
                        </label>
                        <div class="channel-management-actions">
                            <button class="login-btn" on:click=move |_| on_close.run(())>"Cancel"</button>
                            <button class="login-btn sell" on:click=move |_| save.run(())>"Save Channel"</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub(crate) fn ChannelManagementWindow(channel_id: Uuid, on_close: Callback<()>) -> impl IntoView {
    let app_store = use_app_store();
    let channel = app_store.with(|s| s.get_channel(channel_id).cloned());

    let (name, set_name) = signal(channel.as_ref().map(|c| c.name.clone()).unwrap_or_default());
    let (nightly_rate, set_nightly_rate) = signal(
        channel
            .as_ref()
            .and_then(|c| c.nightly_rate_override)
            .map(|r| format!("{:.2}", r))
            .unwrap_or_default(),
    );
    let (commission, set_commission) = signal(
        channel
            .as_ref()
            .and_then(|c| c.commission_percent)
            .map(|p| format!("{:.2}", p))
            .unwrap_or_default(),
    );
    let (min_nights, set_min_nights) = signal(
        channel
            .as_ref()
            .and_then(|c| c.minimum_nights)
            .map(|n| n.to_string())
            .unwrap_or_default(),
    );
    let (max_nights, set_max_nights) = signal(
        channel
            .as_ref()
            .and_then(|c| c.maximum_nights)
            .map(|n| n.to_string())
            .unwrap_or_default(),
    );
    let (sync_direction, set_sync_direction) = signal(
        channel
            .as_ref()
            .map(|c| c.sync_direction.clone())
            .unwrap_or_default(),
    );
    let (connection_status, set_connection_status) = signal(
        channel
            .as_ref()
            .map(|c| c.connection_status.clone())
            .unwrap_or_default(),
    );
    let (enabled, set_enabled) = signal(channel.as_ref().map(|c| c.enabled).unwrap_or(true));
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let save = Callback::new(move |_| {
        set_form_error.set(None);

        let nightly_rate = if nightly_rate.get().trim().is_empty() {
            None
        } else {
            match nightly_rate.get().trim().parse::<f64>() {
                Ok(v) if v >= 0.0 => Some(v),
                Ok(_) => {
                    set_form_error.set(Some("Nightly rate must be positive".to_string()));
                    return;
                }
                Err(_) => {
                    set_form_error.set(Some("Invalid nightly rate".to_string()));
                    return;
                }
            }
        };

        let commission = if commission.get().trim().is_empty() {
            None
        } else {
            match commission.get().trim().parse::<f64>() {
                Ok(v) if v >= 0.0 => Some(v),
                Ok(_) => {
                    set_form_error.set(Some("Commission must be positive".to_string()));
                    return;
                }
                Err(_) => {
                    set_form_error.set(Some("Invalid commission".to_string()));
                    return;
                }
            }
        };

        let min_nights = if min_nights.get().trim().is_empty() {
            None
        } else {
            match min_nights.get().trim().parse::<u32>() {
                Ok(v) => Some(v),
                Err(_) => {
                    set_form_error.set(Some("Invalid minimum nights".to_string()));
                    return;
                }
            }
        };

        let max_nights = if max_nights.get().trim().is_empty() {
            None
        } else {
            match max_nights.get().trim().parse::<u32>() {
                Ok(v) => Some(v),
                Err(_) => {
                    set_form_error.set(Some("Invalid maximum nights".to_string()));
                    return;
                }
            }
        };

        if let (Some(min), Some(max)) = (min_nights, max_nights) {
            if min > max {
                set_form_error.set(Some(
                    "Minimum nights cannot exceed maximum nights".to_string(),
                ));
                return;
            }
        }

        app_store.update(|s| {
            if let Some(c) = s.get_channel_mut(channel_id) {
                c.name = name.get();
                c.nightly_rate_override = nightly_rate;
                c.commission_percent = commission;
                c.minimum_nights = min_nights;
                c.maximum_nights = max_nights;
                c.sync_direction = sync_direction.get();
                c.connection_status = connection_status.get();
                c.enabled = enabled.get();
                c.updated_at = Utc::now();
            }
        });

        on_close.run(());
    });

    view! {
        <div class="doc-modal-overlay" on:click=move |_| on_close.run(())>
            <div class="doc-modal channel-management-window" on:click=|ev| ev.stop_propagation()>
                <div class="doc-modal-header">
                    <span class="doc-modal-title">"Channel Management"</span>
                    <button class="doc-modal-close" aria-label="Close channel management" on:click=move |_| on_close.run(())>"✕"</button>
                </div>
                <div class="doc-modal-body">
                    <div class="channel-management-form">
                        {move || form_error.get().map(|m| view! { <div class="ai-form-error">{m}</div> }.into_any()).unwrap_or_else(|| ().into_any())}
                        <label>"Channel name"
                            <input type="text" prop:value={move || name.get()} on:input=move |ev| set_name.set(event_target_value(&ev)) />
                        </label>
                        <label>"Nightly rate (rent)"
                            <input type="text" prop:value={move || nightly_rate.get()} on:input=move |ev| set_nightly_rate.set(event_target_value(&ev)) placeholder="e.g. 100" />
                        </label>
                        <label>"Commission / promotion (%)"
                            <input type="text" prop:value={move || commission.get()} on:input=move |ev| set_commission.set(event_target_value(&ev)) placeholder="e.g. 12.5" />
                        </label>
                        <div class="channel-management-row">
                            <label>"Min nights"
                                <input type="text" prop:value={move || min_nights.get()} on:input=move |ev| set_min_nights.set(event_target_value(&ev)) />
                            </label>
                            <label>"Max nights"
                                <input type="text" prop:value={move || max_nights.get()} on:input=move |ev| set_max_nights.set(event_target_value(&ev)) />
                            </label>
                        </div>
                        <label>"Sync direction"
                            <select prop:value={move || format!("{:?}", sync_direction.get())} on:change=move |ev| {
                                let v = event_target_value(&ev);
                                set_sync_direction.set(match v.as_str() {
                                    "ImportOnly" => SyncDirection::ImportOnly,
                                    "ExportOnly" => SyncDirection::ExportOnly,
                                    _ => SyncDirection::TwoWay,
                                });
                            }>
                                <option value="ImportOnly">"Import only"</option>
                                <option value="ExportOnly">"Export only"</option>
                                <option value="TwoWay">"Two-way"</option>
                            </select>
                        </label>
                        <label>"Connection status"
                            <select prop:value={move || format!("{:?}", connection_status.get())} on:change=move |ev| {
                                let v = event_target_value(&ev);
                                set_connection_status.set(match v.as_str() {
                                    "Connected" => ConnectionStatus::Connected,
                                    "Error" => ConnectionStatus::Error,
                                    _ => ConnectionStatus::Disconnected,
                                });
                            }>
                                <option value="Disconnected">"Disconnected"</option>
                                <option value="Connected">"Connected"</option>
                                <option value="Error">"Error"</option>
                            </select>
                        </label>
                        <label class="channel-management-checkbox">
                            <input type="checkbox" checked=move || enabled.get() on:change=move |ev| set_enabled.set(event_target_checked(&ev)) />
                            "Enabled"
                        </label>
                        <div class="channel-management-actions">
                            <button class="login-btn" on:click=move |_| on_close.run(())>"Cancel"</button>
                            <button class="login-btn sell" on:click=move |_| save.run(())>"Save"</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Marketplace controls — account connection + auto-filled listing setup
// (FLUF-style). Connections are stored locally per asset session; real
// marketplace API integration is future work.
// ---------------------------------------------------------------------------

fn find_asset_in_store(s: &AppStore, asset_id: Uuid) -> Option<crate::models::Asset> {
    for p in &s.portfolios {
        if let Some(a) = p.assets.iter().find(|a| a.id == asset_id) {
            return Some(a.clone());
        }
        for g in &p.asset_groups {
            if let Some(a) = g.assets.iter().find(|a| a.id == asset_id) {
                return Some(a.clone());
            }
        }
    }
    None
}

/// Lightweight (id, name, file_type) list of an asset's documents for the
/// listing picker — avoids cloning `Document.content` payloads.
fn find_asset_doc_options(s: &AppStore, asset_id: Uuid) -> Vec<(Uuid, String, String)> {
    for p in &s.portfolios {
        let docs = p
            .assets
            .iter()
            .find(|a| a.id == asset_id)
            .map(|a| &a.documents)
            .or_else(|| {
                p.asset_groups.iter().find_map(|g| {
                    g.assets
                        .iter()
                        .find(|a| a.id == asset_id)
                        .map(|a| &a.documents)
                })
            });
        if let Some(docs) = docs {
            return docs
                .iter()
                .map(|d| (d.id, d.name.clone(), d.file_type.clone()))
                .collect();
        }
    }
    Vec::new()
}

fn attach_channel_to_asset(
    s: &mut AppStore,
    asset_id: Uuid,
    portfolio_id: Option<Uuid>,
    channel: Channel,
) {
    let cid = channel.id;
    s.add_channel(channel);
    if let Some(pid) = portfolio_id {
        if let Some(p) = s.get_portfolio_mut(pid) {
            if let Some(a) = p.assets.iter_mut().find(|a| a.id == asset_id) {
                a.channel_ids.push(cid);
            } else {
                for g in p.asset_groups.iter_mut() {
                    if let Some(a) = g.assets.iter_mut().find(|a| a.id == asset_id) {
                        a.channel_ids.push(cid);
                        break;
                    }
                }
            }
        }
    } else {
        for p in s.portfolios.iter_mut() {
            if let Some(a) = p.assets.iter_mut().find(|a| a.id == asset_id) {
                a.channel_ids.push(cid);
                break;
            }
            for g in p.asset_groups.iter_mut() {
                if let Some(a) = g.assets.iter_mut().find(|a| a.id == asset_id) {
                    a.channel_ids.push(cid);
                    break;
                }
            }
        }
    }
}

/// Shared "Sales" data point stored once on the asset
/// (`metadata["sales"]`) — the overlapping listing criteria sent to every
/// marketplace. Per-store details stay in `marketplace_listings[key]`.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SalesProfile {
    #[serde(default)]
    pub price: String,
    #[serde(default)]
    pub condition: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Comma-separated list editing shared by tags / photo URLs / doc names.
pub(crate) fn csv_to_list(buf: &str) -> Vec<String> {
    buf.split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

fn load_sales_profile(s: &AppStore, asset_id: Uuid) -> SalesProfile {
    find_asset_metadata(s, asset_id)
        .and_then(|m| m.get("sales").cloned())
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// One comment on a marketplace listing — stored inside the listing's
/// own metadata so it persists with the draft/published record.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ListingComment {
    pub author: String,
    pub text: String,
    pub at: String,
}

/// One bid placed on a marketplace listing. Only the newest bid stays
/// `active`; earlier ones become `outbid`. `won`/`lost` are reserved for
/// auction-close resolution.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ListingBid {
    pub bidder: String,
    #[serde(default)]
    pub bidder_id: Option<Uuid>,
    pub amount: f64,
    pub at: String,
    /// "active" | "outbid" | "won" | "lost"
    #[serde(default = "default_bid_state")]
    pub state: String,
}

fn default_bid_state() -> String {
    "active".to_string()
}

/// Listing contents bound to an asset + marketplace. Persisted inside
/// `asset.metadata["marketplace_listings"][key]` so drafts survive reloads
/// and published listings can be updated later.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MarketplaceListing {
    /// eBay sites (marketplace ids) this listing publishes to.
    #[serde(default)]
    pub(crate) sites: Vec<String>,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) price: String,
    pub(crate) condition: String,
    pub(crate) location: String,
    /// The asset's actual photo URLs — carried into the listing so the same
    /// photos are used on the marketplace.
    #[serde(default)]
    pub(crate) photos: Vec<String>,
    /// The asset's exact condition label (New / Excellent / …) — an
    /// additional read-only criteria next to the marketplace `condition`.
    #[serde(default)]
    pub(crate) asset_condition: String,
    /// Asset status badges formatted exactly like the asset row meta
    /// ("Lifecycle: X, Availability: Y, Condition: Z, Commercial: W").
    #[serde(default)]
    pub(crate) asset_status: String,
    /// eBay auction/bidding fields — only surfaced on the eBay card.
    #[serde(default)]
    pub(crate) auction_start: String,
    #[serde(default)]
    pub(crate) buy_it_now: String,
    #[serde(default)]
    pub(crate) bid_increment: String,
    #[serde(default)]
    pub(crate) auction_days: String,
    /// Messenger thread id for this listing's conversation — created on
    /// publish so buyer/seller messages thread with the listing embedded.
    #[serde(default)]
    pub(crate) thread_id: Option<Uuid>,
    /// Context pulled from the asset — each field is an editable
    /// per-listing override once the form is open.
    pub(crate) category: String,
    pub(crate) sku: String,
    #[serde(default)]
    pub(crate) docs: Vec<String>,
    pub(crate) photo_count: usize,
    pub(crate) doc_count: usize,
    pub(crate) tags: Vec<String>,
    /// "draft" | "published" | "bidding" | "sold" | …
    pub(crate) status: String,
    pub(crate) saved_at: String,
    /// Buyer/seller comments shown on the marketplace item view.
    #[serde(default)]
    pub(crate) comments: Vec<ListingComment>,
    /// Bids placed on this listing — the displayed status derives from
    /// these (`effective_status`), not the stored `status` string alone.
    #[serde(default)]
    pub(crate) bids: Vec<ListingBid>,
    /// Who bought the listing via buy-now, and when.
    #[serde(default)]
    pub(crate) sold_to: String,
    #[serde(default)]
    pub(crate) sold_at: String,
    /// Price the listing sold for (buy-now amount or winning bid).
    #[serde(default)]
    pub(crate) sale_price: f64,
}

impl MarketplaceListing {
    /// Auto-fill every feasible field from the asset's contents and the
    /// shared Sales profile (single data point sent to every store).
    fn from_asset(a: &Asset, sales: &SalesProfile) -> Self {
        // Shared sales price wins; otherwise fall back to asset values.
        let price = if !sales.price.trim().is_empty() {
            sales.price.trim().to_string()
        } else if a.current_value > 0.0 {
            format!("{:.2}", a.current_value)
        } else if a.purchase_value > 0.0 {
            format!("{:.2}", a.purchase_value)
        } else {
            String::new()
        };
        let category = match a.asset_subtype.as_deref().map(str::trim) {
            Some(st) if !st.is_empty() => {
                format!("{} — {}", a.asset_type.to_input_string(), st)
            }
            _ => a.asset_type.to_input_string(),
        };
        let sku = a
            .reference_code
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| {
                let c = a.code.trim();
                (!c.is_empty()).then(|| c.to_string())
            })
            .unwrap_or_default();
        let photos: Vec<String> = a.images.iter().take(12).cloned().collect();
        let asset_condition = a.condition_status.as_str().to_string();
        // Same status line the asset row shows: "Label: value, …"
        let asset_status = a
            .status_badges()
            .iter()
            .map(|(label, value)| format!("{}: {}", label, value))
            .collect::<Vec<_>>()
            .join(", ");
        // When the asset has no description, compose one in the exact same
        // label format the asset detail uses so listings stay interpretable.
        let description = {
            let d = a.description.clone().unwrap_or_default();
            if d.trim().is_empty() {
                let mut lines = vec![format!("Type: {}", category)];
                if let Some(loc) = a.location.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                    lines.push(format!("Address: {}", loc));
                }
                if a.current_value > 0.0 || a.purchase_value > 0.0 {
                    let v = if a.current_value > 0.0 { a.current_value } else { a.purchase_value };
                    lines.push(format!("Value: ${:.2}", v));
                }
                lines.push(format!("Condition: {}", asset_condition));
                if !asset_status.is_empty() {
                    lines.push(format!("Status: {}", asset_status));
                }
                lines.join("\n")
            } else {
                d
            }
        };
        let condition = if !sales.condition.trim().is_empty() {
            sales.condition.trim().to_string()
        } else {
            marketplace_condition(&a.condition_status).to_string()
        };
        let tags = if !sales.tags.is_empty() {
            sales.tags.clone()
        } else {
            a.tags.clone()
        };
        Self {
            title: a.name.clone(),
            description,
            price: price.clone(),
            condition,
            location: a.location.clone().unwrap_or_default(),
            photos,
            asset_condition,
            asset_status,
            // eBay bidding defaults: start at the shared/asset price, 7 days.
            auction_start: price.clone(),
            buy_it_now: String::new(),
            bid_increment: String::new(),
            auction_days: "7".to_string(),
            sites: Vec::new(),
            category,
            sku,
            docs: a.documents.iter().map(|d| d.name.clone()).collect(),
            photo_count: a.images.len(),
            doc_count: a.documents.len(),
            tags,
            status: "draft".to_string(),
            saved_at: String::new(),
            thread_id: None,
            comments: Vec::new(),
            bids: Vec::new(),
            sold_to: String::new(),
            sold_at: String::new(),
            sale_price: 0.0,
        }
    }

    /// Price actually sent to the store: per-listing override if set,
    /// otherwise the shared sales price.
    fn effective_price<'a>(&'a self, sales: &'a SalesProfile) -> &'a str {
        if !self.price.trim().is_empty() {
            self.price.trim()
        } else {
            sales.price.trim()
        }
    }

    /// The status actually shown/enforced: drafts and sales are explicit,
    /// but "bidding" is derived — any active bid flips a live listing into
    /// bidding automatically (eBay auctions and Red Marketplace alike).
    pub(crate) fn effective_status(&self) -> &str {
        match self.status.as_str() {
            "draft" => "draft",
            "sold" => "sold",
            _ => {
                if self.bids.iter().any(|b| b.state == "active") {
                    "bidding"
                } else {
                    "published"
                }
            }
        }
    }

    /// Highest-value bid still in play, if any.
    pub(crate) fn high_bid(&self) -> Option<&ListingBid> {
        self.bids
            .iter()
            .filter(|b| b.state == "active")
            .max_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Minimum amount a new bid must beat: the current high bid, or 0
    /// when no bids exist yet.
    pub(crate) fn min_next_bid(&self) -> f64 {
        self.high_bid().map(|b| b.amount).unwrap_or(0.0)
    }

    /// Record a bid: every previous active bid becomes `outbid`.
    /// Returns false when the amount doesn't beat the current high bid.
    pub(crate) fn place_bid(
        &mut self,
        bidder: String,
        bidder_id: Option<Uuid>,
        amount: f64,
    ) -> bool {
        if amount <= self.min_next_bid() || amount <= 0.0 {
            return false;
        }
        for b in self.bids.iter_mut() {
            if b.state == "active" {
                b.state = "outbid".to_string();
            }
        }
        self.bids.push(ListingBid {
            bidder,
            bidder_id,
            amount,
            at: chrono::Utc::now().to_rfc3339(),
            state: "active".to_string(),
        });
        true
    }

    /// Buy the listing outright — marks it sold and records the buyer.
    pub(crate) fn buy_now(&mut self, buyer: String, buyer_id: Option<Uuid>, price: f64) {
        self.status = "sold".to_string();
        self.sold_to = buyer;
        self.sold_at = chrono::Utc::now().to_rfc3339();
        self.sale_price = price;
        let _ = buyer_id;
    }
}

fn find_asset_metadata(s: &AppStore, asset_id: Uuid) -> Option<serde_json::Value> {
    for p in &s.portfolios {
        if let Some(a) = p.assets.iter().find(|a| a.id == asset_id) {
            return Some(a.metadata.clone());
        }
        for g in &p.asset_groups {
            if let Some(a) = g.assets.iter().find(|a| a.id == asset_id) {
                return Some(a.metadata.clone());
            }
        }
    }
    None
}

fn find_asset_mut(s: &mut AppStore, asset_id: Uuid) -> Option<&mut Asset> {
    for p in s.portfolios.iter_mut() {
        if let Some(a) = p.assets.iter_mut().find(|a| a.id == asset_id) {
            return Some(a);
        }
        for g in p.asset_groups.iter_mut() {
            if let Some(a) = g.assets.iter_mut().find(|a| a.id == asset_id) {
                return Some(a);
            }
        }
    }
    None
}

fn load_marketplace_listing(
    s: &AppStore,
    asset_id: Uuid,
    mp_key: &str,
) -> Option<MarketplaceListing> {
    let meta = find_asset_metadata(s, asset_id)?;
    // The internal Red Marketplace stores its listing at the shared
    // `metadata["red_listing"]` slot — the same location asset groups and
    // portfolios use, so the Marketplace tab reads one shape everywhere.
    let v = if mp_key == "Red" {
        meta.get("red_listing")?.clone()
    } else {
        meta.get("marketplace_listings")?.get(mp_key)?.clone()
    };
    serde_json::from_value(v).ok()
}

fn save_marketplace_listing(
    s: &mut AppStore,
    asset_id: Uuid,
    mp_key: &str,
    listing: &MarketplaceListing,
) {
    let Some(a) = find_asset_mut(s, asset_id) else {
        return;
    };
    if !a.metadata.is_object() {
        a.metadata = serde_json::json!({});
    }
    if mp_key == "Red" {
        if let Ok(v) = serde_json::to_value(listing) {
            a.metadata
                .as_object_mut()
                .unwrap()
                .insert("red_listing".to_string(), v);
        }
        return;
    }
    let root = a.metadata.as_object_mut().unwrap();
    let listings = root
        .entry("marketplace_listings".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if !listings.is_object() {
        *listings = serde_json::json!({});
    }
    if let Ok(v) = serde_json::to_value(listing) {
        listings
            .as_object_mut()
            .unwrap()
            .insert(mp_key.to_string(), v);
    }
}

/// Persist the portfolio containing an asset so metadata edits
/// (marketplace listings, sales profiles, …) reach the server store.
/// `app_store.update` only mutates in-memory state — without this the
/// listing would vanish on the next reload.
fn persist_asset_portfolio(app_store: RwSignal<AppStore>, asset_id: Uuid) {
    let p = app_store
        .get()
        .portfolios
        .iter()
        .find(|p| {
            p.assets.iter().any(|a| a.id == asset_id)
                || p.asset_groups
                    .iter()
                    .any(|g| g.assets.iter().any(|a| a.id == asset_id))
        })
        .cloned();
    if let Some(p) = p {
        leptos::task::spawn_local(async move {
            let _ = crate::server::save_portfolio(p).await;
        });
    }
}

#[component]
fn MarketplaceCard(
    mp_key: &'static str,
    mp_label: &'static str,
    asset_id: Uuid,
    asset_name: String,
    portfolio_id: Option<Uuid>,
    can_link: bool,
) -> impl IntoView {
    let app_store = use_app_store();
    let messenger_store = use_messenger_store();
    let (asset_name_sig, _) = signal(asset_name);
    let def = marketplace_def(mp_key);
    let mp_icon = def.map(|d| d.icon).unwrap_or("🏪");
    let mp_hint = def.map(|d| d.hint).unwrap_or("");

    // Connection state comes from the account-level channel managed in
    // the Marketplace tab (Channels view) — cards no longer connect.
    let conn = Memo::new(move |_| -> Option<String> {
        if mp_key == "Red" {
            return Some("Red Marketplace".to_string());
        }
        let ty = marketplace_channel_type(mp_key)?;
        app_store.with(|s| {
            s.channels
                .iter()
                .find(|c| {
                    c.is_account_level()
                        && c.channel_type == ty
                        && c.enabled
                        && c.connection_status == ConnectionStatus::Connected
                })
                .map(|c| c.account_label.clone().unwrap_or_else(|| c.name.clone()))
        })
    });
    // eBay sites enabled in the Marketplace tab — listings publish to these.
    let enabled_sites = Memo::new(move |_| {
        app_store.with(|s| {
            s.channels
                .iter()
                .filter(|c| {
                    c.is_account_level() && c.channel_type == ChannelType::EBay && c.enabled
                })
                .filter_map(|c| c.marketplace_site.clone())
                .collect::<Vec<String>>()
        })
    });
    let open_channels_tab = move |_| {
        app_store.update(|s| {
            s.pending_marketplace_channels = true;
            s.expand_tab(crate::types::TabType::Marketplace);
        });
    };

    // Listing setup state — single struct mirroring the persisted draft.
    let (listing_open, set_listing_open) = signal(false);
    let (form, set_form) = signal(MarketplaceListing::default());
    let (published, set_published) = signal(false);
    let (form_err, set_form_err) = signal(Option::<String>::None);

    // Inline editing: which asset-format row is being edited + its buffer.
    // Fields: "title" | "price" | "condition" | "location" | "description".
    let (edit_field, set_edit_field) = signal(Option::<&'static str>::None);
    let (edit_buf, set_edit_buf) = signal(String::new());

    // Commit the edit buffer into the form field and close the editor.
    let commit_edit = move || {
        if let Some(field) = edit_field.get_untracked() {
            let buf = edit_buf.get_untracked();
            set_form.update(|fm| match field {
                "title" => fm.title = buf.clone(),
                "price" => fm.price = buf.clone(),
                "condition" => fm.condition = buf.clone(),
                "location" => fm.location = buf.clone(),
                "description" => fm.description = buf.clone(),
                "tags" => fm.tags = csv_to_list(&buf),
                "photos" => fm.photos = csv_to_list(&buf),
                "docs" => fm.docs = csv_to_list(&buf),
                "category" => fm.category = buf.clone(),
                "sku" => fm.sku = buf.clone(),
                "asset_condition" => fm.asset_condition = buf.clone(),
                "asset_status" => fm.asset_status = buf.clone(),
                "auction_start" => fm.auction_start = buf.clone(),
                "buy_it_now" => fm.buy_it_now = buf.clone(),
                "bid_increment" => fm.bid_increment = buf.clone(),
                "auction_days" => fm.auction_days = buf.clone(),
                _ => {}
            });
        }
        set_edit_field.set(None);
    };
    let cancel_edit = move || set_edit_field.set(None);
    let start_edit = move |field: &'static str| {
        if edit_field.get_untracked() == Some(field) {
            return;
        }
        // Switching rows commits whatever was being edited first.
        commit_edit();
        let current = match field {
            "title" => form.get_untracked().title,
            "price" => form.get_untracked().price,
            "condition" => form.get_untracked().condition,
            "location" => form.get_untracked().location,
            "description" => form.get_untracked().description,
            "tags" => form.get_untracked().tags.join(", "),
            "photos" => form.get_untracked().photos.join(", "),
            "docs" => form.get_untracked().docs.join(", "),
            "category" => form.get_untracked().category,
            "sku" => form.get_untracked().sku,
            "asset_condition" => form.get_untracked().asset_condition,
            "asset_status" => form.get_untracked().asset_status,
            "auction_start" => form.get_untracked().auction_start,
            "buy_it_now" => form.get_untracked().buy_it_now,
            "bid_increment" => form.get_untracked().bid_increment,
            "auction_days" => form.get_untracked().auction_days,
            _ => String::new(),
        };
        set_edit_buf.set(current);
        set_edit_field.set(Some(field));
    };

    // One asset-format row whose value becomes an inline input on
    // double-click (Enter saves, Escape cancels, outside tap saves).
    let af_text_row = move |label: &'static str,
                            field: &'static str,
                            value: String|
     -> AnyView {
        view! {
            <div class="mp-af-row">
                <span class="mp-af-label">{label}</span>
                {move || {
                    if edit_field.get() == Some(field) {
                        view! {
                            <input class="mp-af-input" type="text"
                                prop:value=move || edit_buf.get()
                                on:input=move |ev| set_edit_buf.set(event_target_value(&ev))
                                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                    match ev.key().as_str() {
                                        "Enter" => commit_edit(),
                                        "Escape" => cancel_edit(),
                                        _ => {}
                                    }
                                } />
                        }.into_any()
                    } else {
                        let v = if value.is_empty() { "—".to_string() } else { value.clone() };
                        view! {
                            <span class="mp-af-value mp-af-editable" on:dblclick=move |_| start_edit(field)>{v}</span>
                        }.into_any()
                    }
                }}
            </div>
        }
        .into_any()
    };

    // Outside taps save+close the active row editor as a phantom click.
    let mp_edit_token: &'static str =
        Box::leak(format!("mp-edit-{}-{}", asset_id, mp_key).into_boxed_str());
    {
        let alive = phantom_edit_scope(
            mp_edit_token.to_string(),
            move || edit_field.get_untracked().is_some(),
            move || commit_edit(),
        );
        on_cleanup(move || alive.store(false, Ordering::Relaxed));
    }

    // Focus the inline editor input as soon as a row enters edit mode.
    Effect::new(move |_| {
        if edit_field.get().is_some() {
            #[cfg(target_arch = "wasm32")]
            if let Some(el) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.query_selector(".mp-af-input").ok().flatten())
            {
                if let Ok(h) = el.dyn_into::<web_sys::HtmlElement>() {
                    let _ = h.focus();
                }
            }
        }
    });

    // The asset's own documents — the listing picker chooses which of these
    // are uploaded with the listing (stored as names in `form.docs`).
    let asset_docs = Memo::new(move |_| app_store.with(|s| find_asset_doc_options(s, asset_id)));

    let toggle_doc = move |name: String| {
        let all: Vec<String> = asset_docs
            .get_untracked()
            .iter()
            .map(|d| d.1.clone())
            .collect();
        set_form.update(|fm| {
            // Legacy listings stored only a count — materialize the full
            // asset doc list first so one uncheck doesn't drop all of them.
            if fm.docs.is_empty() && fm.doc_count > 0 && !all.is_empty() {
                fm.docs = all;
            }
            if let Some(i) = fm.docs.iter().position(|d| *d == name) {
                fm.docs.remove(i);
            } else {
                fm.docs.push(name);
            }
            fm.doc_count = fm.docs.len();
        });
    };

    // Last saved draft/published listing for this marketplace (reactive).
    let listing_saved = Memo::new(move |_| {
        app_store.with(|s| load_marketplace_listing(s, asset_id, mp_key))
    });

    // Bid entry on the card — records a bid on the saved listing so its
    // status flips to "bidding" automatically (eBay auctions + Red).
    let (bid_bidder, set_bid_bidder) = signal(String::new());
    let (bid_amount_val, set_bid_amount_val) = signal(String::new());
    let (bid_msg, set_bid_msg) = signal(Option::<String>::None);
    let record_bid = Callback::new(move |_: ()| {
        let bidder = bid_bidder.get().trim().to_string();
        if bidder.is_empty() {
            set_bid_msg.set(Some("Bidder name required".to_string()));
            return;
        }
        let amount: f64 = bid_amount_val.get().trim().parse().unwrap_or(0.0);
        let Some(mut l) = listing_saved.get_untracked() else {
            return;
        };
        if !l.place_bid(bidder, None, amount) {
            set_bid_msg.set(Some(format!(
                "Bid must be more than ${:.2}",
                l.min_next_bid()
            )));
            return;
        }
        l.saved_at = Utc::now().to_rfc3339();
        app_store.update(|s| save_marketplace_listing(s, asset_id, mp_key, &l));
        persist_asset_portfolio(app_store, asset_id);
        set_bid_bidder.set(String::new());
        set_bid_amount_val.set(String::new());
        set_bid_msg.set(Some("Bid recorded".to_string()));
    });

    let already_linked = move || {
        marketplace_channel_type(mp_key)
            .map(|ty| {
                app_store.with(|s| {
                    s.channels_for_asset(asset_id)
                        .iter()
                        .any(|c| c.channel_type == ty)
                })
            })
            .unwrap_or(false)
    };

    // Open the listing form: resume the saved draft if one exists, otherwise
    // auto-fill every feasible field from the asset's current contents.
    let open_listing = move |_| {
        set_form_err.set(None);
        let mut listing = load_marketplace_listing(&app_store.get_untracked(), asset_id, mp_key)
            .or_else(|| {
                let s = app_store.get_untracked();
                let sales = load_sales_profile(&s, asset_id);
                find_asset_in_store(&s, asset_id).map(|a| MarketplaceListing::from_asset(&a, &sales))
            })
            .unwrap_or_else(|| MarketplaceListing {
                title: asset_name_sig.get_untracked(),
                ..Default::default()
            });
        if mp_key == "EBay" && listing.sites.is_empty() {
            listing.sites = enabled_sites.get_untracked();
        }
        set_form.set(listing);
        set_listing_open.set(true);
    };

    let save_listing = Callback::new(move |status: &'static str| {
        commit_edit();
        let mut l = form.get();
        if l.title.trim().is_empty() {
            set_form_err.set(Some("Title is required".to_string()));
            return;
        }
        l.status = status.to_string();
        l.saved_at = Utc::now().to_rfc3339();
        // Published listings get a stable messenger thread id up front so
        // buyer/seller messages always land in the same conversation.
        if status == "published" && l.thread_id.is_none() {
            l.thread_id = Some(Uuid::new_v4());
        }
        if mp_key == "EBay" {
            if l.sites.is_empty() {
                l.sites = enabled_sites.get_untracked();
            }
            if status == "published" && l.sites.is_empty() {
                set_form_err.set(Some(
                    "Enable at least one eBay site in Marketplace → Channels before publishing".to_string(),
                ));
                return;
            }
        }
        let targets: Vec<Option<String>> = if mp_key == "EBay" {
            l.sites.iter().cloned().map(Some).collect()
        } else {
            vec![None]
        };
        let asset_name = asset_name_sig.get_untracked();
        app_store.update(|s| {
            save_marketplace_listing(s, asset_id, mp_key, &l);
            if status != "published" {
                return;
            }
            let Some(channel_type) = marketplace_channel_type(mp_key) else { return };
            // Bind the listing contents to the marketplace channel so the
            // connection carries the listing's key details — including eBay
            // bidding parameters when present.
            let sales = load_sales_profile(s, asset_id);
            let eff_price = l.effective_price(&sales);
            let mut payload = format!(
                "Listing: {} · ${} · {} · {} photo{}",
                l.title,
                if eff_price.is_empty() { "0.00".to_string() } else { eff_price.to_string() },
                l.condition,
                l.photos.len(),
                if l.photos.len() == 1 { "" } else { "s" }
            );
            if mp_key == "EBay"
                && (!l.auction_start.trim().is_empty() || !l.buy_it_now.trim().is_empty())
            {
                payload.push_str(&format!(
                    " · Auction: start ${}, BIN ${}, +${}, {}d",
                    l.auction_start, l.buy_it_now, l.bid_increment, l.auction_days
                ));
            }
            for site in targets.iter() {
                let linked = s.channels_for_asset(asset_id).iter().any(|c| {
                    c.channel_type == channel_type && (site.is_none() || c.marketplace_site == *site)
                });
                if linked {
                    continue;
                }
                let site_def = site.as_deref().and_then(ebay_site);
                let label = site_def
                    .map(|d| format!("{} {}", mp_label, d.label))
                    .unwrap_or_else(|| mp_label.to_string());
                let mut channel = Channel::new_test_channel(
                    format!("{} — {}", label, asset_name),
                    Some(asset_id),
                    portfolio_id,
                );
                channel.channel_type = channel_type.clone();
                channel.marketplace_site = site.clone();
                if let Some(d) = site_def {
                    channel.currency = d.currency();
                }
                channel.last_sync_status = Some(payload.clone());
                attach_channel_to_asset(s, asset_id, portfolio_id, channel);
            }
        });
        // Push the updated listing metadata (and any new channel link) to
        // the server store so published listings survive reloads.
        persist_asset_portfolio(app_store, asset_id);
        if status == "published" {
            set_published.set(true);
            // Open the listing's messenger thread with the listing embedded
            // as the conversation's reference message.
            let uid = app_store.get_untracked().current_user.id;
            ensure_marketplace_thread(messenger_store, uid, asset_id, mp_key, &l);
        }
        set_listing_open.set(false);
    });

    let publish = {
        let sl = save_listing.clone();
        move |_| sl.run("published")
    };
    let save_draft = move |_| save_listing.run("draft");

    // Jump from the card into the listing's messenger thread. Lazily creates
    // the thread for listings published before threads existed.
    let open_listing_messages = move |_| {
        if let Some(mut l) = listing_saved.get_untracked() {
            if l.thread_id.is_none() {
                l.thread_id = Some(Uuid::new_v4());
                let saved = l.clone();
                app_store.update(|s| save_marketplace_listing(s, asset_id, mp_key, &saved));
                persist_asset_portfolio(app_store, asset_id);
            }
            let uid = app_store.get_untracked().current_user.id;
            let tid = ensure_marketplace_thread(messenger_store, uid, asset_id, mp_key, &l);
            messenger_store.update(|ms| {
                ms.set_message_drawer(true);
                ms.set_selected_chat(Some(tid));
            });
        }
    };

    // Listing setup is available before AND after connecting — the draft is
    // bound to this marketplace key either way. Shared by both card states.
    let listing_section = move || -> AnyView {
        view! {
            {move || if published.get() {
                view! { <p class="mp-published">"✓ Listing published to "{mp_label}</p> }.into_any()
            } else if listing_open.get() {
                view! {
                    <div class="mp-listing-form" attr:data-pf-edit-scope=mp_edit_token>
                        <p class="mp-hint">"Listing setup — auto-filled from asset. Double-click a field to edit:"</p>
                        {move || {
                            let f = form.get();
                            view! {
                                <div class="mp-included">
                                    <span class="mp-included-label">"Included from asset — same format:"</span>
                                    <div class="mp-af-grid">
                                        {af_text_row("TITLE", "title", f.title.clone())}
                                        {af_text_row("DESCRIPTION", "description", f.description.clone())}
                                        {af_text_row("TYPE & BUILD", "category", f.category.clone())}
                                        {af_text_row("ADDRESS", "location", f.location.clone())}
                                        {af_text_row("PRICE", "price", {
                                            if f.price.is_empty() { String::new() } else { format!("${}", f.price) }
                                        })}
                                        <div class="mp-af-row">
                                            <span class="mp-af-label">"CONDITION"</span>
                                            {move || {
                                                if edit_field.get() == Some("condition") {
                                                    view! {
                                                        <select class="mp-af-input"
                                                            prop:value=move || edit_buf.get()
                                                            on:change=move |ev| {
                                                                set_edit_buf.set(event_target_value(&ev));
                                                                commit_edit();
                                                            }
                                                            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                                                if ev.key() == "Escape" { cancel_edit(); }
                                                            }>
                                                            <option value="New">"New"</option>
                                                            <option value="Used">"Used"</option>
                                                            <option value="For parts">"For parts"</option>
                                                            <option value="Refurbished">"Refurbished"</option>
                                                        </select>
                                                    }.into_any()
                                                } else {
                                                    let v = if f.condition.is_empty() { "—".to_string() } else { f.condition.clone() };
                                                    view! {
                                                        <span class="mp-af-value mp-af-editable" on:dblclick=move |_| start_edit("condition")>{v}</span>
                                                    }.into_any()
                                                }
                                            }}
                                        </div>
                                        {af_text_row("ASSET CONDITION", "asset_condition", f.asset_condition.clone())}
                                        {af_text_row("STATUS", "asset_status", f.asset_status.clone())}
                                        {af_text_row("REF / SKU", "sku", {
                                            if f.sku.is_empty() { String::new() } else { format!("SKU {}", f.sku) }
                                        })}
                                        {af_text_row("PHOTOS", "photos", {
                                            let n = if !f.photos.is_empty() { f.photos.len() } else { f.photo_count };
                                            if n == 0 { String::new() } else {
                                                format!("{} photo{}", n, if n == 1 { "" } else { "s" })
                                            }
                                        })}
                                        {move || {
                                            let f = form.get();
                                            let docs = asset_docs.get();
                                            // Explicit names win; a bare
                                            // doc_count means "all asset docs".
                                            let selected: Vec<String> =
                                                if !f.docs.is_empty() {
                                                    f.docs
                                                } else if f.doc_count > 0 {
                                                    docs.iter().map(|d| d.1.clone()).collect()
                                                } else {
                                                    Vec::new()
                                                };
                                            let doc_row = move |name: String,
                                                               ftype: String,
                                                               on: bool,
                                                               custom: bool| {
                                                let tn = name.clone();
                                                view! {
                                                    <label class="mp-af-doc"
                                                        class:mp-af-doc-on=on
                                                        class:mp-af-doc-custom=custom>
                                                        <input type="checkbox" class="mp-af-doc-check"
                                                            prop:checked=on
                                                            on:change=move |_| toggle_doc(tn.clone()) />
                                                        <span class="mp-af-doc-name">{name}</span>
                                                        <span class="mp-af-doc-type">{ftype}</span>
                                                    </label>
                                                }
                                            };
                                            if docs.is_empty() && selected.is_empty() {
                                                return view! {
                                                    <div class="mp-af-row">
                                                        <span class="mp-af-label">"DOCS"</span>
                                                        <span class="mp-af-value">"No documents on this asset"</span>
                                                    </div>
                                                }
                                                .into_any();
                                            }
                                            let asset_names: std::collections::HashSet<&str> =
                                                docs.iter().map(|d| d.1.as_str()).collect();
                                            view! {
                                                <div class="mp-af-docs">
                                                    <div class="mp-af-row">
                                                        <span class="mp-af-label">"DOCS"</span>
                                                        <span class="mp-af-value">
                                                            {if selected.is_empty() {
                                                                "—".to_string()
                                                            } else {
                                                                format!("{} selected", selected.len())
                                                            }}
                                                        </span>
                                                    </div>
                                                    <div class="mp-af-doc-list">
                                                        {docs.iter().map(|(_, name, ftype)| {
                                                            let on = selected.iter().any(|s| s == name);
                                                            doc_row(name.clone(), ftype.clone(), on, false)
                                                        }).collect::<Vec<_>>()}
                                                        {selected.iter()
                                                            .filter(|n| !asset_names.contains(n.as_str()))
                                                            .map(|n| doc_row(n.clone(), "custom".to_string(), true, true))
                                                            .collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }
                                            .into_any()
                                        }}
                                        {af_text_row("TAGS", "tags", f.tags.join(", "))}
                                        {if mp_key != "EBay" { ().into_any() } else {
                                            view! {
                                                <div class="mp-af-section">"EBAY BIDDING"</div>
                                                {af_text_row("START BID", "auction_start", f.auction_start.clone())}
                                                {af_text_row("BUY IT NOW", "buy_it_now", f.buy_it_now.clone())}
                                                {af_text_row("BID INCREMENT", "bid_increment", f.bid_increment.clone())}
                                                {af_text_row("DAYS", "auction_days", f.auction_days.clone())}
                                                <div class="mp-af-section">"EBAY SITES"</div>
                                                {move || {
                                                    let sites = enabled_sites.get();
                                                    if sites.is_empty() {
                                                        return view! {
                                                            <p class="mp-hint">"No eBay sites enabled — choose them in Marketplace → Channels."</p>
                                                        }.into_any();
                                                    }
                                                    sites.into_iter().map(|id| {
                                                        let def = ebay_site(&id);
                                                        let name = def.map(|d| format!("{} ({})", d.label, d.domain)).unwrap_or_else(|| id.clone());
                                                        let cur = def.map(|d| d.currency_code).unwrap_or("");
                                                        let on_id = id.clone();
                                                        let on = move || form.with(|fm| fm.sites.contains(&on_id));
                                                        let on2 = on.clone();
                                                        view! {
                                                            <label class="mp-af-doc" class:mp-af-doc-on=on>
                                                                <input type="checkbox" class="mp-af-doc-check"
                                                                    prop:checked=on2
                                                                    on:change=move |_| set_form.update(|fm| {
                                                                        if let Some(i) = fm.sites.iter().position(|x| *x == id) {
                                                                            fm.sites.remove(i);
                                                                        } else {
                                                                            fm.sites.push(id.clone());
                                                                        }
                                                                    })
                                                                />
                                                                <span class="mp-af-doc-name">{name}</span>
                                                                <span class="mp-af-doc-type">{cur}</span>
                                                            </label>
                                                        }
                                                    }).collect::<Vec<_>>().into_any()
                                                }}
                                            }.into_any()
                                        }}
                                    </div>
                                    {if f.photos.is_empty() { ().into_any() } else {
                                        view! {
                                            <div class="mp-af-photos">
                                                {f.photos.iter().take(8).cloned().map(|u| view! {
                                                    <img class="mp-af-photo" src={u} alt="Listing photo" />
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_any()
                                    }}
                                </div>
                            }
                        }}
                        {move || form_err.get().map(|e| view! { <p class="mp-error">{e}</p> })}
                        <div class="mp-form-actions">
                            <button type="button" class="pf-small-btn" on:click=move |_| set_listing_open.set(false)>"Cancel"</button>
                            <button type="button" class="pf-small-btn" on:click=save_draft>"Save draft"</button>
                            <button type="button" class="login-btn sell" on:click=publish>"Publish listing"</button>
                        </div>
                    </div>
                }.into_any()
            } else {
                ().into_any()
            }}
            {move || listing_saved.get().map(|l| {
                // Status is governed by actual bid activity — "bidding"
                // appears automatically while active bids exist.
                let eff = l.effective_status();
                let status_label = match eff {
                    "published" => "Published",
                    "bidding" => "Bidding",
                    "sold" => "Sold",
                    _ => "Draft saved",
                };
                let active_bids = l.bids.iter().filter(|b| b.state == "active").count();
                let high = l.high_bid().map(|b| b.amount);
                let bidding_open = matches!(eff, "published" | "bidding")
                    && (mp_key == "EBay" || mp_key == "Red");
                view! {
                    <div class="mp-listing-status-block">
                        <p class="mp-listing-status">
                            {format!("{} · {}", status_label, l.saved_at)}
                            {if active_bids > 0 {
                                format!(
                                    " · {} bid{} · high ${:.2}",
                                    active_bids,
                                    if active_bids == 1 { "" } else { "s" },
                                    high.unwrap_or(0.0)
                                )
                            } else { String::new() }}
                        </p>
                        {if !l.bids.is_empty() {
                            view! {
                                <div class="mp-bid-list">
                                    {l.bids.iter().rev().take(5).map(|b| {
                                        let is_active = b.state == "active";
                                        view! {
                                            <div class="mp-bid-row" class:mp-bid-active=is_active>
                                                <span class="mp-bid-amount">{format!("${:.2}", b.amount)}</span>
                                                <span class="mp-bid-who">{b.bidder.clone()}</span>
                                                <span class="mp-bid-state">{b.state.clone()}</span>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        } else { ().into_any() }}
                        {if bidding_open {
                            view! {
                                <div class="mp-bid-form">
                                    <input class="login-input" type="text" placeholder="Bidder name"
                                        aria-label="Bidder name"
                                        prop:value=move || bid_bidder.get()
                                        on:input=move |ev| set_bid_bidder.set(event_target_value(&ev)) />
                                    <input class="login-input" type="text"
                                        placeholder={if let Some(h) = high {
                                            format!("Bid > ${:.2}", h)
                                        } else {
                                            "Bid amount".to_string()
                                        }}
                                        aria-label="Bid amount"
                                        prop:value=move || bid_amount_val.get()
                                        on:input=move |ev| set_bid_amount_val.set(event_target_value(&ev)) />
                                    <button type="button" class="pf-small-btn" on:click=move |_| record_bid.run(())>"Record bid"</button>
                                    {move || bid_msg.get().map(|m| view! { <span class="mp-hint">{m}</span> })}
                                </div>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                }
            })}
        }
        .into_any()
    };

    view! {
        <div class="mp-card">
            <div class="mp-card-head">
                <span class="mp-card-icon">{mp_icon}</span>
                <span class="mp-card-name">{mp_label}</span>
                {move || if can_link && !published.get() && !listing_open.get() {
                    view! {
                        <button type="button" class="mp-head-btn" on:click=open_listing>
                            {move || {
                                if listing_saved.get().is_some() {
                                    "Edit listing"
                                } else if already_linked() {
                                    "Update listing"
                                } else {
                                    "Set up listing"
                                }
                            }}
                        </button>
                    }.into_any()
                } else { ().into_any() }}
                {move || if listing_saved.get().map(|l| l.status == "published").unwrap_or(false) {
                    view! {
                        <button type="button" class="mp-head-btn" on:click=open_listing_messages>"Messages"</button>
                    }.into_any()
                } else { ().into_any() }}
                {move || if conn.get().is_some() {
                    view! { <span class="mp-badge mp-badge-on mp-badge-square">"● Connected"</span> }.into_any()
                } else {
                    view! { <span class="mp-badge mp-badge-square">"Not connected"</span> }.into_any()
                }}
            </div>
            {move || if !can_link {
                view! { <p class="mp-hint">"You don't have permission to manage marketplace listings."</p> }.into_any()
            } else if conn.get().is_none() {
                view! {
                    <div class="mp-connect">
                        <p class="mp-hint">{format!("{} isn't connected. Connect it once in the Marketplace tab, under Channels.", mp_label)}</p>
                        {if !mp_hint.is_empty() {
                            view! { <p class="mp-hint mp-hint-sub">{mp_hint}</p> }.into_any()
                        } else { ().into_any() }}
                        <button type="button" class="pf-small-btn" on:click=open_channels_tab>"Open Marketplace channels"</button>
                        {listing_section()}
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="mp-connected">
                        {move || conn.get().map(|account| view! {
                            <p class="mp-account">"✓ Connected — "{account}</p>
                        })}
                        {listing_section()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
pub(crate) fn AssetMarketplaceControls(
    asset_id: Uuid,
    asset_name: String,
    portfolio_id: Option<Uuid>,
    can_link: bool,
) -> impl IntoView {
    view! {
        <div class="ai-marketplace-controls">
            <For
                each=move || MARKETPLACES.to_vec()
                key=|d| d.key
                children=move |def| view! {
                    <MarketplaceCard
                        mp_key=def.key
                        mp_label=def.label
                        asset_id={asset_id}
                        asset_name={asset_name.clone()}
                        portfolio_id={portfolio_id}
                        can_link={can_link}
                    />
                }
            />
        </div>
    }
}
