use crate::pages::transactions::{
    currency_symbol, format_dollars, Contact, Invoice, InvoiceStatus, Wallet,
};
use leptos::prelude::*;

#[component]
pub(crate) fn WalletDashboard(wallets: Vec<Wallet>) -> impl IntoView {
    let total_fiat: f64 = wallets
        .iter()
        .filter(|w| w.wallet_type != "Crypto")
        .map(|w| w.balance)
        .sum();
    let total_crypto: f64 = wallets
        .iter()
        .filter(|w| w.wallet_type == "Crypto" && w.currency != "BTC")
        .map(|w| w.balance)
        .sum();
    let btc_balance: f64 = wallets
        .iter()
        .filter(|w| w.currency == "BTC")
        .map(|w| w.balance)
        .sum();

    view! {
        <div class="wallet-dashboard">
            <div class="wallet-balance-bar">
                <div class="wallet-bal-item">
                    <div class="wallet-bal-label">"FIAT"</div>
                    <div class="wallet-bal-value">{format!("${:.2}", total_fiat)}</div>
                </div>
                <div class="wallet-bal-divider"></div>
                <div class="wallet-bal-item">
                    <div class="wallet-bal-label">"CRYPTO"</div>
                    <div class="wallet-bal-value">{format!("${:.2}", total_crypto)}</div>
                </div>
                <div class="wallet-bal-divider"></div>
                <div class="wallet-bal-item">
                    <div class="wallet-bal-label">"BTC"</div>
                    <div class="wallet-bal-value">{format!("₿{:.4}", btc_balance)}</div>
                </div>
            </div>
        </div>
    }
}

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
    view! {
        <div class="data-card">
            <div class="card-header"><span class="card-title">"Payees"</span></div>
            {contacts.into_iter().map(|c| view! {
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">{c.name}</div>
                        <div class="list-item-subtitle">{c.account}</div>
                    </div>
                    <div class="list-item-right"><div class="list-item-subtitle">{c.currency}</div></div>
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub(crate) fn PayerList(contacts: Vec<Contact>) -> impl IntoView {
    view! {
        <div class="data-card">
            <div class="card-header"><span class="card-title">"Payers"</span></div>
            {contacts.into_iter().map(|c| view! {
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">{c.name}</div>
                        <div class="list-item-subtitle">{c.account}</div>
                    </div>
                    <div class="list-item-right"><div class="list-item-subtitle">{c.currency}</div></div>
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub(crate) fn WalletCards(wallets: Vec<Wallet>) -> impl IntoView {
    view! {
        <div class="dcard-strip">
            {wallets.into_iter().flat_map(|w| {
                let w2 = w.clone();
                w.cards.into_iter().map(move |c| {
                    let w3 = w2.clone();
                    let c2 = c.clone();
                    view! {
                        <super::transaction_card::DigitalCard wallet={w3.clone()} card={c2} on_click=move |_, _| {} />
                    }
                })
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub(crate) fn WalletDetails(wallets: Vec<Wallet>) -> impl IntoView {
    view! {
        <div class="data-card">
            <div class="card-header"><span class="card-title">"Wallet Details"</span></div>
            {wallets.into_iter().map(|w| view! {
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">{format!("{} ({})", w.name, w.wallet_type)}</div>
                        <div class="list-item-subtitle">{format!("{} card{}", w.cards.len(), if w.cards.len() == 1 { "" } else { "s" })}</div>
                    </div>
                    <div class="list-item-right">
                        <div class="list-item-value">{format!("{}{:.2} {}", currency_symbol(&w.currency), w.balance, w.currency)}</div>
                    </div>
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}
