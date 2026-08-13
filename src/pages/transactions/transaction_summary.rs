use crate::models::Transaction;
use crate::pages::transactions::{
    currency_symbol, format_dollars, Bill, BillStatus, Card, Contact, Invoice, InvoiceStatus,
    Wallet,
};
use crate::types::TransactionType;
use leptos::prelude::*;

#[component]
pub(crate) fn SummaryCards(
    invoices: Vec<Invoice>,
    set_active_tab: Callback<String>,
) -> impl IntoView {
    let total_outstanding: f64 = invoices
        .iter()
        .filter(|i| {
            matches!(
                i.status,
                InvoiceStatus::Sent | InvoiceStatus::Viewed | InvoiceStatus::Draft
            )
        })
        .map(|i| i.amount)
        .sum();
    let total_overdue: f64 = invoices
        .iter()
        .filter(|i| i.status == InvoiceStatus::Overdue)
        .map(|i| i.amount)
        .sum();

    view! {
        <div class="tx-summary-cards">
            <div class="tx-summary-card tx-summary-outstanding">
                <div class="tx-summary-label">"Outstanding"</div>
                <div class="tx-summary-value">{format_dollars(total_outstanding, "AUD")}</div>
            </div>
            <div class="tx-summary-card tx-summary-overdue">
                <div class="tx-summary-label">"Overdue"</div>
                <div class="tx-summary-value">{format_dollars(total_overdue, "AUD")}</div>
            </div>
            <div class="tx-summary-card" on:click=move |_| set_active_tab.run("invoices".to_string())>
                <div class="tx-summary-label">"Invoices"</div>
                <div class="tx-summary-value">{format!("{}", invoices.len())}</div>
            </div>
        </div>
    }
}

#[component]
pub(crate) fn PayeeList(contacts: Vec<Contact>) -> impl IntoView {
    let contacts_for = contacts.clone();
    let contacts_memo = Memo::new(move |_| contacts_for.clone());
    view! {
        <div class="data-card">
            <div class="card-header"><span class="card-title">"Payees"</span></div>
            <For
                each=move || contacts_memo.get()
                key=|c| c.id
                children=move |c| view! {
                    <div class="list-item">
                        <div class="list-item-left">
                            <div class="list-item-title">{c.name}</div>
                            <div class="list-item-subtitle">{c.account}</div>
                        </div>
                        <div class="list-item-right"><div class="list-item-subtitle">{c.currency}</div></div>
                    </div>
                }
            />
        </div>
    }
}

#[component]
pub(crate) fn PayerList(contacts: Vec<Contact>) -> impl IntoView {
    let contacts_for = contacts.clone();
    let contacts_memo = Memo::new(move |_| contacts_for.clone());
    view! {
        <div class="data-card">
            <div class="card-header"><span class="card-title">"Payers"</span></div>
            <For
                each=move || contacts_memo.get()
                key=|c| c.id
                children=move |c| view! {
                    <div class="list-item">
                        <div class="list-item-left">
                            <div class="list-item-title">{c.name}</div>
                            <div class="list-item-subtitle">{c.account}</div>
                        </div>
                        <div class="list-item-right"><div class="list-item-subtitle">{c.currency}</div></div>
                    </div>
                }
            />
        </div>
    }
}

#[component]
pub(crate) fn WalletCards(wallets: Vec<Wallet>) -> impl IntoView {
    let wallet_cards: Vec<(Wallet, Card)> = wallets
        .into_iter()
        .flat_map(|w| {
            let w2 = w.clone();
            w.cards.into_iter().map(move |c| (w2.clone(), c))
        })
        .collect();
    let wallet_cards_for = wallet_cards.clone();
    let wallet_cards_memo = Memo::new(move |_| wallet_cards_for.clone());
    view! {
        <div class="dcard-strip">
            <For
                each=move || wallet_cards_memo.get()
                key=|(_, c)| c.id
                children=move |(w, c)| {
                    view! {
                        <super::transaction_card::DigitalCard wallet={w} card={c} on_click=move |_, _| {} />
                    }
                }
            />
        </div>
    }
}

