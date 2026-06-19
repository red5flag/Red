use crate::models::{ChangeType, RecentChange, TrendDirection, TrendingData};
use crate::stores::use_app_store;
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn OverviewPage() -> impl IntoView {
    let _app_store = use_app_store();

    // Mock trending data - would come from API
    let trending_data = Memo::new(move |_| {
        vec![
            TrendingData {
                portfolio_id: Uuid::new_v4(),
                portfolio_name: "Real Estate Portfolio".to_string(),
                change_percent: 5.2,
                volume: 1250000.0,
                trend: TrendDirection::Up,
            },
            TrendingData {
                portfolio_id: Uuid::new_v4(),
                portfolio_name: "Tech Assets".to_string(),
                change_percent: -2.1,
                volume: 850000.0,
                trend: TrendDirection::Down,
            },
            TrendingData {
                portfolio_id: Uuid::new_v4(),
                portfolio_name: "Equipment Group".to_string(),
                change_percent: 0.5,
                volume: 420000.0,
                trend: TrendDirection::Stable,
            },
        ]
    });

    // Mock recent changes
    let recent_changes = Memo::new(move |_| {
        vec![
            RecentChange {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                change_type: ChangeType::Created,
                entity_name: "Warehouse Property".to_string(),
                entity_type: "Asset".to_string(),
                value_change: Some(750000.0),
            },
            RecentChange {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                change_type: ChangeType::Modified,
                entity_name: "Fleet Vehicles".to_string(),
                entity_type: "Asset Group".to_string(),
                value_change: Some(25000.0),
            },
            RecentChange {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                change_type: ChangeType::ValueUpdated,
                entity_name: "Downtown Office".to_string(),
                entity_type: "Asset".to_string(),
                value_change: Some(150000.0),
            },
        ]
    });

    // Mock financial summary
    let financial_summary = Memo::new(move |_| {
        vec![
            ("Total Portfolio Value", "$4.2M", "positive"),
            ("Total Profit/Loss", "+$850K", "positive"),
            ("Monthly Revenue", "$125K", "positive"),
            ("YTD Growth", "+12.5%", "positive"),
        ]
    });

    view! {
        <div class="home-screen">
            <div class="welcome-header">
                <h1>"Dashboard"</h1>
                <p>"Overview of your business at a glance"</p>
            </div>

            // Financial Summary
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Financial Summary"</span>
                </div>
                <div class="card-stats">
                    {move || {
                        financial_summary.get()
                            .into_iter()
                            .map(|(label, value, class)| {
                                view! {
                                    <div class="stat-item">
                                        <div class="stat-label">{label}</div>
                                        <div class={format!("stat-value {}", class)}>{value}</div>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </div>
            </div>

            // Simple Line Graph (SVG)
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Profit/Loss Trend"</span>
                </div>
                <div class="graph-container">
                    <svg class="graph-svg" viewBox="0 0 400 200">
                        // Grid lines
                        <line x1="0" y1="150" x2="400" y2="150" stroke="#2d3748" stroke-width="1"/>
                        <line x1="0" y1="100" x2="400" y2="100" stroke="#2d3748" stroke-width="1"/>
                        <line x1="0" y1="50" x2="400" y2="50" stroke="#2d3748" stroke-width="1"/>

                        // Trend line
                        <polyline
                            fill="none"
                            stroke="#4ade80"
                            stroke-width="2"
                            points="0,180 50,160 100,140 150,120 200,100 250,80 300,60 350,40 400,20"
                        />

                        // Data points
                        <circle cx="0" cy="180" r="4" fill="#4ade80"/>
                        <circle cx="100" cy="140" r="4" fill="#4ade80"/>
                        <circle cx="200" cy="100" r="4" fill="#4ade80"/>
                        <circle cx="300" cy="60" r="4" fill="#4ade80"/>
                        <circle cx="400" cy="20" r="4" fill="#4ade80"/>
                    </svg>
                </div>
            </div>

            // Trending Portfolios
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Trending"</span>
                </div>
                <div class="data-list">
                    {move || {
                        trending_data.get()
                            .into_iter()
                            .map(|item| {
                                let trend_icon = match item.trend {
                                    TrendDirection::Up => "📈",
                                    TrendDirection::Down => "📉",
                                    TrendDirection::Stable => "➡️",
                                };
                                let change_class = if item.change_percent > 0.0 {
                                    "positive"
                                } else if item.change_percent < 0.0 {
                                    "negative"
                                } else {
                                    ""
                                };
                                view! {
                                    <div class="list-item">
                                        <div class="list-item-left">
                                            <div class="list-item-title">{item.portfolio_name}</div>
                                            <div class="list-item-subtitle">
                                                {format!("Volume: ${:.0}", item.volume)}
                                            </div>
                                        </div>
                                        <div class="list-item-right">
                                            <div class={format!("list-item-value {}", change_class)}>
                                                {format!("{:+.1}%", item.change_percent)}
                                            </div>
                                            <div>{trend_icon}</div>
                                        </div>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </div>
            </div>

            // Recent Changes
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Recent Changes"</span>
                </div>
                <div class="data-list">
                    {move || {
                        recent_changes.get()
                            .into_iter()
                            .map(|change| {
                                let change_icon = match change.change_type {
                                    ChangeType::Created => "🆕",
                                    ChangeType::Modified => "📝",
                                    ChangeType::Deleted => "🗑️",
                                    ChangeType::ValueUpdated => "💰",
                                    ChangeType::StatusChanged => "🔄",
                                    ChangeType::DocumentAdded => "📄",
                                };
                                view! {
                                    <div class="list-item">
                                        <div class="list-item-left">
                                            <div class="list-item-title">
                                                {format!("{} {}", change_icon, change.entity_name)}
                                            </div>
                                            <div class="list-item-subtitle">
                                                {format!("{} - {:#?}", change.entity_type, change.change_type)}
                                            </div>
                                        </div>
                                        <div class="list-item-right">
                                            {change.value_change.map(|v| {
                                                view! {
                                                    <div class="list-item-value">
                                                        {format!("${:.0}", v)}
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
