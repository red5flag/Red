use crate::models::{Payment, PaymentSettings, PaymentStatus, User};
use crate::stores::use_app_store;
use crate::types::{PaymentInterval, PaymentMethod, UserRole};
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn NetworkingPage() -> impl IntoView {
    let _app_store = use_app_store();

    // Mock users data
    let users = Memo::new(move |_| {
        vec![
            User {
                id: Uuid::new_v4(),
                name: "John Smith".to_string(),
                email: "john@company.com".to_string(),
                role: UserRole::Owner,
                organization_id: Some(Uuid::new_v4()),
                department: Some("Executive".to_string()),
                phone: Some("+1-555-0100".to_string()),
                address: Some("123 Main St".to_string()),
                hire_date: Some(Utc::now()),
                base_salary: Some(200000.0),
                payment_settings: PaymentSettings {
                    payment_method: PaymentMethod::BankTransfer,
                    account_details: "****1234".to_string(),
                    payment_interval: PaymentInterval::Monthly,
                    currency: crate::types::Currency::USD,
                    automatic_payout: true,
                    payout_threshold: None,
                },
                notification_preferences: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_login: Some(Utc::now()),
                is_active: true,
            },
            User {
                id: Uuid::new_v4(),
                name: "Sarah Johnson".to_string(),
                email: "sarah@company.com".to_string(),
                role: UserRole::Manager,
                organization_id: Some(Uuid::new_v4()),
                department: Some("Operations".to_string()),
                phone: Some("+1-555-0101".to_string()),
                address: None,
                hire_date: Some(Utc::now()),
                base_salary: Some(120000.0),
                payment_settings: PaymentSettings {
                    payment_method: PaymentMethod::DirectDeposit,
                    account_details: "****5678".to_string(),
                    payment_interval: PaymentInterval::BiWeekly,
                    currency: crate::types::Currency::USD,
                    automatic_payout: true,
                    payout_threshold: None,
                },
                notification_preferences: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_login: Some(Utc::now()),
                is_active: true,
            },
            User {
                id: Uuid::new_v4(),
                name: "Mike Williams".to_string(),
                email: "mike@company.com".to_string(),
                role: UserRole::Worker,
                organization_id: Some(Uuid::new_v4()),
                department: Some("Field Operations".to_string()),
                phone: Some("+1-555-0102".to_string()),
                address: None,
                hire_date: Some(Utc::now()),
                base_salary: Some(65000.0),
                payment_settings: PaymentSettings {
                    payment_method: PaymentMethod::BankTransfer,
                    account_details: "****9012".to_string(),
                    payment_interval: PaymentInterval::Weekly,
                    currency: crate::types::Currency::USD,
                    automatic_payout: true,
                    payout_threshold: None,
                },
                notification_preferences: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_login: Some(Utc::now()),
                is_active: true,
            },
        ]
    });

    // Mock transactions
    let transactions = Memo::new(move |_| {
        vec![
            Payment {
                id: Uuid::new_v4(),
                from_user_id: Uuid::new_v4(),
                to_user_id: Uuid::new_v4(),
                amount: 5000.0,
                currency: crate::types::Currency::USD,
                payment_method: PaymentMethod::BankTransfer,
                description: Some("Monthly salary payment".to_string()),
                related_asset_id: None,
                related_portfolio_id: None,
                status: PaymentStatus::Completed,
                scheduled_date: None,
                executed_date: Some(Utc::now()),
                created_at: Utc::now(),
                is_recurring: true,
                recurrence_rule: Some("monthly".to_string()),
            },
            Payment {
                id: Uuid::new_v4(),
                from_user_id: Uuid::new_v4(),
                to_user_id: Uuid::new_v4(),
                amount: 2500.0,
                currency: crate::types::Currency::USD,
                payment_method: PaymentMethod::BankTransfer,
                description: Some("Asset performance bonus".to_string()),
                related_asset_id: Some(Uuid::new_v4()),
                related_portfolio_id: None,
                status: PaymentStatus::Pending,
                scheduled_date: Some(Utc::now()),
                executed_date: None,
                created_at: Utc::now(),
                is_recurring: false,
                recurrence_rule: None,
            },
        ]
    });

    view! {
        <div class="home-screen">
            <div class="welcome-header">
                <h1>"Networking"</h1>
                <p>"Organization members and payments"</p>
            </div>

            // Organization Stats
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Organization Overview"</span>
                </div>
                <div class="card-stats">
                    <div class="stat-item">
                        <div class="stat-label">"Total Members"</div>
                        <div class="stat-value">{move || users.get().len()}</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">"Active Now"</div>
                        <div class="stat-value">"2"</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">"Pending Payments"</div>
                        <div class="stat-value">
                            {move || {
                                transactions.get()
                                    .iter()
                                    .filter(|t| t.status == PaymentStatus::Pending)
                                    .count()
                            }}
                        </div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">"Total Payouts"</div>
                        <div class="stat-value">
                            {move || {
                                let total: f64 = transactions.get()
                                    .iter()
                                    .filter(|t| t.status == PaymentStatus::Completed)
                                    .map(|t| t.amount)
                                    .sum();
                                format!("${:.0}K", total / 1000.0)
                            }}
                        </div>
                    </div>
                </div>
            </div>

            // Users List
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Team Members"</span>
                </div>
                <div class="data-list">
                    {move || {
                        users.get()
                            .into_iter()
                            .map(|user| {
                                let role_icon = match user.role {
                                    UserRole::Owner => "👑",
                                    UserRole::Manager => "⭐",
                                    UserRole::Worker => "👤",
                                };
                                view! {
                                    <div class="list-item">
                                        <div class="list-item-left">
                                            <div class="list-item-title">
                                                {format!("{} {}", role_icon, user.name)}
                                            </div>
                                            <div class="list-item-subtitle">
                                                {format!("{} - {}",
                                                    match user.role {
                                                        UserRole::Owner => "Owner",
                                                        UserRole::Manager => "Manager",
                                                        UserRole::Worker => "Worker",
                                                    },
                                                    user.department.as_deref().unwrap_or("General")
                                                )}
                                            </div>
                                        </div>
                                        <div class="list-item-right">
                                            {user.base_salary.map(|s| {
                                                view! {
                                                    <div class="list-item-value">
                                                        {format!("${:.0}K", s / 1000.0)}
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </div>
            </div>

            // Payment History
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Recent Payments"</span>
                </div>
                <div class="data-list">
                    {move || {
                        transactions.get()
                            .into_iter()
                            .map(|payment| {
                                let status_class = match payment.status {
                                    PaymentStatus::Completed => "positive",
                                    PaymentStatus::Pending => "",
                                    PaymentStatus::Failed => "negative",
                                    _ => "",
                                };
                                let status_icon = match payment.status {
                                    PaymentStatus::Completed => "✓",
                                    PaymentStatus::Pending => "⏳",
                                    PaymentStatus::Scheduled => "📅",
                                    PaymentStatus::Processing => "⚙️",
                                    PaymentStatus::Failed => "✗",
                                    PaymentStatus::Cancelled => "⊘",
                                };
                                view! {
                                    <div class="list-item">
                                        <div class="list-item-left">
                                            <div class="list-item-title">
                                                {format!("{} {}", status_icon,
                                                    payment.description.as_deref().unwrap_or("Payment")
                                                )}
                                            </div>
                                            <div class="list-item-subtitle">
                                                {format!("{:?} - {:?}",
                                                    payment.payment_method,
                                                    payment.status
                                                )}
                                            </div>
                                        </div>
                                        <div class="list-item-right">
                                            <div class={format!("list-item-value {}", status_class)}>
                                                {format!("${:.0}", payment.amount)}
                                            </div>
                                            {payment.is_recurring.then(|| {
                                                view! {
                                                    <div style="font-size: 10px; color: var(--text-secondary);">
                                                        "🔄 Recurring"
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </div>
            </div>
        </div>
    }
}
