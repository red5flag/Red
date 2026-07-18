use crate::models::{CalendarEvent, CalendarEventType};
use crate::pages::portfolios::{AddChannelModal, ChannelManagementWindow};
use crate::stores::{use_app_store, use_calendar_store};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct CalendarScope {
    pub portfolio_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub title: String,
}

impl CalendarScope {
    pub fn global() -> Self {
        Self {
            portfolio_id: None,
            group_id: None,
            asset_id: None,
            title: "Calendar".to_string(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CalendarViewMode {
    Month,
    MultiMonth,
    Year,
}

impl CalendarViewMode {
    fn label(&self) -> &'static str {
        match self {
            CalendarViewMode::Month => "Month",
            CalendarViewMode::MultiMonth => "Multi-month",
            CalendarViewMode::Year => "Year",
        }
    }
}

fn month_label(year: i32, month: u32) -> String {
    let labels = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    format!("{} {}", labels[(month as usize - 1).min(11)], year)
}

#[component]
pub fn CalendarManager(
    scope: CalendarScope,
    #[prop(default = false)] embedded: bool,
) -> impl IntoView {
    let app_store = use_app_store();
    let calendar_store = use_calendar_store();
    let (selected_types, set_selected_types) = signal(HashSet::<CalendarEventType>::from_iter(
        CalendarEventType::all().iter().copied(),
    ));
    let toggle_type = Callback::new(move |t: CalendarEventType| {
        set_selected_types.update(|set| {
            if set.contains(&t) {
                set.remove(&t);
            } else {
                set.insert(t);
            }
        });
    });

    let events_for_scope = Memo::new(move |_| {
        let app = app_store.get();
        let calendar = calendar_store.get();
        let mut events: Vec<CalendarEvent> =
            if scope.portfolio_id.is_none() && scope.group_id.is_none() && scope.asset_id.is_none()
            {
                let mut local = calendar.calendar_events.clone();
                for p in &app.portfolios {
                    local.extend(p.calendar_events.clone());
                    for g in &p.asset_groups {
                        local.extend(g.calendar_events.clone());
                        for a in &g.assets {
                            local.extend(a.calendar_events.clone());
                        }
                    }
                    for a in &p.assets {
                        local.extend(a.calendar_events.clone());
                    }
                }
                local
            } else {
                let mut local = Vec::new();
                if let Some(pid) = scope.portfolio_id {
                    if let Some(p) = app.portfolios.iter().find(|p| p.id == pid) {
                        local.extend(p.calendar_events.clone());
                        for g in &p.asset_groups {
                            local.extend(g.calendar_events.clone());
                            for a in &g.assets {
                                local.extend(a.calendar_events.clone());
                            }
                        }
                        for a in &p.assets {
                            local.extend(a.calendar_events.clone());
                        }
                    }
                } else if let Some(gid) = scope.group_id {
                    for p in &app.portfolios {
                        if let Some(g) = p.asset_groups.iter().find(|g| g.id == gid) {
                            local.extend(g.calendar_events.clone());
                            for a in &g.assets {
                                local.extend(a.calendar_events.clone());
                            }
                            break;
                        }
                    }
                } else if let Some(aid) = scope.asset_id {
                    for p in &app.portfolios {
                        let all: Vec<&crate::models::Asset> = p
                            .assets
                            .iter()
                            .chain(p.asset_groups.iter().flat_map(|g| g.assets.iter()))
                            .collect();
                        if let Some(a) = all.into_iter().find(|a| a.id == aid) {
                            local.extend(a.calendar_events.clone());
                            break;
                        }
                    }
                }
                local
            };
        events.sort_by(|a, b| a.start.cmp(&b.start));
        events.dedup_by(|a, b| a.id == b.id);
        let selected = selected_types.get();
        events.retain(|e| selected.contains(&e.event_type));
        events
    });

    let today = Utc::now().date_naive();
    let (view_year, set_view_year) = signal(today.year());
    let (view_month, set_view_month) = signal(today.month());
    let (calendar_view_mode, set_calendar_view_mode) = signal(CalendarViewMode::Month);
    let (zoom, set_zoom) = signal(1_usize);
    let (selected_date, set_selected_date) = signal::<Option<NaiveDate>>(None);
    let (editing_event, set_editing_event) = signal::<Option<CalendarEvent>>(None);
    let (show_form, set_show_form) = signal(false);

    let prev_month = move |_| {
        let (y, m) = (view_year.get(), view_month.get());
        if m == 1 {
            set_view_year.set(y - 1);
            set_view_month.set(12);
        } else {
            set_view_month.set(m - 1);
        }
    };
    let next_month = move |_| {
        let (y, m) = (view_year.get(), view_month.get());
        if m == 12 {
            set_view_year.set(y + 1);
            set_view_month.set(1);
        } else {
            set_view_month.set(m + 1);
        }
    };
    let go_today = move |_| {
        set_view_year.set(today.year());
        set_view_month.set(today.month());
        set_selected_date.set(Some(today));
    };

    let open_new_event = move |date: NaiveDate| {
        let start = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
        let mut ev = CalendarEvent::new("New event".to_string(), start, start + Duration::hours(1));
        ev.related_portfolio_id = scope.portfolio_id;
        ev.related_group_id = scope.group_id;
        ev.related_asset_id = scope.asset_id;
        ev.category = Some("General".to_string());
        set_editing_event.set(Some(ev));
        set_show_form.set(true);
    };

    let on_add_new = move |_| {
        open_new_event(selected_date.get().unwrap_or(today));
    };

    let on_edit = Callback::new(move |ev: CalendarEvent| {
        set_editing_event.set(Some(ev));
        set_show_form.set(true);
    });

    let on_delete = Callback::new(move |id: Uuid| {
        calendar_store.update(|s| s.remove_calendar_event(id));
        app_store.update(|s| s.remove_calendar_event_from_portfolios(id));
    });

    let on_save = Callback::new(move |ev: CalendarEvent| {
        calendar_store.update(|s| s.upsert_calendar_event(ev.clone()));
        app_store.update(|s| s.sync_calendar_event_to_portfolios(ev));
        set_show_form.set(false);
        set_editing_event.set(None);
    });

    let on_cancel = Callback::new(move |_| {
        set_show_form.set(false);
        set_editing_event.set(None);
    });

    let on_day_click = Callback::new(move |date: NaiveDate| {
        set_selected_date.set(Some(date));
    });

    let on_day_add = Callback::new(move |date: NaiveDate| {
        open_new_event(date);
    });

    let actions_ref = NodeRef::<leptos::html::Div>::new();
    let scroll_actions_left = move |_| {
        if let Some(el) = actions_ref.get() {
            let _ = el.scroll_by_with_x_and_y(-120.0, 0.0);
        }
    };
    let scroll_actions_right = move |_| {
        if let Some(el) = actions_ref.get() {
            let _ = el.scroll_by_with_x_and_y(120.0, 0.0);
        }
    };

    // Build the grid days for the current view month
    let grid_days = Memo::new(move |_| {
        let y = view_year.get();
        let m = view_month.get();
        let first_day = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
        let days_in_month = (NaiveDate::from_ymd_opt(y, m + 1, 1)
            .unwrap_or(NaiveDate::from_ymd_opt(y + 1, 1, 1).unwrap())
            - first_day)
            .num_days() as u32;
        let start_weekday = first_day.weekday().num_days_from_sunday();

        let mut days: Vec<Option<NaiveDate>> = Vec::new();
        // Leading blanks from previous month
        for i in 0..start_weekday {
            if let Some(d) =
                first_day.checked_sub_signed(Duration::days((start_weekday - i) as i64))
            {
                days.push(Some(d));
            } else {
                days.push(None);
            }
        }
        for d in 1..=days_in_month {
            days.push(Some(NaiveDate::from_ymd_opt(y, m, d).unwrap()));
        }
        // Trailing fill to complete the last week (42 cells = 6 rows)
        while days.len() < 42 {
            let last = days.last().and_then(|d| *d).unwrap_or(first_day);
            if let Some(d) = last.checked_add_signed(Duration::days(1)) {
                days.push(Some(d));
            } else {
                days.push(None);
            }
        }
        days
    });

    // Events for the selected day
    let selected_day_events = Memo::new(move |_| {
        let date = selected_date.get();
        let events = events_for_scope.get();
        match date {
            Some(d) => events
                .into_iter()
                .filter(|e| e.start.date_naive() == d)
                .collect::<Vec<_>>(),
            None => Vec::new(),
        }
    });

    view! {
        <div class={if embedded { "calendar-manager embedded" } else { "calendar-manager" }}>
            <div class="calendar-manager-header">
                <div class="calendar-manager-title">
                    {move || month_label(view_year.get(), view_month.get())}
                </div>
                <button class="calendar-scroll-arrow calendar-scroll-arrow-left"
                    title="Scroll controls left"
                    aria-label="Scroll calendar controls left"
                    on:click=scroll_actions_left>"‹"</button>
                <div class="calendar-manager-actions" node_ref=actions_ref>
                    <button class="calendar-nav-btn" on:click=prev_month title="Previous month">"‹"</button>
                    <button class="calendar-today-btn" on:click=go_today>"Today"</button>
                    <button class="calendar-nav-btn" on:click=next_month title="Next month">"›"</button>
                    <div class="calendar-view-controls">
                        {[CalendarViewMode::Month, CalendarViewMode::MultiMonth, CalendarViewMode::Year]
                            .into_iter()
                            .map(|mode| {
                                let mode_clone = mode;
                                view! {
                                    <button
                                        class="calendar-view-btn"
                                        class:calendar-view-active={move || calendar_view_mode.get() == mode_clone}
                                        on:click=move |_| set_calendar_view_mode.set(mode_clone)
                                    >
                                        {mode_clone.label()}
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                    </div>
                    <div class="calendar-zoom-controls">
                        <button
                            class="calendar-view-btn"
                            on:click=move |_| set_zoom.update(|z| *z = z.saturating_sub(1).max(1))
                            title="Zoom out"
                            aria-label="Zoom out"
                        >"−"</button>
                        <span class="calendar-zoom-label">{move || format!("{}x", zoom.get())}</span>
                        <button
                            class="calendar-view-btn"
                            on:click=move |_| set_zoom.update(|z| *z = (*z + 1).min(4))
                            title="Zoom in"
                            aria-label="Zoom in"
                        >"+"</button>
                    </div>
                    <button class="calendar-manager-btn" on:click=on_add_new title="Add event">"+"</button>
                </div>
                <button class="calendar-scroll-arrow calendar-scroll-arrow-right"
                    title="Scroll controls right"
                    aria-label="Scroll calendar controls right"
                    on:click=scroll_actions_right>"›"</button>
            </div>

            <div class="calendar-filter-bar" aria-label="Event type filters">
                {CalendarEventType::all().iter().copied().map(|t| {
                    let toggle = toggle_type.clone();
                    view! {
                        <button
                            class="calendar-filter-btn"
                            class:calendar-filter-active={move || selected_types.get().contains(&t)}
                            on:click=move |_| toggle.run(t)
                            title={format!("Toggle {}", t.display())}
                        >
                            {t.display()}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            {move || if show_form.get() {
                if let Some(ev) = editing_event.get() {
                    view! { <EventEditor event={ev} on_save={on_save.clone()} on_cancel={on_cancel.clone()} /> }.into_any()
                } else { ().into_any() }
            } else { ().into_any() }}

            <div class="calendar-grid-wrapper">
                // Weekday headers
                <div class="calendar-grid-weekdays">
                    {vec!["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].into_iter().map(|d| {
                        view! { <div class="calendar-grid-weekday">{d}</div> }
                    }).collect::<Vec<_>>()}
                </div>
                // Day cells
                <div
                    class="calendar-grid-days"
                    style={move || { let z = zoom.get(); format!("grid-auto-rows: {}px; aspect-ratio: auto;", 70 * z) }}
                >
                    {move || {
                        let days = grid_days.get();
                        let events = events_for_scope.get();
                        let vy = view_year.get();
                        let vm = view_month.get();
                        let sel = selected_date.get();
                        let on_day_click = on_day_click.clone();
                        let on_day_add = on_day_add.clone();
                        let on_edit = on_edit.clone();

                        days.into_iter().map(|opt_date| {
                            let on_dc = on_day_click.clone();
                            let on_da = on_day_add.clone();
                            let on_e = on_edit.clone();

                            match opt_date {
                                Some(date) => {
                                    let is_today = date == today;
                                    let is_current_month = date.year() == vy && date.month() == vm;
                                    let is_selected = sel == Some(date);
                                    let day_events: Vec<CalendarEvent> = events.iter()
                                        .filter(|e| e.start.date_naive() == date)
                                        .cloned()
                                        .collect();
                                    let event_count = day_events.len();
                                    let max_chips = zoom.get() * 3;

                                    view! {
                                        <div class="calendar-grid-day"
                                            class:today={is_today}
                                            class:other-month={!is_current_month}
                                            class:selected={is_selected}
                                            on:click=move |_| on_dc.run(date)
                                        >
                                            <div class="calendar-grid-day-num">{date.day().to_string()}</div>
                                            <div class="calendar-grid-day-events">
                                                {day_events.iter().take(max_chips).map(|ev| {
                                                    let ev_clone = ev.clone();
                                                    let on_e2 = on_e.clone();
                                                    let _cat = ev.category.clone().unwrap_or_else(|| "General".to_string());
                                                    let time_str = if ev.all_day {
                                                        "All day".to_string()
                                                    } else {
                                                        ev.start.format("%H:%M").to_string()
                                                    };
                                                    view! {
                                                        <div class="calendar-grid-event-chip"
                                                            on:click=move |e: leptos::ev::MouseEvent| {
                                                                e.stop_propagation();
                                                                on_e2.run(ev_clone.clone());
                                                            }
                                                        >
                                                            <span class="calendar-grid-event-time">{time_str}</span>
                                                            <span class="calendar-grid-event-title">{ev.title.clone()}</span>
                                                        </div>
                                                    }
                                                }).collect::<Vec<_>>()}
                                                {if event_count > max_chips {
                                                    view! {
                                                        <div class="calendar-grid-event-more">
                                                            {format!("+{} more", event_count - max_chips)}
                                                        </div>
                                                    }.into_any()
                                                } else { ().into_any() }}
                                            </div>
                                            <button class="calendar-grid-day-add"
                                                on:click=move |e: leptos::ev::MouseEvent| {
                                                    e.stop_propagation();
                                                    on_da.run(date);
                                                }
                                                title="Add event"
                                            >"+"</button>
                                        </div>
                                    }.into_any()
                                }
                                None => view! { <div class="calendar-grid-day empty"></div> }.into_any(),
                            }
                        }).collect::<Vec<_>>()
                    }}
                </div>
            </div>

            // Selected day detail panel
            {move || selected_date.get().map(|date| {
                let events = selected_day_events.get();
                let on_edit2 = on_edit.clone();
                let on_delete2 = on_delete.clone();
                let on_add2 = on_day_add.clone();
                let date_label = date.format("%A, %d %B %Y").to_string();
                view! {
                    <div class="calendar-day-detail">
                        <div class="calendar-day-detail-header">
                            <div class="calendar-day-detail-date">{date_label}</div>
                            <div class="calendar-day-detail-actions">
                                <button class="calendar-day-detail-add" on:click=move |_| on_add2.run(date)>"+ Add event"</button>
                                <button class="calendar-day-detail-close" on:click=move |_| set_selected_date.set(None)>"✕"</button>
                            </div>
                        </div>
                        <div class="calendar-day-detail-body">
                            {if events.is_empty() {
                                view! { <div class="calendar-day-detail-empty">"No events on this day"</div> }.into_any()
                            } else {
                                events.into_iter().map(|ev| {
                                    let on_e3 = on_edit2.clone();
                                    let on_d3 = on_delete2.clone();
                                    let ev_for_edit = ev.clone();
                                    let id = ev.id;
                                    let time = if ev.all_day {
                                        "All day".to_string()
                                    } else {
                                        format!("{} – {}", ev.start.format("%H:%M"), ev.end.format("%H:%M"))
                                    };
                                    let cat = ev.category.clone().unwrap_or_else(|| "General".to_string());
                                    let desc = ev.description.clone();
                                    view! {
                                        <div class="calendar-day-detail-event">
                                            <div class="calendar-day-detail-event-time">{time}</div>
                                            <div class="calendar-day-detail-event-body">
                                                <div class="calendar-day-detail-event-title">{ev.title.clone()}</div>
                                                <div class="calendar-day-detail-event-cat">{cat}</div>
                                                {desc.map(|d| view! {
                                                    <div class="calendar-day-detail-event-desc">{d}</div>
                                                }.into_any()).unwrap_or_else(|| ().into_any())}
                                            </div>
                                            <div class="calendar-day-detail-event-actions">
                                                <button class="calendar-event-row-btn" on:click=move |_| on_e3.run(ev_for_edit.clone())>"✎"</button>
                                                <button class="calendar-event-row-btn danger" on:click=move |_| on_d3.run(id)>"✕"</button>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>().into_any()
                            }}
                        </div>
                    </div>
                }.into_any()
            }).unwrap_or_else(|| ().into_any())}
        </div>
    }
}

#[component]
fn EventEditor(
    event: CalendarEvent,
    on_save: Callback<CalendarEvent>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (title, set_title) = signal(event.title.clone());
    let (date, set_date) = signal(event.start.format("%Y-%m-%d").to_string());
    let (time, set_time) = signal(event.start.format("%H:%M").to_string());
    let (end_date, set_end_date) = signal(event.end.format("%Y-%m-%d").to_string());
    let (end_time, set_end_time) = signal(event.end.format("%H:%M").to_string());
    let (all_day, set_all_day) = signal(event.all_day);
    let (category, set_category) = signal(event.category.clone().unwrap_or_default());
    let (description, set_description) = signal(event.description.clone().unwrap_or_default());

    let app_store = use_app_store();
    let (show_add_channel, set_show_add_channel) = signal(false);
    let (show_manage_channel, set_show_manage_channel) = signal(false);
    let related_asset_id = event.related_asset_id;
    let related_portfolio_id = event.related_portfolio_id;
    let related_channel_id = event.related_channel_id;
    let can_link_channel = related_asset_id.is_some();

    let asset_name_for_modal = Memo::new(move |_| {
        related_asset_id
            .and_then(|aid| {
                app_store
                    .get()
                    .portfolios
                    .iter()
                    .flat_map(|p| p.assets.iter().chain(p.asset_groups.iter().flat_map(|g| g.assets.iter())))
                    .find(|a| a.id == aid)
                    .map(|a| a.name.clone())
            })
            .unwrap_or_else(|| "Asset".to_string())
    });

    let ev = event.clone();
    let save = move |_| {
        let Ok(d) = NaiveDate::parse_from_str(&date.get(), "%Y-%m-%d") else {
            return;
        };
        let ad = all_day.get();

        let (start, end) = if ad {
            let s = d.and_hms_opt(0, 0, 0).unwrap();
            let e = NaiveDate::parse_from_str(&end_date.get(), "%Y-%m-%d")
                .unwrap_or(d)
                .and_hms_opt(23, 59, 0)
                .unwrap_or(s);
            (s.and_utc(), e.and_utc())
        } else {
            let time_str = time.get();
            let parts: Vec<&str> = time_str.split(':').collect();
            if parts.len() != 2 {
                return;
            }
            let Ok(h) = parts[0].parse::<u32>() else {
                return;
            };
            let Ok(m) = parts[1].parse::<u32>() else {
                return;
            };
            let Some(start) = d.and_hms_opt(h, m, 0) else {
                return;
            };

            let end_dt = NaiveDate::parse_from_str(&end_date.get(), "%Y-%m-%d").unwrap_or(d);
            let end_time_str = end_time.get();
            let end_parts: Vec<&str> = end_time_str.split(':').collect();
            let end = if end_parts.len() == 2 {
                let Ok(eh) = end_parts[0].parse::<u32>() else {
                    return;
                };
                let Ok(em) = end_parts[1].parse::<u32>() else {
                    return;
                };
                end_dt.and_hms_opt(eh, em, 0).unwrap_or(start)
            } else {
                start + Duration::hours(1)
            };
            (start.and_utc(), end.and_utc())
        };

        let mut updated = ev.clone();
        updated.title = title.get();
        updated.start = start;
        updated.end = end;
        updated.all_day = ad;
        updated.category = if category.get().trim().is_empty() {
            None
        } else {
            Some(category.get())
        };
        updated.description = if description.get().trim().is_empty() {
            None
        } else {
            Some(description.get())
        };
        on_save.run(updated);
    };

    view! {
        <div class="calendar-event-editor-overlay" on:click=move |_| on_cancel.run(())>
            <div class="calendar-event-editor" on:click=|ev| ev.stop_propagation()>
                <div class="calendar-event-editor-title">"Edit Event"</div>
                <input class="calendar-event-editor-input" type="text" prop:value={move || title.get()} on:input=move |e| set_title.set(event_target_value(&e)) placeholder="Title" />

                <div class="calendar-event-editor-row">
                    <label class="calendar-event-editor-checkbox">
                        <input type="checkbox" prop:checked={move || all_day.get()} on:change=move |e| set_all_day.set(event_target_checked(&e)) />
                        "All day"
                    </label>
                </div>

                <div class="calendar-event-editor-row">
                    <div class="calendar-event-editor-field">
                        <label>"Start date"</label>
                        <input class="calendar-event-editor-input" type="date" prop:value={move || date.get()} on:change=move |e| set_date.set(event_target_value(&e)) />
                    </div>
                    {move || if !all_day.get() {
                        view! {
                            <div class="calendar-event-editor-field">
                                <label>"Start time"</label>
                                <input class="calendar-event-editor-input" type="time" prop:value={move || time.get()} on:change=move |e| set_time.set(event_target_value(&e)) />
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                </div>

                <div class="calendar-event-editor-row">
                    <div class="calendar-event-editor-field">
                        <label>"End date"</label>
                        <input class="calendar-event-editor-input" type="date" prop:value={move || end_date.get()} on:change=move |e| set_end_date.set(event_target_value(&e)) />
                    </div>
                    {move || if !all_day.get() {
                        view! {
                            <div class="calendar-event-editor-field">
                                <label>"End time"</label>
                                <input class="calendar-event-editor-input" type="time" prop:value={move || end_time.get()} on:change=move |e| set_end_time.set(event_target_value(&e)) />
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                </div>

                <input class="calendar-event-editor-input" type="text" prop:value={move || category.get()} on:input=move |e| set_category.set(event_target_value(&e)) placeholder="Category (e.g. Bills, Books, Upcoming)" />
                <textarea class="calendar-event-editor-textarea" prop:value={move || description.get()} on:input=move |e| set_description.set(event_target_value(&e)) placeholder="Description"></textarea>

                <div class="calendar-event-editor-actions">
                    <button class="calendar-event-editor-save" on:click=save>"Save"</button>
                    {move || if can_link_channel { view! {
                        <button
                            class="calendar-event-editor-save"
                            on:click=move |_| {
                                if related_channel_id.is_some() {
                                    set_show_manage_channel.set(true);
                                } else {
                                    set_show_add_channel.set(true);
                                }
                            }
                        >
                            {if related_channel_id.is_some() { "Manage Channel" } else { "Link Channel" }}
                        </button>
                    }.into_any() } else { ().into_any() }}
                    <button class="calendar-event-editor-cancel" on:click=move |_| on_cancel.run(())>"Cancel"</button>
                </div>

                {move || if show_add_channel.get() { view! {
                    <AddChannelModal
                        asset_id={related_asset_id.unwrap()}
                        asset_name={asset_name_for_modal.get()}
                        portfolio_id={related_portfolio_id}
                        on_close={Callback::new(move |_| set_show_add_channel.set(false))}
                    />
                }.into_any() } else { ().into_any() }}

                {move || if show_manage_channel.get() {
                    if let Some(cid) = related_channel_id {
                        view! {
                            <ChannelManagementWindow channel_id={cid} on_close={Callback::new(move |_| set_show_manage_channel.set(false))} />
                        }.into_any()
                    } else { ().into_any() }
                } else { ().into_any() }}
            </div>
        </div>
    }
}
