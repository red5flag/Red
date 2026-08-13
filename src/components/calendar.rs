use crate::models::{CalendarEvent, CalendarEventType};
use crate::pages::portfolios::{AddChannelModal, ChannelManagementWindow};
use crate::stores::{use_app_store, use_calendar_store};
use chrono::{Datelike, Duration, Months, NaiveDate, Utc};
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
#[allow(dead_code)]
enum CalendarViewMode {
    Month,
    MultiMonth,
    Year,
    Day,
}

#[allow(dead_code)]
impl CalendarViewMode {
    fn label(&self) -> &'static str {
        match self {
            CalendarViewMode::Month => "Month",
            CalendarViewMode::MultiMonth => "Multi-month",
            CalendarViewMode::Year => "Year",
            CalendarViewMode::Day => "Day",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CalendarScale {
    Minute,
    Hour,
    Date,
    Month,
    Year,
}

impl CalendarScale {
    fn all() -> [Self; 4] {
        [Self::Minute, Self::Hour, Self::Date, Self::Month]
    }

    fn label(&self) -> &'static str {
        match self {
            CalendarScale::Minute => "Minute",
            CalendarScale::Hour => "Hour",
            CalendarScale::Date => "Date",
            CalendarScale::Month => "Month",
            CalendarScale::Year => "Year",
        }
    }

    fn format_now(&self, now: chrono::DateTime<chrono::Utc>) -> String {
        match self {
            CalendarScale::Minute => now.format("%Y-%m-%d %H:%M").to_string(),
            CalendarScale::Hour => now.format("%Y-%m-%d %H:00").to_string(),
            CalendarScale::Date => now.format("%Y-%m-%d").to_string(),
            CalendarScale::Month => now.format("%B %Y").to_string(),
            CalendarScale::Year => now.format("%Y").to_string(),
        }
    }
}

fn scale_for_level(level: f64) -> CalendarScale {
    if level >= 2.0 {
        CalendarScale::Minute
    } else if level >= 1.0 {
        CalendarScale::Hour
    } else if level >= 0.0 {
        CalendarScale::Date
    } else if level >= -1.0 {
        CalendarScale::Month
    } else {
        CalendarScale::Year
    }
}

fn view_mode_for_scale(scale: CalendarScale) -> CalendarViewMode {
    match scale {
        CalendarScale::Minute | CalendarScale::Hour => CalendarViewMode::Day,
        CalendarScale::Date => CalendarViewMode::Month,
        CalendarScale::Month => CalendarViewMode::MultiMonth,
        CalendarScale::Year => CalendarViewMode::Year,
    }
}

fn base_level_for_scale(scale: CalendarScale) -> f64 {
    match scale {
        CalendarScale::Minute => 2.5,
        CalendarScale::Hour => 1.0,
        CalendarScale::Date => 0.5,
        CalendarScale::Month => -0.5,
        CalendarScale::Year => -1.5,
    }
}

fn day_zoom_for_level(level: f64) -> f64 {
    if level >= 2.0 {
        6.0 + (level - 2.0) * 6.0
    } else if level >= 1.0 {
        3.0 + (level - 1.0) * 3.0
    } else if level >= 0.0 {
        1.0 + level * 2.0
    } else {
        1.0
    }
}

fn month_zoom_for_level(level: f64) -> f64 {
    if level >= 1.0 {
        2.5 + (level - 1.0) * 1.5
    } else if level >= 0.0 {
        1.0 + level * 1.5
    } else if level >= -1.0 {
        1.0 + level * 0.5
    } else {
        0.5 + (level + 1.0) * 0.5
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
    let _toggle_type = Callback::new(move |t: CalendarEventType| {
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
    let (zoom, set_zoom) = signal(1.0_f64);
    let (day_zoom, set_day_zoom) = signal(1.0_f64);
    let multi_month_count = Memo::new(move |_| 2u32);
    let year_month_count = Memo::new(move |_| (12.0 / zoom.get()).clamp(12.0, 36.0).round() as u32);
    let (pinch_start, set_pinch_start) = signal(Option::<(f64, f64)>::None);
    let (zoom_level, set_zoom_level) = signal(-0.5_f64);
    let calendar_scale = Memo::new(move |_| scale_for_level(zoom_level.get()));
    let (scale_dropdown_open, set_scale_dropdown_open) = signal(false);
    let (now, _set_now) = signal(chrono::Utc::now());
    let (frozen_now, set_frozen_now) = signal(chrono::Utc::now());

    Effect::new(move |_| {
        let level = zoom_level.get();
        let scale = scale_for_level(level);
        set_calendar_view_mode.set(view_mode_for_scale(scale));
        set_day_zoom.set(day_zoom_for_level(level));
        set_zoom.set(month_zoom_for_level(level).clamp(0.5, 3.0));
    });

    #[cfg(feature = "hydrate")]
    {
        use futures::StreamExt;
        use gloo_timers::future::IntervalStream;
        leptos::task::spawn_local(async move {
            let mut interval = IntervalStream::new(1_000);
            while interval.next().await.is_some() {
                _set_now.set(chrono::Utc::now());
            }
        });
    }
    let (selected_date, set_selected_date) = signal::<Option<NaiveDate>>(None);
    let (editing_event, set_editing_event) = signal::<Option<CalendarEvent>>(None);
    let (show_form, set_show_form) = signal(false);

    // Date range signals for rangefinder view
    let (range_start, set_range_start) = signal::<Option<NaiveDate>>(None);
    let (range_end, set_range_end) = signal::<Option<NaiveDate>>(None);

    // Calendar search signal
    let (cal_search, set_cal_search) = signal(String::new());

    // Month/year dial dropdown state
    let (month_dial_open, set_month_dial_open) = signal(false);
    let (year_dial_open, set_year_dial_open) = signal(false);

    let step_months = move |mode: CalendarViewMode| -> u32 {
        match mode {
            CalendarViewMode::Month => 1,
            CalendarViewMode::MultiMonth => multi_month_count.get_untracked(),
            CalendarViewMode::Year => 12,
            CalendarViewMode::Day => 1,
        }
    };

    let prev_period = move |_| {
        if calendar_view_mode.get_untracked() == CalendarViewMode::Day {
            let current = selected_date.get().unwrap_or(today);
            if let Some(d) = current.checked_sub_signed(Duration::days(1)) {
                set_selected_date.set(Some(d));
                set_view_year.set(d.year());
                set_view_month.set(d.month());
            }
        } else {
            let base = NaiveDate::from_ymd_opt(view_year.get(), view_month.get(), 1).unwrap();
            let step = step_months(calendar_view_mode.get_untracked());
            if let Some(d) = base.checked_sub_months(Months::new(step)) {
                set_view_year.set(d.year());
                set_view_month.set(d.month());
            }
        }
    };
    let next_period = move |_| {
        if calendar_view_mode.get_untracked() == CalendarViewMode::Day {
            let current = selected_date.get().unwrap_or(today);
            if let Some(d) = current.checked_add_signed(Duration::days(1)) {
                set_selected_date.set(Some(d));
                set_view_year.set(d.year());
                set_view_month.set(d.month());
            }
        } else {
            let base = NaiveDate::from_ymd_opt(view_year.get(), view_month.get(), 1).unwrap();
            let step = step_months(calendar_view_mode.get_untracked());
            if let Some(d) = base.checked_add_months(Months::new(step)) {
                set_view_year.set(d.year());
                set_view_month.set(d.month());
            }
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
        // Range selection: first click sets start, second click sets end
        let rs = range_start.get();
        let re = range_end.get();
        if rs.is_none() || (rs.is_some() && re.is_some()) {
            set_range_start.set(Some(date));
            set_range_end.set(None);
        } else if let Some(start) = rs {
            if date < start {
                set_range_start.set(Some(date));
                set_range_end.set(Some(start));
            } else {
                set_range_end.set(Some(date));
            }
        }
        set_selected_date.set(Some(date));
        if calendar_view_mode.get_untracked() != CalendarViewMode::Day {
            set_zoom_level.set(1.0);
        }
    });

    let on_day_add = Callback::new(move |date: NaiveDate| {
        open_new_event(date);
    });

    // Events for the selected day
    let selected_day_events = Memo::new(move |_| {
        let date = selected_date.get();
        let events = events_for_scope.get();
        let search = cal_search.get();
        match date {
            Some(d) => events
                .into_iter()
                .filter(|e| e.start.date_naive() == d)
                .filter(|e| {
                    search.trim().is_empty()
                        || e.title.to_lowercase().contains(&search.to_lowercase())
                        || e.description
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&search.to_lowercase())
                })
                .collect::<Vec<_>>(),
            None => Vec::new(),
        }
    });

    // Search-filtered events for the current view
    let filtered_events = Memo::new(move |_| {
        let search = cal_search.get();
        if search.trim().is_empty() {
            return events_for_scope.get();
        }
        let search_lower = search.to_lowercase();
        events_for_scope
            .get()
            .into_iter()
            .filter(|e| {
                e.title.to_lowercase().contains(&search_lower)
                    || e.description
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&search_lower)
                    || e.category
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&search_lower)
            })
            .collect::<Vec<_>>()
    });

    let _period_label = Memo::new(move |_| match calendar_view_mode.get() {
        CalendarViewMode::Month => month_label(view_year.get(), view_month.get()),
        CalendarViewMode::Day => {
            let date = selected_date.get().unwrap_or(today);
            date.format("%A, %d %B %Y").to_string()
        }
        CalendarViewMode::Year => {
            let base = NaiveDate::from_ymd_opt(view_year.get(), view_month.get(), 1).unwrap();
            let count = year_month_count.get();
            let half = count / 2;
            let start = base.checked_sub_months(Months::new(half)).unwrap_or(base);
            let end = start
                .checked_add_months(Months::new(count.saturating_sub(1)))
                .unwrap_or(start);
            if start.year() == end.year() {
                format!(
                    "{} – {}",
                    month_label(start.year(), start.month()),
                    month_label(end.year(), end.month())
                )
            } else {
                format!(
                    "{} {} – {} {}",
                    start.month(),
                    start.year(),
                    end.month(),
                    end.year()
                )
            }
        }
        CalendarViewMode::MultiMonth => {
            let base = NaiveDate::from_ymd_opt(view_year.get(), view_month.get(), 1).unwrap();
            let count = multi_month_count.get();
            let half = count / 2;
            let start = base.checked_sub_months(Months::new(half)).unwrap_or(base);
            let end = start
                .checked_add_months(Months::new(count.saturating_sub(1)))
                .unwrap_or(start);
            if start.year() == end.year() {
                format!(
                    "{} – {}",
                    month_label(start.year(), start.month()),
                    month_label(end.year(), end.month())
                )
            } else {
                format!(
                    "{} {} – {} {}",
                    start.month(),
                    start.year(),
                    end.month(),
                    end.year()
                )
            }
        }
    });

    // Build the grid days for an arbitrary month
    fn month_days(year: i32, month: u32) -> Vec<Option<NaiveDate>> {
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let days_in_month = (NaiveDate::from_ymd_opt(year, month + 1, 1)
            .unwrap_or(NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
            - first_day)
            .num_days() as u32;
        let start_weekday = first_day.weekday().num_days_from_sunday();

        let mut days: Vec<Option<NaiveDate>> = Vec::new();
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
            days.push(Some(NaiveDate::from_ymd_opt(year, month, d).unwrap()));
        }
        while days.len() < 42 {
            let last = days.last().and_then(|d| *d).unwrap_or(first_day);
            if let Some(d) = last.checked_add_signed(Duration::days(1)) {
                days.push(Some(d));
            } else {
                days.push(None);
            }
        }
        days
    }

    // Render a single month grid (full or mini)
    fn month_grid(
        year: i32,
        month: u32,
        today: NaiveDate,
        events: Memo<Vec<CalendarEvent>>,
        selected_date: ReadSignal<Option<NaiveDate>>,
        zoom: ReadSignal<f64>,
        on_day_click: Callback<NaiveDate>,
        on_day_add: Callback<NaiveDate>,
        on_edit: Callback<CalendarEvent>,
        show_label: bool,
        mini: bool,
    ) -> impl IntoView {
        view! {
            <div class="calendar-month-wrapper" class:mini={mini}>
                {if show_label { view! { <div class="calendar-month-label">{month_label(year, month)}</div> }.into_any() } else { ().into_any() }}
                <div class="calendar-grid-wrapper">
                    <div class="calendar-grid-weekdays"
                        style={move || {
                            let z = zoom.get();
                            if mini {
                                "grid-template-columns: repeat(7, 1fr);".to_string()
                            } else {
                                let col_base = 90.0;
                                format!("grid-template-columns: repeat(7, {}px);", col_base * z)
                            }
                        }}
                    >
                        {vec!["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].into_iter().map(|d| {
                            view! { <div class="calendar-grid-weekday">{d}</div> }
                        }).collect::<Vec<_>>()}
                    </div>
                    <div
                        class="calendar-grid-days"
                        style={move || {
                            let z = zoom.get();
                            let base = if mini { 32.0 } else { 70.0 };
                            if mini {
                                format!("grid-auto-rows: 1fr; grid-template-columns: repeat(7, 1fr); aspect-ratio: 7 / 6; --calendar-zoom: {};", z)
                            } else {
                                let col_base = 90.0;
                                format!("grid-auto-rows: {}px; grid-template-columns: repeat(7, {}px); aspect-ratio: auto; --calendar-zoom: {};", base * z, col_base * z, z)
                            }
                        }}
                    >
                        {move || {
                            let days = month_days(year, month);
                            let events = events.get();
                            let sel = selected_date.get();
                            let on_day_click = on_day_click.clone();
                            let on_day_add = on_day_add.clone();
                            let on_edit = on_edit.clone();
                            let z = zoom.get();
                            let max_chips = if mini { (z * 2.0).max(0.0) as usize } else { (z * 4.0).max(1.0) as usize };
                            let show_day_content = !mini || z > 1.2;

                            days.into_iter().map(|opt_date| {
                                let on_dc = on_day_click.clone();
                                let on_da = on_day_add.clone();
                                let on_e = on_edit.clone();

                                match opt_date {
                                    Some(date) => {
                                        let is_today = date == today;
                                        let is_current_month = date.year() == year && date.month() == month;
                                        let is_selected = sel == Some(date);
                                        let day_events: Vec<CalendarEvent> = events
                                            .iter()
                                            .filter(|e| e.start.date_naive() == date)
                                            .cloned()
                                            .collect();
                                        let event_count = day_events.len();
                                        let has_event = event_count > 0;

                                        view! {
                                            <div class="calendar-grid-day"
                                                class:today={is_today}
                                                class:other-month={!is_current_month}
                                                class:selected={is_selected}
                                                class:mini={mini}
                                                class:has-event={has_event}
                                                on:click=move |_| on_dc.run(date)
                                            >
                                                <div class="calendar-grid-day-num">{date.day().to_string()}</div>
                                                {if show_day_content { view! {
                                                    <div class="calendar-grid-day-events">
                                                        {day_events.iter().take(max_chips).map(|ev| {
                                                            let ev_clone = ev.clone();
                                                            let on_e2 = on_e.clone();
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
                                                        {if event_count > max_chips { view! { <div class="calendar-grid-event-more">{format!("+{} more", event_count - max_chips)}</div> }.into_any() } else { ().into_any() }}
                                                    </div>
                                                    <button class="calendar-grid-day-add"
                                                        on:click=move |e: leptos::ev::MouseEvent| {
                                                            e.stop_propagation();
                                                            on_da.run(date);
                                                        }
                                                        title="Add event"
                                                    >"+"</button>
                                                }.into_any() } else { ().into_any() }}
                                            </div>
                                        }.into_any()
                                    }
                                    None => view! { <div class="calendar-grid-day empty"></div> }.into_any(),
                                }
                            }).collect::<Vec<_>>()
                        }}
                    </div>
                </div>
            </div>
        }
    }

    fn day_view(
        date: NaiveDate,
        events: Memo<Vec<CalendarEvent>>,
        day_zoom: ReadSignal<f64>,
        on_edit: Callback<CalendarEvent>,
        on_add: Callback<NaiveDate>,
    ) -> impl IntoView {
        let day_start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let day_end = day_start + Duration::days(1);
        let hours: Vec<u32> = (0..24).collect();

        view! {
            <div class="calendar-day-view">
                <div class="calendar-day-view-header">
                    <div class="calendar-day-view-date">{date.format("%A, %d %B %Y").to_string()}</div>
                    <button class="calendar-day-view-add" on:click=move |_| on_add.run(date)>"+ Add"</button>
                </div>
                {move || {
                    let z = day_zoom.get();
                    let all_day: Vec<CalendarEvent> = events
                        .get()
                        .iter()
                        .filter(|e| e.all_day && e.start.date_naive() <= date && e.end.date_naive() >= date)
                        .cloned()
                        .collect();
                    let mut timed: Vec<CalendarEvent> = events
                        .get()
                        .iter()
                        .filter(|e| !e.all_day && e.end > day_start && e.start < day_end)
                        .cloned()
                        .collect();
                    timed.sort_by(|a, b| a.start.cmp(&b.start));

                    view! {
                        <div class="calendar-day-view-scroll">
                            {if all_day.is_empty() { ().into_any() } else { view! {
                                <div class="calendar-day-view-allday">
                                    {all_day.iter().map(|ev| {
                                        let ev_clone = ev.clone();
                                        view! {
                                            <div
                                                class="calendar-day-view-allday-chip"
                                                on:click=move |e: leptos::ev::MouseEvent| {
                                                    e.stop_propagation();
                                                    on_edit.run(ev_clone.clone());
                                                }
                                            >
                                                {ev.title.clone()}
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any() }}
                            <div class="calendar-day-view-content" style={format!("height: {}px;", 1440.0 * z)}>
                                <div class="calendar-day-view-time-col">
                                    {hours.iter().map(|h| {
                                        view! {
                                            <div class="calendar-day-view-hour-row" style={format!("height: {}px;", 60.0 * z)}>
                                                <span class="calendar-day-view-hour-label">{format!("{:02}:00", h)}</span>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                <div class="calendar-day-view-events-col">
                                    {hours.iter().map(|_h| {
                                        view! {
                                            <div
                                                class="calendar-day-view-hour-row"
                                                style={format!("height: {}px; position: relative;", 60.0 * z)}
                                            >
                                                {(1..12).filter_map(|s| {
                                                    let top = (s * 5) as f64 * z;
                                                    let show_label = z >= 3.0;
                                                    let label = if show_label { Some(format!(":{:02}", s * 5)) } else { None };
                                                    Some(view! {
                                                        <div class="calendar-day-view-minute-marker" style={format!("top: {}px;", top)}>
                                                            {if let Some(l) = label { view! { <span class="calendar-day-view-minute-label">{l}</span> }.into_any() } else { ().into_any() }}
                                                        </div>
                                                    }.into_any())
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                    <div class="calendar-day-view-events-layer">
                                        {move || {
                                            let mut lanes: Vec<i64> = Vec::new();
                                            let mut positioned: Vec<(CalendarEvent, usize, i64, i64)> = Vec::new();
                                            for ev in &timed {
                                                let start_min = (ev.start - day_start).num_minutes().clamp(0, 1440);
                                                let end_min = (ev.end - day_start).num_minutes().clamp(0, 1440);
                                                let mut lane = 0usize;
                                                for (i, end) in lanes.iter().enumerate() {
                                                    if *end <= start_min {
                                                        lane = i;
                                                        break;
                                                    }
                                                    lane = i + 1;
                                                }
                                                if lane >= lanes.len() {
                                                    lanes.push(end_min);
                                                } else {
                                                    lanes[lane] = end_min;
                                                }
                                                positioned.push((ev.clone(), lane, start_min, end_min));
                                            }
                                            let lane_count = lanes.len().max(1);
                                            positioned.into_iter().map(|(ev, lane, start, end)| {
                                                let duration = (end - start).max(5);
                                                let top = start as f64 * z;
                                                let height = duration as f64 * z;
                                                let left = (lane as f64 / lane_count as f64) * 100.0;
                                                let width = (100.0 / lane_count as f64) - 2.0;
                                                let ev_for_click = ev.clone();
                                                view! {
                                                    <div
                                                        class="calendar-day-view-event"
                                                        style={format!("top: {}px; height: {}px; left: {}%; width: {}%;", top, height, left, width)}
                                                        on:click=move |e: leptos::ev::MouseEvent| {
                                                            e.stop_propagation();
                                                            on_edit.run(ev_for_click.clone());
                                                        }
                                                    >
                                                        <div class="calendar-day-view-event-time">{format!("{} – {}", ev.start.format("%H:%M"), ev.end.format("%H:%M"))}</div>
                                                        <div class="calendar-day-view-event-title">{ev.title.clone()}</div>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()
                                        }}
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                }}
            </div>
        }
    }

    view! {
        <div class={if embedded { "calendar-manager embedded" } else { "calendar-manager" }}>
            <div class="calendar-manager-header">
                <div class="calendar-dial-row">
                    <div class="calendar-dial-group">
                        <div class="calendar-dial-wrap">
                            <button
                                class="calendar-dial-btn"
                                class:calendar-dial-open={move || month_dial_open.get()}
                                on:click=move |_| {
                                    set_month_dial_open.update(|v| *v = !*v);
                                    set_year_dial_open.set(false);
                                }
                            >
                                {move || {
                                    let m = view_month.get();
                                    month_label(view_year.get(), m).split_whitespace().next().unwrap_or("").to_string()
                                }}
                                <span class="calendar-dial-caret">"▾"</span>
                            </button>
                            {move || if month_dial_open.get() { view! {
                                <div class="calendar-dial-dropdown" on:click=|ev| ev.stop_propagation()>
                                    {(1u32..=12u32).map(|m| {
                                        let is_current = move || view_month.get() == m;
                                        view! {
                                            <button
                                                class="calendar-dial-option"
                                                class:calendar-dial-active={is_current}
                                                on:click=move |_| {
                                                    set_view_month.set(m);
                                                    set_month_dial_open.set(false);
                                                }
                                            >
                                                {month_label(view_year.get(), m).split_whitespace().next().unwrap_or("").to_string()}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any() } else { ().into_any() }}
                        </div>
                        <div class="calendar-dial-wrap">
                            <button
                                class="calendar-dial-btn"
                                class:calendar-dial-open={move || year_dial_open.get()}
                                on:click=move |_| {
                                    set_year_dial_open.update(|v| *v = !*v);
                                    set_month_dial_open.set(false);
                                }
                            >
                                {move || view_year.get().to_string()}
                                <span class="calendar-dial-caret">"▾"</span>
                            </button>
                            {move || if year_dial_open.get() { view! {
                                <div class="calendar-dial-dropdown" on:click=|ev| ev.stop_propagation()>
                                    {((view_year.get() - 10)..=(view_year.get() + 10)).map(|y| {
                                        let is_current = move || view_year.get() == y;
                                        view! {
                                            <button
                                                class="calendar-dial-option"
                                                class:calendar-dial-active={is_current}
                                                on:click=move |_| {
                                                    set_view_year.set(y);
                                                    set_year_dial_open.set(false);
                                                }
                                            >
                                                {y.to_string()}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any() } else { ().into_any() }}
                        </div>
                    </div>
                    <button class="calendar-today-btn" on:click=go_today>"Today"</button>
                    <button class="calendar-nav-btn" on:click=prev_period title="Previous">"‹"</button>
                    <button class="calendar-nav-btn" on:click=next_period title="Next">"›"</button>
                    <div class="calendar-view-dropdown-wrap calendar-scale-dropdown-wrap">
                        <button
                            class="calendar-view-dropdown-toggle calendar-scale-dropdown-toggle"
                            aria-haspopup="menu"
                            aria-expanded={move || scale_dropdown_open.get()}
                            on:click=move |_| {
                                if !scale_dropdown_open.get_untracked() {
                                    set_frozen_now.set(now.get());
                                }
                                set_scale_dropdown_open.update(|v| *v = !*v);
                            }
                        >
                            {move || {
                                let display_now = if scale_dropdown_open.get() { frozen_now.get() } else { now.get() };
                                let scale = calendar_scale.get();
                                scale.format_now(display_now)
                            }}
                        </button>
                        {move || if scale_dropdown_open.get() { view! {
                            <div class="calendar-view-dropdown calendar-scale-dropdown" on:click=|ev| ev.stop_propagation()>
                                {CalendarScale::all()
                                    .into_iter()
                                    .map(|scale| {
                                        let scale_clone = scale;
                                        view! {
                                            <button
                                                class="calendar-view-dropdown-option calendar-scale-dropdown-option"
                                                class:calendar-scale-dropdown-active={move || calendar_scale.get() == scale_clone}
                                                on:click=move |_| {
                                                    set_zoom_level.set(base_level_for_scale(scale_clone));
                                                    set_scale_dropdown_open.set(false);
                                                }
                                            >
                                                {scale_clone.label()}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                            </div>
                        }.into_any() } else { ().into_any() }}
                    </div>
                    <button class="calendar-manager-btn" on:click=on_add_new title="Add event">"+"</button>
                </div>
            </div>
            <div class="calendar-controls-row">
                <div class="calendar-range-group">
                    {move || {
                        let rs = range_start.get();
                        let re = range_end.get();
                        let rs_str = rs.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
                        let re_str = re.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
                        view! {
                            <div class="calendar-range-inputs">
                                <input
                                    class="calendar-range-input"
                                    type="date"
                                    prop:value={rs_str.clone()}
                                    on:change=move |ev| {
                                        let v = event_target_value(&ev);
                                        if let Ok(d) = NaiveDate::parse_from_str(&v, "%Y-%m-%d") {
                                            set_range_start.set(Some(d));
                                        } else {
                                            set_range_start.set(None);
                                        }
                                    }
                                />
                                <input
                                    class="calendar-range-input"
                                    type="date"
                                    prop:value={re_str.clone()}
                                    on:change=move |ev| {
                                        let v = event_target_value(&ev);
                                        if let Ok(d) = NaiveDate::parse_from_str(&v, "%Y-%m-%d") {
                                            set_range_end.set(Some(d));
                                        } else {
                                            set_range_end.set(None);
                                        }
                                    }
                                />
                                {(rs.is_some() && re.is_some()).then(|| {
                                    let days = (re.unwrap() - rs.unwrap()).num_days() + 1;
                                    view! {
                                        <span class="calendar-range-info">{format!("{} days", days)}</span>
                                    }
                                })}
                            </div>
                        }.into_any()
                    }}
                </div>
            </div>
            <div class="calendar-search-wrap">
                <input
                    class="calendar-search-input"
                    type="text"
                    placeholder="Search events in current view..."
                    prop:value={move || cal_search.get()}
                    on:input=move |ev| set_cal_search.set(event_target_value(&ev))
                />
                {move || if !cal_search.get().is_empty() {
                    view! {
                        <button
                            class="calendar-search-clear"
                            on:click=move |_| set_cal_search.set(String::new())
                            title="Clear search"
                        >"✕"</button>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            {move || if show_form.get() {
                if let Some(ev) = editing_event.get() {
                    view! { <EventEditor event={ev} on_save={on_save.clone()} on_cancel={on_cancel.clone()} /> }.into_any()
                } else { ().into_any() }
            } else { ().into_any() }}

            <div class="calendar-grids"
                class:calendar-day-mode={move || calendar_view_mode.get() == CalendarViewMode::Day}
                on:wheel=move |ev: leptos::ev::WheelEvent| {
                    ev.prevent_default();
                    let delta = ev.delta_y() as f64;
                    set_zoom_level.update(|l| *l = (*l - delta * 0.004).clamp(-0.5, 3.0));
                }
                on:touchstart=move |ev: leptos::ev::TouchEvent| {
                    let touches = ev.touches();
                    if touches.length() == 2 {
                        if let (Some(t0), Some(t1)) = (touches.get(0), touches.get(1)) {
                            let dx = (t1.client_x() - t0.client_x()) as f64;
                            let dy = (t1.client_y() - t0.client_y()) as f64;
                            let dist = (dx * dx + dy * dy).sqrt();
                            let start_level = zoom_level.get();
                            set_pinch_start.set(Some((dist, start_level)));
                        }
                    }
                }
                on:touchmove=move |ev: leptos::ev::TouchEvent| {
                    let touches = ev.touches();
                    if touches.length() == 2 {
                        if let (Some(t0), Some(t1)) = (touches.get(0), touches.get(1)) {
                            let dx = (t1.client_x() - t0.client_x()) as f64;
                            let dy = (t1.client_y() - t0.client_y()) as f64;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if let Some((start_dist, start_level)) = pinch_start.get() {
                                ev.prevent_default();
                                let ratio = dist / start_dist;
                                let new_level = start_level + ratio.ln() * 0.8;
                                set_zoom_level.set(new_level.clamp(-0.5, 3.0));
                            }
                        }
                    }
                }
                on:touchend=move |_ev: leptos::ev::TouchEvent| {
                    set_pinch_start.set(None);
                }
                on:touchcancel=move |_ev: leptos::ev::TouchEvent| {
                    set_pinch_start.set(None);
                }
            >
                {move || match calendar_view_mode.get() {
                    CalendarViewMode::Month => view! {
                        {month_grid(
                            view_year.get(), view_month.get(), today,
                            filtered_events, selected_date, zoom,
                            on_day_click.clone(), on_day_add.clone(), on_edit.clone(),
                            false, false
                        )}
                    }.into_any(),
                    CalendarViewMode::Day => {
                        let date = selected_date.get().unwrap_or(today);
                        day_view(
                            date,
                            filtered_events,
                            day_zoom,
                            on_edit.clone(),
                            on_day_add.clone(),
                        ).into_any()
                    }
                    CalendarViewMode::MultiMonth => {
                        let base = NaiveDate::from_ymd_opt(view_year.get(), view_month.get(), 1).unwrap();
                        let count = multi_month_count.get();
                        let half = (count / 2) as i32;
                        let months: Vec<(i32, u32)> = (-half..(count as i32 - half))
                            .filter_map(|offset| {
                                let d = if offset < 0 {
                                    base.checked_sub_months(Months::new((-offset) as u32))
                                } else {
                                    base.checked_add_months(Months::new(offset as u32))
                                };
                                d.map(|d| (d.year(), d.month()))
                            })
                            .collect();
                        view! {
                            <div class="calendar-manager-grids calendar-manager-grids-multi">
                                {months.into_iter().map(|(y, m)| view! {
                                    {month_grid(
                                        y, m, today,
                                        filtered_events, selected_date, zoom,
                                        on_day_click.clone(), on_day_add.clone(), on_edit.clone(),
                                        true, true
                                    )}
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    },
                    CalendarViewMode::Year => {
                        let base = NaiveDate::from_ymd_opt(view_year.get(), view_month.get(), 1).unwrap();
                        let count = year_month_count.get();
                        let half = count / 2;
                        let start = base.checked_sub_months(Months::new(half)).unwrap_or(base);
                        let months: Vec<_> = (0..count)
                            .filter_map(|i| start.checked_add_months(Months::new(i)))
                            .map(|d| (d.year(), d.month()))
                            .collect();
                        view! {
                            <div class="calendar-manager-grids calendar-manager-grids-year">
                                {months.into_iter().map(|(y, m)| view! {
                                    {month_grid(
                                        y, m, today,
                                        filtered_events, selected_date, zoom,
                                        on_day_click.clone(), on_day_add.clone(), on_edit.clone(),
                                        true, true
                                    )}
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    },
                }}
            </div>

            // Selected day detail panel
            {move || if calendar_view_mode.get() != CalendarViewMode::Day {
                selected_date.get().map(|date| {
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
                }).unwrap_or_else(|| ().into_any())
            } else {
                ().into_any()
            }}
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
                    .flat_map(|p| {
                        p.assets
                            .iter()
                            .chain(p.asset_groups.iter().flat_map(|g| g.assets.iter()))
                    })
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
