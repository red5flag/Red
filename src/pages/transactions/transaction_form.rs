use crate::models::{EntityReference, EntityType, Transaction};
use crate::pages::transactions::{Contact, Payment, Wallet};
use crate::types::{Currency, TransactionType};
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub(crate) fn PaymentForm(
    wallets: Vec<Wallet>,
    contacts: Vec<Contact>,
    direction: &'static str,
    on_submit: Callback<Payment>,
) -> impl IntoView {
    let (from_wallet, set_from_wallet) =
        signal(wallets.first().map(|w| w.id).unwrap_or_else(Uuid::new_v4));
    let (from_card, set_from_card) = signal(None::<Uuid>);
    let (to_contact, set_to_contact) =
        signal(contacts.first().map(|c| c.id).unwrap_or_else(Uuid::new_v4));
    let (amount, set_amount) = signal(String::new());
    let (currency, set_currency) = signal("AUD".to_string());
    let (is_routine, set_is_routine) = signal(false);
    let (interval, set_interval) = signal("Monthly".to_string());
    let (is_auto, set_is_auto) = signal(false);

    let submit = move |_| {
        let value: f64 = amount.get().parse().unwrap_or(0.0);
        if value <= 0.0 {
            return;
        }
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
            <super::transaction_filters::WalletSelector
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
pub(crate) fn CreateTransactionRecord(
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
        if value <= 0.0 {
            return;
        }
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
            "Pending" => crate::models::TransactionStatus::Pending,
            "Approved" => crate::models::TransactionStatus::Approved,
            "Executed" => crate::models::TransactionStatus::Executed,
            "Cancelled" => crate::models::TransactionStatus::Cancelled,
            _ => crate::models::TransactionStatus::Draft,
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
            description: if description.get().is_empty() {
                None
            } else {
                Some(description.get())
            },
            from_entity: EntityReference {
                entity_type: EntityType::Organization,
                entity_id: Uuid::new_v4(),
                name: if from_name.get().is_empty() {
                    "Internal".to_string()
                } else {
                    from_name.get()
                },
            },
            to_entity: EntityReference {
                entity_type: EntityType::External,
                entity_id: Uuid::new_v4(),
                name: if to_name.get().is_empty() {
                    "External".to_string()
                } else {
                    to_name.get()
                },
            },
            related_portfolio_id: None,
            related_asset_group_id: None,
            related_asset_id: None,
            executed_by: Uuid::new_v4(),
            status: st.clone(),
            created_at: Utc::now(),
            executed_at: if st == crate::models::TransactionStatus::Executed {
                Some(Utc::now())
            } else {
                None
            },
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
