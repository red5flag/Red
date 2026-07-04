use crate::models::{EntityReference, EntityType, Transaction, TransactionStatus};
use crate::stores::use_app_store;
use crate::types::{Currency, SortMode, TransactionType};
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug)]
struct Card {
    id: Uuid,
    label: String,
    last4: String,
}

#[derive(Clone, Debug)]
struct Wallet {
    id: Uuid,
    name: String,
    wallet_type: String,
    balance: f64,
    currency: String,
    cards: Vec<Card>,
}

#[derive(Clone, Debug)]
struct Contact {
    id: Uuid,
    name: String,
    account: String,
    currency: String,
}

#[derive(Clone, Debug)]
struct Payment {
    _id: Uuid,
    direction: String,
    from_wallet_id: Uuid,
    from_card_id: Option<Uuid>,
    to_contact_id: Uuid,
    amount: f64,
    currency: String,
    is_routine: bool,
    interval: String,
    is_auto: bool,
    created_at: chrono::DateTime<Utc>,
}

#[derive(Clone, Debug)]
struct Invoice {
    id: Uuid,
    number: String,
    issue_date: chrono::DateTime<Utc>,
    due_date: chrono::DateTime<Utc>,
    amount: f64,
    currency: String,
    status: InvoiceStatus,
    from_name: String,
    to_name: String,
    description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum InvoiceStatus {
    Draft,
    Sent,
    Viewed,
    Paid,
    Overdue,
    Cancelled,
}

impl InvoiceStatus {
    fn label(&self) -> &'static str {
        match self {
            InvoiceStatus::Draft => "Draft",
            InvoiceStatus::Sent => "Sent",
            InvoiceStatus::Viewed => "Viewed",
            InvoiceStatus::Paid => "Paid",
            InvoiceStatus::Overdue => "Overdue",
            InvoiceStatus::Cancelled => "Cancelled",
        }
    }
}

fn mock_wallets() -> Vec<Wallet> {
    vec![
        Wallet {
            id: Uuid::new_v4(),
            name: "Business Bank".to_string(),
            wallet_type: "Bank".to_string(),
            balance: 1_250_000.0,
            currency: "AUD".to_string(),
            cards: vec![
                Card { id: Uuid::new_v4(), label: "Business Debit".to_string(), last4: "4821".to_string() },
                Card { id: Uuid::new_v4(), label: "Business Credit".to_string(), last4: "9912".to_string() },
            ],
        },
        Wallet {
            id: Uuid::new_v4(),
            name: "Coinspot".to_string(),
            wallet_type: "Crypto".to_string(),
            balance: 45_000.0,
            currency: "AUD".to_string(),
            cards: vec![Card { id: Uuid::new_v4(), label: "Coinspot AUD".to_string(), last4: "SPOT".to_string() }],
        },
        Wallet {
            id: Uuid::new_v4(),
            name: "Coinbase".to_string(),
            wallet_type: "Crypto".to_string(),
            balance: 28_000.0,
            currency: "USD".to_string(),
            cards: vec![Card { id: Uuid::new_v4(), label: "Coinbase USD".to_string(), last4: "BASE".to_string() }],
        },
        Wallet {
            id: Uuid::new_v4(),
            name: "Cold Wallet".to_string(),
            wallet_type: "Crypto".to_string(),
            balance: 3.5,
            currency: "BTC".to_string(),
            cards: vec![Card { id: Uuid::new_v4(), label: "BTC Wallet".to_string(), last4: "BTC".to_string() }],
        },
    ]
}

fn mock_payees() -> Vec<Contact> {
    vec![
        Contact { id: Uuid::new_v4(), name: "Tech Supplies Inc".to_string(), account: "BSB 062-000 12345678".to_string(), currency: "AUD".to_string() },
        Contact { id: Uuid::new_v4(), name: "Property Manager".to_string(), account: "BSB 032-002 98765432".to_string(), currency: "AUD".to_string() },
        Contact { id: Uuid::new_v4(), name: "Crypto Exchange".to_string(), account: "0x71C7656EC7ab88b098defB751B7401B5f6d8976F".to_string(), currency: "ETH".to_string() },
    ]
}

