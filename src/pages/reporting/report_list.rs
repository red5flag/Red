use crate::models::{Asset, ConditionStatus, Document, LifecycleStatus, Transaction};
use crate::pages::reporting::{fmt_dollars, table_head};
use crate::stores::{use_transaction_store, use_ui_store, AppStore};
use crate::types::{ReportSortMode, TransactionType};
use leptos::prelude::*;
use uuid::Uuid;

pub(crate) fn find_asset_name(app_store: &RwSignal<AppStore>, id: Uuid) -> String {
    let store = app_store.get();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            if a.id == id {
                return a.name.clone();
            }
        }
    }
    "—".to_string()
}

pub(crate) fn find_portfolio_name(app_store: &RwSignal<AppStore>, id: Uuid) -> String {
    let store = app_store.get();
    store
        .portfolios
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "—".to_string())
}

pub(crate) fn sort_transactions(
    items: &mut [(Transaction, String, String)],
    sort: &ReportSortMode,
) {
    items.sort_by(|a, b| {
        let (ta, asset_a, pf_a) = a;
        let (tb, asset_b, pf_b) = b;
        match sort {
            ReportSortMode::Recent => tb.created_at.cmp(&ta.created_at),
            ReportSortMode::Oldest => ta.created_at.cmp(&tb.created_at),
            ReportSortMode::HighestValue => tb
                .amount
                .partial_cmp(&ta.amount)
                .unwrap_or(std::cmp::Ordering::Equal),
            ReportSortMode::LowestValue => ta
                .amount
                .partial_cmp(&tb.amount)
                .unwrap_or(std::cmp::Ordering::Equal),
            ReportSortMode::ByStatus => format!("{:?}", ta.status).cmp(&format!("{:?}", tb.status)),
            ReportSortMode::ByName => ta
                .from_entity
                .name
                .to_lowercase()
                .cmp(&tb.from_entity.name.to_lowercase()),
            ReportSortMode::ByCalendarDate => ta.created_at.cmp(&tb.created_at),
            ReportSortMode::ByDocumentType => std::cmp::Ordering::Equal,
            ReportSortMode::BySales
            | ReportSortMode::ByPurchases
            | ReportSortMode::ByBills
            | ReportSortMode::ByInvoices
            | ReportSortMode::ByNotices
            | ReportSortMode::ByStatements
            | ReportSortMode::BySummaries
            | ReportSortMode::ByCompliance => std::cmp::Ordering::Equal,
            ReportSortMode::ByOrganization => std::cmp::Ordering::Equal,
            ReportSortMode::ByPortfolio => pf_a.to_lowercase().cmp(&pf_b.to_lowercase()),
            ReportSortMode::ByAssetGroup | ReportSortMode::ByDirectAsset => {
                asset_a.to_lowercase().cmp(&asset_b.to_lowercase())
            }
            ReportSortMode::ByRole | ReportSortMode::ByUser => std::cmp::Ordering::Equal,
        }
    });
}

