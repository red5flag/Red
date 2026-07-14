use leptos::prelude::*;

pub mod booking_report;
pub mod page;
pub mod report_filters;
pub mod report_list;
pub mod report_summary;

pub use page::ReportingPage;

pub(crate) fn table_head(cols: &[&'static str]) -> impl IntoView {
    let cols = cols.to_vec();
    view! {
        <div class="reporting-table-head">
            {cols.into_iter().map(|c| view! { <div class="reporting-th">{c}</div> }).collect::<Vec<_>>()}
        </div>
    }
}

/// Active tab in the reporting page.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReportTab {
    Sales,
    Purchases,
    Bills,
    Invoices,
    Notices,
    Documents,
    Statements,
    Summaries,
    #[allow(dead_code)]
    CamScan,
    Assets,
    Compliance,
    Transactions,
    ExportedRecords,
    Bookings,
}

impl ReportTab {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            ReportTab::Sales => "Sales",
            ReportTab::Purchases => "Purchases",
            ReportTab::Bills => "Bills",
            ReportTab::Invoices => "Invoices",
            ReportTab::Notices => "Notices",
            ReportTab::Documents => "Documents",
            ReportTab::Statements => "Statements",
            ReportTab::Summaries => "Summaries",
            ReportTab::CamScan => "CamScan",
            ReportTab::Assets => "Assets",
            ReportTab::Compliance => "Compliance",
            ReportTab::Transactions => "Transactions",
            ReportTab::ExportedRecords => "Exported Records",
            ReportTab::Bookings => "Bookings & Channels",
        }
    }
}

#[cfg(feature = "hydrate")]
pub(crate) fn scroll_tabs_node_ref(node_ref: &NodeRef<leptos::html::Div>, delta: i32) {
    use wasm_bindgen::JsCast;
    if let Some(el) = node_ref.get() {
        if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
            html_el.set_scroll_left(html_el.scroll_left() + delta);
        }
    }
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn scroll_tabs_node_ref(_node_ref: &NodeRef<leptos::html::Div>, _delta: i32) {}

#[cfg(feature = "hydrate")]
pub(crate) fn start_scroll_interval(
    node_ref: &NodeRef<leptos::html::Div>,
    delta: i32,
) -> Option<i32> {
    use wasm_bindgen::prelude::*;
    let node_ref = node_ref.clone();
    let closure = Closure::wrap(Box::new(move || {
        scroll_tabs_node_ref(&node_ref, delta);
    }) as Box<dyn FnMut()>);
    let id = web_sys::window()?
        .set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            50,
        )
        .ok()?;
    closure.forget();
    Some(id)
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn start_scroll_interval(
    _node_ref: &NodeRef<leptos::html::Div>,
    _delta: i32,
) -> Option<i32> {
    None
}

#[cfg(feature = "hydrate")]
pub(crate) fn stop_scroll_interval(id: Option<i32>) {
    if let Some(id) = id {
        if let Some(window) = web_sys::window() {
            window.clear_interval_with_handle(id);
        }
    }
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn stop_scroll_interval(_id: Option<i32>) {}

/// Format a number as whole-dollar currency with thousands separators.
pub(crate) fn fmt_dollars(v: f64) -> String {
    let s = format!("{:.0}", v);
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    format!("${}", out.chars().rev().collect::<String>())
}