fn mock_payers() -> Vec<Contact> {
    vec![
        Contact { id: Uuid::new_v4(), name: "Buyer Corp".to_string(), account: "BSB 082-001 55556666".to_string(), currency: "AUD".to_string() },
        Contact { id: Uuid::new_v4(), name: "Tenant LLC".to_string(), account: "BSB 062-000 77778888".to_string(), currency: "AUD".to_string() },
    ]
}

fn mock_invoices() -> Vec<Invoice> {
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

fn create_mock_transaction(
    transaction_type: TransactionType,
    amount: f64,
    description: &str,
    from: &str,
    to: &str,
    status: TransactionStatus,
) -> Transaction {
    Transaction {
        id: Uuid::new_v4(),
        transaction_type,
        amount,
        currency: Currency::USD,
        description: Some(description.to_string()),
        from_entity: EntityReference { entity_type: EntityType::Organization, entity_id: Uuid::new_v4(), name: from.to_string() },
        to_entity: EntityReference { entity_type: EntityType::External, entity_id: Uuid::new_v4(), name: to.to_string() },
        related_portfolio_id: None,
        related_asset_group_id: None,
        related_asset_id: None,
        executed_by: Uuid::new_v4(),
        status,
        created_at: Utc::now(),
        executed_at: Some(Utc::now()),
        metadata: serde_json::json!({}),
    }
}

fn status_label(status: &TransactionStatus) -> &'static str {
    match status {
        TransactionStatus::Draft => "Draft",
        TransactionStatus::Pending => "Pending",
        TransactionStatus::Approved => "Approved",
        TransactionStatus::Rejected => "Rejected",
        TransactionStatus::Executed => "Executed",
        TransactionStatus::Cancelled => "Cancelled",
    }
}

fn transaction_status_class(status: &TransactionStatus) -> &'static str {
    match status {
        TransactionStatus::Executed => "tx-status-success",
        TransactionStatus::Approved => "tx-status-pending",
        TransactionStatus::Pending => "tx-status-pending",
        TransactionStatus::Draft => "tx-status-draft",
        TransactionStatus::Rejected => "tx-status-danger",
        TransactionStatus::Cancelled => "tx-status-muted",
    }
}

fn invoice_status_class(status: &InvoiceStatus) -> &'static str {
    match status {
        InvoiceStatus::Draft => "tx-status-draft",
        InvoiceStatus::Sent => "tx-status-pending",
        InvoiceStatus::Viewed => "tx-status-pending",
        InvoiceStatus::Paid => "tx-status-success",
        InvoiceStatus::Overdue => "tx-status-danger",
        InvoiceStatus::Cancelled => "tx-status-muted",
    }
}

fn currency_label(currency: &crate::types::Currency) -> String {
    currency.to_string()
}

fn format_dollars(amount: f64, currency: &str) -> String {
    let whole = amount.trunc() as i64;
    let cents = ((amount.fract() * 100.0).round() as i64).abs();
    let s = whole.abs().to_string();
    let mut grouped = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    let grouped = grouped.chars().rev().collect::<String>();
    let sign = if amount < 0.0 { "-" } else { "" };
    format!("{}{}{}.{:02} {}", sign, currency_symbol(currency), grouped, cents, currency)
}

fn format_date(d: &chrono::DateTime<Utc>) -> String {
    d.format("%d %b %Y").to_string()
}

fn type_icon(transaction_type: &TransactionType) -> &'static str {
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

fn currency_symbol(currency: &str) -> &str {
    match currency {
        "USD" => "$",
        "AUD" => "A$",
        "EUR" => "€",
        "GBP" => "£",
        "BTC" => "₿",
        "ETH" => "Ξ",
        _ => "$",
    }
}