pub(crate) fn sort_documents(items: &mut [(Document, String, String)], sort: &ReportSortMode) {
    items.sort_by(|a, b| {
        let (da, asset_a, pf_a) = a;
        let (db, asset_b, pf_b) = b;
        match sort {
            ReportSortMode::Recent => db.uploaded_at.cmp(&da.uploaded_at),
            ReportSortMode::Oldest => da.uploaded_at.cmp(&db.uploaded_at),
            ReportSortMode::HighestValue => std::cmp::Ordering::Equal,
            ReportSortMode::LowestValue => std::cmp::Ordering::Equal,
            ReportSortMode::ByStatus => std::cmp::Ordering::Equal,
            ReportSortMode::ByName => da.name.to_lowercase().cmp(&db.name.to_lowercase()),
            ReportSortMode::ByDocumentType => da
                .file_type
                .to_lowercase()
                .cmp(&db.file_type.to_lowercase()),
            ReportSortMode::ByCalendarDate => da.uploaded_at.cmp(&db.uploaded_at),
            // Document category sort: group by matching name keyword
            ReportSortMode::BySales => {
                let a_match = da.name.to_lowercase().contains("sale");
                let b_match = db.name.to_lowercase().contains("sale");
                b_match.cmp(&a_match)
            }
            ReportSortMode::ByPurchases => {
                let a_match = da.name.to_lowercase().contains("purchase");
                let b_match = db.name.to_lowercase().contains("purchase");
                b_match.cmp(&a_match)
            }
            ReportSortMode::ByBills => {
                let a_match = da.name.to_lowercase().contains("bill");
                let b_match = db.name.to_lowercase().contains("bill");
                b_match.cmp(&a_match)
            }
            ReportSortMode::ByInvoices => {
                let a_match = da.name.to_lowercase().contains("invoice")
                    || da.name.to_lowercase().contains("receipt");
                let b_match = db.name.to_lowercase().contains("invoice")
                    || db.name.to_lowercase().contains("receipt");
                b_match.cmp(&a_match)
            }
            ReportSortMode::ByNotices => {
                let a_match = da.name.to_lowercase().contains("notice")
                    || da.name.to_lowercase().contains("delivery")
                    || da.name.to_lowercase().contains("registration");
                let b_match = db.name.to_lowercase().contains("notice")
                    || db.name.to_lowercase().contains("delivery")
                    || db.name.to_lowercase().contains("registration");
                b_match.cmp(&a_match)
            }
            ReportSortMode::ByStatements => {
                let a_match = da.name.to_lowercase().contains("statement");
                let b_match = db.name.to_lowercase().contains("statement");
                b_match.cmp(&a_match)
            }
            ReportSortMode::BySummaries => {
                let a_match = da.name.to_lowercase().contains("summary")
                    || da.name.to_lowercase().contains("report");
                let b_match = db.name.to_lowercase().contains("summary")
                    || db.name.to_lowercase().contains("report");
                b_match.cmp(&a_match)
            }
            ReportSortMode::ByCompliance => {
                let a_match = da.name.to_lowercase().contains("compliance")
                    || da.name.to_lowercase().contains("audit");
                let b_match = db.name.to_lowercase().contains("compliance")
                    || db.name.to_lowercase().contains("audit");
                b_match.cmp(&a_match)
            }
            // Parent entity sort
            ReportSortMode::ByOrganization => std::cmp::Ordering::Equal,
            ReportSortMode::ByPortfolio => pf_a.to_lowercase().cmp(&pf_b.to_lowercase()),
            ReportSortMode::ByAssetGroup | ReportSortMode::ByDirectAsset => {
                asset_a.to_lowercase().cmp(&asset_b.to_lowercase())
            }
            ReportSortMode::ByRole | ReportSortMode::ByUser => std::cmp::Ordering::Equal,
        }
    });
}

pub(crate) fn sort_assets(items: &mut [(Asset, String)], sort: &ReportSortMode) {
    items.sort_by(|a, b| {
        let (aa, pf_a) = a;
        let (ab, pf_b) = b;
        match sort {
            ReportSortMode::Recent => ab.purchase_date.cmp(&aa.purchase_date),
            ReportSortMode::Oldest => aa.purchase_date.cmp(&ab.purchase_date),
            ReportSortMode::HighestValue => ab
                .current_value
                .partial_cmp(&aa.current_value)
                .unwrap_or(std::cmp::Ordering::Equal),
            ReportSortMode::LowestValue => aa
                .current_value
                .partial_cmp(&ab.current_value)
                .unwrap_or(std::cmp::Ordering::Equal),
            ReportSortMode::ByStatus => {
                let status_a = format!(
                    "{} {} {} {}",
                    aa.lifecycle_status.as_str(),
                    aa.availability_status.as_str(),
                    aa.condition_status.as_str(),
                    aa.commercial_status.as_str()
                );
                let status_b = format!(
                    "{} {} {} {}",
                    ab.lifecycle_status.as_str(),
                    ab.availability_status.as_str(),
                    ab.condition_status.as_str(),
                    ab.commercial_status.as_str()
                );
                status_a.cmp(&status_b)
            }
            ReportSortMode::ByName => aa.name.to_lowercase().cmp(&ab.name.to_lowercase()),
            ReportSortMode::ByDocumentType => {
                format!("{:?}", aa.asset_type).cmp(&format!("{:?}", ab.asset_type))
            }
            ReportSortMode::ByCalendarDate => ab.purchase_date.cmp(&aa.purchase_date),
            ReportSortMode::BySales
            | ReportSortMode::ByPurchases
            | ReportSortMode::ByBills
            | ReportSortMode::ByInvoices
            | ReportSortMode::ByNotices
            | ReportSortMode::ByStatements
            | ReportSortMode::BySummaries
            | ReportSortMode::ByCompliance => std::cmp::Ordering::Equal,
            ReportSortMode::ByOrganization => std::cmp::Ordering::Equal,
            ReportSortMode::ByPortfolio => pf_a.to_lowercase().cmp(&pf_b.to_lowercase()),
            ReportSortMode::ByAssetGroup | ReportSortMode::ByDirectAsset => {
                aa.name.to_lowercase().cmp(&ab.name.to_lowercase())
            }
            ReportSortMode::ByRole | ReportSortMode::ByUser => std::cmp::Ordering::Equal,
        }
    });
}

