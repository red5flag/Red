use crate::models::TransactionStatus;
use crate::pages::transactions::approval_panel::PrintRecordButton;
use crate::pages::transactions::transaction_detail::{InvoiceForm, InvoiceList};
use crate::pages::transactions::transaction_form::{CreateTransactionRecord, PaymentForm};
use crate::pages::transactions::transaction_list::{PaymentList, TransactionList};
use crate::pages::transactions::transaction_summary::{
    PayeeList, PayerList, SummaryCards, WalletCards, WalletDashboard, WalletDetails,
};
use crate::pages::transactions::{
    create_mock_transaction, Card, Contact, Invoice, Payment, Wallet,
};
use crate::stores::use_app_store;
use crate::types::{SortMode, TransactionType};
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

fn mock_wallets() -> Vec<Wallet> {
    vec![
        Wallet {
            id: Uuid::new_v4(),
            name: "Business Bank".to_string(),
            wallet_type: "Bank".to_string(),
            balance: 1_250_000.0,
            currency: "AUD".to_string(),
            cards: vec![
                Card {
                    id: Uuid::new_v4(),
                    label: "Business Debit".to_string(),
                    last4: "4821".to_string(),
                },
                Card {
                    id: Uuid::new_v4(),
                    label: "Business Credit".to_string(),
                    last4: "9912".to_string(),
                },
            ],
        },
        Wallet {
            id: Uuid::new_v4(),
            name: "Coinspot".to_string(),
            wallet_type: "Crypto".to_string(),
            balance: 45_000.0,
            currency: "AUD".to_string(),
            cards: vec![Card {
                id: Uuid::new_v4(),
                label: "Coinspot AUD".to_string(),
                last4: "SPOT".to_string(),
            }],
        },
        Wallet {
            id: Uuid::new_v4(),
            name: "Coinbase".to_string(),
            wallet_type: "Crypto".to_string(),
            balance: 28_000.0,
            currency: "USD".to_string(),
            cards: vec![Card {
                id: Uuid::new_v4(),
                label: "Coinbase USD".to_string(),
                last4: "BASE".to_string(),
            }],
        },
        Wallet {
            id: Uuid::new_v4(),
            name: "Cold Wallet".to_string(),
            wallet_type: "Crypto".to_string(),
            balance: 3.5,
            currency: "BTC".to_string(),
            cards: vec![Card {
                id: Uuid::new_v4(),
                label: "BTC Wallet".to_string(),
                last4: "BTC".to_string(),
            }],
        },
    ]
}

fn mock_payees() -> Vec<Contact> {
    vec![
        Contact {
            id: Uuid::new_v4(),
            name: "Tech Supplies Inc".to_string(),
            account: "BSB 062-000 12345678".to_string(),
            currency: "AUD".to_string(),
        },
        Contact {
            id: Uuid::new_v4(),
            name: "Property Manager".to_string(),
            account: "BSB 032-002 98765432".to_string(),
            currency: "AUD".to_string(),
        },
        Contact {
            id: Uuid::new_v4(),
            name: "Crypto Exchange".to_string(),
            account: "0x71C7656EC7ab88b098defB751B7401B5f6d8976F".to_string(),
            currency: "ETH".to_string(),
        },
    ]
}

fn mock_payers() -> Vec<Contact> {
    vec![
        Contact {
            id: Uuid::new_v4(),
            name: "Buyer Corp".to_string(),
            account: "BSB 082-001 55556666".to_string(),
            currency: "AUD".to_string(),
        },
        Contact {
            id: Uuid::new_v4(),
            name: "Tenant LLC".to_string(),
            account: "BSB 062-000 77778888".to_string(),
            currency: "AUD".to_string(),
        },
    ]
}

