use crate::models::{Asset, AssetGroup, AssetStatus, Portfolio, PortfolioStatus};
use crate::stores::use_app_store;
use crate::types::{AssetType, ViewMode};
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn PortfoliosPage() -> impl IntoView {
    let app_store = use_app_store();

    // Mock portfolios data
    let portfolios = Memo::new(move |_| {
        let mut portfolios = vec![
            Portfolio {
                id: Uuid::new_v4(),
                name: "Commercial Real Estate".to_string(),
                description: Some("Office buildings and retail spaces".to_string()),
                owner_id: Uuid::new_v4(),
                organization_id: None,
                asset_groups: vec![
                    create_mock_asset_group("Downtown Properties", vec![
                        create_mock_asset("Main Office Building", AssetType::RealEstate, 2500000.0, 3200000.0),
                        create_mock_asset("Retail Plaza", AssetType::RealEstate, 1200000.0, 1450000.0),
                    ]),
                    create_mock_asset_group("Suburban Offices", vec![
                        create_mock_asset("Tech Park Building A", AssetType::RealEstate, 1800000.0, 2100000.0),
                        create_mock_asset("Tech Park Building B", AssetType::RealEstate, 1600000.0, 1850000.0),
                    ]),
                ],
                currency: crate::types::Currency::USD,
                total_value: 8600000.0,
                purchase_value: 7100000.0,
                profit_loss: 1500000.0,
                profit_loss_percent: 21.1,
                revenue: 450000.0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                tags: vec!["real-estate".to_string(), "commercial".to_string()],
                status: PortfolioStatus::Active,
                view_mode: ViewMode::List,
                documents: vec![],
            },
            Portfolio {
                id: Uuid::new_v4(),
                name: "Fleet & Equipment".to_string(),
                description: Some("Company vehicles and machinery".to_string()),
                owner_id: Uuid::new_v4(),
                organization_id: None,
                asset_groups: vec![
                    create_mock_asset_group("Delivery Fleet", vec![
                        create_mock_asset("Truck Fleet", AssetType::Vehicle, 450000.0, 380000.0),
                        create_mock_asset("Van Fleet", AssetType::Vehicle, 180000.0, 165000.0),
                    ]),
                    create_mock_asset_group("Manufacturing Equipment", vec![
                        create_mock_asset("CNC Machines", AssetType::Equipment, 800000.0, 920000.0),
                        create_mock_asset("Assembly Line", AssetType::Equipment, 600000.0, 650000.0),
                    ]),
                ],
                currency: crate::types::Currency::USD,
                total_value: 2115000.0,
                purchase_value: 2030000.0,
                profit_loss: 85000.0,
                profit_loss_percent: 4.2,
                revenue: 180000.0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                tags: vec!["fleet".to_string(), "equipment".to_string()],
                status: PortfolioStatus::Active,
                view_mode: ViewMode::List,
                documents: vec![],
            },
        ];

        // Recalculate values
        for p in &mut portfolios {
            for g in &mut p.asset_groups {
                g.recalculate_values();
            }
            p.recalculate_values();
        }

        portfolios
    });

    let view_mode = move || app_store.get().portfolio_view_mode.clone();

    view! {
        <div class="home-screen">
            <div class="welcome-header">
                <h1>"Portfolios"</h1>
                <p>"Manage your asset portfolios"</p>
            </div>

            // View Toggle
            <div class="view-toggle">
                <button
                    class="view-btn"
                    class:active={move || view_mode() == ViewMode::List}
                    on:click=move |_| {
                        app_store.update(|s| s.portfolio_view_mode = ViewMode::List);
                    }
                >
                    "📋 List"
                </button>
                <button
                    class="view-btn"
                    class:active={move || view_mode() == ViewMode::Grid}
                    on:click=move |_| {
                        app_store.update(|s| s.portfolio_view_mode = ViewMode::Grid);
                    }
                >
                    "⊞ Grid"
                </button>
            </div>

            // Portfolios List
            <div class={move || {
                if view_mode() == ViewMode::Grid {
                    "grid-view"
                } else {
                    "data-list"
                }
            }}>
                {move || {
                    let mode = view_mode();
                    portfolios.get()
                        .into_iter()
                        .map(|portfolio| {
                            let pl_class = if portfolio.profit_loss >= 0.0 {
                                "positive"
                            } else {
                                "negative"
                            };

                            if mode == ViewMode::Grid {
                                view! {
                                    <div class="grid-item">
                                        <div class="grid-item-img">"🏢"</div>
                                        <div>
                                            <div class="list-item-title">{portfolio.name.clone()}</div>
                                            <div class={format!("list-item-value {}", pl_class)}>
                                                {format!("${:.1}M", portfolio.total_value / 1000000.0)}
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="data-card">
                                        <div class="card-header">
                                            <span class="card-title">{portfolio.name.clone()}</span>
                                            <div class="card-actions">
                                                <button class="card-btn">"View"</button>
                                                <button class="card-btn sell">"Quick Sale"</button>
                                            </div>
                                        </div>
                                        <div class="card-stats">
                                            <div class="stat-item">
                                                <div class="stat-label">"Current Value"</div>
                                                <div class="stat-value">
                                                    {format!("${:.2}M", portfolio.total_value / 1000000.0)}
                                                </div>
                                            </div>
                                            <div class="stat-item">
                                                <div class="stat-label">"Profit/Loss"</div>
                                                <div class={format!("stat-value {}", pl_class)}>
                                                    {format!("${:+.0}K", portfolio.profit_loss / 1000.0)}
                                                </div>
                                            </div>
                                            <div class="stat-item">
                                                <div class="stat-label">"Asset Groups"</div>
                                                <div class="stat-value">
                                                    {portfolio.asset_groups.len()}
                                                </div>
                                            </div>
                                            <div class="stat-item">
                                                <div class="stat-label">"Total Assets"</div>
                                                <div class="stat-value">
                                                    {portfolio.get_all_assets().len()}
                                                </div>
                                            </div>
                                        </div>
                                        <div style="margin-top: 12px; padding-top: 12px; border-top: 2px solid var(--border-color);">
                                            <div style="font-size: 11px; color: var(--text-secondary);">
                                                {format!("Revenue: ${:.0}K | Purchase Value: ${:.2}M",
                                                    portfolio.revenue / 1000.0,
                                                    portfolio.purchase_value / 1000000.0)}
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}

// Helper functions to create mock data
fn create_mock_asset(name: &str, asset_type: AssetType, purchase: f64, current: f64) -> Asset {
    Asset {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: None,
        asset_type,
        purchase_value: purchase,
        current_value: current,
        profit_loss: current - purchase,
        profit_loss_percent: ((current - purchase) / purchase) * 100.0,
        revenue: 0.0,
        purchase_date: chrono::Utc::now(),
        images: vec![],
        documents: vec![],
        tags: vec![],
        status: AssetStatus::Active,
        metadata: serde_json::json!({}),
        assigned_workers: vec![],
        quick_sale_enabled: false,
        notification_settings: vec![],
    }
}

fn create_mock_asset_group(name: &str, assets: Vec<Asset>) -> AssetGroup {
    let mut group = AssetGroup {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: None,
        assets,
        total_value: 0.0,
        purchase_value: 0.0,
        profit_loss: 0.0,
        profit_loss_percent: 0.0,
        revenue: 0.0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        tags: vec![],
        documents: vec![],
    };
    group.recalculate_values();
    group
}
