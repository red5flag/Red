use crate::models::{ApprovalAction, ApprovalRecord, TransactionStatus};
use crate::pages::transactions::approval_panel::PrintRecordButton;
use crate::pages::transactions::transaction_detail::{BillForm, BillList, InvoiceForm};
use crate::pages::transactions::transaction_form::{CreateTransactionRecord, PaymentForm};
use crate::pages::transactions::transaction_list::TransactionList;
use crate::pages::transactions::{
    create_mock_bill, create_mock_transaction, currency_symbol, AccountType, Bill,
    BillStatus, Card, Contact, Invoice, Payment, Wallet,
};
use crate::stores::{
    create_action, use_account_store, use_app_store, use_organization_store,
    use_transaction_store, use_undo_redo_store,
};
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
                    account_id: Uuid::nil(),
                    label: "Business Debit".to_string(),
                    last4: "4821".to_string(),
                    card_type: "Debit".to_string(),
                    active: true,
                },
                Card {
                    id: Uuid::new_v4(),
                    account_id: Uuid::nil(),
                    label: "Business Credit".to_string(),
                    last4: "9912".to_string(),
                    card_type: "Credit".to_string(),
                    active: true,
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
                account_id: Uuid::nil(),
                label: "Coinspot AUD".to_string(),
                last4: "SPOT".to_string(),
                card_type: "Crypto".to_string(),
                active: true,
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
                account_id: Uuid::nil(),
                label: "Coinbase USD".to_string(),
                last4: "BASE".to_string(),
                card_type: "Crypto".to_string(),
                active: true,
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
                account_id: Uuid::nil(),
                label: "BTC Wallet".to_string(),
                last4: "BTC".to_string(),
                card_type: "Crypto".to_string(),
                active: true,
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
    let account_store = use_account_store();
    let organization_store = use_organization_store();
    let undo_store = use_undo_redo_store();

    let (form_mode, set_form_mode) = signal(TxFormMode::None);
    let (payment_direction, set_payment_direction) = signal("send".to_string());
    let (show_options, set_show_options) = signal(false);

    // Dropdown-driven navigation state
    let (selected_section, set_selected_section) = signal("finance".to_string());
    let (selected_subview, set_selected_subview) = signal("transactions".to_string());
    let (selected_account_id, set_selected_account_id) = signal::<Option<Uuid>>(None);
    let (selected_org_id, set_selected_org_id) = signal::<Option<Uuid>>(app_store.get().current_user.organization_id);
    let (selected_portfolio_id, set_selected_portfolio_id) = signal::<Option<Uuid>>(None);

    let (wallets, _set_wallets) = signal(mock_wallets());
    let (payees, _set_payees) = signal(mock_payees());
    let (payers, _set_payers) = signal(mock_payers());
    let (_payments, set_payments) = signal(Vec::<Payment>::new());
    let (_invoices, set_invoices) = signal(mock_invoices());
    let (bills, set_bills) = signal(mock_bills());
    let (sort_mode, set_sort_mode) = signal(SortMode::Recent);
    let (sort_ascending, _set_sort_ascending) = signal(false);
    let (status_filter, _set_status_filter) = signal::<Option<TransactionStatus>>(None);
    let (view_count, _set_view_count) = signal(TxViewCount::V20);

    let filtered_transactions = move || {
        let vc = view_count.get().value();
        let org_id = selected_org_id.get();
        let portfolio_id = selected_portfolio_id.get();
        let portfolios = app_store.get().portfolios.clone();
        let org_portfolio_ids: std::collections::HashSet<Uuid> = if let Some(oid) = org_id {
            portfolios.iter().filter(|p| p.organization_id == Some(oid)).map(|p| p.id).collect()
        } else {
            std::collections::HashSet::new()
        };
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
                // Filter by portfolio if selected
                if let Some(pid) = portfolio_id {
                    if t.related_portfolio_id != Some(pid) {
                        return false;
                    }
                }
                // Filter by organization: check if the tx's portfolio belongs to the org
                if let Some(_oid) = org_id {
                    if portfolio_id.is_none() {
                        // Only apply org filter when no specific portfolio is selected
                        if let Some(tx_pid) = t.related_portfolio_id {
                            if !org_portfolio_ids.contains(&tx_pid) {
                                return false;
                            }
                        } else {
                            return false;
                        }
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
                SortMode::HighestValue => b
                    .amount
                    .partial_cmp(&a.amount)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortMode::LowestValue => a
                    .amount
                    .partial_cmp(&b.amount)
                    .unwrap_or(std::cmp::Ordering::Equal),
                _ => b.created_at.cmp(&a.created_at),
            };
            if sort_ascending.get() {
                ord.reverse()
            } else {
                ord
            }
        });
        items.into_iter().take(vc).collect::<Vec<_>>()
    };

    // Seed accounts and transactions on first render.
    let current_org_id = app_store.get().current_user.organization_id;
    account_store.update(|store| {
        store.seed_mock_data(current_org_id);
    });

    transaction_store.update(|store| {
        if store.transactions.is_empty() {
            let accounts = account_store.get().accounts_for_org_owned(current_org_id);
            let bank_account_id = accounts
                .iter()
                .find(|a| matches!(a.account_type, AccountType::Bank))
                .map(|a| a.id);
            let bank_card_id = accounts
                .iter()
                .find(|a| matches!(a.account_type, AccountType::Bank))
                .and_then(|a| a.cards.first())
                .map(|c| c.id);
            let crypto_account_id = accounts
                .iter()
                .find(|a| matches!(a.account_type, AccountType::Crypto))
                .map(|a| a.id);
            let savings_account_id = accounts
                .iter()
                .find(|a| matches!(a.account_type, AccountType::Savings))
                .map(|a| a.id);

            let mut tx1 = create_mock_transaction(
                TransactionType::Purchase,
                125000.0,
                "Office equipment purchase",
                "Main Org",
                "Tech Supplies Inc",
                TransactionStatus::Executed,
            );
            tx1.account_id = bank_account_id;
            tx1.card_id = bank_card_id;

            let mut tx2 = create_mock_transaction(
                TransactionType::Sale,
                450000.0,
                "Property sale - downtown plaza",
                "Real Estate Holdings",
                "Buyer Corp",
                TransactionStatus::Approved,
            );
            tx2.account_id = savings_account_id;

            let mut tx3 = create_mock_transaction(
                TransactionType::Rent,
                8500.0,
                "Monthly warehouse rent",
                "Tenant LLC",
                "Property Manager",
                TransactionStatus::Executed,
            );
            tx3.account_id = bank_account_id;
            tx3.card_id = bank_card_id;

            let mut tx4 = create_mock_transaction(
                TransactionType::Fee,
                1200.0,
                "Bank processing fee",
                "Main Org",
                "Banking Partner",
                TransactionStatus::Executed,
            );
            tx4.account_id = bank_account_id;

            let mut tx5 = create_mock_transaction(
                TransactionType::Transfer,
                50000.0,
                "Inter-account transfer",
                "Business Bank",
                "Savings Account",
                TransactionStatus::Pending,
            );
            tx5.account_id = bank_account_id;

            let mut tx6 = create_mock_transaction(
                TransactionType::Transfer,
                5000.0,
                "Crypto purchase",
                "Business Bank",
                "Coinspot",
                TransactionStatus::Executed,
            );
            tx6.account_id = crypto_account_id;

            store.add_transaction(tx1);
            store.add_transaction(tx2);
            store.add_transaction(tx3);
            store.add_transaction(tx4);
            store.add_transaction(tx5);
            store.add_transaction(tx6);
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

    let all_accounts = move || {
        let org_id = selected_org_id.get().or(app_store.get().current_user.organization_id);
        account_store.get().accounts_for_org_owned(org_id)
    };

    let all_organizations = move || organization_store.get().organizations.clone();

    let portfolios_for_org = move || {
        let org_id = selected_org_id.get();
        app_store.get().portfolios.iter()
            .filter(|p| {
                if let Some(oid) = org_id {
                    p.organization_id == Some(oid) || p.secondary_organization_ids.contains(&oid)
                } else {
                    true
                }
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    let users_for_org = move || {
        let org_id = selected_org_id.get();
        organization_store.get().organization_users.iter()
            .filter(|u| u.organization_id == org_id)
            .cloned()
            .collect::<Vec<_>>()
    };

    let workers_for_org = move || {
        use crate::types::UserRole;
        users_for_org().into_iter()
            .filter(|u| matches!(u.role, UserRole::Worker | UserRole::DocumentWorker))
            .collect::<Vec<_>>()
    };

    let managers_for_org = move || {
        use crate::types::UserRole;
        users_for_org().into_iter()
            .filter(|u| matches!(u.role, UserRole::Manager | UserRole::SeniorManager | UserRole::Director))
            .collect::<Vec<_>>()
    };

    // Derived node key for content panel routing
    let selected_node = move || {
        let section = selected_section.get();
        let subview = selected_subview.get();
        if section == "finance" {
            if subview == "accounts" {
                if let Some(id) = selected_account_id.get() {
                    return format!("finance.accounts.{}", id);
                }
                return "finance.accounts".to_string();
            }
            if subview == "cards" {
                return "finance.cards".to_string();
            }
            return format!("finance.{}", subview);
        }
        section
    };

    let currency_label = |c: &crate::types::Currency| format!("{:?}", c);

    view! {
        <div class="home-screen tx-page tx-tree-page home-screen-tx">
            <div class="portfolio-controls-bar tx-controls-bar">
                    <button class="nav-portfolio-btn" on:click=move |_| set_show_options.update(|v| *v = !*v)>
                        {move || if show_options.get() { "Close" } else { "+ New" }}
                    </button>
                    {move || if show_options.get() { view! {
                        <TransactionsOptionsPanel
                            on_new_transaction={Callback::new(move |_| {
                                set_form_mode.set(TxFormMode::CreateTransaction);
                                set_show_options.set(false);
                            })}
                            on_new_payment={Callback::new(move |_| {
                                set_form_mode.set(TxFormMode::CreatePayment);
                                set_show_options.set(false);
                            })}
                            on_new_invoice={Callback::new(move |_| {
                                set_form_mode.set(TxFormMode::CreateInvoice);
                                set_show_options.set(false);
                            })}
                            on_new_bill={Callback::new(move |_| {
                                set_form_mode.set(TxFormMode::CreateBill);
                                set_show_options.set(false);
                            })}
                        />
                    }.into_any() } else { ().into_any() }}
                    <select class="nav-portfolio-btn nav-portfolio-sort" aria-label="Section"
                        prop:value={move || selected_section.get()}
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            set_selected_section.set(v.clone());
                            if v == "finance" {
                                set_selected_subview.set("transactions".to_string());
                            } else {
                                set_selected_subview.set(String::new());
                            }
                            set_selected_account_id.set(None);
                        }
                    >
                        <option value="finance">"Finance"</option>
                        <option value="workers">"Workers"</option>
                        <option value="teams">"Teams"</option>
                        <option value="managers">"Managers"</option>
                        <option value="assets">"Assets"</option>
                        <option value="projects">"Projects"</option>
                        <option value="channels">"Channels"</option>
                    </select>
                    {move || if selected_section.get() == "finance" {
                        view! {
                            <select class="nav-portfolio-btn nav-portfolio-sort" aria-label="View"
                                prop:value={move || selected_subview.get()}
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    set_selected_subview.set(v);
                                    set_selected_account_id.set(None);
                                }
                            >
                                <option value="transactions">"Transactions"</option>
                                <option value="transfers">"Transfers"</option>
                                <option value="expenses">"Expenses"</option>
                                <option value="reimbursements">"Reimbursements"</option>
                                <option value="bills">"Bills"</option>
                                <option value="budgets">"Budgets"</option>
                                <option value="reports">"Reports"</option>
                                <option value="accounts">"Accounts"</option>
                                <option value="cards">"Cards"</option>
                            </select>
                        }.into_any()
                    } else { ().into_any() }}
                    {move || if selected_section.get() == "finance" && selected_subview.get() == "accounts" {
                        let accts = all_accounts();
                        view! {
                            <select class="nav-portfolio-btn nav-portfolio-sort" aria-label="Account"
                                prop:value={move || selected_account_id.get().map(|id| id.to_string()).unwrap_or_default()}
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    if v.is_empty() {
                                        set_selected_account_id.set(None);
                                    } else if let Ok(id) = Uuid::parse_str(&v) {
                                        set_selected_account_id.set(Some(id));
                                    }
                                }
                            >
                                <option value="">"All Accounts"</option>
                                {accts.iter().map(|a| {
                                    let id_str = a.id.to_string();
                                    let label = format!("{} — {}", a.name, a.account_type.label());
                                    view! {
                                        <option value={id_str}>{label}</option>
                                    }
                                }).collect::<Vec<_>>()}
                            </select>
                        }.into_any()
                    } else { ().into_any() }}
                    <select class="nav-portfolio-btn nav-portfolio-sort" aria-label="Organization"
                        prop:value={move || selected_org_id.get().map(|id| id.to_string()).unwrap_or_default()}
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            if v.is_empty() {
                                set_selected_org_id.set(None);
                            } else if let Ok(id) = Uuid::parse_str(&v) {
                                set_selected_org_id.set(Some(id));
                            }
                            set_selected_portfolio_id.set(None);
                            set_selected_account_id.set(None);
                        }
                    >
                        <option value="">"All Orgs"</option>
                        {move || all_organizations().iter().map(|org| {
                            let id_str = org.id.to_string();
                            let name = org.name.clone();
                            view! {
                                <option value={id_str.clone()}>{name}</option>
                            }
                        }).collect::<Vec<_>>()}
                    </select>
                    {move || {
                        let portfolios = portfolios_for_org();
                        if portfolios.is_empty() {
                            ().into_any()
                        } else {
                            view! {
                                <select class="nav-portfolio-btn nav-portfolio-sort" aria-label="Portfolio"
                                    prop:value={move || selected_portfolio_id.get().map(|id| id.to_string()).unwrap_or_default()}
                                    on:change=move |ev| {
                                        let v = event_target_value(&ev);
                                        if v.is_empty() {
                                            set_selected_portfolio_id.set(None);
                                        } else if let Ok(id) = Uuid::parse_str(&v) {
                                            set_selected_portfolio_id.set(Some(id));
                                        }
                                    }
                                >
                                    <option value="">"All Portfolios"</option>
                                    {portfolios.iter().map(|p| {
                                        let id_str = p.id.to_string();
                                        let name = p.name.clone();
                                        view! {
                                            <option value={id_str.clone()}>{name}</option>
                                        }
                                    }).collect::<Vec<_>>()}
                                </select>
                            }.into_any()
                        }
                    }}
                    <select class="nav-portfolio-btn nav-portfolio-sort" aria-label="Sort transactions"
                        prop:value={move || match sort_mode.get() {
                            SortMode::Recent => "sort_recent",
                            SortMode::Oldest => "sort_oldest",
                            SortMode::HighestValue => "sort_highest",
                            SortMode::LowestValue => "sort_lowest",
                            _ => "sort_recent",
                        }.to_string()}
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            set_sort_mode.set(match v.as_str() {
                                "sort_oldest" => SortMode::Oldest,
                                "sort_highest" => SortMode::HighestValue,
                                "sort_lowest" => SortMode::LowestValue,
                                _ => SortMode::Recent,
                            });
                        }
                    >
                        <option value="sort_recent">"Sort: Recent"</option>
                        <option value="sort_oldest">"Sort: Oldest"</option>
                        <option value="sort_highest">"Sort: High Val"</option>
                        <option value="sort_lowest">"Sort: Low Val"</option>
                    </select>
                    <PrintRecordButton />
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
                view! {
                    <div class="tx-content-panel">
                        {move || {
                            let node = selected_node();
                            let sm = sort_mode.get();
                            let user = current_user();
                            let on_app = on_approval_action.clone();

                            if node == "finance.transactions" {
                                let txns = filtered_transactions();
                                view! {
                                    <div class="data-card">
                                        <div class="card-header"><span class="card-title">"Transactions"</span></div>
                                        <TransactionList transactions={txns} sort_mode={sm} current_user={user} on_approval_action={on_app} />
                                    </div>
                                }.into_any()
                            } else if node == "finance.bills" {
                                view! {
                                    <BillList bills={bills.get()} on_new_bill={Some(Callback::new(move |_| set_form_mode.set(TxFormMode::CreateBill)))} />
                                }.into_any()
                            } else if node.starts_with("finance.accounts.") {
                                let id_str = node.strip_prefix("finance.accounts.").unwrap_or("");
                                let accts = all_accounts();
                                if let Some(a) = accts.iter().find(|a| a.id.to_string() == id_str) {
                                    let txns = account_store.get().all_transactions_for_account(a.id, &transaction_store.get().transactions);
                                    view! {
                                        <div class="data-card">
                                            <div class="card-header">
                                                <span class="card-title">{format!("{} — {}", a.name, a.account_type.label())}</span>
                                                <span class="card-meta">{format!("{}{:.2}", currency_symbol(&currency_label(&a.currency)), a.balance)}</span>
                                            </div>
                                            <TransactionList transactions={txns} sort_mode={sm} current_user={user} on_approval_action={on_app} />
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div class="tx-empty">"Account not found."</div> }.into_any()
                                }
                            } else if node == "finance.accounts" {
                                let accts = all_accounts();
                                view! {
                                    <div class="data-card">
                                        <div class="card-header"><span class="card-title">"All Accounts"</span></div>
                                        {accts.iter().map(|a| {
                                            let name = a.name.clone();
                                            let bal = format!("{}{:.2}", currency_symbol(&currency_label(&a.currency)), a.balance);
                                            view! {
                                                <div class="tx-account-row">
                                                    <span class="tx-account-name">{name}</span>
                                                    <span class="tx-account-type">{a.account_type.label()}</span>
                                                    <span class="tx-account-balance">{bal}</span>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            } else if node == "finance.cards" {
                                view! {
                                    <div class="data-card">
                                        <div class="card-header"><span class="card-title">"Cards"</span></div>
                                        <div class="tx-empty">"Select an account to view its cards."</div>
                                    </div>
                                }.into_any()
                            } else if node == "workers" {
                                let workers = workers_for_org();
                                view! {
                                    <div class="data-card">
                                        <div class="card-header"><span class="card-title">"Workers"</span></div>
                                        {if workers.is_empty() {
                                            view! { <div class="tx-empty">"No workers in this organization."</div> }.into_any()
                                        } else {
                                            workers.iter().map(|w| {
                                                let name = w.name.clone();
                                                let role = format!("{:?}", w.role);
                                                let dept = w.department.clone().unwrap_or("—".to_string());
                                                view! {
                                                    <div class="tx-person-row">
                                                        <span class="tx-person-name">{name}</span>
                                                        <span class="tx-person-role">{role}</span>
                                                        <span class="tx-person-dept">{dept}</span>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>().into_any()
                                        }}
                                    </div>
                                }.into_any()
                            } else if node == "managers" {
                                let managers = managers_for_org();
                                view! {
                                    <div class="data-card">
                                        <div class="card-header"><span class="card-title">"Managers"</span></div>
                                        {if managers.is_empty() {
                                            view! { <div class="tx-empty">"No managers in this organization."</div> }.into_any()
                                        } else {
                                            managers.iter().map(|m| {
                                                let name = m.name.clone();
                                                let role = format!("{:?}", m.role);
                                                let dept = m.department.clone().unwrap_or("—".to_string());
                                                view! {
                                                    <div class="tx-person-row">
                                                        <span class="tx-person-name">{name}</span>
                                                        <span class="tx-person-role">{role}</span>
                                                        <span class="tx-person-dept">{dept}</span>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>().into_any()
                                        }}
                                    </div>
                                }.into_any()
                            } else if node == "teams" {
                                let users = users_for_org();
                                view! {
                                    <div class="data-card">
                                        <div class="card-header"><span class="card-title">"Teams"</span></div>
                                        {if users.is_empty() {
                                            view! { <div class="tx-empty">"No team members in this organization."</div> }.into_any()
                                        } else {
                                            users.iter().map(|u| {
                                                let name = u.name.clone();
                                                let role = format!("{:?}", u.role);
                                                let dept = u.department.clone().unwrap_or("—".to_string());
                                                view! {
                                                    <div class="tx-person-row">
                                                        <span class="tx-person-name">{name}</span>
                                                        <span class="tx-person-role">{role}</span>
                                                        <span class="tx-person-dept">{dept}</span>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>().into_any()
                                        }}
                                    </div>
                                }.into_any()
                            } else if node == "assets" {
                                let portfolios = portfolios_for_org();
                                view! {
                                    <div class="data-card">
                                        <div class="card-header"><span class="card-title">"Assets"</span></div>
                                        {if portfolios.is_empty() {
                                            view! { <div class="tx-empty">"No portfolios in this organization."</div> }.into_any()
                                        } else {
                                            portfolios.iter().map(|p| {
                                                let name = p.name.clone();
                                                let asset_count = p.assets.len();
                                                let value = format!("{:.2}", p.total_value);
                                                view! {
                                                    <div class="tx-person-row">
                                                        <span class="tx-person-name">{name}</span>
                                                        <span class="tx-person-role">{format!("{} assets", asset_count)}</span>
                                                        <span class="tx-person-dept">{format!("${}", value)}</span>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>().into_any()
                                        }}
                                    </div>
                                }.into_any()
                            } else if node == "projects" {
                                let portfolios = portfolios_for_org();
                                view! {
                                    <div class="data-card">
                                        <div class="card-header"><span class="card-title">"Projects"</span></div>
                                        {if portfolios.is_empty() {
                                            view! { <div class="tx-empty">"No projects in this organization."</div> }.into_any()
                                        } else {
                                            portfolios.iter().map(|p| {
                                                let name = p.name.clone();
                                                let status = format!("{:?}", p.status);
                                                let groups = p.asset_groups.len();
                                                view! {
                                                    <div class="tx-person-row">
                                                        <span class="tx-person-name">{name}</span>
                                                        <span class="tx-person-role">{status}</span>
                                                        <span class="tx-person-dept">{format!("{} groups", groups)}</span>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>().into_any()
                                        }}
                                    </div>
                                }.into_any()
                            } else if node == "channels" {
                                let channels = app_store.get().channels.clone();
                                view! {
                                    <div class="data-card">
                                        <div class="card-header"><span class="card-title">"Channels"</span></div>
                                        {if channels.is_empty() {
                                            view! { <div class="tx-empty">"No channels available."</div> }.into_any()
                                        } else {
                                            channels.iter().map(|c| {
                                                let name = c.name.clone();
                                                view! {
                                                    <div class="tx-person-row">
                                                        <span class="tx-person-name">{name}</span>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>().into_any()
                                        }}
                                    </div>
                                }.into_any()
                            } else {
                                let label = node.split('.').last().unwrap_or(&node);
                                view! {
                                    <div class="data-card">
                                        <div class="card-header"><span class="card-title">{label}</span></div>
                                        <div class="tx-empty">"No data yet for this section."</div>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
