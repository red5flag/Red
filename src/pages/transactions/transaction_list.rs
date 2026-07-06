use crate::models::TransactionStatus;
use crate::pages::transactions::{format_dollars, Contact, Payment, Transaction, Wallet};
use crate::types::{Currency, SortMode, TransactionType};
use leptos::prelude::*;

pub(crate) fn status_label(status: &TransactionStatus) -> &'static str {
    match status {
        TransactionStatus::Draft => "Draft",
        TransactionStatus::Pending => "Pending",
        TransactionStatus::Approved => "Approved",
        TransactionStatus::Rejected => "Rejected",
        TransactionStatus::Executed => "Executed",
        TransactionStatus::Cancelled => "Cancelled",
    }
}

pub(crate) fn transaction_status_class(status: &TransactionStatus) -> &'static str {
    match status {
        TransactionStatus::Executed => "tx-status-success",
        TransactionStatus::Approved => "tx-status-pending",
        TransactionStatus::Pending => "tx-status-pending",
        TransactionStatus::Draft => "tx-status-draft",
        TransactionStatus::Rejected => "tx-status-danger",
        TransactionStatus::Cancelled => "tx-status-muted",
    }
}

pub(crate) fn type_icon(transaction_type: &TransactionType) -> &'static str {
    match transaction_type {
        TransactionType::Purchase => "🛒",
        TransactionType::Sale => "💰",
        TransactionType::Rent => "🏠",
        TransactionType::Lease => "📄",
        TransactionType::Payout => "💵",
        TransactionType::Dividend => "📈",
        TransactionType::Fee => "⚠",
        TransactionType::Tax => "🏛",
        TransactionType::Transfer => "🔄",
        TransactionType::Adjustment => "🔧",
    }
}

pub(crate) fn currency_label(currency: &Currency) -> String {
    currency.to_string()
}

#[component]
pub(crate) fn TransactionList(
    transactions: Vec<Transaction>,
    sort_mode: SortMode,
) -> impl IntoView {
    let mut items = transactions;
    items.sort_by(|a, b| match sort_mode {
        SortMode::Recent => b.created_at.cmp(&a.created_at),
        SortMode::Oldest => a.created_at.cmp(&b.created_at),
        SortMode::HighestValue => b
            .amount
            .partial_cmp(&a.amount)
            .unwrap_or(std::cmp::Ordering::Equal),
        SortMode::LowestValue => a
            .amount
            .partial_cmp(&b.amount)
            .unwrap_or(std::cmp::Ordering::Equal),
        _ => b.created_at.cmp(&a.created_at),
    });

    let txs = items.clone();
    let txs_memo = Memo::new(move |_| txs.clone());

    if txs_memo.get().is_empty() {
        view! { <div class="tx-empty">"No transactions yet."</div> }.into_any()
    } else {
        view! {
            <div class="tx-statement-list">
                <For
                    each=move || txs_memo.get()
                    key=|t| t.id
                    children=move |t| {
                        let icon = type_icon(&t.transaction_type);
                        let status = t.status.clone();
                        let status_label = status_label(&status);
                        let status_class = transaction_status_class(&status);
                        let amount = format_dollars(t.amount, &currency_label(&t.currency));
                        let desc = t.description.unwrap_or_default();
                        let date = t.created_at.format("%d %b %Y").to_string();
                        view! {
                            <div class="tx-statement-row">
                                <div class="tx-statement-icon">{icon}</div>
                                <div class="tx-statement-main">
                                    <div class="tx-statement-title">{desc}</div>
                                    <div class="tx-statement-meta">{format!("{} · {} → {}", date, t.from_entity.name, t.to_entity.name)}</div>
                                </div>
                                <div class="tx-statement-right">
                                    <div class="tx-statement-amount">{amount}</div>
                                    <div class={format!("tx-status-badge {}", status_class)}>{status_label}</div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        }.into_any()
    }
}

#[component]
pub(crate) fn PaymentList(
    payments: Vec<Payment>,
    wallets: Vec<Wallet>,
    payees: Vec<Contact>,
    payers: Vec<Contact>,
) -> impl IntoView {
    let payments_for = payments.clone();
    let payments_reversed = payments_for
        .iter()
        .rev()
        .take(5)
        .cloned()
        .collect::<Vec<_>>();

    if payments_reversed.is_empty() {
        view! { <div class="tx-empty">"No payments yet"</div> }.into_any()
    } else {
        view! {
            <div class="tx-statement-list">
                <For
                    each=move || payments_reversed.clone()
                    key=|p| p._id
                    children=move |p| {
                        let wallet = wallets.iter().find(|w| w.id == p.from_wallet_id).cloned();
                        let contact = if p.direction == "send" {
                            payees.iter().find(|c| c.id == p.to_contact_id).cloned()
                        } else {
                            payers.iter().find(|c| c.id == p.to_contact_id).cloned()
                        };
                        let wallet_name = wallet.as_ref().map(|w| w.name.clone()).unwrap_or_else(|| "Unknown".to_string());
                        let contact_name = contact.map(|c| c.name).unwrap_or_else(|| "Unknown".to_string());
                        let card_label = wallet.and_then(|w| p.from_card_id.and_then(|cid| w.cards.iter().find(|c| c.id == cid).map(|c| format!("{} ending {}", c.label, c.last4)))).unwrap_or_else(|| "wallet balance".to_string());
                        let routine = if p.is_routine { "routine" } else { "one-time" };
                        let date = p.created_at.format("%d %b %H:%M").to_string();
                        view! {
                            <div class="tx-statement-row">
                                <div class="tx-statement-icon">{if p.direction == "send" { "📤" } else { "📥" }}</div>
                                <div class="tx-statement-main">
                                    <div class="tx-statement-title">{format!("{} - {}", p.direction.to_uppercase(), contact_name)}</div>
                                    <div class="tx-statement-meta">{format!("{} · {} via {} · {}", date, wallet_name, card_label, routine)}</div>
                                </div>
                                <div class="tx-statement-right">
                                    <div class="tx-statement-amount">{format_dollars(p.amount, &p.currency)}</div>
                                    <div class="tx-status-badge tx-status-draft">{if p.is_auto { "Auto" } else { "Manual" }}</div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        }.into_any()
    }
}
