use crate::pages::transactions::{
    format_date, format_dollars, invoice_status_class, Invoice, InvoiceStatus,
};
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub(crate) fn InvoiceForm(on_create: Callback<Invoice>) -> impl IntoView {
    let (number, set_number) = signal(String::new());
    let (amount, set_amount) = signal(String::new());
    let (currency, set_currency) = signal("AUD".to_string());
    let (to_name, set_to_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (due_days, set_due_days) = signal("30".to_string());

    let submit = move |_| {
        let value: f64 = amount.get().parse().unwrap_or(0.0);
        if value <= 0.0 || to_name.get().trim().is_empty() {
            return;
        }
        let days: i64 = due_days.get().parse().unwrap_or(30);
        let invoice = Invoice {
            id: Uuid::new_v4(),
            number: if number.get().trim().is_empty() {
                format!(
                    "INV-{}",
                    Uuid::new_v4().to_string().split('-').next().unwrap()
                )
            } else {
                number.get()
            },
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
pub(crate) fn InvoiceList(
    invoices: Vec<Invoice>,
    #[prop(default = None)] on_new_invoice: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="data-card">
            <div class="card-header">
                <span class="card-title">"Invoices"</span>
                {match on_new_invoice {
                    Some(cb) => view! {
                        <button class="tx-action-btn tx-action-btn-small" on:click=move |_| cb.run(())>"+ New Invoice"</button>
                    }.into_any(),
                    None => ().into_any(),
                }}
            </div>
            {if invoices.is_empty() {
                view! { <div class="tx-empty">"No invoices yet."</div> }.into_any()
            } else {
                view! {
                    <div class="tx-invoice-list">
                        {invoices.into_iter().map(|inv| {
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
    }
}