fn mock_invoices() -> Vec<Invoice> {
    use crate::pages::transactions::InvoiceStatus;
    vec![
        Invoice {
            id: Uuid::new_v4(),
            number: "INV-2026-001".to_string(),
            issue_date: Utc::now() - chrono::Duration::days(5),
            due_date: Utc::now() + chrono::Duration::days(25),
            amount: 12_500.0,
            currency: "AUD".to_string(),
            status: InvoiceStatus::Sent,
            from_name: "Carly Holdings".to_string(),
            to_name: "Buyer Corp".to_string(),
            description: "Property consultation services".to_string(),
        },
        Invoice {
            id: Uuid::new_v4(),
            number: "INV-2026-002".to_string(),
            issue_date: Utc::now() - chrono::Duration::days(35),
            due_date: Utc::now() - chrono::Duration::days(5),
            amount: 8_400.0,
            currency: "AUD".to_string(),
            status: InvoiceStatus::Overdue,
            from_name: "Carly Holdings".to_string(),
            to_name: "Tenant LLC".to_string(),
            description: "Monthly warehouse rent".to_string(),
        },
        Invoice {
            id: Uuid::new_v4(),
            number: "INV-2026-003".to_string(),
            issue_date: Utc::now() - chrono::Duration::days(20),
            due_date: Utc::now() + chrono::Duration::days(10),
            amount: 45_000.0,
            currency: "USD".to_string(),
            status: InvoiceStatus::Paid,
            from_name: "Carly Holdings".to_string(),
            to_name: "Tech Supplies Inc".to_string(),
            description: "Equipment purchase invoice".to_string(),
        },
    ]
}

