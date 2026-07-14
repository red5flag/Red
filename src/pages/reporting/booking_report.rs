use crate::models::{BookingStatus, ChannelType, ServiceTaskStatus};
use crate::stores::use_app_store;
use leptos::prelude::*;

pub(crate) fn bookings_view() -> impl IntoView {
    let app_store = use_app_store();

    let channels = Memo::new(move |_| app_store.get().channels.clone());
    let bookings = Memo::new(move |_| app_store.get().bookings.clone());
    let tasks = Memo::new(move |_| app_store.get().service_tasks.clone());

    let metrics = Memo::new(move |_| {
        let ch = channels.get();
        let bk = bookings.get();
        let tk = tasks.get();
        let total_channels = ch.len();
        let test_channels = ch
            .iter()
            .filter(|c| matches!(c.channel_type, ChannelType::Test))
            .count();
        let total_bookings = bk.len();
        let active_bookings = bk.iter().filter(|b| b.is_active()).count();
        let total_revenue: f64 = bk
            .iter()
            .filter(|b| !matches!(b.status, BookingStatus::Cancelled))
            .map(|b| b.total)
            .sum();
        let upcoming_nights: u32 = bk.iter().filter(|b| b.is_active()).map(|b| b.nights).sum();
        let completed_tasks = tk
            .iter()
            .filter(|t| matches!(t.status, ServiceTaskStatus::Done))
            .count();
        let pending_tasks = tk
            .iter()
            .filter(|t| {
                !matches!(
                    t.status,
                    ServiceTaskStatus::Done | ServiceTaskStatus::Cancelled
                )
            })
            .count();
        (
            total_channels,
            test_channels,
            total_bookings,
            active_bookings,
            total_revenue,
            upcoming_nights,
            completed_tasks,
            pending_tasks,
        )
    });

    view! {
        <div class="reporting-section">
            <div class="reporting-section-header">
                <h3 class="reporting-section-title">"Bookings & Channel Metrics"</h3>
                <button class="reporting-btn" disabled title="Export via clipboard/file will be enabled in a later stage">"Export (deferred)"</button>
            </div>

            {move || {
                let (tc, ttc, tb, ab, rev, nights, ct, pt) = metrics.get();
                view! {
                    <div class="reporting-metrics">
                        <div class="reporting-metric-card">
                            <div class="reporting-metric-value">{tc}</div>
                            <div class="reporting-metric-label">"Total Channels"</div>
                            <div class="reporting-metric-sub">{format!("{} Test", ttc)}</div>
                        </div>
                        <div class="reporting-metric-card">
                            <div class="reporting-metric-value">{tb}</div>
                            <div class="reporting-metric-label">"Total Bookings"</div>
                            <div class="reporting-metric-sub">{format!("{} Active", ab)}</div>
                        </div>
                        <div class="reporting-metric-card">
                            <div class="reporting-metric-value">{format!("${:.2}", rev)}</div>
                            <div class="reporting-metric-label">"Booking Revenue"</div>
                            <div class="reporting-metric-sub">{format!("{} upcoming nights", nights)}</div>
                        </div>
                        <div class="reporting-metric-card">
                            <div class="reporting-metric-value">{ct}</div>
                            <div class="reporting-metric-label">"Tasks Done"</div>
                            <div class="reporting-metric-sub">{format!("{} pending", pt)}</div>
                        </div>
                    </div>
                }.into_any()
            }}

            <div class="reporting-table">
                {super::table_head(&["Channel", "Type", "Status", "Linked Asset", "Last Sync"])}
                {move || {
                    let list = channels.get();
                    if list.is_empty() {
                        view! { <div class="reporting-empty">"No channels configured."</div> }.into_any()
                    } else {
                        view! {
                            <div>
                                {list.into_iter().map(|c| {
                                    let type_label = match c.channel_type {
                                        ChannelType::Test => "Test Channel",
                                        ChannelType::Airbnb => "Airbnb",
                                        ChannelType::BookingCom => "Booking.com",
                                        ChannelType::Expedia => "Expedia",
                                        ChannelType::Vrbo => "Vrbo",
                                        ChannelType::LinkedIn => "LinkedIn",
                                        ChannelType::Other(_) => "Other",
                                    };
                                    let status = format!("{:?}", c.connection_status);
                                    let linked = c.linked_asset_id.map(|id| format!("Asset {}", id)).unwrap_or_else(|| "—".to_string());
                                    let last = c.last_sync_status.clone().unwrap_or_else(|| "Never".to_string());
                                    view! {
                                        <div class="reporting-table-row">
                                            <div class="reporting-td">{c.name}</div>
                                            <div class="reporting-td">{type_label}</div>
                                            <div class="reporting-td">{status}</div>
                                            <div class="reporting-td">{linked}</div>
                                            <div class="reporting-td">{last}</div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            <div class="reporting-table">
                {super::table_head(&["Guest", "Asset", "Dates", "Nights", "Total", "Status"])}
                {move || {
                    let list = bookings.get();
                    if list.is_empty() {
                        view! { <div class="reporting-empty">"No bookings recorded."</div> }.into_any()
                    } else {
                        view! {
                            <div>
                                {list.into_iter().map(|b| {
                                    let start = b.start_datetime.format("%d %b %Y").to_string();
                                    let end = b.end_datetime.format("%d %b %Y").to_string();
                                    let status = format!("{:?}", b.status);
                                    let asset_label = b.asset_id.to_string();
                                    view! {
                                        <div class="reporting-table-row">
                                            <div class="reporting-td">{b.guest_name}</div>
                                            <div class="reporting-td">{asset_label}</div>
                                            <div class="reporting-td">{start}" → "{end}</div>
                                            <div class="reporting-td">{b.nights}</div>
                                            <div class="reporting-td">{format!("${:.2}", b.total)}</div>
                                            <div class="reporting-td">{status}</div>
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