#[component]
fn WalletSelector(
    wallets: Vec<Wallet>,
    selected_wallet: ReadSignal<Uuid>,
    set_selected_wallet: WriteSignal<Uuid>,
    selected_card: ReadSignal<Option<Uuid>>,
    set_selected_card: WriteSignal<Option<Uuid>>,
) -> impl IntoView {
    view! {
        <div class="tx-form-row">
            <label class="tx-form-label">"From wallet"</label>
            <select
                class="form-select"
                prop:value={move || selected_wallet.get().to_string()}
                on:change=move |ev| {
                    let v = event_target_value(&ev);
                    if let Ok(id) = Uuid::parse_str(&v) {
                        set_selected_wallet.set(id);
                        set_selected_card.set(None);
                    }
                }
            >
                {wallets.clone().into_iter().map(|w| {
                    let id = w.id.to_string();
                    view! { <option value={id.clone()}>{format!("{} ({}{:.2} {})", w.name, currency_symbol(&w.currency), w.balance, w.currency)}</option> }
                }).collect::<Vec<_>>()}
            </select>
        </div>
        {move || {
            let wallet = wallets.iter().find(|w| w.id == selected_wallet.get()).cloned();
            wallet.map(|w| {
                if w.cards.is_empty() {
                    ().into_any()
                } else {
                    view! {
                        <div class="tx-form-row">
                            <label class="tx-form-label">"From card"</label>
                            <select
                                class="form-select"
                                prop:value={move || selected_card.get().map(|id| id.to_string()).unwrap_or_default()}
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    set_selected_card.set(if v.is_empty() { None } else { Uuid::parse_str(&v).ok() });
                                }
                            >
                                <option value="">"Default wallet balance"</option>
                                {w.cards.into_iter().map(|c| {
                                    let id = c.id.to_string();
                                    view! { <option value={id.clone()}>{format!("{} ending {}", c.label, c.last4)}</option> }
                                }).collect::<Vec<_>>()}
                            </select>
                        </div>
                    }.into_any()
                }
            }).unwrap_or(().into_any())
        }}
    }
}

#[component]
fn PaymentForm(
    wallets: Vec<Wallet>,
    contacts: Vec<Contact>,
    direction: &'static str,
    on_submit: Callback<Payment>,
) -> impl IntoView {
    let (from_wallet, set_from_wallet) = signal(wallets.first().map(|w| w.id).unwrap_or_else(Uuid::new_v4));
    let (from_card, set_from_card) = signal(None::<Uuid>);
    let (to_contact, set_to_contact) = signal(contacts.first().map(|c| c.id).unwrap_or_else(Uuid::new_v4));
    let (amount, set_amount) = signal(String::new());
    let (currency, set_currency) = signal("AUD".to_string());
    let (is_routine, set_is_routine) = signal(false);
    let (interval, set_interval) = signal("Monthly".to_string());
    let (is_auto, set_is_auto) = signal(false);

    let submit = move |_| {
        let value: f64 = amount.get().parse().unwrap_or(0.0);
        if value <= 0.0 { return; }
        let payment = Payment {
            _id: Uuid::new_v4(),
            direction: direction.to_string(),
            from_wallet_id: from_wallet.get(),
            from_card_id: from_card.get(),
            to_contact_id: to_contact.get(),
            amount: value,
            currency: currency.get(),
            is_routine: is_routine.get(),
            interval: interval.get(),
            is_auto: is_auto.get(),
            created_at: Utc::now(),
        };
        on_submit.run(payment);
        set_amount.set(String::new());
    };

    view! {
        <div class="tx-form">
            <WalletSelector
                wallets={wallets}
                selected_wallet={from_wallet}
                set_selected_wallet={set_from_wallet}
                selected_card={from_card}
                set_selected_card={set_from_card}
            />
            <div class="tx-form-row">
                <label class="tx-form-label">{"To ".to_string() + if direction == "send" { "payee" } else { "payer" }}</label>
                <select
                    class="form-select"
                    prop:value={move || to_contact.get().to_string()}
                    on:change=move |ev| {
                        let v = event_target_value(&ev);
                        if let Ok(id) = Uuid::parse_str(&v) {
                            set_to_contact.set(id);
                        }
                    }
                >
                    {contacts.clone().into_iter().map(|c| {
                        let id = c.id.to_string();
                        view! { <option value={id.clone()}>{format!("{} ({})", c.name, c.currency)}</option> }
                    }).collect::<Vec<_>>()}
                </select>
            </div>
            <div class="tx-form-row">
                <label class="tx-form-label">"Amount"</label>
                <input
                    class="form-input"
                    type="number"
                    placeholder="0.00"
                    prop:value={move || amount.get()}
                    on:input=move |ev| set_amount.set(event_target_value(&ev))
                />
            </div>
            <div class="tx-form-row">
                <label class="tx-form-label">"Currency"</label>
                <select
                    class="form-select"
                    prop:value={move || currency.get()}
                    on:change=move |ev| set_currency.set(event_target_value(&ev))
                >
                    <option value="AUD">"AUD (FIAT)"</option>
                    <option value="USD">"USD (FIAT)"</option>
                    <option value="EUR">"EUR (FIAT)"</option>
                    <option value="BTC">"BTC (Crypto)"</option>
                    <option value="ETH">"ETH (Crypto)"</option>
                    <option value="USDC">"USDC (Crypto)"</option>
                </select>
            </div>
            <div class="tx-form-row tx-toggle-row">
                <label class="tx-form-label">"Payment type"</label>
                <div class="tx-toggle">
                    <button class="tx-toggle-btn" class:active={move || !is_routine.get()} on:click=move |_| set_is_routine.set(false)>"One-time"</button>
                    <button class="tx-toggle-btn" class:active={move || is_routine.get()} on:click=move |_| set_is_routine.set(true)>"Routine"</button>
                </div>
            </div>
            {move || if is_routine.get() {
                view! {
                    <div class="tx-form-row">
                        <label class="tx-form-label">"Interval"</label>
                        <select
                            class="form-select"
                            prop:value={move || interval.get()}
                            on:change=move |ev| set_interval.set(event_target_value(&ev))
                        >
                            <option value="Hourly">"Hourly"</option>
                            <option value="Daily">"Daily"</option>
                            <option value="Weekly">"Weekly"</option>
                            <option value="Bi-weekly">"Bi-weekly"</option>
                            <option value="Monthly">"Monthly"</option>
                            <option value="Quarterly">"Quarterly"</option>
                            <option value="Annually">"Annually"</option>
                        </select>
                    </div>
                }.into_any()
            } else { ().into_any() }}
            <div class="tx-form-row tx-toggle-row">
                <label class="tx-form-label">"Conversion"</label>
                <div class="tx-toggle">
                    <button class="tx-toggle-btn" class:active={move || !is_auto.get()} on:click=move |_| set_is_auto.set(false)>"Manual"</button>
                    <button class="tx-toggle-btn" class:active={move || is_auto.get()} on:click=move |_| set_is_auto.set(true)>"Automatic"</button>
                </div>
            </div>
            <button class="login-btn" on:click=submit>{if direction == "send" { "Send Payment" } else { "Receive Payment" }}</button>
        </div>
    }
}