#[component]
pub fn TransactionsPage() -> impl IntoView {
    let _app_store = use_app_store();

    let (active_tab, set_active_tab) = signal("recent".to_string());
    let (wallets, _set_wallets) = signal(mock_wallets());
    let (payees, _set_payees) = signal(mock_payees());
    let (payers, _set_payers) = signal(mock_payers());
    let (payments, set_payments) = signal(Vec::<Payment>::new());
    let (invoices, set_invoices) = signal(mock_invoices());
    let (transactions, set_transactions) = signal(vec![
        create_mock_transaction(
            TransactionType::Purchase,
            125000.0,
            "Office equipment purchase",
            "Main Org",
            "Tech Supplies Inc",
            TransactionStatus::Executed,
        ),
        create_mock_transaction(
            TransactionType::Sale,
            450000.0,
            "Property sale - downtown plaza",
            "Real Estate Holdings",
            "Buyer Corp",
            TransactionStatus::Approved,
        ),
        create_mock_transaction(
            TransactionType::Rent,
            8500.0,
            "Monthly warehouse rent",
            "Tenant LLC",
            "Property Manager",
            TransactionStatus::Executed,
        ),
        create_mock_transaction(
            TransactionType::Fee,
            1200.0,
            "Bank processing fee",
            "Main Org",
            "Banking Partner",
            TransactionStatus::Executed,
        ),
        create_mock_transaction(
            TransactionType::Transfer,
            50000.0,
            "Inter-portfolio transfer",
            "Portfolio A",
            "Portfolio B",
            TransactionStatus::Pending,
        ),
    ]);
    let (sort_mode, set_sort_mode) = signal(SortMode::Recent);

    let on_send = Callback::new(move |p: Payment| {
        set_payments.update(|list| list.push(p));
    });
    let on_receive = Callback::new(move |p: Payment| {
        set_payments.update(|list| list.push(p));
    });
    let on_create_txn = Callback::new(move |t: crate::models::Transaction| {
        set_transactions.update(|list| list.insert(0, t));
    });
    let on_create_invoice = Callback::new(move |inv: Invoice| {
        set_invoices.update(|list| list.insert(0, inv));
    });

    let tab_btn = |label: &str, key: &str| {
        let key = key.to_string();
        let key_active = key.clone();
        let key_click = key.clone();
        let label = label.to_string();
        view! {
            <button
                class="tx-tab-btn"
                class:active={move || active_tab.get() == key_active}
                on:click=move |_| { let key_click = key_click.clone(); set_active_tab.set(key_click) }
            >
                {label.clone()}
            </button>
        }
    };

    let set_active_tab_callback = Callback::new(move |key: String| {
        set_active_tab.set(key);
    });

    view! {
        <div class="home-screen tx-page">
            // Wallet dashboard always visible at top
            {move || view! { <WalletDashboard wallets={wallets.get()} /> }}

            <div class="tx-tabs">
                {tab_btn("Recent", "recent")}
                {tab_btn("Send", "send")}
                {tab_btn("Receive", "receive")}
                {tab_btn("Invoices", "invoices")}
                {tab_btn("Payees", "payees")}
                {tab_btn("Payers", "payers")}
                {tab_btn("Wallets", "wallets")}
            </div>

            // Action bar
            <div class="tx-action-bar">
                <button class="tx-action-btn" on:click=move |_| set_active_tab.set("create".to_string())>"📝 Transaction"</button>
                <button class="tx-action-btn tx-action-btn-invoice" on:click=move |_| set_active_tab.set("create_invoice".to_string())>"🧾 Invoice"</button>
                <PrintRecordButton />
            </div>

            {move || match active_tab.get().as_str() {
                "create" => view! {
                    <div class="data-card">
                        <div class="card-header"><span class="card-title">"Create Transaction Record"</span></div>
                        <CreateTransactionRecord _wallets={wallets.get()} on_create={on_create_txn.clone()} />
                    </div>
                }.into_any(),
                "create_invoice" => view! {
                    <div class="data-card">
                        <div class="card-header"><span class="card-title">"Create Invoice"</span></div>
                        <InvoiceForm on_create={on_create_invoice.clone()} />
                    </div>
                }.into_any(),
                "invoices" => view! {
                    <InvoiceList
                        invoices={invoices.get()}
                        on_new_invoice={Some(Callback::new(move |_| set_active_tab.set("create_invoice".to_string())))}
                    />
                }.into_any(),
                "payees" => view! { <PayeeList contacts={payees.get()} /> }.into_any(),
                "payers" => view! { <PayerList contacts={payers.get()} /> }.into_any(),
                "wallets" => view! {
                    <div class="tx-wallets-section">
                        <WalletCards wallets={wallets.get()} />
                        <WalletDetails wallets={wallets.get()} />
                    </div>
                }.into_any(),
                "send" => view! {
                    <div class="data-card">
                        <div class="card-header"><span class="card-title">"Send Payment"</span></div>
                        <PaymentForm wallets={wallets.get()} contacts={payees.get()} direction="send" on_submit={on_send.clone()} />
                    </div>
                }.into_any(),
                "receive" => view! {
                    <div class="data-card">
                        <div class="card-header"><span class="card-title">"Receive Payment"</span></div>
                        <PaymentForm wallets={wallets.get()} contacts={payers.get()} direction="receive" on_submit={on_receive.clone()} />
                    </div>
                }.into_any(),
                _ => view! {
                    <SummaryCards invoices={invoices.get()} set_active_tab={set_active_tab_callback.clone()} />
                    <div class="data-card">
                        <div class="card-header">
                            <span class="card-title">"Recent Transactions"</span>
                            <select
                                class="form-select tx-sort-select"
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    let mode = match v.as_str() {
                                        "oldest" => SortMode::Oldest,
                                        "highest_amount" => SortMode::HighestValue,
                                        "lowest_amount" => SortMode::LowestValue,
                                        _ => SortMode::Recent,
                                    };
                                    set_sort_mode.set(mode);
                                }
                            >
                                <option value="recent">"Recent"</option>
                                <option value="oldest">"Oldest"</option>
                                <option value="highest_amount">"Highest Amount"</option>
                                <option value="lowest_amount">"Lowest Amount"</option>
                            </select>
                        </div>
                        {move || view! {
                            <TransactionList transactions={transactions.get()} sort_mode={sort_mode.get()} />
                        }.into_any()}
                    </div>
                    <div class="data-card">
                        <div class="card-header"><span class="card-title">"Recent Payments"</span></div>
                        {move || view! {
                            <PaymentList payments={payments.get()} wallets={wallets.get()} payees={payees.get()} payers={payers.get()} />
                        }.into_any()}
                    </div>
                }.into_any(),
            }}
        </div>
    }
}
