use crate::stores::use_ui_store;
use crate::types::ReportSortMode;
use leptos::prelude::*;

pub(crate) fn report_filters() -> impl IntoView {
    let ui_store = use_ui_store();
    view! {
        // General sort
        <select
            class="reporting-sort-select"
            prop:value={move || {
                match ui_store.get().reporting_sort_mode {
                    ReportSortMode::Recent => "sort_recent",
                    ReportSortMode::Oldest => "sort_oldest",
                    ReportSortMode::HighestValue => "sort_highest_value",
                    ReportSortMode::LowestValue => "sort_lowest_value",
                    ReportSortMode::ByStatus => "sort_by_status",
                    ReportSortMode::ByName => "sort_by_name",
                    ReportSortMode::ByDocumentType => "sort_by_doc_type",
                    ReportSortMode::ByCalendarDate => "sort_by_calendar",
                    _ => "sort_recent",
                }.to_string()
            }}
            on:change=move |ev| {
                let v = event_target_value(&ev);
                let mode = match v.as_str() {
                    "sort_oldest" => ReportSortMode::Oldest,
                    "sort_highest_value" => ReportSortMode::HighestValue,
                    "sort_lowest_value" => ReportSortMode::LowestValue,
                    "sort_by_status" => ReportSortMode::ByStatus,
                    "sort_by_name" => ReportSortMode::ByName,
                    "sort_by_doc_type" => ReportSortMode::ByDocumentType,
                    "sort_by_calendar" => ReportSortMode::ByCalendarDate,
                    _ => ReportSortMode::Recent,
                };
                ui_store.update(|s| s.reporting_sort_mode = mode);
            }
        >
            <option value="sort_recent">"Sort: Recent"</option>
            <option value="sort_oldest">"Sort: Oldest"</option>
            <option value="sort_highest_value">"Sort: Highest Value"</option>
            <option value="sort_lowest_value">"Sort: Lowest Value"</option>
            <option value="sort_by_status">"Sort: By Status"</option>
            <option value="sort_by_name">"Sort: By Name"</option>
            <option value="sort_by_doc_type">"Sort: By Document Type"</option>
            <option value="sort_by_calendar">"Sort: By Calendar Date"</option>
        </select>
        // Document category sort
        <select
            class="reporting-sort-select"
            prop:value={move || {
                match ui_store.get().reporting_sort_mode {
                    ReportSortMode::BySales => "cat_sales",
                    ReportSortMode::ByPurchases => "cat_purchases",
                    ReportSortMode::ByBills => "cat_bills",
                    ReportSortMode::ByInvoices => "cat_invoices",
                    ReportSortMode::ByNotices => "cat_notices",
                    ReportSortMode::ByStatements => "cat_statements",
                    ReportSortMode::BySummaries => "cat_summaries",
                    ReportSortMode::ByCompliance => "cat_compliance",
                    _ => "cat_none",
                }.to_string()
            }}
            on:change=move |ev| {
                let v = event_target_value(&ev);
                let mode = match v.as_str() {
                    "cat_sales" => ReportSortMode::BySales,
                    "cat_purchases" => ReportSortMode::ByPurchases,
                    "cat_bills" => ReportSortMode::ByBills,
                    "cat_invoices" => ReportSortMode::ByInvoices,
                    "cat_notices" => ReportSortMode::ByNotices,
                    "cat_statements" => ReportSortMode::ByStatements,
                    "cat_summaries" => ReportSortMode::BySummaries,
                    "cat_compliance" => ReportSortMode::ByCompliance,
                    _ => ReportSortMode::Recent,
                };
                ui_store.update(|s| s.reporting_sort_mode = mode);
            }
        >
            <option value="cat_none">"Category: All"</option>
            <option value="cat_sales">"Category: Sales"</option>
            <option value="cat_purchases">"Category: Purchases"</option>
            <option value="cat_bills">"Category: Bills"</option>
            <option value="cat_invoices">"Category: Invoices"</option>
            <option value="cat_notices">"Category: Notices"</option>
            <option value="cat_statements">"Category: Statements"</option>
            <option value="cat_summaries">"Category: Summaries"</option>
            <option value="cat_compliance">"Category: Compliance"</option>
        </select>
        // Parent entity sort
        <select
            class="reporting-sort-select"
            prop:value={move || {
                match ui_store.get().reporting_sort_mode {
                    ReportSortMode::ByOrganization => "parent_org",
                    ReportSortMode::ByPortfolio => "parent_portfolio",
                    ReportSortMode::ByAssetGroup => "parent_group",
                    ReportSortMode::ByDirectAsset => "parent_asset",
                    ReportSortMode::ByRole => "parent_role",
                    ReportSortMode::ByUser => "parent_user",
                    _ => "parent_none",
                }.to_string()
            }}
            on:change=move |ev| {
                let v = event_target_value(&ev);
                let mode = match v.as_str() {
                    "parent_org" => ReportSortMode::ByOrganization,
                    "parent_portfolio" => ReportSortMode::ByPortfolio,
                    "parent_group" => ReportSortMode::ByAssetGroup,
                    "parent_asset" => ReportSortMode::ByDirectAsset,
                    "parent_role" => ReportSortMode::ByRole,
                    "parent_user" => ReportSortMode::ByUser,
                    _ => ReportSortMode::Recent,
                };
                ui_store.update(|s| s.reporting_sort_mode = mode);
            }
        >
            <option value="parent_none">"Parent: None"</option>
            <option value="parent_org">"Parent: Organization"</option>
            <option value="parent_portfolio">"Parent: Portfolio"</option>
            <option value="parent_group">"Parent: Asset Group"</option>
            <option value="parent_asset">"Parent: Direct Asset"</option>
            <option value="parent_role">"Parent: Role"</option>
            <option value="parent_user">"Parent: User"</option>
        </select>
        <button
            class="reporting-sort-direction"
            title={move || if ui_store.get().reporting_sort_ascending { "Ascending ↑" } else { "Descending ↓" }}
            on:click=move |_| ui_store.update(|s| s.toggle_reporting_sort_direction())
        >
            {move || if ui_store.get().reporting_sort_ascending { "↑" } else { "↓" }}
        </button>
    }
}
