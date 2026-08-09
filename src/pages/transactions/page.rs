use crate::models::{ApprovalAction, ApprovalRecord, TransactionStatus};
use crate::pages::transactions::approval_panel::PrintRecordButton;
use crate::pages::transactions::transaction_detail::{BillForm, BillList, InvoiceForm, InvoiceList};
use crate::pages::transactions::transaction_form::{CreateTransactionRecord, PaymentForm};
use crate::pages::transactions::transaction_list::{PaymentList, TransactionList};
use crate::pages::transactions::transaction_summary::{
    CashFlowSummary, PayeeList, PayerList, SummaryCards, WalletCards, WalletDetails,
};
use crate::pages::transactions::{
    create_mock_bill, create_mock_transaction, Bill, BillStatus, Card, Contact, Invoice, Payment,
    Wallet,
};
use crate::stores::{create_action, use_app_store, use_transaction_store, use_undo_redo_store};
use crate::types::{ActionType, SortMode, TransactionType};
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
            from_name: "Red Holdings".to_string(),
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
            from_name: "Red Holdings".to_string(),
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
            from_name: "Red Holdings".to_string(),
            to_name: "Tech Supplies Inc".to_string(),
            description: "Equipment purchase invoice".to_string(),
        },
    ]
}

#[derive(Clone, Copy, PartialEq)]
enum TxFormMode {
    None,
    CreateTransaction,
    CreateInvoice,
    CreateBill,
    CreatePayment,
}

#[derive(Clone, Copy, PartialEq)]
enum TxViewCount {
    V10,
    V20,
    V50,
    V100,
}

impl TxViewCount {
    fn value(&self) -> usize {
        match self {
            TxViewCount::V10 => 10,
            TxViewCount::V20 => 20,
            TxViewCount::V50 => 50,
            TxViewCount::V100 => 100,
        }
    }
}

fn mock_bills() -> Vec<Bill> {
    vec![
        create_mock_bill(
            "BILL-2026-001",
            5_200.0,
            "AUD",
            "Tech Supplies Inc",
            "Red Holdings",
            "Office supplies Q1",
            BillStatus::AwaitingPayment,
            10,
        ),
        create_mock_bill(
            "BILL-2026-002",
            18_000.0,
            "AUD",
            "Property Manager",
            "Red Holdings",
            "Building maintenance",
            BillStatus::AwaitingApproval,
            20,
        ),
        create_mock_bill(
            "BILL-2026-003",
            3_400.0,
            "AUD",
            "Insurance Co",
            "Red Holdings",
            "Annual insurance premium",
            BillStatus::Overdue,
            -5,
        ),
        create_mock_bill(
            "BILL-2026-004",
            12_000.0,
            "USD",
            "Cloud Services Ltd",
            "Red Holdings",
            "Annual cloud infrastructure",
            BillStatus::Paid,
            -30,
        ),
    ]
}

#[component]
fn TransactionsOptionsPanel(
    on_new_transaction: Callback<()>,
    on_new_payment: Callback<()>,
    on_new_invoice: Callback<()>,
    on_new_bill: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="tx-options-panel" on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()>
            <button class="context-menu-item tx-options-item" on:click=move |_| on_new_transaction.run(())>
                "New Transaction"
            </button>
            <button class="context-menu-item tx-options-item" on:click=move |_| on_new_payment.run(())>
                "New Payment"
            </button>
            <button class="context-menu-item tx-options-item" on:click=move |_| on_new_invoice.run(())>
                "New Invoice"
            </button>
            <button class="context-menu-item tx-options-item" on:click=move |_| on_new_bill.run(())>
                "New Bill"
            </button>
        </div>
    }
}