#[component]
fn DigitalCard(
    wallet: Wallet,
    card: Card,
    on_click: impl Fn(Uuid, Uuid) + 'static,
) -> impl IntoView {
    let w_name = wallet.name.clone();
    let w_type = wallet.wallet_type.clone();
    let w_currency = wallet.currency.clone();
    let w_balance = wallet.balance;
    let c_label = card.label.clone();
    let c_last4 = card.last4.clone();
    let wid = wallet.id;
    let cid = card.id;
    let is_crypto = w_type == "Crypto";

    view! {
        <div class="dcard" class:dcard-crypto={is_crypto} on:click=move |_| on_click(wid, cid)>
            <div class="dcard-top">
                <span class="dcard-brand">{if is_crypto { "⚡" } else { "💳" }}</span>
                <span class="dcard-type">{w_type.clone()}</span>
            </div>
            <div class="dcard-number">"•••• •••• •••• "{c_last4.clone()}</div>
            <div class="dcard-bottom">
                <div class="dcard-info">
                    <div class="dcard-label">{c_label.clone()}</div>
                    <div class="dcard-wallet">{w_name.clone()}</div>
                </div>
                <div class="dcard-balance">
                    <div class="dcard-bal-num">{format!("{}{:.2}", currency_symbol(&w_currency), w_balance)}</div>
                    <div class="dcard-bal-cur">{w_currency.clone()}</div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn WalletDashboard(wallets: Vec<Wallet>) -> impl IntoView {
    let total_fiat: f64 = wallets.iter().filter(|w| w.wallet_type != "Crypto").map(|w| w.balance).sum();
    let total_crypto: f64 = wallets.iter().filter(|w| w.wallet_type == "Crypto" && w.currency != "BTC").map(|w| w.balance).sum();
    let btc_balance: f64 = wallets.iter().filter(|w| w.currency == "BTC").map(|w| w.balance).sum();

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
fn CreateTransactionRecord(
    _wallets: Vec<Wallet>,
    on_create: Callback<Transaction>,
) -> impl IntoView {
    let (tx_type, set_tx_type) = signal("Transfer".to_string());
    let (amount, set_amount) = signal(String::new());
    let (currency, set_currency) = signal("AUD".to_string());
    let (description, set_description) = signal(String::new());
    let (from_name, set_from_name) = signal(String::new());
    let (to_name, set_to_name) = signal(String::new());
    let (status, set_status) = signal("Draft".to_string());

    let submit = move |_| {
        let value: f64 = amount.get().parse().unwrap_or(0.0);
        if value <= 0.0 { return; }
        let ttype = match tx_type.get().as_str() {
            "Purchase" => TransactionType::Purchase,
            "Sale" => TransactionType::Sale,
            "Rent" => TransactionType::Rent,
            "Lease" => TransactionType::Lease,
            "Payout" => TransactionType::Payout,
            "Dividend" => TransactionType::Dividend,
            "Fee" => TransactionType::Fee,
            "Tax" => TransactionType::Tax,
            "Transfer" => TransactionType::Transfer,
            _ => TransactionType::Adjustment,
        };
        let st = match status.get().as_str() {
            "Pending" => TransactionStatus::Pending,
            "Approved" => TransactionStatus::Approved,
            "Executed" => TransactionStatus::Executed,
            "Cancelled" => TransactionStatus::Cancelled,
            _ => TransactionStatus::Draft,
        };
        let cur = match currency.get().as_str() {
            "AUD" => Currency::AUD,
            "EUR" => Currency::EUR,
            "GBP" => Currency::GBP,
            _ => Currency::USD,
        };
        let txn = Transaction {
            id: Uuid::new_v4(),
            transaction_type: ttype,
            amount: value,
            currency: cur,
            description: if description.get().is_empty() { None } else { Some(description.get()) },
            from_entity: EntityReference {
                entity_type: EntityType::Organization,
                entity_id: Uuid::new_v4(),
                name: if from_name.get().is_empty() { "Internal".to_string() } else { from_name.get() },
            },
            to_entity: EntityReference {
                entity_type: EntityType::External,
                entity_id: Uuid::new_v4(),
                name: if to_name.get().is_empty() { "External".to_string() } else { to_name.get() },
            },
            related_portfolio_id: None,
            related_asset_group_id: None,
            related_asset_id: None,
            executed_by: Uuid::new_v4(),
            status: st.clone(),
            created_at: Utc::now(),
            executed_at: if st == TransactionStatus::Executed { Some(Utc::now()) } else { None },
            metadata: serde_json::json!({}),
        };
        on_create.run(txn);
        set_amount.set(String::new());
        set_description.set(String::new());
        set_from_name.set(String::new());
        set_to_name.set(String::new());
    };

    view! {
        <div class="tx-form">
            <div class="tx-form-row">
                <label class="tx-form-label">"Type"</label>
                <select class="form-select" prop:value={move || tx_type.get()} on:change=move |ev| set_tx_type.set(event_target_value(&ev))>
                    <option value="Transfer">"Transfer"</option>
                    <option value="Purchase">"Purchase"</option>
                    <option value="Sale">"Sale"</option>
                    <option value="Rent">"Rent"</option>
                    <option value="Lease">"Lease"</option>
                    <option value="Payout">"Payout"</option>
                    <option value="Dividend">"Dividend"</option>
                    <option value="Fee">"Fee"</option>
                    <option value="Tax">"Tax"</option>
                    <option value="Adjustment">"Adjustment"</option>
                </select>
            </div>
            <div class="tx-form-row">
                <label class="tx-form-label">"Amount"</label>
                <input class="form-input" type="number" placeholder="0.00" prop:value={move || amount.get()} on:input=move |ev| set_amount.set(event_target_value(&ev)) />
            </div>
            <div class="tx-form-row">
                <label class="tx-form-label">"Currency"</label>
                <select class="form-select" prop:value={move || currency.get()} on:change=move |ev| set_currency.set(event_target_value(&ev))>
                    <option value="AUD">"AUD"</option>
                    <option value="USD">"USD"</option>
                    <option value="EUR">"EUR"</option>
                    <option value="GBP">"GBP"</option>
                </select>
            </div>
            <div class="tx-form-row">
                <label class="tx-form-label">"Description"</label>
                <input class="form-input" type="text" placeholder="Transaction description" prop:value={move || description.get()} on:input=move |ev| set_description.set(event_target_value(&ev)) />
            </div>
            <div class="tx-form-row">
                <label class="tx-form-label">"From"</label>
                <input class="form-input" type="text" placeholder="Source entity" prop:value={move || from_name.get()} on:input=move |ev| set_from_name.set(event_target_value(&ev)) />
            </div>
            <div class="tx-form-row">
                <label class="tx-form-label">"To"</label>
                <input class="form-input" type="text" placeholder="Destination entity" prop:value={move || to_name.get()} on:input=move |ev| set_to_name.set(event_target_value(&ev)) />
            </div>
            <div class="tx-form-row">
                <label class="tx-form-label">"Status"</label>
                <select class="form-select" prop:value={move || status.get()} on:change=move |ev| set_status.set(event_target_value(&ev))>
                    <option value="Draft">"Draft"</option>
                    <option value="Pending">"Pending"</option>
                    <option value="Approved">"Approved"</option>
                    <option value="Executed">"Executed"</option>
                    <option value="Cancelled">"Cancelled"</option>
                </select>
            </div>
            <button class="login-btn" on:click=submit>"Create Record"</button>
        </div>
    }
}

#[component]
fn InvoiceForm(on_create: Callback<Invoice>) -> impl IntoView {
    let (number, set_number) = signal(String::new());
    let (amount, set_amount) = signal(String::new());
    let (currency, set_currency) = signal("AUD".to_string());
    let (to_name, set_to_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (due_days, set_due_days) = signal("30".to_string());

    let submit = move |_| {
        let value: f64 = amount.get().parse().unwrap_or(0.0);
        if value <= 0.0 || to_name.get().trim().is_empty() { return; }
        let days: i64 = due_days.get().parse().unwrap_or(30);
        let invoice = Invoice {
            id: Uuid::new_v4(),
            number: if number.get().trim().is_empty() { format!("INV-{}", Uuid::new_v4().to_string().split('-').next().unwrap()) } else { number.get() },
            issue_date: Utc::now(),
            due_date: Utc::now() + chrono::Duration::days(days),
            amount: value,
            currency: currency.get(),
            status: InvoiceStatus::Draft,
            from_name: "Carly Holdings".to_string(),
            to_name: to_name.get(),
            description: description.get(),
        };
        on_create.run(invoice);
        set_number.set(String::new());
        set_amount.set(String::new());
        set_to_name.set(String::new());
        set_description.set(String::new());
        set_due_days.set("30".to_string());
    };

    view! {
        <div class="tx-form">
            <div class="tx-form-row">
                <label class="tx-form-label">"Invoice #"</label>
                <input class="form-input" type="text" placeholder="Auto-generated if empty" prop:value={move || number.get()} on:input=move |ev| set_number.set(event_target_value(&ev)) />
            </div>
            <div class="tx-form-row">
                <label class="tx-form-label">"Bill to"</label>
                <input class="form-input" type="text" placeholder="Customer or payer name" prop:value={move || to_name.get()} on:input=move |ev| set_to_name.set(event_target_value(&ev)) />
            </div>
            <div class="tx-form-row">
                <label class="tx-form-label">"Description"</label>
                <input class="form-input" type="text" placeholder="What this invoice is for" prop:value={move || description.get()} on:input=move |ev| set_description.set(event_target_value(&ev)) />
            </div>
            <div class="tx-form-row">
                <label class="tx-form-label">"Amount"</label>
                <input class="form-input" type="number" placeholder="0.00" prop:value={move || amount.get()} on:input=move |ev| set_amount.set(event_target_value(&ev)) />
            </div>
            <div class="tx-form-row">
                <label class="tx-form-label">"Currency"</label>
                <select class="form-select" prop:value={move || currency.get()} on:change=move |ev| set_currency.set(event_target_value(&ev))>
                    <option value="AUD">"AUD"</option>
                    <option value="USD">"USD"</option>
                    <option value="EUR">"EUR"</option>
                    <option value="GBP">"GBP"</option>
                    <option value="BTC">"BTC"</option>
                    <option value="ETH">"ETH"</option>
                </select>
            </div>
            <div class="tx-form-row">
                <label class="tx-form-label">"Due in (days)"</label>
                <input class="form-input" type="number" placeholder="30" prop:value={move || due_days.get()} on:input=move |ev| set_due_days.set(event_target_value(&ev)) />
            </div>
            <button class="login-btn" on:click=submit>"Create Invoice"</button>
        </div>
    }
}

#[component]
fn PrintRecordButton() -> impl IntoView {
    let on_print = move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.print();
        }
    };
    view! {
        <button class="tx-print-btn" on:click=on_print>"🖨 Print Record"</button>
    }
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
        create_mock_transaction(TransactionType::Purchase, 125000.0, "Office equipment purchase", "Main Org", "Tech Supplies Inc", TransactionStatus::Executed),
        create_mock_transaction(TransactionType::Sale, 450000.0, "Property sale - downtown plaza", "Real Estate Holdings", "Buyer Corp", TransactionStatus::Approved),
        create_mock_transaction(TransactionType::Rent, 8500.0, "Monthly warehouse rent", "Tenant LLC", "Property Manager", TransactionStatus::Executed),
        create_mock_transaction(TransactionType::Fee, 1200.0, "Bank processing fee", "Main Org", "Banking Partner", TransactionStatus::Executed),
        create_mock_transaction(TransactionType::Transfer, 50000.0, "Inter-portfolio transfer", "Portfolio A", "Portfolio B", TransactionStatus::Pending),
    ]);
    let (sort_mode, set_sort_mode) = signal(SortMode::Recent);

    let on_send = Callback::new(move |p: Payment| {
        set_payments.update(|list| list.push(p));
    });
    let on_receive = Callback::new(move |p: Payment| {
        set_payments.update(|list| list.push(p));
    });
    let on_create_txn = Callback::new(move |t: Transaction| {
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
                    <div class="data-card">
                        <div class="card-header">
                            <span class="card-title">"Invoices"</span>
                            <button class="tx-action-btn tx-action-btn-small" on:click=move |_| set_active_tab.set("create_invoice".to_string())>"+ New Invoice"</button>
                        </div>
                        {move || if invoices.get().is_empty() {
                            view! { <div class="tx-empty">"No invoices yet."</div> }.into_any()
                        } else {
                            view! {
                                <div class="tx-invoice-list">
                                    {invoices.get().into_iter().map(|inv| {
                                        let status = inv.status.clone();
                                        let status_class = invoice_status_class(&status);
                                        let amount = format_dollars(inv.amount, &inv.currency);
                                        let issue = format_date(&inv.issue_date);
                                        let due = format_date(&inv.due_date);
                                        view! {
                                            <div class="tx-invoice-item">
                                                <div class="tx-invoice-main">
                                                    <div class="tx-invoice-number">{inv.number}</div>
                                                    <div class="tx-invoice-desc">{inv.description}</div>
                                                    <div class="tx-invoice-parties">{format!("{} → {}", inv.from_name, inv.to_name)}</div>
                                                    <div class="tx-invoice-dates">{format!("Issued {} · Due {}", issue, due)}</div>
                                                </div>
                                                <div class="tx-invoice-right">
                                                    <div class="tx-invoice-amount">{amount}</div>
                                                    <div class={format!("tx-status-badge {}", status_class)}>{status.label()}</div>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }}
                    </div>
                }.into_any(),
                "payees" => view! {
                    <div class="data-card">
                        <div class="card-header"><span class="card-title">"Payees"</span></div>
                        {payees.get().into_iter().map(|c| view! {
                            <div class="list-item">
                                <div class="list-item-left">
                                    <div class="list-item-title">{c.name}</div>
                                    <div class="list-item-subtitle">{c.account}</div>
                                </div>
                                <div class="list-item-right"><div class="list-item-subtitle">{c.currency}</div></div>
                            </div>
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any(),
                "payers" => view! {
                    <div class="data-card">
                        <div class="card-header"><span class="card-title">"Payers"</span></div>
                        {payers.get().into_iter().map(|c| view! {
                            <div class="list-item">
                                <div class="list-item-left">
                                    <div class="list-item-title">{c.name}</div>
                                    <div class="list-item-subtitle">{c.account}</div>
                                </div>
                                <div class="list-item-right"><div class="list-item-subtitle">{c.currency}</div></div>
                            </div>
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any(),
                "wallets" => view! {
                    <div class="tx-wallets-section">
                        <div class="dcard-strip">
                            {wallets.get().into_iter().flat_map(|w| {
                                let w2 = w.clone();
                                w.cards.into_iter().map(move |c| {
                                    let w3 = w2.clone();
                                    let c2 = c.clone();
                                    view! {
                                        <DigitalCard wallet={w3.clone()} card={c2} on_click=move |_, _| {} />
                                    }
                                })
                            }).collect::<Vec<_>>()}
                        </div>
                        <div class="data-card">
                            <div class="card-header"><span class="card-title">"Wallet Details"</span></div>
                            {wallets.get().into_iter().map(|w| view! {
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
                    <div class="tx-summary-cards">
                        {move || {
                            let invs = invoices.get();
                            let total_outstanding: f64 = invs.iter().filter(|i| matches!(i.status, InvoiceStatus::Sent | InvoiceStatus::Viewed | InvoiceStatus::Draft)).map(|i| i.amount).sum();
                            let total_overdue: f64 = invs.iter().filter(|i| i.status == InvoiceStatus::Overdue).map(|i| i.amount).sum();
                            view! {
                                <div class="tx-summary-card tx-summary-outstanding">
                                    <div class="tx-summary-label">"Outstanding"</div>
                                    <div class="tx-summary-value">{format_dollars(total_outstanding, "AUD")}</div>
                                </div>
                                <div class="tx-summary-card tx-summary-overdue">
                                    <div class="tx-summary-label">"Overdue"</div>
                                    <div class="tx-summary-value">{format_dollars(total_overdue, "AUD")}</div>
                                </div>
                                <div class="tx-summary-card" on:click=move |_| set_active_tab.set("invoices".to_string())>
                                    <div class="tx-summary-label">"Invoices"</div>
                                    <div class="tx-summary-value">{format!("{}", invs.len())}</div>
                                </div>
                            }.into_any()
                        }}
                    </div>
                    <div class="data-card">
                        <div class="card-header">
                            <span class="card-title">"Recent Transactions"</span>
                            <select
                                class="form-select"
                                style="width: auto; min-width: 120px;"
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
                        {move || {
                            let sort = sort_mode.get();
                            let mut items = transactions.get().clone();
                            items.sort_by(|a, b| match sort {
                                SortMode::Recent => b.created_at.cmp(&a.created_at),
                                SortMode::Oldest => a.created_at.cmp(&b.created_at),
                                SortMode::HighestValue => b.amount.partial_cmp(&a.amount).unwrap_or(std::cmp::Ordering::Equal),
                                SortMode::LowestValue => a.amount.partial_cmp(&b.amount).unwrap_or(std::cmp::Ordering::Equal),
                                _ => b.created_at.cmp(&a.created_at),
                            });
                            if items.is_empty() {
                                view! { <div class="tx-empty">"No transactions yet."</div> }.into_any()
                            } else {
                                view! {
                                    <div class="tx-statement-list">
                                        {items.into_iter().map(|t| {
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
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                    <div class="data-card">
                        <div class="card-header"><span class="card-title">"Recent Payments"</span></div>
                        {move || if payments.get().is_empty() {
                            view! { <div class="tx-empty">"No payments yet"</div> }.into_any()
                        } else {
                            view! {
                                <div class="tx-statement-list">
                                    {payments.get().into_iter().rev().take(5).map(|p| {
                                        let wallet = wallets.get().iter().find(|w| w.id == p.from_wallet_id).cloned();
                                        let contact = if p.direction == "send" {
                                            payees.get().iter().find(|c| c.id == p.to_contact_id).cloned()
                                        } else {
                                            payers.get().iter().find(|c| c.id == p.to_contact_id).cloned()
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
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }}
                    </div>
                }.into_any(),
            }}
        </div>
    }
}
