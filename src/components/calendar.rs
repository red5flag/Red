use crate::models::CalendarEvent;
use crate::stores::use_app_store;
use chrono::{Datelike, Duration, NaiveDate, Utc};
use leptos::prelude::*;
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

#[component]
pub fn CalendarManager(
    scope: CalendarScope,
    #[prop(default = false)] embedded: bool,
) -> impl IntoView {
    let app_store = use_app_store();

    let events_for_scope = Memo::new(move |_| {
        let store = app_store.get();
        let mut events: Vec<CalendarEvent> = if scope.portfolio_id.is_none()
            && scope.group_id.is_none()
            && scope.asset_id.is_none()
        {
            store.calendar_events.clone()
        } else {
            let mut local = Vec::new();
            if let Some(pid) = scope.portfolio_id {
                if let Some(p) = store.portfolios.iter().find(|p| p.id == pid) {
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
                for p in &store.portfolios {
                    if let Some(g) = p.asset_groups.iter().find(|g| g.id == gid) {
                        local.extend(g.calendar_events.clone());
                        for a in &g.assets {
                            local.extend(a.calendar_events.clone());
                        }
                        break;
                    }
                }
            } else if let Some(aid) = scope.asset_id {
                for p in &store.portfolios {
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
        events
    });

    let (expanded_month, set_expanded_month) = signal::<Option<(i32, u32)>>(None);
    let (editing_event, set_editing_event) = signal::<Option<CalendarEvent>>(None);
    let (show_form, set_show_form) = signal(false);

    let today = Utc::now().date_naive();
    let current_year = today.year();
    let current_month = today.month();

    let months = Memo::new(move |_| {
        let mut list = Vec::new();
        for offset in -2..=12i64 {
            let date = NaiveDate::from_ymd_opt(current_year, current_month, 1)
                .unwrap()
                .checked_add_signed(Duration::days(offset * 30))
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(current_year, current_month, 1).unwrap());
            list.push((date.year(), date.month(), date.format("%B %Y").to_string()));
        }
        list
    });

    let month_event_count = move |year: i32, month: u32| {
        events_for_scope
            .get()
            .iter()
            .filter(|e| e.start.year() == year && e.start.month() == month)
            .count()
    };

    let month_events = move |year: i32, month: u32| {
        events_for_scope
            .get()
            .into_iter()
            .filter(move |e| e.start.year() == year && e.start.month() == month)
            .collect::<Vec<_>>()
    };

    let on_add_new = move |_| {
        let mut ev = CalendarEvent::new("New event".to_string(), Utc::now(), Utc::now() + Duration::hours(1));
        ev.related_portfolio_id = scope.portfolio_id;
        ev.related_group_id = scope.group_id;
        ev.related_asset_id = scope.asset_id;
        ev.category = Some("Upcoming".to_string());
        set_editing_event.set(Some(ev));
        set_show_form.set(true);
    };

    let on_edit = Callback::new(move |ev: CalendarEvent| {
        set_editing_event.set(Some(ev));
        set_show_form.set(true);
    });

    let on_delete = Callback::new(move |id: Uuid| {
        app_store.update(|s| s.remove_calendar_event(id));
    });

    let on_save = Callback::new(move |ev: CalendarEvent| {
        app_store.update(|s| s.upsert_calendar_event(ev));
        set_show_form.set(false);
        set_editing_event.set(None);
    });

    let on_cancel = Callback::new(move |_| {
        set_show_form.set(false);
        set_editing_event.set(None);
    });

    let toggle_month = Callback::new(move |key: (i32, u32)| {
        let current = expanded_month.get();
        set_expanded_month.set(if current == Some(key) { None } else { Some(key) });
    });

    view! {
        <div class={if embedded { "calendar-manager embedded" } else { "calendar-manager" }}>
            <div class="calendar-manager-header">
                <div class="calendar-manager-title">{scope.title.clone()}</div>
                <div class="calendar-manager-actions">
                    <button class="calendar-manager-btn" on:click=on_add_new>"+"</button>
                </div>
            </div>

            {move || if show_form.get() {
                if let Some(ev) = editing_event.get() {
                    view! { <EventEditor event={ev} on_save={on_save.clone()} on_cancel={on_cancel.clone()} /> }.into_any()
                } else { ().into_any() }
            } else { ().into_any() }}

            <div class="calendar-manager-months">
                {move || {
                    let months_list = months.get();
                    months_list.into_iter().map(|(year, month, label)| {
                        let key = (year, month);
                        let is_expanded = expanded_month.get() == Some(key);
                        let count = month_event_count(year, month);
                        let toggle = toggle_month.clone();
                        let events = month_events(year, month);
                        view! {
                            <div class="calendar-month-row" class:expanded={is_expanded}>
                                <div class="calendar-month-header" on:click=move |_| toggle.run(key)>
                                    <span class="calendar-month-arrow">{if is_expanded { "▲" } else { "▼" }}</span>
                                    <span class="calendar-month-name">{label}</span>
                                    <span class="calendar-month-count">{"# "}{count.to_string()}</span>
                                    <button class="calendar-month-add" on:click=move |ev| {
                                        ev.stop_propagation();
                                        let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap().and_hms_opt(12, 0, 0).unwrap().and_utc();
                                        let mut new_ev = CalendarEvent::new("New event".to_string(), start, start + Duration::hours(1));
                                        new_ev.related_portfolio_id = scope.portfolio_id;
                                        new_ev.related_group_id = scope.group_id;
                                        new_ev.related_asset_id = scope.asset_id;
                                        new_ev.category = Some("Upcoming".to_string());
                                        set_editing_event.set(Some(new_ev));
                                        set_show_form.set(true);
                                    }>"+"</button>
                                </div>
                                {if is_expanded {
                                    view! {
                                        <div class="calendar-month-body">
                                            <MiniMonthGrid year={year} month={month} events={events.clone()} />
                                            <div class="calendar-month-events">
                                                <div class="calendar-month-section-title">"Books, Bills & Upcoming"</div>
                                                {if events.is_empty() {
                                                    view! { <div class="calendar-month-empty">"No events this month"</div> }.into_any()
                                                } else {
                                                    view! {
                                                        {events.into_iter().map(|ev| {
                                                            let on_edit = on_edit.clone();
                                                            let on_delete = on_delete.clone();
                                                            view! {
                                                                <CalendarEventRow event={ev} on_edit={on_edit} on_delete={on_delete} />
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    }.into_any()
                                                }}
                                            </div>
                                        </div>
                                    }.into_any()
                                } else { ().into_any() }}
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}

#[component]
fn MiniMonthGrid(year: i32, month: u32, events: Vec<CalendarEvent>) -> impl IntoView {
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let days_in_month = (NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap_or(NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()) - first_day).num_days() as u32;
    let start_weekday = first_day.weekday().num_days_from_sunday();

    let event_days: Vec<u32> = events.iter().map(|e| e.start.day()).collect();

    let mut days = Vec::new();
    for _ in 0..start_weekday {
        days.push(None);
    }
    for d in 1..=days_in_month {
        days.push(Some(d));
    }

    view! {
        <div class="calendar-mini-grid">
            <div class="calendar-mini-weekdays">
                <span>"S"</span><span>"M"</span><span>"T"</span><span>"W"</span><span>"T"</span><span>"F"</span><span>"S"</span>
            </div>
            <div class="calendar-mini-days">
                {days.into_iter().map(|d| {
                    let has_event = d.map(|day| event_days.contains(&day)).unwrap_or(false);
                    view! {
                        <div class="calendar-mini-day" class:has-event={has_event} class:empty={d.is_none()}>
                            {d.map(|n| n.to_string()).unwrap_or_default()}
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn CalendarEventRow(
    event: CalendarEvent,
    on_edit: Callback<CalendarEvent>,
    on_delete: Callback<Uuid>,
) -> impl IntoView {
    let ev = event.clone();
    let date = event.start.format("%d %b").to_string();
    let time = event.start.format("%H:%M").to_string();
    let cat = event.category.clone().unwrap_or_else(|| "General".to_string());
    let id = event.id;

    view! {
        <div class="calendar-event-row">
            <div class="calendar-event-row-main">
                <div class="calendar-event-row-date">{date}<span class="calendar-event-row-time">{time}</span></div>
                <div class="calendar-event-row-title">{event.title}</div>
                <div class="calendar-event-row-category">{cat}</div>
            </div>
            <div class="calendar-event-row-actions">
                <button class="calendar-event-row-btn" on:click=move |_| on_edit.run(ev.clone())>"✎"</button>
                <button class="calendar-event-row-btn danger" on:click=move |_| on_delete.run(id)>"✕"</button>
            </div>
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
    let (category, set_category) = signal(event.category.clone().unwrap_or_default());

    let ev = event.clone();
    let save = move |_| {
        let Ok(d) = NaiveDate::parse_from_str(&date.get(), "%Y-%m-%d") else { return; };
        let time_str = time.get();
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 { return; }
        let Ok(h) = parts[0].parse::<u32>() else { return; };
        let Ok(m) = parts[1].parse::<u32>() else { return; };
        let Some(start) = d.and_hms_opt(h, m, 0) else { return; };
        let mut updated = ev.clone();
        updated.title = title.get();
        updated.start = start.and_utc();
        updated.end = start.and_utc() + Duration::hours(1);
        updated.category = if category.get().trim().is_empty() { None } else { Some(category.get()) };
        on_save.run(updated);
    };

    view! {
        <div class="calendar-event-editor">
            <div class="calendar-event-editor-title">"Edit Event"</div>
            <input class="calendar-event-editor-input" type="text" prop:value={move || title.get()} on:input=move |e| set_title.set(event_target_value(&e)) placeholder="Title" />
            <div class="calendar-event-editor-row">
                <input class="calendar-event-editor-input" type="date" prop:value={move || date.get()} on:change=move |e| set_date.set(event_target_value(&e)) />
                <input class="calendar-event-editor-input" type="time" prop:value={move || time.get()} on:change=move |e| set_time.set(event_target_value(&e)) />
            </div>
            <input class="calendar-event-editor-input" type="text" prop:value={move || category.get()} on:input=move |e| set_category.set(event_target_value(&e)) placeholder="Category (e.g. Bills, Books, Upcoming)" />
            <div class="calendar-event-editor-actions">
                <button class="calendar-event-editor-save" on:click=save>"Save"</button>
                <button class="calendar-event-editor-cancel" on:click=move |_| on_cancel.run(())>"Cancel"</button>
            </div>
        </div>
    }
}