#[component]
pub(crate) fn WalletDetails(wallets: Vec<Wallet>) -> impl IntoView {
    let wallets_for = wallets.clone();
    let wallets_memo = Memo::new(move |_| wallets_for.clone());
    view! {
        <div class="data-card">
            <div class="card-header"><span class="card-title">"Wallet Details"</span></div>
            <For
                each=move || wallets_memo.get()
                key=|w| w.id
                children=move |w| view! {
                    <div class="list-item">
                        <div class="list-item-left">
                            <div class="list-item-title">{format!("{} ({})", w.name, w.wallet_type)}</div>
                            <div class="list-item-subtitle">{format!("{} card{}", w.cards.len(), if w.cards.len() == 1 { "" } else { "s" })}</div>
                        </div>
                        <div class="list-item-right">
                            <div class="list-item-value">{format!("{}{:.2} {}", currency_symbol(&w.currency), w.balance, w.currency)}</div>
                        </div>
                    </div>
                }
            />
        </div>
    }
}

#[component]
pub(crate) fn CashFlowSummary(
    transactions: Vec<Transaction>,
    invoices: Vec<Invoice>,
    bills: Vec<Bill>,
    wallets: Vec<Wallet>,
) -> impl IntoView {
    let total_inflow: f64 = transactions
        .iter()
        .filter(|t| {
            t.status == crate::models::TransactionStatus::Executed
                && matches!(
                    t.transaction_type,
                    TransactionType::Sale | TransactionType::Rent | TransactionType::Dividend
                )
        })
        .map(|t| t.amount)
        .sum();
    let total_outflow: f64 = transactions
        .iter()
        .filter(|t| {
            t.status == crate::models::TransactionStatus::Executed
                && matches!(
                    t.transaction_type,
                    TransactionType::Purchase
                        | TransactionType::Fee
                        | TransactionType::Tax
                        | TransactionType::Payout
                )
        })
        .map(|t| t.amount)
        .sum();
    let net_flow = total_inflow - total_outflow;

    let invoices_outstanding: f64 = invoices
        .iter()
        .filter(|i| {
            matches!(
                i.status,
                InvoiceStatus::Sent | InvoiceStatus::Viewed | InvoiceStatus::Draft
            )
        })
        .map(|i| i.amount)
        .sum();
    let invoices_overdue: f64 = invoices
        .iter()
        .filter(|i| i.status == InvoiceStatus::Overdue)
        .map(|i| i.amount)
        .sum();

    let bills_owing: f64 = bills
        .iter()
        .filter(|b| {
            matches!(
                b.status,
                BillStatus::AwaitingApproval | BillStatus::AwaitingPayment
            )
        })
        .map(|b| b.amount)
        .sum();
    let bills_overdue: f64 = bills
        .iter()
        .filter(|b| b.status == BillStatus::Overdue)
        .map(|b| b.amount)
        .sum();

    let total_balance: f64 = wallets.iter().map(|w| w.balance).sum();

    view! {
        <div class="tx-cashflow-grid">
            <div class="tx-cashflow-card tx-cashflow-inflow">
                <div class="tx-cashflow-label">"Money In"</div>
                <div class="tx-cashflow-value">{format_dollars(total_inflow, "AUD")}</div>
            </div>
            <div class="tx-cashflow-card tx-cashflow-outflow">
                <div class="tx-cashflow-label">"Money Out"</div>
                <div class="tx-cashflow-value">{format_dollars(total_outflow, "AUD")}</div>
            </div>
            <div class="tx-cashflow-card tx-cashflow-net">
                <div class="tx-cashflow-label">"Net Flow"</div>
                <div class="tx-cashflow-value" class:negative={net_flow < 0.0}>{format_dollars(net_flow, "AUD")}</div>
            </div>
            <div class="tx-cashflow-card tx-cashflow-balance">
                <div class="tx-cashflow-label">"Bank Balance"</div>
                <div class="tx-cashflow-value">{format_dollars(total_balance, "AUD")}</div>
            </div>
            <div class="tx-cashflow-card tx-cashflow-receivable">
                <div class="tx-cashflow-label">"Owed to You"</div>
                <div class="tx-cashflow-value">{format_dollars(invoices_outstanding, "AUD")}</div>
                {if invoices_overdue > 0.0 {
                    view! { <div class="tx-cashflow-sub tx-cashflow-danger">{format!("{} overdue", format_dollars(invoices_overdue, "AUD"))}</div> }.into_any()
                } else { ().into_any() }}
            </div>
            <div class="tx-cashflow-card tx-cashflow-payable">
                <div class="tx-cashflow-label">"You Owe"</div>
                <div class="tx-cashflow-value">{format_dollars(bills_owing, "AUD")}</div>
                {if bills_overdue > 0.0 {
                    view! { <div class="tx-cashflow-sub tx-cashflow-danger">{format!("{} overdue", format_dollars(bills_overdue, "AUD"))}</div> }.into_any()
                } else { ().into_any() }}
            </div>
        </div>
    }
}