pub(crate) fn sort_compliance(
    items: &mut [(String, String, String, String, usize)],
    sort: &ReportSortMode,
) {
    items.sort_by(|a, b| {
        let (asset_a, pf_a, status_a, risk_a, docs_a) = a;
        let (asset_b, pf_b, status_b, risk_b, docs_b) = b;
        match sort {
            ReportSortMode::Recent => docs_b.cmp(docs_a),
            ReportSortMode::Oldest => docs_a.cmp(docs_b),
            ReportSortMode::HighestValue => docs_b.cmp(docs_a),
            ReportSortMode::LowestValue => docs_a.cmp(docs_b),
            ReportSortMode::ByStatus => status_a.to_lowercase().cmp(&status_b.to_lowercase()),
            ReportSortMode::ByName => asset_a.to_lowercase().cmp(&asset_b.to_lowercase()),
            ReportSortMode::ByDocumentType => risk_a.to_lowercase().cmp(&risk_b.to_lowercase()),
            ReportSortMode::ByCalendarDate => std::cmp::Ordering::Equal,
            ReportSortMode::BySales
            | ReportSortMode::ByPurchases
            | ReportSortMode::ByBills
            | ReportSortMode::ByInvoices
            | ReportSortMode::ByNotices
            | ReportSortMode::ByStatements
            | ReportSortMode::BySummaries
            | ReportSortMode::ByCompliance => std::cmp::Ordering::Equal,
            ReportSortMode::ByOrganization => std::cmp::Ordering::Equal,
            ReportSortMode::ByPortfolio => pf_a.to_lowercase().cmp(&pf_b.to_lowercase()),
            ReportSortMode::ByAssetGroup | ReportSortMode::ByDirectAsset => {
                asset_a.to_lowercase().cmp(&asset_b.to_lowercase())
            }
            ReportSortMode::ByRole | ReportSortMode::ByUser => std::cmp::Ordering::Equal,
        }
    });
}

