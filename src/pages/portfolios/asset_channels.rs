use crate::models::{
    Booking, BookingSource, BookingStatus, Channel, ChannelType, ConnectionStatus, SyncDirection,
};
use crate::stores::{use_app_store, use_calendar_store};
use chrono::{Duration, NaiveDate, Utc};
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;
use wasm_bindgen::JsCast;

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
        let channel_name =
            channel_id.and_then(|cid| app_store.get().get_channel(cid).map(|c| c.name.clone()));

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
        let channel_name =
            channel_id.and_then(|cid| app_store.get().get_channel(cid).map(|c| c.name.clone()));
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
    let channel = app_store.get().get_channel(channel_id).cloned();

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
