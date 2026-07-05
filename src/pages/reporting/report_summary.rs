use crate::pages::reporting::{fmt_dollars, table_head};
use crate::stores::{use_transaction_store, AppStore};
use leptos::prelude::*;

pub(crate) fn summaries_view(app_store: &RwSignal<AppStore>) -> impl IntoView {
    let store = app_store.get();
    let portfolios: Vec<_> = store
        .portfolios
        .iter()
        .map(|p| {
            let assets = p.get_all_assets();
            let total_value: f64 = assets.iter().map(|a| a.current_value).sum();
            let total_pl: f64 = assets.iter().map(|a| a.profit_loss).sum();
            let doc_count: usize = assets.iter().map(|a| a.documents.len()).sum();
            (
                p.name.clone(),
                assets.len(),
                doc_count,
                total_value,
                total_pl,
            )
        })
        .collect();
    let count = portfolios.len();
    let grand_total: f64 = portfolios.iter().map(|(_, _, _, v, _)| *v).sum();
    let grand_pl: f64 = portfolios.iter().map(|(_, _, _, _, pl)| *pl).sum();
    view! {
        <div class="reporting-section">
            <div class="reporting-section-meta">{format!("{} portfolios · Total: {} · P/L: {}", count, fmt_dollars(grand_total), fmt_dollars(grand_pl))}</div>
            <div class="reporting-table">
                {table_head(&["Portfolio", "Assets", "Documents", "Value", "P/L"])}
                {if portfolios.is_empty() {
                    view! { <div class="reporting-empty">"No portfolios to summarize."</div> }.into_any()
                } else {
                    view! {
                        {portfolios.into_iter().map(|(name, assets, docs, value, pl)| {
                            let pl_cls = if pl >= 0.0 { "positive" } else { "negative" };
                            view! {
                                <div class="reporting-row">
                                    <div class="reporting-td">{name}</div>
                                    <div class="reporting-td">{assets}</div>
                                    <div class="reporting-td">{docs}</div>
                                    <div class="reporting-td">{fmt_dollars(value)}</div>
                                    <div class={format!("reporting-td {}", pl_cls)}>{fmt_dollars(pl)}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    }.into_any()
                }}
            </div>
        </div>
    }
}

pub(crate) fn exported_records_view(app_store: &RwSignal<AppStore>) -> impl IntoView {
    let store = app_store.get();
    let transaction_store = use_transaction_store();
    let tx_count = transaction_store.get().transactions.len();
    let asset_count: usize = store
        .portfolios
        .iter()
        .map(|p| p.get_all_assets().len())
        .sum();
    let doc_count: usize = store
        .portfolios
        .iter()
        .flat_map(|p| p.get_all_assets())
        .map(|a| a.documents.len())
        .sum();
    view! {
        <div class="reporting-section">
            <div class="reporting-section-meta">"Download or export data from the system"</div>
            <div class="reporting-export-grid">
                <div class="reporting-export-card">
                    <div class="reporting-export-icon">"📊"</div>
                    <div class="reporting-export-title">"Transactions"</div>
                    <div class="reporting-export-count">{format!("{} records", tx_count)}</div>
                    <button class="reporting-export-btn">"Export CSV"</button>
                </div>
                <div class="reporting-export-card">
                    <div class="reporting-export-icon">"🏢"</div>
                    <div class="reporting-export-title">"Assets"</div>
                    <div class="reporting-export-count">{format!("{} assets", asset_count)}</div>
                    <button class="reporting-export-btn">"Export CSV"</button>
                </div>
                <div class="reporting-export-card">
                    <div class="reporting-export-icon">"📄"</div>
                    <div class="reporting-export-title">"Documents"</div>
                    <div class="reporting-export-count">{format!("{} documents", doc_count)}</div>
                    <button class="reporting-export-btn">"Export CSV"</button>
                </div>
                <div class="reporting-export-card">
                    <div class="reporting-export-icon">"📦"</div>
                    <div class="reporting-export-title">"Full Backup"</div>
                    <div class="reporting-export-count">"All data"</div>
                    <button class="reporting-export-btn">"Export JSON"</button>
                </div>
            </div>
        </div>
    }
}
