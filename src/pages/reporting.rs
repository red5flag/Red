use crate::stores::use_app_store;
use crate::types::TransactionType;
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

fn sales_view(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>) -> impl IntoView {
    let store = app_store.get();
    let items: Vec<_> = store.transactions.iter()
        .filter(|t| t.transaction_type == TransactionType::Sale)
        .map(|t| (t.clone(), find_asset_name(app_store, t.related_asset_id.unwrap_or_default()), find_portfolio_name(app_store, t.related_portfolio_id.unwrap_or_default())))
        .collect();
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
    let items: Vec<_> = store.transactions.iter()
        .filter(|t| t.transaction_type == TransactionType::Purchase)
        .map(|t| (t.clone(), find_asset_name(app_store, t.related_asset_id.unwrap_or_default()), find_portfolio_name(app_store, t.related_portfolio_id.unwrap_or_default())))
        .collect();
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
    let mut items: Vec<_> = Vec::new();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            items.push((a.clone(), p.name.clone()));
        }
    }
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
    let mut items: Vec<_> = Vec::new();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            let docs = a.documents.len();
            let status = format!("{:?}", a.status);
            let risk = if docs == 0 { "Missing docs" } else if status == "UnderReview" { "Under review" } else { "OK" };
            items.push((a.name.clone(), p.name.clone(), status, risk.to_string(), docs));
        }
    }
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
    let items: Vec<_> = store.transactions.iter()
        .map(|t| (t.clone(), find_asset_name(app_store, t.related_asset_id.unwrap_or_default()), find_portfolio_name(app_store, t.related_portfolio_id.unwrap_or_default())))
        .collect();
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