pub(crate) fn sales_view(app_store: &RwSignal<AppStore>) -> impl IntoView {
    let _store = app_store.get();
    let transaction_store = use_transaction_store();
    let ui_store = use_ui_store();
    let sort = ui_store.get().effective_reporting_sort_mode();
    let mut items: Vec<_> = transaction_store
        .get()
        .transactions
        .iter()
        .filter(|t| t.transaction_type == TransactionType::Sale)
        .map(|t| {
            (
                t.clone(),
                find_asset_name(app_store, t.related_asset_id.unwrap_or_default()),
                find_portfolio_name(app_store, t.related_portfolio_id.unwrap_or_default()),
            )
        })
        .collect();
    sort_transactions(&mut items, &sort);
    let count = items.len();
    let items_for = items.clone();
    let items_memo = Memo::new(move |_| items_for.clone());
    view! {
        <div class="reporting-section">
            <div class="reporting-section-meta">{format!("{} asset-linked sale records", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Asset", "Portfolio", "Amount", "Status", "Counterparty"])}
                {if items_memo.get().is_empty() {
                    view! { <div class="reporting-empty">"No sales recorded."</div> }.into_any()
                } else {
                    view! {
                        <For
                            each=move || items_memo.get()
                            key=|(t, _, _)| t.id
                            children=move |(t, asset, portfolio)| {
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
                            }
                        />
                    }.into_any()
                }}
            </div>
        </div>
    }
}

pub(crate) fn purchases_view(app_store: &RwSignal<AppStore>) -> impl IntoView {
    let _store = app_store.get();
    let transaction_store = use_transaction_store();
    let ui_store = use_ui_store();
    let sort = ui_store.get().effective_reporting_sort_mode();
    let mut items: Vec<_> = transaction_store
        .get()
        .transactions
        .iter()
        .filter(|t| t.transaction_type == TransactionType::Purchase)
        .map(|t| {
            (
                t.clone(),
                find_asset_name(app_store, t.related_asset_id.unwrap_or_default()),
                find_portfolio_name(app_store, t.related_portfolio_id.unwrap_or_default()),
            )
        })
        .collect();
    sort_transactions(&mut items, &sort);
    let count = items.len();
    let items_for = items.clone();
    let items_memo = Memo::new(move |_| items_for.clone());
    view! {
        <div class="reporting-section">
            <div class="reporting-section-meta">{format!("{} asset-linked purchase records", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Asset", "Portfolio", "Amount", "Status", "Seller"])}
                {if items_memo.get().is_empty() {
                    view! { <div class="reporting-empty">"No purchases recorded."</div> }.into_any()
                } else {
                    view! {
                        <For
                            each=move || items_memo.get()
                            key=|(t, _, _)| t.id
                            children=move |(t, asset, portfolio)| {
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
                            }
                        />
                    }.into_any()
                }}
            </div>
        </div>
    }
}

pub(crate) fn bills_view(app_store: &RwSignal<AppStore>) -> impl IntoView {
    let store = app_store.get();
    let ui_store = use_ui_store();
    let sort = ui_store.get().effective_reporting_sort_mode();
    let mut items: Vec<_> = Vec::new();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            for d in &a.documents {
                if d.name.to_lowercase().contains("bill")
                    || d.file_type.to_lowercase().contains("bill")
                {
                    items.push((d.clone(), a.name.clone(), p.name.clone()));
                }
            }
        }
    }
    sort_documents(&mut items, &sort);
    let count = items.len();
    let items_for = items.clone();
    let items_memo = Memo::new(move |_| items_for.clone());
    view! {
        <div class="reporting-section">
            <div class="reporting-section-meta">{format!("{} asset-linked bills", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Document", "Asset", "Portfolio", "Type"])}
                {if items_memo.get().is_empty() {
                    view! { <div class="reporting-empty">"No bills found. Document names containing 'bill' will appear here."</div> }.into_any()
                } else {
                    view! {
                        <For
                            each=move || items_memo.get()
                            key=|(d, _, _)| d.id
                            children=move |(d, asset, portfolio)| {
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
                            }
                        />
                    }.into_any()
                }}
            </div>
        </div>
    }
}

pub(crate) fn invoices_view(app_store: &RwSignal<AppStore>) -> impl IntoView {
    let store = app_store.get();
    let ui_store = use_ui_store();
    let sort = ui_store.get().effective_reporting_sort_mode();
    let mut items: Vec<_> = Vec::new();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            for d in &a.documents {
                if d.name.to_lowercase().contains("invoice")
                    || d.name.to_lowercase().contains("receipt")
                {
                    items.push((d.clone(), a.name.clone(), p.name.clone()));
                }
            }
        }
    }
    sort_documents(&mut items, &sort);
    let count = items.len();
    let items_for = items.clone();
    let items_memo = Memo::new(move |_| items_for.clone());
    view! {
        <div class="reporting-section">
            <div class="reporting-section-meta">{format!("{} asset-linked invoices / receipts", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Document", "Asset", "Portfolio", "Type"])}
                {if items_memo.get().is_empty() {
                    view! { <div class="reporting-empty">"No invoices found. Document names containing 'invoice' or 'receipt' will appear here."</div> }.into_any()
                } else {
                    view! {
                        <For
                            each=move || items_memo.get()
                            key=|(d, _, _)| d.id
                            children=move |(d, asset, portfolio)| {
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
                            }
                        />
                    }.into_any()
                }}
            </div>
        </div>
    }
}

pub(crate) fn notices_view(app_store: &RwSignal<AppStore>) -> impl IntoView {
    let store = app_store.get();
    let ui_store = use_ui_store();
    let sort = ui_store.get().effective_reporting_sort_mode();
    let mut items: Vec<_> = Vec::new();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            for d in &a.documents {
                if d.name.to_lowercase().contains("notice")
                    || d.name.to_lowercase().contains("delivery")
                    || d.name.to_lowercase().contains("registration")
                {
                    items.push((d.clone(), a.name.clone(), p.name.clone()));
                }
            }
        }
    }
    sort_documents(&mut items, &sort);
    let count = items.len();
    let items_for = items.clone();
    let items_memo = Memo::new(move |_| items_for.clone());
    view! {
        <div class="reporting-section">
            <div class="reporting-section-meta">{format!("{} asset-linked notices", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Document", "Asset", "Portfolio", "Type"])}
                {if items_memo.get().is_empty() {
                    view! { <div class="reporting-empty">"No notices found. Document names containing 'notice', 'delivery', or 'registration' will appear here."</div> }.into_any()
                } else {
                    view! {
                        <For
                            each=move || items_memo.get()
                            key=|(d, _, _)| d.id
                            children=move |(d, asset, portfolio)| {
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
                            }
                        />
                    }.into_any()
                }}
            </div>
        </div>
    }
}

pub(crate) fn documents_view(app_store: &RwSignal<AppStore>) -> impl IntoView {
    let store = app_store.get();
    let ui_store = use_ui_store();
    let sort = ui_store.get().effective_reporting_sort_mode();
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
    let items_for = items.clone();
    let items_memo = Memo::new(move |_| items_for.clone());
    view! {
        <div class="reporting-section">
            <div class="reporting-section-meta">{format!("{} asset-linked documents", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Document", "Asset / Scope", "Portfolio", "Type"])}
                {if items_memo.get().is_empty() {
                    view! { <div class="reporting-empty">"No documents found."</div> }.into_any()
                } else {
                    view! {
                        <For
                            each=move || items_memo.get()
                            key=|(d, _, _)| d.id
                            children=move |(d, asset, portfolio)| {
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
                            }
                        />
                    }.into_any()
                }}
            </div>
        </div>
    }
}

pub(crate) fn assets_view(app_store: &RwSignal<AppStore>) -> impl IntoView {
    let store = app_store.get();
    let ui_store = use_ui_store();
    let sort = ui_store.get().effective_reporting_sort_mode();
    let mut items: Vec<_> = Vec::new();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            items.push((a.clone(), p.name.clone()));
        }
    }
    sort_assets(&mut items, &sort);
    let count = items.len();
    let items_for = items.clone();
    let items_memo = Memo::new(move |_| items_for.clone());
    view! {
        <div class="reporting-section">
            <div class="reporting-section-meta">{format!("{} assets across portfolios", count)}</div>
            <div class="reporting-table">
                {table_head(&["Name", "Portfolio", "Type", "Status", "Current Value", "P&L %"])}
                {if items_memo.get().is_empty() {
                    view! { <div class="reporting-empty">"No assets found."</div> }.into_any()
                } else {
                    view! {
                        <For
                            each=move || items_memo.get()
                            key=|(a, _)| a.id
                            children=move |(a, portfolio)| {
                                let pl_cls = if a.profit_loss_percent >= 0.0 { "positive" } else { "negative" };
                                view! {
                                    <div class="reporting-row">
                                        <div class="reporting-td">{a.name}</div>
                                        <div class="reporting-td">{portfolio}</div>
                                        <div class="reporting-td">{format!("{:?}", a.asset_type)}</div>
                                        <div class="reporting-td">{format!("{} | {} | {} | {}", a.lifecycle_status.as_str(), a.availability_status.as_str(), a.condition_status.as_str(), a.commercial_status.as_str())}</div>
                                        <div class="reporting-td">{fmt_dollars(a.current_value)}</div>
                                        <div class={format!("reporting-td {}", pl_cls)}>{format!("{:+.1}%", a.profit_loss_percent)}</div>
                                    </div>
                                }
                            }
                        />
                    }.into_any()
                }}
            </div>
        </div>
    }
}

pub(crate) fn compliance_view(app_store: &RwSignal<AppStore>) -> impl IntoView {
    let store = app_store.get();
    let ui_store = use_ui_store();
    let sort = ui_store.get().effective_reporting_sort_mode();
    let mut items: Vec<_> = Vec::new();
    for p in &store.portfolios {
        for a in p.get_all_assets() {
            let docs = a.documents.len();
            let status = format!(
                "{} / {} / {} / {}",
                a.lifecycle_status.as_str(),
                a.availability_status.as_str(),
                a.condition_status.as_str(),
                a.commercial_status.as_str()
            );
            let risk = if docs == 0 {
                "Missing docs"
            } else if matches!(a.lifecycle_status, LifecycleStatus::Disposed)
                || matches!(a.condition_status, ConditionStatus::Unsafe)
            {
                "Under review"
            } else {
                "OK"
            };
            items.push((
                a.name.clone(),
                p.name.clone(),
                status,
                risk.to_string(),
                docs,
            ));
        }
    }
    sort_compliance(&mut items, &sort);
    let count = items.len();
    view! {
        <div class="reporting-section">
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

pub(crate) fn transactions_view(app_store: &RwSignal<AppStore>) -> impl IntoView {
    let _store = app_store.get();
    let transaction_store = use_transaction_store();
    let ui_store = use_ui_store();
    let sort = ui_store.get().effective_reporting_sort_mode();
    let mut items: Vec<_> = transaction_store
        .get()
        .transactions
        .iter()
        .map(|t| {
            (
                t.clone(),
                find_asset_name(app_store, t.related_asset_id.unwrap_or_default()),
                find_portfolio_name(app_store, t.related_portfolio_id.unwrap_or_default()),
            )
        })
        .collect();
    sort_transactions(&mut items, &sort);
    let count = items.len();
    let items_for = items.clone();
    let items_memo = Memo::new(move |_| items_for.clone());
    view! {
        <div class="reporting-section">
            <div class="reporting-section-meta">{format!("{} all-time transactions", count)}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Type", "Asset", "Portfolio", "Amount", "Status"])}
                {if items_memo.get().is_empty() {
                    view! { <div class="reporting-empty">"No transactions recorded."</div> }.into_any()
                } else {
                    view! {
                        <For
                            each=move || items_memo.get()
                            key=|(t, _, _)| t.id
                            children=move |(t, asset, portfolio)| {
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
                            }
                        />
                    }.into_any()
                }}
            </div>
        </div>
    }
}

pub(crate) fn statements_view(app_store: &RwSignal<AppStore>) -> impl IntoView {
    let _store = app_store.get();
    let transaction_store = use_transaction_store();
    let ui_store = use_ui_store();
    let sort = ui_store.get().effective_reporting_sort_mode();
    let mut items: Vec<_> = transaction_store
        .get()
        .transactions
        .iter()
        .filter(|t| {
            matches!(
                t.transaction_type,
                TransactionType::Sale | TransactionType::Purchase
            )
        })
        .map(|t| {
            (
                t.clone(),
                find_asset_name(app_store, t.related_asset_id.unwrap_or_default()),
                find_portfolio_name(app_store, t.related_portfolio_id.unwrap_or_default()),
            )
        })
        .collect();
    sort_transactions(&mut items, &sort);
    let count = items.len();
    let total: f64 = items.iter().map(|(t, _, _)| t.amount).sum();
    let items_for = items.clone();
    let items_memo = Memo::new(move |_| items_for.clone());
    view! {
        <div class="reporting-section">
            <div class="reporting-section-meta">{format!("{} statements · Total: {}", count, fmt_dollars(total))}</div>
            <div class="reporting-table">
                {table_head(&["Date", "Type", "Asset", "Portfolio", "Amount", "Status"])}
                {if items_memo.get().is_empty() {
                    view! { <div class="reporting-empty">"No statements generated."</div> }.into_any()
                } else {
                    view! {
                        <For
                            each=move || items_memo.get()
                            key=|(t, _, _)| t.id
                            children=move |(t, asset, portfolio)| {
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
                            }
                        />
                    }.into_any()
                }}
            </div>
        </div>
    }
}