#[component]
pub fn TransactionsPage() -> impl IntoView {
    let app_store = use_app_store();
    let transaction_store = use_transaction_store();
    let undo_store = use_undo_redo_store();

    let (active_tab, set_active_tab) = signal("dashboard".to_string());
    let (form_mode, set_form_mode) = signal(TxFormMode::None);
    let (payment_direction, set_payment_direction) = signal("send".to_string());
    let (contact_type, set_contact_type) = signal("payees".to_string());
    let (show_options, set_show_options) = signal(false);

    let (wallets, _set_wallets) = signal(mock_wallets());
    let (payees, _set_payees) = signal(mock_payees());
    let (payers, _set_payers) = signal(mock_payers());
    let (payments, set_payments) = signal(Vec::<Payment>::new());
    let (invoices, set_invoices) = signal(mock_invoices());
    let (bills, set_bills) = signal(mock_bills());
    let (sort_mode, set_sort_mode) = signal(SortMode::Recent);
    let (sort_ascending, set_sort_ascending) = signal(false);
    let (status_filter, set_status_filter) = signal::<Option<TransactionStatus>>(None);
    let (search_query, set_search_query) = signal(String::new());
    let (view_count, set_view_count) = signal(TxViewCount::V20);

    let filtered_transactions = move || {
        let search = search_query.get().to_lowercase();
        let vc = view_count.get().value();
        let mut items: Vec<_> = transaction_store
            .get()
            .transactions
            .iter()
            .filter(|t| {
                if let Some(s) = status_filter.get() {
                    if t.status != s {
                        return false;
                    }
                }
                if !search.is_empty() {
                    let desc = t.description.as_deref().unwrap_or("").to_lowercase();
                    let from = t.from_entity.name.to_lowercase();
                    let to = t.to_entity.name.to_lowercase();
                    if !desc.contains(&search) && !from.contains(&search) && !to.contains(&search) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        items.sort_by(|a, b| {
            let ord = match sort_mode.get() {
                SortMode::Recent => b.created_at.cmp(&a.created_at),
                SortMode::Oldest => a.created_at.cmp(&b.created_at),
                SortMode::HighestValue => b.amount.partial_cmp(&a.amount).unwrap_or(std::cmp::Ordering::Equal),
                SortMode::LowestValue => a.amount.partial_cmp(&b.amount).unwrap_or(std::cmp::Ordering::Equal),
                _ => b.created_at.cmp(&a.created_at),
            };
            if sort_ascending.get() { ord.reverse() } else { ord }
        });
        items.into_iter().take(vc).collect::<Vec<_>>()
    };

    // Seed the transaction store with mock transactions on first render.
    transaction_store.update(|store| {
        if store.transactions.is_empty() {
            store.add_transaction(create_mock_transaction(
                TransactionType::Purchase,
                125000.0,
                "Office equipment purchase",
                "Main Org",
                "Tech Supplies Inc",
                TransactionStatus::Executed,
            ));
            store.add_transaction(create_mock_transaction(
                TransactionType::Sale,
                450000.0,
                "Property sale - downtown plaza",
                "Real Estate Holdings",
                "Buyer Corp",
                TransactionStatus::Approved,
            ));
            store.add_transaction(create_mock_transaction(
                TransactionType::Rent,
                8500.0,
                "Monthly warehouse rent",
                "Tenant LLC",
                "Property Manager",
                TransactionStatus::Executed,
            ));
            store.add_transaction(create_mock_transaction(
                TransactionType::Fee,
                1200.0,
                "Bank processing fee",
                "Main Org",
                "Banking Partner",
                TransactionStatus::Executed,
            ));
            store.add_transaction(create_mock_transaction(
                TransactionType::Transfer,
                50000.0,
                "Inter-portfolio transfer",
                "Portfolio A",
                "Portfolio B",
                TransactionStatus::Pending,
            ));
        }
    });

    let current_user = move || app_store.get().current_user;
    let actor_name = move || current_user().name.clone();
    let actor_role = move || format!("{:?}", current_user().role);
    let actor_id = move || current_user().id;
    let org_id = move || current_user().organization_id;

    let record_action = move |action_type: ActionType, description: String| {
        let action = create_action(
            action_type,
            "Transaction",
            &description,
            actor_id(),
            &actor_name(),
            &actor_role(),
            org_id(),
            None,
        );
        undo_store.update(|u| u.record_action(action));
    };

    let on_payment = Callback::new(move |p: Payment| {
        let direction = p.direction.clone();
        set_payments.update(|list| list.push(p));
        record_action(ActionType::Create, format!("Created {} payment", direction));
    });
    let on_create_txn = Callback::new(move |t: crate::models::Transaction| {
        transaction_store.update(|store| store.add_transaction(t));
        set_form_mode.set(TxFormMode::None);
        record_action(ActionType::Create, "Created transaction record".to_string());
    });
    let on_create_invoice = Callback::new(move |inv: Invoice| {
        set_invoices.update(|list| list.insert(0, inv));
        set_form_mode.set(TxFormMode::None);
    });
    let on_create_bill = Callback::new(move |bill: Bill| {
        set_bills.update(|list| list.insert(0, bill));
        set_form_mode.set(TxFormMode::None);
    });

    let on_approval_action = Callback::new(
        move |(tx_id, action, comment): (Uuid, ApprovalAction, Option<String>)| {
            let user = current_user();
            let record = ApprovalRecord::new(
                user.id,
                user.name.clone(),
                format!("{:?}", user.role),
                action,
                comment,
            );
            let description = format!("{:?} transaction", action);
            transaction_store.update(|store| {
                let _ = store.record_approval(tx_id, record);
            });
            record_action(ActionType::Update, description);
        },
    );

    let set_active_tab_callback = Callback::new(move |key: String| {
        set_active_tab.set(key);
        set_form_mode.set(TxFormMode::None);
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
                on:click=move |_| {
                    let key_click = key_click.clone();
                    set_active_tab.set(key_click);
                    set_form_mode.set(TxFormMode::None);
                }
            >
                {label.clone()}
            </button>
        }
    };

    view! {
        <div class="home-screen tx-page">
            // Controls bar (portfolios-style, fixed below navbar)
            <div class="tx-controls-bar">
                <div class="tx-controls-left">
                    <button class="tx-action-btn" on:click=move |_| set_show_options.update(|v| *v = !*v)>
                        {move || if show_options.get() { "Close" } else { "+ New" }}
                    </button>
                    {move || if show_options.get() { view! {
                        <TransactionsOptionsPanel
                            on_new_transaction={Callback::new(move |_| {
                                set_form_mode.set(TxFormMode::CreateTransaction);
                                set_show_options.set(false);
                            })}
                            on_new_payment={Callback::new(move |_| {
                                set_active_tab.set("payments".to_string());
                                set_form_mode.set(TxFormMode::CreatePayment);
                                set_show_options.set(false);
                            })}
                            on_new_invoice={Callback::new(move |_| {
                                set_active_tab.set("invoices".to_string());
                                set_form_mode.set(TxFormMode::CreateInvoice);
                                set_show_options.set(false);
                            })}
                            on_new_bill={Callback::new(move |_| {
                                set_active_tab.set("bills".to_string());
                                set_form_mode.set(TxFormMode::CreateBill);
                                set_show_options.set(false);
                            })}
                        />
                    }.into_any() } else { ().into_any() }}
                </div>

                <div class="tx-controls-right">
                    // Search bar
                    <div class="tx-search-wrap">
                        <input
                            class="tx-search-input"
                            type="text"
                            placeholder="Search transactions…"
                            prop:value={move || search_query.get()}
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                        />
                        {move || if !search_query.get().is_empty() {
                            view! {
                                <button class="tx-search-clear" on:click=move |_| set_search_query.set(String::new())>"×"</button>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>

                    // Sort dropdown
                    <select
                        class="tx-ctrl-select"
                        aria-label="Sort transactions"
                        prop:value={move || match sort_mode.get() {
                            SortMode::Recent => "sort_recent",
                            SortMode::Oldest => "sort_oldest",
                            SortMode::HighestValue => "sort_highest",
                            SortMode::LowestValue => "sort_lowest",
                            _ => "sort_recent",
                        }.to_string()}
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            let mode = match v.as_str() {
                                "sort_oldest" => SortMode::Oldest,
                                "sort_highest" => SortMode::HighestValue,
                                "sort_lowest" => SortMode::LowestValue,
                                _ => SortMode::Recent,
                            };
                            set_sort_mode.set(mode);
                        }
                    >
                        <option value="sort_recent">"Sort: Recent"</option>
                        <option value="sort_oldest">"Sort: Oldest"</option>
                        <option value="sort_highest">"Sort: High ₵"</option>
                        <option value="sort_lowest">"Sort: Low ₵"</option>
                    </select>

                    // Sort direction toggle
                    <button
                        class="tx-ctrl-btn sort-direction-btn"
                        aria-label={move || if sort_ascending.get() { "Ascending" } else { "Descending" }}
                        title={move || if sort_ascending.get() { "Ascending ↑" } else { "Descending ↓" }}
                        on:click=move |_| set_sort_ascending.update(|v| *v = !*v)
                    >
                        {move || if sort_ascending.get() { "↑" } else { "↓" }}
                    </button>

                    // View count selector
                    <select
                        class="tx-ctrl-select"
                        aria-label="View count"
                        prop:value={move || match view_count.get() {
                            TxViewCount::V10 => "view_10",
                            TxViewCount::V20 => "view_20",
                            TxViewCount::V50 => "view_50",
                            TxViewCount::V100 => "view_100",
                        }.to_string()}
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            let vc = match v.as_str() {
                                "view_10" => TxViewCount::V10,
                                "view_20" => TxViewCount::V20,
                                "view_50" => TxViewCount::V50,
                                "view_100" => TxViewCount::V100,
                                _ => TxViewCount::V20,
                            };
                            set_view_count.set(vc);
                        }
                    >
                        <option value="view_10">"View: 10"</option>
                        <option value="view_20">"View: 20"</option>
                        <option value="view_50">"View: 50"</option>
                        <option value="view_100">"View: 100"</option>
                    </select>

                    <PrintRecordButton />
                </div>
            </div>

            // Tab bar (Xero-like navigation)
            <div class="tx-tabs">
                {tab_btn("Dashboard", "dashboard")}
                {tab_btn("Bank Accounts", "wallets")}
                {tab_btn("Invoices", "invoices")}
                {tab_btn("Bills", "bills")}
                {tab_btn("Contacts", "contacts")}
                {tab_btn("Transactions", "transactions")}
            </div>

            {move || if form_mode.get() == TxFormMode::CreateTransaction {
                view! {
                    <div class="data-card">
                        <div class="card-header">
                            <span class="card-title">"New Transaction"</span>
                            <button class="tx-action-btn tx-action-btn-small" on:click=move |_| set_form_mode.set(TxFormMode::None)>"Cancel"</button>
                        </div>
                        <CreateTransactionRecord _wallets={wallets.get()} on_create={on_create_txn.clone()} />
                    </div>
                }.into_any()
            } else if form_mode.get() == TxFormMode::CreateInvoice {
                view! {
                    <div class="data-card">
                        <div class="card-header">
                            <span class="card-title">"New Invoice"</span>
                            <button class="tx-action-btn tx-action-btn-small" on:click=move |_| set_form_mode.set(TxFormMode::None)>"Cancel"</button>
                        </div>
                        <InvoiceForm on_create={on_create_invoice.clone()} />
                    </div>
                }.into_any()
            } else if form_mode.get() == TxFormMode::CreateBill {
                view! {
                    <div class="data-card">
                        <div class="card-header">
                            <span class="card-title">"New Bill"</span>
                            <button class="tx-action-btn tx-action-btn-small" on:click=move |_| set_form_mode.set(TxFormMode::None)>"Cancel"</button>
                        </div>
                        <BillForm on_create={on_create_bill.clone()} />
                    </div>
                }.into_any()
            } else if form_mode.get() == TxFormMode::CreatePayment {
                view! {
                    <div class="data-card">
                        <div class="card-header">
                            <span class="card-title">{move || if payment_direction.get() == "send" { "Send Payment" } else { "Receive Payment" }}</span>
                            <div class="tx-toggle">
                                <button class="tx-toggle-btn" class:active={move || payment_direction.get() == "send"} on:click=move |_| set_payment_direction.set("send".to_string())>"Out"</button>
                                <button class="tx-toggle-btn" class:active={move || payment_direction.get() == "receive"} on:click=move |_| set_payment_direction.set("receive".to_string())>"In"</button>
                            </div>
                        </div>
                        {move || if payment_direction.get() == "send" {
                            view! { <PaymentForm wallets={wallets.get()} contacts={payees.get()} direction="send" on_submit={on_payment.clone()} /> }.into_any()
                        } else {
                            view! { <PaymentForm wallets={wallets.get()} contacts={payers.get()} direction="receive" on_submit={on_payment.clone()} /> }.into_any()
                        }}
                        <div class="card-header" style="margin-top:12px">
                            <button class="tx-action-btn tx-action-btn-small" on:click=move |_| set_form_mode.set(TxFormMode::None)>"Done"</button>
                        </div>
                    </div>
                }.into_any()
            } else {
                match active_tab.get().as_str() {
                    "dashboard" => view! {
                        <CashFlowSummary
                            transactions={transaction_store.get().transactions.clone()}
                            invoices={invoices.get()}
                            bills={bills.get()}
                            wallets={wallets.get()}
                        />
                        <SummaryCards invoices={invoices.get()} set_active_tab={set_active_tab_callback.clone()} />
                        <div class="data-card">
                            <div class="card-header">
                                <span class="card-title">"Recent Transactions"</span>
                                <div class="tx-header-controls">
                                    <select
                                        class="form-select tx-filter-select"
                                        on:change=move |ev| {
                                            let v = event_target_value(&ev);
                                            let f = match v.as_str() {
                                                "draft" => Some(TransactionStatus::Draft),
                                                "pending" => Some(TransactionStatus::Pending),
                                                "approved" => Some(TransactionStatus::Approved),
                                                "rejected" => Some(TransactionStatus::Rejected),
                                                "executed" => Some(TransactionStatus::Executed),
                                                "cancelled" => Some(TransactionStatus::Cancelled),
                                                _ => None,
                                            };
                                            set_status_filter.set(f);
                                        }
                                    >
                                        <option value="">"All statuses"</option>
                                        <option value="draft">"Draft"</option>
                                        <option value="pending">"Pending"</option>
                                        <option value="approved">"Approved"</option>
                                        <option value="rejected">"Rejected"</option>
                                        <option value="executed">"Executed"</option>
                                        <option value="cancelled">"Cancelled"</option>
                                    </select>
                                </div>
                            </div>
                            <div class="tx-status-filters" role="group" aria-label="Filter transactions by status">
                                {[("All", None), ("Draft", Some(TransactionStatus::Draft)), ("Pending", Some(TransactionStatus::Pending)), ("Approved", Some(TransactionStatus::Approved)), ("Rejected", Some(TransactionStatus::Rejected)), ("Executed", Some(TransactionStatus::Executed))]
                                    .iter().map(|(label, status)| {
                                        let label = *label;
                                        let status_active = status.clone();
                                        let status_aria = status.clone();
                                        let status_click = status.clone();
                                        view! {
                                            <button
                                                class="tx-status-filter-btn"
                                                class:active={move || status_filter.get() == status_active}
                                                aria-pressed={move || status_filter.get() == status_aria}
                                                on:click=move |_| set_status_filter.set(status_click.clone())
                                            >
                                                {label}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                            </div>
                            {move || view! {
                                <TransactionList
                                    transactions={filtered_transactions()}
                                    sort_mode={sort_mode.get()}
                                    current_user={current_user()}
                                    on_approval_action={on_approval_action.clone()}
                                />
                            }.into_any()}
                        </div>
                        <div class="data-card">
                            <div class="card-header"><span class="card-title">"Recent Payments"</span></div>
                            {move || view! {
                                <PaymentList payments={payments.get()} wallets={wallets.get()} payees={payees.get()} payers={payers.get()} />
                            }.into_any()}
                        </div>
                    }.into_any(),
                    "transactions" => view! {
                        <div class="data-card">
                            <div class="card-header">
                                <span class="card-title">"All Transactions"</span>
                                <div class="tx-header-controls">
                                    <select
                                        class="form-select tx-filter-select"
                                        on:change=move |ev| {
                                            let v = event_target_value(&ev);
                                            let f = match v.as_str() {
                                                "draft" => Some(TransactionStatus::Draft),
                                                "pending" => Some(TransactionStatus::Pending),
                                                "approved" => Some(TransactionStatus::Approved),
                                                "rejected" => Some(TransactionStatus::Rejected),
                                                "executed" => Some(TransactionStatus::Executed),
                                                "cancelled" => Some(TransactionStatus::Cancelled),
                                                _ => None,
                                            };
                                            set_status_filter.set(f);
                                        }
                                    >
                                        <option value="">"All statuses"</option>
                                        <option value="draft">"Draft"</option>
                                        <option value="pending">"Pending"</option>
                                        <option value="approved">"Approved"</option>
                                        <option value="rejected">"Rejected"</option>
                                        <option value="executed">"Executed"</option>
                                        <option value="cancelled">"Cancelled"</option>
                                    </select>
                                </div>
                            </div>
                            <div class="tx-status-filters" role="group" aria-label="Filter transactions by status">
                                {[("All", None), ("Draft", Some(TransactionStatus::Draft)), ("Pending", Some(TransactionStatus::Pending)), ("Approved", Some(TransactionStatus::Approved)), ("Rejected", Some(TransactionStatus::Rejected)), ("Executed", Some(TransactionStatus::Executed))]
                                    .iter().map(|(label, status)| {
                                        let label = *label;
                                        let status_active = status.clone();
                                        let status_aria = status.clone();
                                        let status_click = status.clone();
                                        view! {
                                            <button
                                                class="tx-status-filter-btn"
                                                class:active={move || status_filter.get() == status_active}
                                                aria-pressed={move || status_filter.get() == status_aria}
                                                on:click=move |_| set_status_filter.set(status_click.clone())
                                            >
                                                {label}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                            </div>
                            {move || view! {
                                <TransactionList
                                    transactions={filtered_transactions()}
                                    sort_mode={sort_mode.get()}
                                    current_user={current_user()}
                                    on_approval_action={on_approval_action.clone()}
                                />
                            }.into_any()}
                        </div>
                    }.into_any(),
                    "payments" => view! {
                        <div class="data-card">
                            <div class="card-header">
                                <span class="card-title">{move || if payment_direction.get() == "send" { "Send Payment" } else { "Receive Payment" }}</span>
                                <div class="tx-toggle">
                                    <button class="tx-toggle-btn" class:active={move || payment_direction.get() == "send"} on:click=move |_| set_payment_direction.set("send".to_string())>"Out"</button>
                                    <button class="tx-toggle-btn" class:active={move || payment_direction.get() == "receive"} on:click=move |_| set_payment_direction.set("receive".to_string())>"In"</button>
                                </div>
                            </div>
                            {move || if payment_direction.get() == "send" {
                                view! { <PaymentForm wallets={wallets.get()} contacts={payees.get()} direction="send" on_submit={on_payment.clone()} /> }.into_any()
                            } else {
                                view! { <PaymentForm wallets={wallets.get()} contacts={payers.get()} direction="receive" on_submit={on_payment.clone()} /> }.into_any()
                            }}
                        </div>
                        <div class="data-card">
                            <div class="card-header"><span class="card-title">"Recent Payments"</span></div>
                            {move || view! {
                                <PaymentList payments={payments.get()} wallets={wallets.get()} payees={payees.get()} payers={payers.get()} />
                            }.into_any()}
                        </div>
                    }.into_any(),
                    "invoices" => view! {
                        <InvoiceList
                            invoices={invoices.get()}
                            on_new_invoice={Some(Callback::new(move |_| set_form_mode.set(TxFormMode::CreateInvoice)))}
                        />
                    }.into_any(),
                    "bills" => view! {
                        <BillList
                            bills={bills.get()}
                            on_new_bill={Some(Callback::new(move |_| set_form_mode.set(TxFormMode::CreateBill)))}
                        />
                    }.into_any(),
                    "contacts" => view! {
                        <div class="data-card">
                            <div class="card-header">
                                <span class="card-title">{move || if contact_type.get() == "payees" { "Payees" } else { "Payers" }}</span>
                                <div class="tx-toggle">
                                    <button class="tx-toggle-btn" class:active={move || contact_type.get() == "payees"} on:click=move |_| set_contact_type.set("payees".to_string())>"Payees"</button>
                                    <button class="tx-toggle-btn" class:active={move || contact_type.get() == "payers"} on:click=move |_| set_contact_type.set("payers".to_string())>"Payers"</button>
                                </div>
                            </div>
                            {move || if contact_type.get() == "payees" {
                                view! { <PayeeList contacts={payees.get()} /> }.into_any()
                            } else {
                                view! { <PayerList contacts={payers.get()} /> }.into_any()
                            }}
                        </div>
                    }.into_any(),
                    "wallets" => view! {
                        <div class="tx-wallets-section">
                            <WalletCards wallets={wallets.get()} />
                            <WalletDetails wallets={wallets.get()} />
                        </div>
                    }.into_any(),
                    _ => view! {
                        <CashFlowSummary
                            transactions={transaction_store.get().transactions.clone()}
                            invoices={invoices.get()}
                            bills={bills.get()}
                            wallets={wallets.get()}
                        />
                        <div class="data-card">
                            <div class="card-header"><span class="card-title">"Dashboard"</span></div>
                            <div class="tx-empty">"Select a tab to get started."</div>
                        </div>
                    }.into_any(),
                }
            }}
        </div>
    }
}
