use crate::stores::use_app_store;
use crate::types::{ReportSortMode, TransactionType};
use leptos::prelude::*;
use uuid::Uuid;

/// Format a number as whole-dollar currency with thousands separators.
fn fmt_dollars(v: f64) -> String {
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReportTab {
    Sales,
    Purchases,
    Bills,
    Invoices,
    Notices,
    Documents,
    CamScan,
    Assets,
    Compliance,
    Transactions,
}

impl ReportTab {
    fn label(&self) -> &'static str {
        match self {
            ReportTab::Sales => "Sales",
            ReportTab::Purchases => "Purchases",
            ReportTab::Bills => "Bills",
            ReportTab::Invoices => "Invoices",
            ReportTab::Notices => "Notices",
            ReportTab::Documents => "Documents",
            ReportTab::CamScan => "CamScan",
            ReportTab::Assets => "Assets",
            ReportTab::Compliance => "Compliance",
            ReportTab::Transactions => "Transactions",
        }
    }
}

#[component]
pub fn ReportingPage() -> impl IntoView {
    let app_store = use_app_store();
    let (active_tab, set_active_tab) = signal(ReportTab::Sales);

    let seed_demo = move |_| {
        let user = app_store.get().current_user.id;
        app_store.update(|s| {
            let t = crate::models::Transaction::new(
                TransactionType::Sale,
                1_250_000.0,
                crate::types::Currency::AUD,
                crate::models::EntityReference {
                    entity_type: crate::models::EntityType::External,
                    entity_id: Uuid::new_v4(),
                    name: "Buyer".to_string(),
                },
                crate::models::EntityReference {
                    entity_type: crate::models::EntityType::Organization,
                    entity_id: Uuid::new_v4(),
                    name: "Carly".to_string(),
                },
                user,
            );
            s.transactions.push(t);
        });
    };

    let tabs = [
        ReportTab::Sales,
        ReportTab::Purchases,
        ReportTab::Bills,
        ReportTab::Invoices,
        ReportTab::Notices,
        ReportTab::Documents,
        ReportTab::CamScan,
        ReportTab::Assets,
        ReportTab::Compliance,
        ReportTab::Transactions,
    ];

    view! {
        <div class="reporting-page">
            <div class="reporting-header">
                <h2 class="reporting-title">"Reporting"</h2>
                <div class="reporting-subtitle">"Asset-linked reports: sales, purchases, bills, invoices, notices, documents, compliance, and transactions."</div>
            </div>

            <div class="reporting-actions">
                <button class="reporting-btn" on:click=seed_demo>"+ Seed Demo Sale"</button>
            </div>

            <div class="reporting-controls-bar">
                <select
                    class="reporting-sort-select"
                    prop:value={move || {
                        match app_store.get().reporting_sort_mode {
                            ReportSortMode::Recent => "sort_recent",
                            ReportSortMode::Oldest => "sort_oldest",
                            ReportSortMode::HighestValue => "sort_highest_value",
                            ReportSortMode::LowestValue => "sort_lowest_value",
                            ReportSortMode::ByStatus => "sort_by_status",
                            ReportSortMode::ByName => "sort_by_name",
                            ReportSortMode::ByDocumentType => "sort_by_doc_type",
                            ReportSortMode::ByCalendarDate => "sort_by_calendar",
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
                        app_store.update(|s| s.reporting_sort_mode = mode);
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
                <button
                    class="reporting-sort-direction"
                    title={move || if app_store.get().reporting_sort_ascending { "Ascending ↑" } else { "Descending ↓" }}
                    on:click=move |_| app_store.update(|s| s.toggle_reporting_sort_direction())
                >
                    {move || if app_store.get().reporting_sort_ascending { "↑" } else { "↓" }}
                </button>
            </div>

            <div class="reporting-tabs">
                {tabs.iter().map(|tab| {
                    let t = *tab;
                    view! {
                        <button
                            class="reporting-tab"
                            class:active={move || active_tab.get() == t}
                            on:click=move |_| set_active_tab.set(t)
                        >
                            {t.label()}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="reporting-body">
                {move || match active_tab.get() {
                    ReportTab::Sales => sales_view(&app_store).into_any(),
                    ReportTab::Purchases => purchases_view(&app_store).into_any(),
                    ReportTab::Bills => bills_view(&app_store).into_any(),
                    ReportTab::Invoices => invoices_view(&app_store).into_any(),
                    ReportTab::Notices => notices_view(&app_store).into_any(),
                    ReportTab::Documents => documents_view(&app_store).into_any(),
                    ReportTab::CamScan => camscan_view(&app_store).into_any(),
                    ReportTab::Assets => assets_view(&app_store).into_any(),
                    ReportTab::Compliance => compliance_view(&app_store).into_any(),
                    ReportTab::Transactions => transactions_view(&app_store).into_any(),
                }}
            </div>
        </div>
    }
}

fn find_asset_name(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>, id: Uuid) -> String {
    let store = app_store.get();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            if a.id == id { return a.name.clone(); }
        }
    }
    "—".to_string()
}

fn find_portfolio_name(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>, id: Uuid) -> String {
    let store = app_store.get();
    store.portfolios.iter().find(|p| p.id == id).map(|p| p.name.clone()).unwrap_or_else(|| "—".to_string())
}

fn table_head(cols: &[&'static str]) -> impl IntoView {
    let cols = cols.to_vec();
    view! {
        <div class="reporting-table-head">
            {cols.into_iter().map(|c| view! { <div class="reporting-th">{c}</div> }).collect::<Vec<_>>()}
        </div>
    }
}

fn sort_transactions(items: &mut [(crate::models::Transaction, String, String)], sort: &ReportSortMode) {
    use crate::types::ReportSortMode;
    items.sort_by(|a, b| {
        let (ta, _, _) = a;
        let (tb, _, _) = b;
        match sort {
            ReportSortMode::Recent => tb.created_at.cmp(&ta.created_at),
            ReportSortMode::Oldest => ta.created_at.cmp(&tb.created_at),
            ReportSortMode::HighestValue => tb.amount.partial_cmp(&ta.amount).unwrap_or(std::cmp::Ordering::Equal),
            ReportSortMode::LowestValue => ta.amount.partial_cmp(&tb.amount).unwrap_or(std::cmp::Ordering::Equal),
            ReportSortMode::ByStatus => format!("{:?}", ta.status).cmp(&format!("{:?}", tb.status)),
            ReportSortMode::ByName => ta.from_entity.name.to_lowercase().cmp(&tb.from_entity.name.to_lowercase()),
            ReportSortMode::ByCalendarDate => ta.created_at.cmp(&tb.created_at),
            ReportSortMode::ByDocumentType => std::cmp::Ordering::Equal,
        }
    });
}

fn sort_documents(items: &mut [(crate::models::Document, String, String)], sort: &ReportSortMode) {
    use crate::types::ReportSortMode;
    items.sort_by(|a, b| {
        let (da, _, _) = a;
        let (db, _, _) = b;
        match sort {
            ReportSortMode::Recent => db.uploaded_at.cmp(&da.uploaded_at),
            ReportSortMode::Oldest => da.uploaded_at.cmp(&db.uploaded_at),
            ReportSortMode::HighestValue => std::cmp::Ordering::Equal,
            ReportSortMode::LowestValue => std::cmp::Ordering::Equal,
            ReportSortMode::ByStatus => std::cmp::Ordering::Equal,
            ReportSortMode::ByName => da.name.to_lowercase().cmp(&db.name.to_lowercase()),
            ReportSortMode::ByDocumentType => da.file_type.to_lowercase().cmp(&db.file_type.to_lowercase()),
            ReportSortMode::ByCalendarDate => da.uploaded_at.cmp(&db.uploaded_at),
        }
    });
}

fn sort_assets(items: &mut [(crate::models::Asset, String)], sort: &ReportSortMode) {
    use crate::types::ReportSortMode;
    items.sort_by(|a, b| {
        let (aa, _) = a;
        let (ab, _) = b;
        match sort {
            ReportSortMode::Recent => ab.purchase_date.cmp(&aa.purchase_date),
            ReportSortMode::Oldest => aa.purchase_date.cmp(&ab.purchase_date),
            ReportSortMode::HighestValue => ab.current_value.partial_cmp(&aa.current_value).unwrap_or(std::cmp::Ordering::Equal),
            ReportSortMode::LowestValue => aa.current_value.partial_cmp(&ab.current_value).unwrap_or(std::cmp::Ordering::Equal),
            ReportSortMode::ByStatus => format!("{:?}", aa.status).cmp(&format!("{:?}", ab.status)),
            ReportSortMode::ByName => aa.name.to_lowercase().cmp(&ab.name.to_lowercase()),
            ReportSortMode::ByDocumentType => format!("{:?}", aa.asset_type).cmp(&format!("{:?}", ab.asset_type)),
            ReportSortMode::ByCalendarDate => ab.purchase_date.cmp(&aa.purchase_date),
        }
    });
}

fn sort_compliance(items: &mut [(String, String, String, String, usize)], sort: &ReportSortMode) {
    use crate::types::ReportSortMode;
    items.sort_by(|a, b| {
        let (asset_a, _, status_a, risk_a, docs_a) = a;
        let (asset_b, _, status_b, risk_b, docs_b) = b;
        match sort {
            ReportSortMode::Recent => docs_b.cmp(docs_a),
            ReportSortMode::Oldest => docs_a.cmp(docs_b),
            ReportSortMode::HighestValue => docs_b.cmp(docs_a),
            ReportSortMode::LowestValue => docs_a.cmp(docs_b),
            ReportSortMode::ByStatus => status_a.to_lowercase().cmp(&status_b.to_lowercase()),
            ReportSortMode::ByName => asset_a.to_lowercase().cmp(&asset_b.to_lowercase()),
            ReportSortMode::ByDocumentType => risk_a.to_lowercase().cmp(&risk_b.to_lowercase()),
            ReportSortMode::ByCalendarDate => std::cmp::Ordering::Equal,
        }
    });
}

fn sales_view(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>) -> impl IntoView {
    let store = app_store.get();
    let sort = store.effective_reporting_sort_mode();
    let mut items: Vec<_> = store.transactions.iter()
        .filter(|t| t.transaction_type == TransactionType::Sale)
        .map(|t| (t.clone(), find_asset_name(app_store, t.related_asset_id.unwrap_or_default()), find_portfolio_name(app_store, t.related_portfolio_id.unwrap_or_default())))
        .collect();
    sort_transactions(&mut items, &sort);
    let count = items.len();
    view! {
        <div class="reporting-section">
            <div class="reporting-section-title">"Sales"</div>
            <div class="reporting-section-meta">{format!("{} asset-linked sale records", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Asset", "Portfolio", "Amount", "Status", "Counterparty"])}
                {if items.is_empty() {
                    view! { <div class="reporting-empty">"No sales recorded."</div> }.into_any()
                } else {
                    view! {
                        {items.into_iter().map(|(t, asset, portfolio)| {
                            let date = t.created_at.format("%d %b %Y").to_string();
                            let status = format!("{:?}", t.status);
                            view! {
                                <div class="reporting-row">
                                    <div class="reporting-td">{date}</div>
                                    <div class="reporting-td">{asset}</div>
                                    <div class="reporting-td">{portfolio}</div>
                                    <div class="reporting-td">{fmt_dollars(t.amount)}</div>
                                    <div class="reporting-td">{status}</div>
                                    <div class="reporting-td">{t.from_entity.name}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    }.into_any()
                }}
            </div>
        </div>
    }
}

fn purchases_view(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>) -> impl IntoView {
    let store = app_store.get();
    let sort = store.effective_reporting_sort_mode();
    let mut items: Vec<_> = store.transactions.iter()
        .filter(|t| t.transaction_type == TransactionType::Purchase)
        .map(|t| (t.clone(), find_asset_name(app_store, t.related_asset_id.unwrap_or_default()), find_portfolio_name(app_store, t.related_portfolio_id.unwrap_or_default())))
        .collect();
    sort_transactions(&mut items, &sort);
    let count = items.len();
    view! {
        <div class="reporting-section">
            <div class="reporting-section-title">"Purchases"</div>
            <div class="reporting-section-meta">{format!("{} asset-linked purchase records", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Asset", "Portfolio", "Amount", "Status", "Seller"])}
                {if items.is_empty() {
                    view! { <div class="reporting-empty">"No purchases recorded."</div> }.into_any()
                } else {
                    view! {
                        {items.into_iter().map(|(t, asset, portfolio)| {
                            let date = t.created_at.format("%d %b %Y").to_string();
                            let status = format!("{:?}", t.status);
                            view! {
                                <div class="reporting-row">
                                    <div class="reporting-td">{date}</div>
                                    <div class="reporting-td">{asset}</div>
                                    <div class="reporting-td">{portfolio}</div>
                                    <div class="reporting-td">{fmt_dollars(t.amount)}</div>
                                    <div class="reporting-td">{status}</div>
                                    <div class="reporting-td">{t.from_entity.name}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    }.into_any()
                }}
            </div>
        </div>
    }
}

fn bills_view(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>) -> impl IntoView {
    let store = app_store.get();
    let sort = store.effective_reporting_sort_mode();
    let mut items: Vec<_> = Vec::new();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            for d in &a.documents {
                if d.name.to_lowercase().contains("bill") || d.file_type.to_lowercase().contains("bill") {
                    items.push((d.clone(), a.name.clone(), p.name.clone()));
                }
            }
        }
    }
    sort_documents(&mut items, &sort);
    let count = items.len();
    view! {
        <div class="reporting-section">
            <div class="reporting-section-title">"Bills"</div>
            <div class="reporting-section-meta">{format!("{} asset-linked bills", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Document", "Asset", "Portfolio", "Type"])}
                {if items.is_empty() {
                    view! { <div class="reporting-empty">"No bills found. Document names containing 'bill' will appear here."</div> }.into_any()
                } else {
                    view! {
                        {items.into_iter().map(|(d, asset, portfolio)| {
                            let date = d.uploaded_at.format("%d %b %Y").to_string();
                            view! {
                                <div class="reporting-row">
                                    <div class="reporting-td">{date}</div>
                                    <div class="reporting-td">{d.name}</div>
                                    <div class="reporting-td">{asset}</div>
                                    <div class="reporting-td">{portfolio}</div>
                                    <div class="reporting-td">{d.file_type.to_uppercase()}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    }.into_any()
                }}
            </div>
        </div>
    }
}

fn invoices_view(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>) -> impl IntoView {
    let store = app_store.get();
    let sort = store.effective_reporting_sort_mode();
    let mut items: Vec<_> = Vec::new();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            for d in &a.documents {
                if d.name.to_lowercase().contains("invoice") || d.name.to_lowercase().contains("receipt") {
                    items.push((d.clone(), a.name.clone(), p.name.clone()));
                }
            }
        }
    }
    sort_documents(&mut items, &sort);
    let count = items.len();
    view! {
        <div class="reporting-section">
            <div class="reporting-section-title">"Invoices"</div>
            <div class="reporting-section-meta">{format!("{} asset-linked invoices / receipts", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Document", "Asset", "Portfolio", "Type"])}
                {if items.is_empty() {
                    view! { <div class="reporting-empty">"No invoices found. Document names containing 'invoice' or 'receipt' will appear here."</div> }.into_any()
                } else {
                    view! {
                        {items.into_iter().map(|(d, asset, portfolio)| {
                            let date = d.uploaded_at.format("%d %b %Y").to_string();
                            view! {
                                <div class="reporting-row">
                                    <div class="reporting-td">{date}</div>
                                    <div class="reporting-td">{d.name}</div>
                                    <div class="reporting-td">{asset}</div>
                                    <div class="reporting-td">{portfolio}</div>
                                    <div class="reporting-td">{d.file_type.to_uppercase()}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    }.into_any()
                }}
            </div>
        </div>
    }
}

fn notices_view(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>) -> impl IntoView {
    let store = app_store.get();
    let sort = store.effective_reporting_sort_mode();
    let mut items: Vec<_> = Vec::new();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            for d in &a.documents {
                if d.name.to_lowercase().contains("notice") || d.name.to_lowercase().contains("delivery") || d.name.to_lowercase().contains("registration") {
                    items.push((d.clone(), a.name.clone(), p.name.clone()));
                }
            }
        }
    }
    sort_documents(&mut items, &sort);
    let count = items.len();
    view! {
        <div class="reporting-section">
            <div class="reporting-section-title">"Notices"</div>
            <div class="reporting-section-meta">{format!("{} asset-linked notices", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Document", "Asset", "Portfolio", "Type"])}
                {if items.is_empty() {
                    view! { <div class="reporting-empty">"No notices found. Document names containing 'notice', 'delivery', or 'registration' will appear here."</div> }.into_any()
                } else {
                    view! {
                        {items.into_iter().map(|(d, asset, portfolio)| {
                            let date = d.uploaded_at.format("%d %b %Y").to_string();
                            view! {
                                <div class="reporting-row">
                                    <div class="reporting-td">{date}</div>
                                    <div class="reporting-td">{d.name}</div>
                                    <div class="reporting-td">{asset}</div>
                                    <div class="reporting-td">{portfolio}</div>
                                    <div class="reporting-td">{d.file_type.to_uppercase()}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    }.into_any()
                }}
            </div>
        </div>
    }
}

fn documents_view(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>) -> impl IntoView {
    let store = app_store.get();
    let sort = store.effective_reporting_sort_mode();
    let mut items: Vec<_> = Vec::new();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            for d in &a.documents {
                items.push((d.clone(), a.name.clone(), p.name.clone()));
            }
        }
        for d in &p.documents {
            items.push((d.clone(), "Portfolio".to_string(), p.name.clone()));
        }
    }
    sort_documents(&mut items, &sort);
    let count = items.len();
    view! {
        <div class="reporting-section">
            <div class="reporting-section-title">"Documents"</div>
            <div class="reporting-section-meta">{format!("{} asset-linked documents", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Document", "Asset / Scope", "Portfolio", "Type"])}
                {if items.is_empty() {
                    view! { <div class="reporting-empty">"No documents found."</div> }.into_any()
                } else {
                    view! {
                        {items.into_iter().map(|(d, asset, portfolio)| {
                            let date = d.uploaded_at.format("%d %b %Y").to_string();
                            view! {
                                <div class="reporting-row">
                                    <div class="reporting-td">{date}</div>
                                    <div class="reporting-td">{d.name}</div>
                                    <div class="reporting-td">{asset}</div>
                                    <div class="reporting-td">{portfolio}</div>
                                    <div class="reporting-td">{d.file_type.to_uppercase()}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    }.into_any()
                }}
            </div>
        </div>
    }
}

fn assets_view(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>) -> impl IntoView {
    let store = app_store.get();
    let sort = store.effective_reporting_sort_mode();
    let mut items: Vec<_> = Vec::new();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            items.push((a.clone(), p.name.clone()));
        }
    }
    sort_assets(&mut items, &sort);
    let count = items.len();
    view! {
        <div class="reporting-section">
            <div class="reporting-section-title">"Assets"</div>
            <div class="reporting-section-meta">{format!("{} assets across portfolios", count)}</div>
            <div class="reporting-table">
                {table_head(&["Name", "Portfolio", "Type", "Status", "Current Value", "P&L %"])}
                {if items.is_empty() {
                    view! { <div class="reporting-empty">"No assets found."</div> }.into_any()
                } else {
                    view! {
                        {items.into_iter().map(|(a, portfolio)| {
                            let pl_cls = if a.profit_loss_percent >= 0.0 { "positive" } else { "negative" };
                            view! {
                                <div class="reporting-row">
                                    <div class="reporting-td">{a.name}</div>
                                    <div class="reporting-td">{portfolio}</div>
                                    <div class="reporting-td">{format!("{:?}", a.asset_type)}</div>
                                    <div class="reporting-td">{format!("{:?}", a.status)}</div>
                                    <div class="reporting-td">{fmt_dollars(a.current_value)}</div>
                                    <div class={format!("reporting-td {}", pl_cls)}>{format!("{:+.1}%", a.profit_loss_percent)}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    }.into_any()
                }}
            </div>
        </div>
    }
}

fn compliance_view(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>) -> impl IntoView {
    let store = app_store.get();
    let sort = store.effective_reporting_sort_mode();
    let mut items: Vec<_> = Vec::new();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            let docs = a.documents.len();
            let status = format!("{:?}", a.status);
            let risk = if docs == 0 { "Missing docs" } else if status == "UnderReview" { "Under review" } else { "OK" };
            items.push((a.name.clone(), p.name.clone(), status, risk.to_string(), docs));
        }
    }
    sort_compliance(&mut items, &sort);
    let count = items.len();
    view! {
        <div class="reporting-section">
            <div class="reporting-section-title">"Compliance"</div>
            <div class="reporting-section-meta">{format!("{} asset compliance checks", count)}</div>
            <div class="reporting-table">
                {table_head(&["Asset", "Portfolio", "Status", "Risk", "Docs"])}
                {if items.is_empty() {
                    view! { <div class="reporting-empty">"No assets to assess."</div> }.into_any()
                } else {
                    view! {
                        {items.into_iter().map(|(asset, portfolio, status, risk, docs)| {
                            view! {
                                <div class="reporting-row">
                                    <div class="reporting-td">{asset}</div>
                                    <div class="reporting-td">{portfolio}</div>
                                    <div class="reporting-td">{status}</div>
                                    <div class="reporting-td">{risk}</div>
                                    <div class="reporting-td">{docs}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    }.into_any()
                }}
            </div>
        </div>
    }
}

fn transactions_view(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>) -> impl IntoView {
    let store = app_store.get();
    let sort = store.effective_reporting_sort_mode();
    let mut items: Vec<_> = store.transactions.iter()
        .map(|t| (t.clone(), find_asset_name(app_store, t.related_asset_id.unwrap_or_default()), find_portfolio_name(app_store, t.related_portfolio_id.unwrap_or_default())))
        .collect();
    sort_transactions(&mut items, &sort);
    let count = items.len();
    view! {
        <div class="reporting-section">
            <div class="reporting-section-title">"Transactions"</div>
            <div class="reporting-section-meta">{format!("{} all-time transactions", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Type", "Asset", "Portfolio", "Amount", "Status"])}
                {if items.is_empty() {
                    view! { <div class="reporting-empty">"No transactions recorded."</div> }.into_any()
                } else {
                    view! {
                        {items.into_iter().map(|(t, asset, portfolio)| {
                            let date = t.created_at.format("%d %b %Y").to_string();
                            view! {
                                <div class="reporting-row">
                                    <div class="reporting-td">{date}</div>
                                    <div class="reporting-td">{format!("{:?}", t.transaction_type)}</div>
                                    <div class="reporting-td">{asset}</div>
                                    <div class="reporting-td">{portfolio}</div>
                                    <div class="reporting-td">{fmt_dollars(t.amount)}</div>
                                    <div class="reporting-td">{format!("{:?}", t.status)}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    }.into_any()
                }}
            </div>
        </div>
    }
}
