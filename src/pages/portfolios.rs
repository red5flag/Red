use crate::components::tabs::use_tab_edit_mode;
use crate::models::{Asset, AssetGroup, AssetStatus, Document, Portfolio};
use crate::stores::{create_action, use_app_store, use_undo_redo_store};
use crate::types::{ActionType, AssetType, SortMode, UserRole, ViewMode};
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub fn PortfoliosPage() -> impl IntoView {
    let app_store = use_app_store();

    // Read portfolios from AppStore and filter by current user visibility
    let filtered_portfolios = Memo::new(move |_| {
        let user = app_store.get().current_user.clone();
        let can_view_all = user.can_view_all();
        let user_id = user.id;
        app_store.get()
            .portfolios
            .iter()
            .filter(|p| p.is_visible_to(user_id, can_view_all))
            .cloned()
            .collect::<Vec<_>>()
    });
    let view_mode = move || app_store.get().portfolio_view_mode.clone();
    let sort_mode = move || app_store.get().portfolio_sort_mode.clone();
    let selected_id = move || app_store.get().selected_portfolio_id;
    let edit_mode = use_tab_edit_mode();
    let can_edit = move || {
        let role = app_store.get().current_user.role.clone();
        edit_mode.get() && (role == UserRole::Owner || role == UserRole::Manager)
    };

    // Form signals for add portfolio
    let (new_name, set_new_name) = signal(String::new());
    let (new_desc, set_new_desc) = signal(String::new());

    // Form signals for add asset group
    let (show_add_group, set_show_add_group) = signal(Option::<Uuid>::None);
    let (new_group_name, set_new_group_name) = signal(String::new());

    // Form signals for add asset
    let (show_add_asset, set_show_add_asset) = signal(AssetTarget::default());

    // Top-level add group/asset signals (from navbar via AppStore)
    let (top_add_group_pid, set_top_add_group_pid) = signal(Option::<Uuid>::None);
    let (top_add_asset_pid, set_top_add_asset_pid) = signal(Option::<Uuid>::None);
    let (top_add_asset_gid, set_top_add_asset_gid) = signal(Option::<Uuid>::None);

    // Modal signal for editing portfolio assets
    let (edit_portfolio_id, set_edit_portfolio_id) = signal(Option::<Uuid>::None);
    let (context_menu, set_context_menu) = signal(Option::<(Uuid, i32, i32)>::None);
    let (new_asset_name, set_new_asset_name) = signal(String::new());
    let (new_asset_type, set_new_asset_type) = signal(AssetType::RealEstate);
    let (new_asset_value, set_new_asset_value) = signal(String::new());

    let on_toggle_view = move |id: Uuid| {
        app_store.update(|s| {
            if s.selected_portfolio_id == Some(id) {
                s.selected_portfolio_id = None;
            } else {
                s.selected_portfolio_id = Some(id);
            }
        });
    };

    let on_add_portfolio = move |_| {
        let name = new_name.get();
        if name.trim().is_empty() {
            return;
        }
        let owner_id = app_store.get().current_user.id;
        let mut p = Portfolio::new(name, owner_id, crate::types::Currency::USD);
        p.description = if new_desc.get().trim().is_empty() {
            None
        } else {
            Some(new_desc.get())
        };
        app_store.update(|s| s.add_portfolio(p));
        set_new_name.set(String::new());
        set_new_desc.set(String::new());
        app_store.update(|s| s.show_add_portfolio = false);
    };

    let on_delete_portfolio = move |id: Uuid| {
        app_store.update(|s| {
            s.remove_portfolio(id);
            if s.selected_portfolio_id == Some(id) {
                s.selected_portfolio_id = None;
            }
        });
        set_edit_portfolio_id.set(None);
    };

    let on_open_edit = move |id: Uuid| {
        set_edit_portfolio_id.set(Some(id));
    };

    let on_close_edit = move |_| {
        set_edit_portfolio_id.set(None);
    };

    let on_add_group = Callback::new(move |portfolio_id: Uuid| {
        let name = new_group_name.get();
        if name.trim().is_empty() {
            return;
        }
        let group = create_mock_asset_group(&name, vec![]);
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(portfolio_id) {
                p.asset_groups.push(group);
                p.recalculate_values();
            }
        });
        set_new_group_name.set(String::new());
        set_show_add_group.set(None);
    });

    let on_add_asset = Callback::new(move |target: AssetTarget| {
        let name = new_asset_name.get();
        if name.trim().is_empty() {
            return;
        }
        let value: f64 = new_asset_value.get().parse().unwrap_or(0.0);
        let _asset = create_mock_asset(&name, new_asset_type.get(), value, value);
        app_store.update(|s| {
            match target {
                AssetTarget::PortfolioDirect(pid) => {
                    if let Some(p) = s.get_portfolio_mut(pid) {
                        p.assets.push(_asset);
                        p.recalculate_values();
                    }
                }
                AssetTarget::Group(pid, gid) => {
                    if let Some(p) = s.get_portfolio_mut(pid) {
                        if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                            g.assets.push(_asset);
                            g.recalculate_values();
                        }
                        p.recalculate_values();
                    }
                }
                AssetTarget::None => {}
            }
        });
        set_new_asset_name.set(String::new());
        set_new_asset_value.set(String::new());
        set_show_add_asset.set(AssetTarget::default());
    });

    let on_top_add_group = move |_| {
        let pid = top_add_group_pid.get();
        if pid.is_none() { return; }
        on_add_group.run(pid.unwrap());
        app_store.update(|s| s.show_top_add_group = false);
        set_top_add_group_pid.set(None);
    };

    let on_top_add_asset = move |_| {
        let pid = top_add_asset_pid.get();
        if pid.is_none() { return; }
        let target = match top_add_asset_gid.get() {
            Some(gid) => AssetTarget::Group(pid.unwrap(), gid),
            None => AssetTarget::PortfolioDirect(pid.unwrap()),
        };
        on_add_asset.run(target);
        app_store.update(|s| s.show_top_add_asset = false);
        set_top_add_asset_pid.set(None);
        set_top_add_asset_gid.set(None);
    };

    let _selected_portfolio = move || {
        selected_id().and_then(|id| filtered_portfolios.get().into_iter().find(|p| p.id == id))
    };

    view! {
        <div class="home-screen">
            // Portfolio controls bar (attached below navbar)
            <div class="portfolio-controls-bar">
                <button
                    class="nav-portfolio-btn"
                    class:active={move || view_mode() == ViewMode::List}
                    on:click=move |_| app_store.update(|s| s.portfolio_view_mode = ViewMode::List)
                >
                    "☰ List"
                </button>
                <button
                    class="nav-portfolio-btn"
                    class:active={move || view_mode() == ViewMode::Grid}
                    on:click=move |_| app_store.update(|s| s.portfolio_view_mode = ViewMode::Grid)
                >
                    "⊞ Grid"
                </button>
                <select
                    class="nav-portfolio-btn nav-portfolio-sort"
                    prop:value={move || {
                        match app_store.get().portfolio_sort_mode {
                            SortMode::Recent => "sort_recent",
                            SortMode::Oldest => "sort_oldest",
                            SortMode::HighestValue => "sort_highest_value",
                            SortMode::LowestValue => "sort_lowest_value",
                            SortMode::HighestProfit => "sort_highest_profit",
                            SortMode::LowestProfit => "sort_lowest_profit",
                            SortMode::HighestRevenue => "sort_highest_revenue",
                            SortMode::LowestRevenue => "sort_lowest_revenue",
                            SortMode::ByOrganization => "sort_by_organization",
                        }.to_string()
                    }}
                    on:change=move |ev| {
                        let v = event_target_value(&ev);
                        let mode = match v.as_str() {
                            "sort_oldest" => SortMode::Oldest,
                            "sort_highest_value" => SortMode::HighestValue,
                            "sort_lowest_value" => SortMode::LowestValue,
                            "sort_highest_profit" => SortMode::HighestProfit,
                            "sort_lowest_profit" => SortMode::LowestProfit,
                            "sort_highest_revenue" => SortMode::HighestRevenue,
                            "sort_lowest_revenue" => SortMode::LowestRevenue,
                            "sort_by_organization" => SortMode::ByOrganization,
                            _ => SortMode::Recent,
                        };
                        app_store.update(|s| s.portfolio_sort_mode = mode);
                    }
                >
                    <option value="sort_recent">"Sort: Recent"</option>
                    <option value="sort_oldest">"Sort: Oldest"</option>
                    <option value="sort_highest_value">"Sort: Highest Value"</option>
                    <option value="sort_lowest_value">"Sort: Lowest Value"</option>
                    <option value="sort_highest_profit">"Sort: Highest Profit"</option>
                    <option value="sort_lowest_profit">"Sort: Lowest Profit"</option>
                    <option value="sort_highest_revenue">"Sort: Highest Revenue"</option>
                    <option value="sort_lowest_revenue">"Sort: Lowest Revenue"</option>
                    <option value="sort_by_organization">"Sort: By Organization"</option>
                </select>
            </div>

            // Edit portfolio assets modal
            {move || edit_portfolio_id.get().map(|pid| {
                let pid_add_asset = pid;
                let pid_add_group = pid;
                let pid_delete = pid;
                view! {
                    <div class="modal-overlay" on:click=move |_| on_close_edit(())>
                        <div class="modal" on:click=|ev| ev.stop_propagation()>
                            <div class="modal-header">
                                <span class="modal-title">"Edit Portfolio Assets"</span>
                                <button class="modal-close" on:click=move |_| on_close_edit(())>"×"</button>
                            </div>
                            <div class="modal-body">
                                <div class="edit-actions">
                                    <button
                                        class="login-btn"
                                        on:click=move |_| {
                                            set_show_add_asset.set(AssetTarget::PortfolioDirect(pid_add_asset));
                                            on_close_edit(());
                                        }
                                    >
                                        "+ Add Asset"
                                    </button>
                                    <button
                                        class="login-btn"
                                        on:click=move |_| {
                                            set_show_add_group.set(Some(pid_add_group));
                                            on_close_edit(());
                                        }
                                    >
                                        "+ Add Group"
                                    </button>
                                    <button
                                        class="login-btn sell"
                                        on:click=move |_| {
                                            on_delete_portfolio(pid_delete);
                                        }
                                    >
                                        "🗑 Delete Portfolio"
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any()
            })}

            // Grid columns selector (only visible in grid mode)
            {move || if view_mode() == ViewMode::Grid {
                view! {
                    <div class="view-toggle">
                        <select
                            class="view-btn sort-select"
                            prop:value={move || format!("grid_{}", app_store.get().portfolio_grid_columns)}
                            on:change=move |ev| {
                                let v = event_target_value(&ev);
                                if let Some(suffix) = v.strip_prefix("grid_") {
                                    if let Ok(n) = suffix.parse::<usize>() {
                                        app_store.update(|s| s.set_portfolio_grid_columns(n));
                                    }
                                }
                            }
                        >
                            <option value="grid_2">"Grid: 2"</option>
                            <option value="grid_3">"Grid: 3"</option>
                            <option value="grid_4">"Grid: 4"</option>
                            <option value="grid_6">"Grid: 6"</option>
                            <option value="grid_8">"Grid: 8"</option>
                            <option value="grid_12">"Grid: 12"</option>
                        </select>
                    </div>
                }.into_any()
                } else { ().into_any() }}

            // Add Portfolio Form (toggled from navbar)
            {move || app_store.get().show_add_portfolio.then(|| view! {
                <div class="add-form">
                    <input
                        class="login-input"
                        type="text"
                        placeholder="Portfolio name"
                        on:input=move |ev| set_new_name.set(event_target_value(&ev))
                    />
                    <input
                        class="login-input"
                        type="text"
                        placeholder="Description (optional)"
                        on:input=move |ev| set_new_desc.set(event_target_value(&ev))
                    />
                    <button class="login-btn" on:click=on_add_portfolio>"Create Portfolio"</button>
                </div>
            })}

            // Top-level Add Group Form (toggled from navbar)
            {move || app_store.get().show_top_add_group.then(|| view! {
                <div class="add-form">
                    <select
                        class="login-input"
                        prop:value={move || top_add_group_pid.get().map(|id| id.to_string()).unwrap_or_default()}
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            if let Ok(uuid) = Uuid::parse_str(&v) {
                                set_top_add_group_pid.set(Some(uuid));
                            } else {
                                set_top_add_group_pid.set(None);
                            }
                        }
                    >
                        <option value="">"Select portfolio"</option>
                        {filtered_portfolios.get().into_iter().map(|p| view! {
                            <option value={p.id.to_string()}>{p.name.clone()}</option>
                        }).collect::<Vec<_>>()}
                    </select>
                    <input
                        class="login-input"
                        type="text"
                        placeholder="Group name"
                        on:input=move |ev| set_new_group_name.set(event_target_value(&ev))
                    />
                    <button class="login-btn" on:click=on_top_add_group>"Create Group"</button>
                </div>
            })}

            // Top-level Add Asset Form (toggled from navbar)
            {move || app_store.get().show_top_add_asset.then(|| view! {
                <div class="add-form">
                    <select
                        class="login-input"
                        prop:value={move || top_add_asset_pid.get().map(|id| id.to_string()).unwrap_or_default()}
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            if let Ok(uuid) = Uuid::parse_str(&v) {
                                set_top_add_asset_pid.set(Some(uuid));
                                set_top_add_asset_gid.set(None);
                            } else {
                                set_top_add_asset_pid.set(None);
                                set_top_add_asset_gid.set(None);
                            }
                        }
                    >
                        <option value="">"Select portfolio"</option>
                        {filtered_portfolios.get().into_iter().map(|p| view! {
                            <option value={p.id.to_string()}>{p.name.clone()}</option>
                        }).collect::<Vec<_>>()}
                    </select>
                    {move || {
                        let pid = top_add_asset_pid.get();
                        if pid.is_none() { return ().into_any(); }
                        let pid = pid.unwrap();
                        let groups = filtered_portfolios.get().into_iter().find(|p| p.id == pid).map(|p| p.asset_groups).unwrap_or_default();
                        view! {
                            <select
                                class="login-input"
                                prop:value={move || top_add_asset_gid.get().map(|id| id.to_string()).unwrap_or_default()}
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    if v.is_empty() {
                                        set_top_add_asset_gid.set(None);
                                    } else if let Ok(uuid) = Uuid::parse_str(&v) {
                                        set_top_add_asset_gid.set(Some(uuid));
                                    }
                                }
                            >
                                <option value="">"No group — add to portfolio"</option>
                                {groups.into_iter().map(|g| view! {
                                    <option value={g.id.to_string()}>{g.name.clone()}</option>
                                }).collect::<Vec<_>>()}
                            </select>
                        }.into_any()
                    }}
                    <input
                        class="login-input"
                        type="text"
                        placeholder="Asset name"
                        on:input=move |ev| set_new_asset_name.set(event_target_value(&ev))
                    />
                    <input
                        class="login-input"
                        type="text"
                        placeholder="Value"
                        on:input=move |ev| set_new_asset_value.set(event_target_value(&ev))
                    />
                    <select
                        class="login-input"
                        prop:value={move || format!("{:?}", new_asset_type.get())}
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            let t = match v.as_str() {
                                "RealEstate" => AssetType::RealEstate,
                                "Vehicle" => AssetType::Vehicle,
                                "Equipment" => AssetType::Equipment,
                                "Stock" => AssetType::Stock,
                                "Bond" => AssetType::Bond,
                                "Commodity" => AssetType::Commodity,
                                "Digital" => AssetType::Digital,
                                "IntellectualProperty" => AssetType::IntellectualProperty,
                                _ => AssetType::RealEstate,
                            };
                            set_new_asset_type.set(t);
                        }
                    >
                        <option value="RealEstate">"Real Estate"</option>
                        <option value="Vehicle">"Vehicle"</option>
                        <option value="Equipment">"Equipment"</option>
                        <option value="Stock">"Stock"</option>
                        <option value="Bond">"Bond"</option>
                        <option value="Commodity">"Commodity"</option>
                        <option value="Digital">"Digital"</option>
                        <option value="IntellectualProperty">"Intellectual Property"</option>
                    </select>
                    <button class="login-btn" on:click=on_top_add_asset>"Create Asset"</button>
                </div>
            })}

            // Portfolios List
            <div class={move || {
                if view_mode() == ViewMode::Grid {
                    format!("grid-view grid-cols-{}", app_store.get().portfolio_grid_columns)
                } else { "pf-accordion".to_string() }
            }}>
                {move || {
                    let can = can_edit();
                    let sort = sort_mode();
                    let mut items: Vec<_> = filtered_portfolios.get().into_iter().collect();
                    items.sort_by(|a, b| match sort {
                        SortMode::Recent => b.created_at.cmp(&a.created_at),
                        SortMode::Oldest => a.created_at.cmp(&b.created_at),
                        SortMode::HighestValue => b.total_value.partial_cmp(&a.total_value).unwrap_or(std::cmp::Ordering::Equal),
                        SortMode::LowestValue => a.total_value.partial_cmp(&b.total_value).unwrap_or(std::cmp::Ordering::Equal),
                        SortMode::HighestProfit => b.profit_loss.partial_cmp(&a.profit_loss).unwrap_or(std::cmp::Ordering::Equal),
                        SortMode::LowestProfit => a.profit_loss.partial_cmp(&b.profit_loss).unwrap_or(std::cmp::Ordering::Equal),
                        SortMode::HighestRevenue => b.revenue.partial_cmp(&a.revenue).unwrap_or(std::cmp::Ordering::Equal),
                        SortMode::LowestRevenue => a.revenue.partial_cmp(&b.revenue).unwrap_or(std::cmp::Ordering::Equal),
                        SortMode::ByOrganization => a.organization_id.cmp(&b.organization_id),
                    });
                    items.into_iter().map(move |portfolio| {
                        let portfolio_id = portfolio.id;
                        let is_expanded = selected_id() == Some(portfolio_id);

                        view! {
                            <PortfolioListItem
                                portfolio={portfolio}
                                can_edit={can}
                                expanded={is_expanded}
                                on_toggle=move || on_toggle_view(portfolio_id)
                                on_context=move |ev: leptos::ev::MouseEvent| {
                                    ev.prevent_default();
                                    set_context_menu.set(Some((portfolio_id, ev.client_x(), ev.client_y())));
                                }
                                show_add_group={show_add_group.get()}
                                set_show_add_group={set_show_add_group}
                                _new_group_name={new_group_name}
                                set_new_group_name={set_new_group_name}
                                on_add_group={on_add_group}
                                show_add_asset={show_add_asset}
                                set_show_add_asset={set_show_add_asset}
                                new_asset_name={new_asset_name}
                                set_new_asset_name={set_new_asset_name}
                                new_asset_type={new_asset_type}
                                set_new_asset_type={set_new_asset_type}
                                new_asset_value={new_asset_value}
                                set_new_asset_value={set_new_asset_value}
                                on_add_asset={on_add_asset}
                                view_mode={view_mode()}
                            />
                        }.into_any()
                    })
                    .collect::<Vec<_>>()
                }}
            </div>

            // Context menu for portfolio press-and-hold
            {move || context_menu.get().map(|(pid, x, y)| {
                view! {
                    <div
                        class="context-menu-overlay"
                        on:click=move |_| set_context_menu.set(None)
                    >
                        <div
                            class="context-menu"
                            style={format!("left: {}px; top: {}px;", x, y)}
                        >
                            <button
                                class="context-menu-item"
                                on:click=move |_| {
                                    set_context_menu.set(None);
                                    on_open_edit(pid);
                                }
                            >
                                "Edit"
                            </button>
                            <button
                                class="context-menu-item"
                                on:click=move |_| {
                                    set_context_menu.set(None);
                                    on_toggle_view(pid);
                                }
                            >
                                "Overview"
                            </button>
                        </div>
                    </div>
                }.into_any()
            })}
        </div>
    }
}

#[derive(Clone, PartialEq, Default)]
pub enum AssetTarget {
    #[default]
    None,
    PortfolioDirect(Uuid),
    Group(Uuid, Uuid),
}

#[component]
fn AssetViewer(
    portfolio: Portfolio,
    can_edit: bool,
    view_mode: ViewMode,
    show_add_group: Option<Uuid>,
    set_show_add_group: WriteSignal<Option<Uuid>>,
    _new_group_name: ReadSignal<String>,
    set_new_group_name: WriteSignal<String>,
    on_add_group: Callback<Uuid>,
    show_add_asset: ReadSignal<AssetTarget>,
    set_show_add_asset: WriteSignal<AssetTarget>,
    new_asset_name: ReadSignal<String>,
    set_new_asset_name: WriteSignal<String>,
    new_asset_type: ReadSignal<AssetType>,
    set_new_asset_type: WriteSignal<AssetType>,
    new_asset_value: ReadSignal<String>,
    set_new_asset_value: WriteSignal<String>,
    on_add_asset: Callback<AssetTarget>,
) -> impl IntoView {
    let pid = portfolio.id;
    let app_store_inner = use_app_store();
    let current_user = app_store_inner.get().current_user.clone();
    let user_id = current_user.id;
    let can_view_all = current_user.can_view_all();
    let portfolio_visible_to_user = portfolio.is_visible_to(user_id, can_view_all);

    let (expanded_groups, set_expanded_groups) = signal(HashSet::<Uuid>::new());
    let toggle_group = Callback::new(move |gid: Uuid| {
        set_expanded_groups.update(|set| {
            if !set.remove(&gid) {
                set.insert(gid);
            }
        });
    });

    let (show_groups, set_show_groups) = signal(true);
    let (show_direct_assets, set_show_direct_assets) = signal(true);

    let (grid_columns, _set_grid_columns) = signal(3usize);
    let (selected_asset, set_selected_asset) = signal::<Option<Asset>>(None);

    let on_select_asset = Callback::new(move |asset: Asset| {
        set_selected_asset.set(Some(asset));
    });

    let on_close_asset = Callback::new(move |_| {
        set_selected_asset.set(None);
    });

    let view_mode = view_mode.clone();
    let view_mode_groups_title = view_mode.clone();
    let view_mode_groups_content = view_mode.clone();
    let view_mode_direct_title = view_mode.clone();
    let view_mode_direct_content = view_mode.clone();
    let portfolio_groups = portfolio.clone();
    let portfolio_direct = portfolio.clone();

    view! {
        <div class="asset-viewer">
            // Asset Groups section
            <div class="asset-section">
                <div class="asset-section-title">
                    <span class="asset-section-arrow"
                            on:click=move |_| set_show_groups.update(|v| *v = !*v)
                        >
                        {move || if show_groups.get() { "▼" } else { "▶" }}
                    </span>
                    <span class="asset-section-label"
                        on:click=move |_| set_show_groups.update(|v| *v = !*v)
                    >"Asset Groups"</span>
                    <div class="section-title-right">
                        {{
                            move || if show_groups.get() && view_mode_groups_title == ViewMode::Grid {
                                view! {
                                    <button class="sort-btn">"Sort ↕"</button>
                                }.into_any()
                            } else { ().into_any() }
                        }}
                        {move || if can_edit {
                            let pid2 = pid;
                            view! {
                                <button
                                    class="add-btn-small"
                                    on:click=move |_| set_show_add_group.set(Some(pid2))
                                >
                                    "+"
                                </button>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                </div>

                {move || if show_groups.get() {
                    let visible_groups: Vec<_> = portfolio_groups.asset_groups.clone().into_iter().filter(|g| portfolio_visible_to_user || g.is_visible_to(user_id, can_view_all)).collect();
                    view! {
                        <div>
                            {move || show_add_group.map(|gp| {
                                if gp == pid {
                                    view! {
                                        <div class="add-form">
                                            <input class="login-input" type="text" placeholder="Group name"
                                                on:input=move |ev| set_new_group_name.set(event_target_value(&ev)) />
                                            <button class="login-btn" on:click=move |_| on_add_group.run(pid)>
                                                "Add Group"
                                            </button>
                                        </div>
                                    }.into_any()
                                } else { ().into_any() }
                            })}

                            {if visible_groups.is_empty() {
                                view! {
                                    <div class="empty-state">
                                        <div class="empty-text">"No asset groups"</div>
                                    </div>
                                }.into_any()
                            } else {
                                let group_class = if view_mode_groups_content == ViewMode::Grid { "grid-view" } else { "asset-list" };
                                let view_mode_clone = view_mode_groups_content.clone();
                                let portfolio_name = portfolio_groups.name.clone();
                                view! {
                                    <div class={group_class}>
                                        {visible_groups.into_iter().map(move |group| {
                                            let gid = group.id;
                                            let pid2 = pid;
                                            let is_expanded = Memo::new(move |_| expanded_groups.get().contains(&gid));
                                            view! {
                                                <AssetGroupItem
                                                    group={group}
                                                    can_edit={can_edit}
                                                    pid={pid2}
                                                    gid={gid}
                                                    expanded={is_expanded}
                                                    view_mode={view_mode_clone.clone()}
                                                    grid_columns={grid_columns.get()}
                                                    on_toggle={toggle_group}
                                                    show_add_asset={show_add_asset}
                                                    set_show_add_asset={set_show_add_asset}
                                                    _new_asset_name={new_asset_name}
                                                    set_new_asset_name={set_new_asset_name}
                                                    _new_asset_type={new_asset_type}
                                                    set_new_asset_type={set_new_asset_type}
                                                    _new_asset_value={new_asset_value}
                                                    set_new_asset_value={set_new_asset_value}
                                                    on_add_asset={on_add_asset}
                                                    on_select_asset={on_select_asset}
                                                    portfolio_name={portfolio_name.clone()}
                                                />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            // Direct Assets section
            <div class="asset-section">
                <div class="asset-section-title">
                    <span class="asset-section-arrow"
                            on:click=move |_| set_show_direct_assets.update(|v| *v = !*v)
                        >
                        {move || if show_direct_assets.get() { "▼" } else { "▶" }}
                    </span>
                    <span class="asset-section-label"
                        on:click=move |_| set_show_direct_assets.update(|v| *v = !*v)
                    >"Direct Assets"</span>
                    <div class="section-title-right">
                        {{
                            move || if show_direct_assets.get() && view_mode_direct_title == ViewMode::Grid {
                                view! {
                                    <button class="sort-btn">"Sort ↕"</button>
                                }.into_any()
                            } else { ().into_any() }
                        }}
                        {move || if can_edit {
                            let pid2 = pid;
                            view! {
                                <button
                                    class="add-btn-small"
                                    on:click=move |_| set_show_add_asset.set(AssetTarget::PortfolioDirect(pid2))
                                >
                                    "+"
                                </button>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                </div>

                {move || if show_direct_assets.get() {
                    let visible_direct_assets: Vec<_> = portfolio_direct.assets.clone().into_iter().filter(|a| portfolio_visible_to_user || a.is_visible_to(user_id, can_view_all)).collect();
                    view! {
                        <div>
                            {move || {
                                if show_add_asset.get() == AssetTarget::PortfolioDirect(pid) {
                                    view! {
                                        <div class="add-form">
                                            <input class="login-input" type="text" placeholder="Asset name"
                                                on:input=move |ev| set_new_asset_name.set(event_target_value(&ev)) />
                                            <select class="login-input"
                                                on:change=move |ev| {
                                                    let v = event_target_value(&ev);
                                                    let t = match v.as_str() {
                                                        "RealEstate" => AssetType::RealEstate,
                                                        "Vehicle" => AssetType::Vehicle,
                                                        "Equipment" => AssetType::Equipment,
                                                        "Stock" => AssetType::Stock,
                                                        "Bond" => AssetType::Bond,
                                                        "Commodity" => AssetType::Commodity,
                                                        "Digital" => AssetType::Digital,
                                                        "IntellectualProperty" => AssetType::IntellectualProperty,
                                                        _ => AssetType::RealEstate,
                                                    };
                                                    set_new_asset_type.set(t);
                                                }
                                            >
                                                <option value="RealEstate">"Real Estate"</option>
                                                <option value="Vehicle">"Vehicle"</option>
                                                <option value="Equipment">"Equipment"</option>
                                                <option value="Stock">"Stock"</option>
                                                <option value="Bond">"Bond"</option>
                                                <option value="Commodity">"Commodity"</option>
                                                <option value="Digital">"Digital"</option>
                                                <option value="IntellectualProperty">"IP"</option>
                                            </select>
                                            <input class="login-input" type="number" placeholder="Value ($)"
                                                on:input=move |ev| set_new_asset_value.set(event_target_value(&ev)) />
                                            <button class="login-btn" on:click=move |_| on_add_asset.run(AssetTarget::PortfolioDirect(pid))>
                                                "Add Asset"
                                            </button>
                                        </div>
                                    }.into_any()
                                } else { ().into_any() }
                            }}

                            {if visible_direct_assets.is_empty() {
                                view! {
                                    <div class="empty-state">
                                        <div class="empty-text">"No direct assets"</div>
                                    </div>
                                }.into_any()
                            } else {
                                let direct_class = if view_mode_direct_content == ViewMode::Grid {
                                    format!("grid-view-{}", grid_columns.get())
                                } else {
                                    "asset-list".to_string()
                                };
                                let view_mode_clone = view_mode_direct_content.clone();
                                let portfolio_name = portfolio_direct.name.clone();
                                view! {
                                    <div class={direct_class}>
                                        {visible_direct_assets.into_iter().map(move |asset| view! {
                                            <AssetItem asset={asset} portfolio_name={portfolio_name.clone()} portfolio_id={Some(pid)} view_mode={view_mode_clone.clone()} on_select={on_select_asset} can_edit={can_edit} />
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            {move || selected_asset.get().map(|asset| view! {
                <AssetDetailView asset={asset} on_close={on_close_asset} />
            })}
        </div>
    }
}

#[component]
fn AssetGroupItem(
    group: AssetGroup,
    #[prop(default = false)] can_edit: bool,
    pid: Uuid,
    gid: Uuid,
    expanded: Memo<bool>,
    view_mode: ViewMode,
    grid_columns: usize,
    on_toggle: Callback<Uuid>,
    show_add_asset: ReadSignal<AssetTarget>,
    set_show_add_asset: WriteSignal<AssetTarget>,
    _new_asset_name: ReadSignal<String>,
    set_new_asset_name: WriteSignal<String>,
    _new_asset_type: ReadSignal<AssetType>,
    set_new_asset_type: WriteSignal<AssetType>,
    _new_asset_value: ReadSignal<String>,
    set_new_asset_value: WriteSignal<String>,
    on_add_asset: Callback<AssetTarget>,
    on_select_asset: Callback<Asset>,
    portfolio_name: String,
) -> impl IntoView {
    let app_store = use_app_store();
    let _ = view_mode;

    let current_user = app_store.get().current_user.clone();
    let user_id = current_user.id;
    let can_view_all = current_user.can_view_all();
    let group_visible_to_user = group.is_visible_to(user_id, can_view_all);

    let can_edit_here = can_edit;

    let (show_doc_modal, set_show_doc_modal) = signal(false);
    let (show_context_add, set_show_context_add) = signal(false);
    let (edit_name, set_edit_name) = signal(group.name.clone());
    let (edit_desc, set_edit_desc) = signal(group.description.clone().unwrap_or_default());

    let g_name = group.name.clone();
    let g_desc = group.description.clone().unwrap_or_default();
    let g_name_for_modal = group.name.clone();
    let docs = group.documents.clone();
    let doc_count = docs.len();
    let assigned_users = group.assigned_users.clone();
    let org_users = move || app_store.get().organization_users.clone();
    let gid_for_assign = gid;
    let pid_for_assign = pid;
    let toggle_group_assignment = Callback::new(move |uid: Uuid| {
        let gid = gid_for_assign;
        let pid = pid_for_assign;
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                    if g.assigned_users.contains(&uid) {
                        g.assigned_users.retain(|&id| id != uid);
                    } else {
                        g.assigned_users.push(uid);
                    }
                }
            }
        });
        if let Some(p) = app_store.get().get_portfolio(pid).cloned() {
            leptos::task::spawn_local(async move {
                let _ = crate::server::save_portfolio(p).await;
            });
        }
    });

    let save_group_edit = move |_| {
        let n = edit_name.get();
        let d = edit_desc.get();
        if n.trim().is_empty() { return; }
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                    g.name = n.clone();
                    g.description = if d.trim().is_empty() { None } else { Some(d.clone()) };
                    g.updated_at = chrono::Utc::now();
                }
            }
        });
    };

    let add_group_doc = move |n: String| {
        if n.trim().is_empty() { return; }
        let doc = crate::models::Document {
            id: Uuid::new_v4(),
            name: n.clone(),
            file_type: "pdf".to_string(),
            content: None,
            url: "#".to_string(),
            uploaded_at: chrono::Utc::now(),
            uploaded_by: Uuid::nil(),
        };
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                    g.documents.push(doc);
                }
            }
        });
    };

    view! {
        <div class="asset-group" class:expanded={move || expanded.get()}>
            <div class="asset-group-header"
                on:click=move |_| if !can_edit_here { on_toggle.run(gid) }>
                <span class="asset-group-arrow">
                    {move || if expanded.get() { "▲" } else { "▼" }}
                </span>
                <div class="asset-group-icon">"📁"</div>
                <div class="asset-group-info-wrap" on:click=|ev| ev.stop_propagation()>
                    {let asset_count = group.assets.len();
                    let g_name_header = g_name.clone();
                    let g_desc_header = g_desc.clone();
                    move || if can_edit_here {
                        view! {
                            <div class="asset-group-edit-form">
                                <input class="pf-edit-input" placeholder="Group name"
                                    prop:value=move || edit_name.get()
                                    on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                    on:blur=save_group_edit />
                                <input class="pf-edit-input" placeholder="Description"
                                    prop:value=move || edit_desc.get()
                                    on:input=move |ev| set_edit_desc.set(event_target_value(&ev))
                                    on:blur=save_group_edit />
                                <UserAssignmentPanel assigned={assigned_users.clone()} users={org_users()} on_toggle={toggle_group_assignment} />
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                <div class="asset-group-name">{g_name_header.clone()}</div>
                                {if !g_desc_header.is_empty() {
                                    view! { <div class="asset-group-desc">{g_desc_header.clone()}</div> }.into_any()
                                } else { ().into_any() }}
                                <div class="asset-group-count">{format!("{} assets", asset_count)}</div>
                            </div>
                        }.into_any()
                    }}
                </div>
                // Action buttons
                <div class="pf-list-actions" on:click=|ev| ev.stop_propagation()>
                    <button class="pf-action-btn"
                        class:active=move || show_doc_modal.get()
                        on:click=move |_| set_show_doc_modal.set(true)>
                        {format!("📄 {}", doc_count)}
                    </button>
                    {move || if can_edit_here {
                        let pid2 = pid; let gid2 = gid;
                        view! {
                            <div class="pf-context-add-wrap">
                                <button class="pf-action-btn pf-context-add-btn"
                                    class:active=move || show_context_add.get()
                                    on:click=move |ev| { ev.stop_propagation(); set_show_context_add.update(|v| *v = !*v); }
                                >"+"</button>
                                {move || if show_context_add.get() {
                                    view! {
                                        <div class="pf-context-add-dropdown">
                                            <button class="pf-context-add-item" on:click=move |ev| {
                                                ev.stop_propagation();
                                                set_show_context_add.set(false);
                                                set_show_add_asset.set(AssetTarget::Group(pid2, gid2));
                                            }>"📦 Add Asset"</button>
                                            <button class="pf-context-add-item" on:click=move |ev| {
                                                ev.stop_propagation();
                                                set_show_context_add.set(false);
                                                set_show_doc_modal.set(true);
                                            }>"📄 Add Document"</button>
                                            <button class="pf-context-add-item" on:click=move |ev| {
                                                ev.stop_propagation();
                                                set_show_context_add.set(false);
                                            }>"👤 Add User"</button>
                                            <button class="pf-context-add-item" on:click=move |ev| {
                                                ev.stop_propagation();
                                                set_show_context_add.set(false);
                                            }>"🏢 Add Organization"</button>
                                            <button class="pf-context-add-item" on:click=move |ev| {
                                                ev.stop_propagation();
                                                set_show_context_add.set(false);
                                            }>"🔑 Add Role"</button>
                                        </div>
                                    }.into_any()
                                } else { ().into_any() }}
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
            </div>
            // Docs modal for group
            {move || if show_doc_modal.get() {
                let docs_snap = docs.clone();
                let modal_title = g_name_for_modal.clone();
                let add_cb = if can_edit_here { Some(Callback::new(move |n: String| add_group_doc(n))) } else { None };
                view! {
                    <DocModal
                        docs={docs_snap}
                        title={modal_title}
                        on_close=move || set_show_doc_modal.set(false)
                        can_edit={can_edit_here}
                        on_add={add_cb}
                    />
                }.into_any()
            } else { ().into_any() }}

            <div class="asset-group-content" class:hidden={move || !expanded.get()}>
                {move || if show_add_asset.get() == AssetTarget::Group(pid, gid) {
                    view! {
                        <div class="add-form">
                            <input class="login-input" type="text" placeholder="Asset name"
                                on:input=move |ev| set_new_asset_name.set(event_target_value(&ev)) />
                            <select class="login-input"
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    let t = match v.as_str() {
                                        "RealEstate" => AssetType::RealEstate,
                                        "Vehicle" => AssetType::Vehicle,
                                        "Equipment" => AssetType::Equipment,
                                        "Stock" => AssetType::Stock,
                                        "Bond" => AssetType::Bond,
                                        "Commodity" => AssetType::Commodity,
                                        "Digital" => AssetType::Digital,
                                        "IntellectualProperty" => AssetType::IntellectualProperty,
                                        _ => AssetType::RealEstate,
                                    };
                                    set_new_asset_type.set(t);
                                }
                            >
                                <option value="RealEstate">"Real Estate"</option>
                                <option value="Vehicle">"Vehicle"</option>
                                <option value="Equipment">"Equipment"</option>
                                <option value="Stock">"Stock"</option>
                                <option value="Bond">"Bond"</option>
                                <option value="Commodity">"Commodity"</option>
                                <option value="Digital">"Digital"</option>
                                <option value="IntellectualProperty">"IP"</option>
                            </select>
                            <input class="login-input" type="number" placeholder="Value ($)"
                                on:input=move |ev| set_new_asset_value.set(event_target_value(&ev)) />
                            <button class="login-btn"
                                on:click=move |_| on_add_asset.run(AssetTarget::Group(pid, gid))>
                                "Add Asset"
                            </button>
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                {{
                    let view_mode = view_mode.clone();
                    let group_assets: Vec<_> = group.assets.into_iter().filter(|a| group_visible_to_user || a.is_visible_to(user_id, can_view_all)).collect();
                    let class_str = if view_mode == ViewMode::Grid {
                        format!("asset-group-assets grid-view-{}", grid_columns)
                    } else {
                        "asset-group-assets asset-list".to_string()
                    };
                    view! {
                        <div class={class_str}>
                            {group_assets.into_iter().map({
                                let view_mode = view_mode.clone();
                                move |asset| view! {
                                    <AssetItem asset={asset} portfolio_name={portfolio_name.clone()} portfolio_id={Some(pid)} view_mode={view_mode.clone()} on_select={on_select_asset} can_edit={can_edit_here} />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }
                }}
            </div>
        </div>
    }
}

/// Portfolio list row — accordion style matching AssetGroupItem.
#[component]
fn PortfolioListItem(
    portfolio: crate::models::Portfolio,
    #[prop(default = false)] can_edit: bool,
    expanded: bool,
    on_toggle: impl Fn() + 'static,
    on_context: impl Fn(leptos::ev::MouseEvent) + 'static,
    // AssetViewer props forwarded for expanded content
    show_add_group: Option<Uuid>,
    set_show_add_group: WriteSignal<Option<Uuid>>,
    _new_group_name: ReadSignal<String>,
    set_new_group_name: WriteSignal<String>,
    on_add_group: Callback<Uuid>,
    show_add_asset: ReadSignal<AssetTarget>,
    set_show_add_asset: WriteSignal<AssetTarget>,
    new_asset_name: ReadSignal<String>,
    set_new_asset_name: WriteSignal<String>,
    new_asset_type: ReadSignal<AssetType>,
    set_new_asset_type: WriteSignal<AssetType>,
    new_asset_value: ReadSignal<String>,
    set_new_asset_value: WriteSignal<String>,
    on_add_asset: Callback<AssetTarget>,
    view_mode: ViewMode,
) -> impl IntoView {
    let app_store = use_app_store();
    let (show_doc_modal, set_show_doc_modal) = signal(false);
    let (show_context_add, set_show_context_add) = signal(false);
    let (edit_name, set_edit_name) = signal(portfolio.name.clone());
    let (edit_desc, set_edit_desc) = signal(portfolio.description.clone().unwrap_or_default());
    let pid = portfolio.id;
    let doc_count = portfolio.documents.len();
    let docs = portfolio.documents.clone();
    let name = portfolio.name.clone();
    let name_for_modal = portfolio.name.clone();
    let desc = portfolio.description.clone().unwrap_or_default();
    let asset_count = portfolio.get_all_assets().len();
    let can_edit_here = can_edit;

    let save_edit = move |_| {
        let n = edit_name.get();
        let d = edit_desc.get();
        if n.trim().is_empty() { return; }
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                p.name = n.clone();
                p.description = if d.trim().is_empty() { None } else { Some(d.clone()) };
                p.updated_at = chrono::Utc::now();
            }
        });
    };

    let add_doc = move |n: String| {
        if n.trim().is_empty() { return; }
        let doc = crate::models::Document {
            id: Uuid::new_v4(),
            name: n.clone(),
            file_type: "pdf".to_string(),
            url: "#".to_string(),
            uploaded_at: chrono::Utc::now(),
            uploaded_by: Uuid::nil(),
            content: None,
        };
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                p.documents.push(doc);
            }
        });
    };

    let portfolio_for_viewer = portfolio.clone();
    let assigned_users = portfolio.assigned_users.clone();
    let org_users = move || app_store.get().organization_users.clone();

    let toggle_portfolio_assignment = Callback::new(move |uid: Uuid| {
        let pid = portfolio.id;
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                if p.assigned_users.contains(&uid) {
                    p.assigned_users.retain(|&id| id != uid);
                } else {
                    p.assigned_users.push(uid);
                }
                p.updated_at = chrono::Utc::now();
            }
        });
        let portfolio_clone = portfolio.clone();
        leptos::task::spawn_local(async move {
            let _ = crate::server::save_portfolio(portfolio_clone).await;
        });
    });

    view! {
        <div class="asset-group" class:expanded={expanded} on:contextmenu=on_context>
            // Header row — same structure as asset-group-header
            <div class="asset-group-header"
                on:click=move |_| if !can_edit_here { on_toggle() }>
                <span class="asset-group-arrow">
                    {if expanded { "▲" } else { "▼" }}
                </span>
                <div class="asset-group-icon">"🏢"</div>
                <div class="asset-group-info-wrap" on:click=|ev| ev.stop_propagation()>
                    {let name_header = name.clone();
                    let desc_header = desc.clone();
                    move || if can_edit_here {
                        view! {
                            <div class="asset-group-edit-form">
                                <input class="pf-edit-input" placeholder="Portfolio name"
                                    prop:value=move || edit_name.get()
                                    on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                    on:blur=save_edit />
                                <input class="pf-edit-input" placeholder="Description"
                                    prop:value=move || edit_desc.get()
                                    on:input=move |ev| set_edit_desc.set(event_target_value(&ev))
                                    on:blur=save_edit />
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                <div class="asset-group-name">{name_header.clone()}</div>
                                {if !desc_header.is_empty() {
                                    view! { <div class="asset-group-desc">{desc_header.clone()}</div> }.into_any()
                                } else { ().into_any() }}
                                <div class="asset-group-count">
                                    {format!("{} asset{}", asset_count, if asset_count == 1 { "" } else { "s" })}
                                </div>
                            </div>
                        }.into_any()
                    }}
                </div>
                // Action strip
                <div class="pf-list-actions" on:click=|ev| ev.stop_propagation()>
                    <button class="pf-action-btn"
                        class:active=move || show_doc_modal.get()
                        on:click=move |_| set_show_doc_modal.set(true)>
                        {format!("📄 {}", doc_count)}
                    </button>
                    {move || if can_edit_here {
                        let pid_add_group = pid;
                        let pid_add_asset = pid;
                        view! {
                            <div class="pf-context-add-wrap">
                                <button class="pf-action-btn pf-context-add-btn"
                                    class:active=move || show_context_add.get()
                                    on:click=move |ev| { ev.stop_propagation(); set_show_context_add.update(|v| *v = !*v); }
                                >"+"</button>
                                {move || if show_context_add.get() {
                                    view! {
                                        <div class="pf-context-add-dropdown">
                                            <button class="pf-context-add-item" on:click=move |ev| {
                                                ev.stop_propagation();
                                                set_show_context_add.set(false);
                                                set_show_add_group.set(Some(pid_add_group));
                                            }>"📁 Add Group"</button>
                                            <button class="pf-context-add-item" on:click=move |ev| {
                                                ev.stop_propagation();
                                                set_show_context_add.set(false);
                                                set_show_add_asset.set(AssetTarget::PortfolioDirect(pid_add_asset));
                                            }>"📦 Add Asset"</button>
                                            <button class="pf-context-add-item" on:click=move |ev| {
                                                ev.stop_propagation();
                                                set_show_context_add.set(false);
                                                set_show_doc_modal.set(true);
                                            }>"📄 Add Document"</button>
                                            <button class="pf-context-add-item" on:click=move |ev| {
                                                ev.stop_propagation();
                                                set_show_context_add.set(false);
                                            }>"👤 Add User"</button>
                                            <button class="pf-context-add-item" on:click=move |ev| {
                                                ev.stop_propagation();
                                                set_show_context_add.set(false);
                                            }>"🏢 Add Organization"</button>
                                            <button class="pf-context-add-item" on:click=move |ev| {
                                                ev.stop_propagation();
                                                set_show_context_add.set(false);
                                            }>"🔑 Add Role"</button>
                                        </div>
                                    }.into_any()
                                } else { ().into_any() }}
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
            </div>

            // Docs modal for portfolio
            {move || if show_doc_modal.get() {
                let docs_snap = docs.clone();
                let modal_title = name_for_modal.clone();
                let add_cb = if can_edit_here { Some(Callback::new(move |n: String| add_doc(n))) } else { None };
                view! {
                    <DocModal
                        docs={docs_snap}
                        title={modal_title}
                        on_close=move || set_show_doc_modal.set(false)
                        can_edit={can_edit_here}
                        on_add={add_cb}
                    />
                }.into_any()
            } else { ().into_any() }}

            {move || if can_edit_here {
                let users = org_users();
                let assigned = assigned_users.clone();
                view! {
                    <UserAssignmentPanel assigned={assigned} users={users} on_toggle={toggle_portfolio_assignment} />
                }.into_any()
            } else { ().into_any() }}

            // Expanded content — AssetViewer
            <div class="asset-group-content" class:hidden={!expanded}>
                <AssetViewer
                    portfolio={portfolio_for_viewer}
                    can_edit={can_edit_here}
                    view_mode={view_mode}
                    show_add_group={show_add_group}
                    set_show_add_group={set_show_add_group}
                    _new_group_name={_new_group_name}
                    set_new_group_name={set_new_group_name}
                    on_add_group={on_add_group}
                    show_add_asset={show_add_asset}
                    set_show_add_asset={set_show_add_asset}
                    new_asset_name={new_asset_name}
                    set_new_asset_name={set_new_asset_name}
                    new_asset_type={new_asset_type}
                    set_new_asset_type={set_new_asset_type}
                    new_asset_value={new_asset_value}
                    set_new_asset_value={set_new_asset_value}
                    on_add_asset={on_add_asset}
                />
            </div>
        </div>
    }
}

fn asset_placeholder_url(asset_type: &AssetType, name: &str) -> String {
    let text = match asset_type {
        AssetType::RealEstate => "House",
        AssetType::Vehicle => "Car",
        AssetType::Equipment => "Gear",
        AssetType::Stock => "Stock",
        AssetType::Bond => "Bond",
        AssetType::Commodity => "Goods",
        AssetType::Digital => "Crypto",
        AssetType::IntellectualProperty => "IP",
        AssetType::Custom(_) => "Asset",
    };
    let seed = name.replace(' ', "+");
    let seed = if seed.len() > 12 { &seed[..12] } else { &seed };
    format!("https://placehold.co/400x400/2d3748/FFF?text={}%2B{}", text, seed)
}

/// Stored snapshot for the detail panel so nested closures can read/clone data without moving it.
#[derive(Clone)]
struct AssetDetailSnapshot {
    a_type: String,
    a_addr: String,
    asset_name_for_modal: String,
    docs: Vec<Document>,
}

#[component]
fn AssetItem(
    asset: Asset,
    portfolio_name: String,
    #[prop(default = None)] portfolio_id: Option<Uuid>,
    view_mode: ViewMode,
    on_select: Callback<Asset>,
    #[prop(default = false)] can_edit: bool,
) -> impl IntoView {
    let app_store = use_app_store();
    let _ = can_edit;
    let image_url = asset
        .images
        .first()
        .cloned()
        .unwrap_or_else(|| asset_placeholder_url(&asset.asset_type, &asset.name));

    let (expanded_detail, set_expanded_detail) = signal(false);
    let (show_doc_modal, set_show_doc_modal) = signal(false);
    let (_editing, set_editing) = signal(false);
    let (edit_name, set_edit_name) = signal(asset.name.clone());
    let (edit_desc, set_edit_desc) = signal(asset.description.clone().unwrap_or_default());
    let (edit_loc, set_edit_loc) = signal(asset.location.clone().unwrap_or_default());

    let can_edit_here = can_edit;
    // doc sort: 0 = recent, 1 = name
    let (doc_sort, set_doc_sort) = signal(0u8);
    let (detail_tab, set_detail_tab) = signal(0u8);

    let asset_id = asset.id;
    let pname = portfolio_name.clone();
    let docs = asset.documents.clone();
    let _doc_count = docs.len();
    let a_name = asset.name.clone();
    let a_addr = asset.location.clone().unwrap_or_default();
    let asset_name_for_modal = asset.name.clone();
    let (_asset_name_signal, _set_asset_name) = signal(a_name.clone());
    // snapshot values for the detail panel
    let a_type     = format!("{:?}", asset.asset_type);
    let _a_desc     = asset.description.clone().unwrap_or_else(|| "—".to_string());
    let _a_status   = format!("{:?}", asset.status);
    let a_purchase_val = asset.purchase_value;
    let a_current_val  = asset.current_value;
    let a_pl           = asset.profit_loss;
    let a_pl_pct       = asset.profit_loss_percent;
    let a_revenue      = asset.revenue;
    let a_pl_cls       = if asset.profit_loss >= 0.0 { "positive" } else { "negative" };
    let _a_purchase_date = asset.purchase_date.format("%d %b %Y").to_string();

    let detail = StoredValue::new(AssetDetailSnapshot {
        a_type: a_type.clone(),
        a_addr: a_addr.clone(),
        asset_name_for_modal: asset_name_for_modal.clone(),
        docs: docs.clone(),
    });

    let save_edit = move || {
        let n = edit_name.get();
        let d = edit_desc.get();
        let l = edit_loc.get();
        if n.trim().is_empty() { return; }
        app_store.update(|s| {
            for p in s.portfolios.iter_mut() {
                let all: Vec<_> = p.assets.iter_mut()
                    .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                    .collect();
                for a in all {
                    if a.id == asset_id {
                        a.name = n.clone();
                        a.description = if d.trim().is_empty() { None } else { Some(d.clone()) };
                        a.location = if l.trim().is_empty() { None } else { Some(l.clone()) };
                        return;
                    }
                }
            }
        });
        set_editing.set(false);
    };

    let add_doc = move |n: String| {
        if n.trim().is_empty() { return; }
        let doc = crate::models::Document {
            id: Uuid::new_v4(),
            name: n.clone(),
            file_type: "pdf".to_string(),
            url: "#".to_string(),
            uploaded_at: chrono::Utc::now(),
            uploaded_by: Uuid::nil(),
            content: None,
        };
        app_store.update(|s| {
            for p in s.portfolios.iter_mut() {
                let all: Vec<_> = p.assets.iter_mut()
                    .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                    .collect();
                for a in all {
                    if a.id == asset_id {
                        a.documents.push(doc.clone());
                        return;
                    }
                }
            }
        });
    };
    let add_cb = if can_edit { Some(Callback::new(add_doc)) } else { None };

    let asset_id_for_assign = asset_id;
    let portfolio_id_for_assign = portfolio_id;
    let toggle_asset_assignment = Callback::new(move |uid: Uuid| {
        let aid = asset_id_for_assign;
        app_store.update(|s| {
            for p in s.portfolios.iter_mut() {
                let all: Vec<_> = p.assets.iter_mut()
                    .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                    .collect();
                for a in all {
                    if a.id == aid {
                        if a.assigned_workers.contains(&uid) {
                            a.assigned_workers.retain(|&id| id != uid);
                        } else {
                            a.assigned_workers.push(uid);
                        }
                        return;
                    }
                }
            }
        });
        if let Some(pid) = portfolio_id_for_assign {
            let portfolio_clone = app_store.get().get_portfolio(pid).cloned();
            if let Some(p) = portfolio_clone {
                leptos::task::spawn_local(async move {
                    let _ = crate::server::save_portfolio(p).await;
                });
            }
        }
    });

    let get_asset_assigned_users = move || {
        app_store.get().portfolios.iter()
            .flat_map(|p| p.assets.iter().chain(p.asset_groups.iter().flat_map(|g| g.assets.iter())))
            .find(|a| a.id == asset_id)
            .map(|a| a.assigned_workers.clone())
            .unwrap_or_default()
    };
    let get_org_users = move || app_store.get().organization_users.clone();

    if view_mode == ViewMode::Grid {
        let asset_for_click = asset.clone();
        let short_name = shorthand_name(&a_name);
        view! {
            <div class="asset-grid-card" on:click=move |_| on_select.run(asset_for_click.clone())>
                <img class="asset-grid-image" src={image_url.clone()} alt={a_name.clone()} />
                <div class="asset-grid-name">{short_name}</div>
            </div>
        }.into_any()
    } else {
    view! {
        <div class="ai-item" class:ai-item-expanded={move || expanded_detail.get()}>
            <div class="ai-list-card" on:click=move |_| {
                if !can_edit_here && detail_tab.get() != 1 { set_expanded_detail.update(|v| *v = !*v); }
            }>
                <img class="ai-list-image" src={image_url.clone()} alt={a_name.clone()} />
                <div class="ai-list-body" on:click=|ev| ev.stop_propagation()>
                    <div class="ai-list-portfolio">{pname.clone()}</div>
                    {move || if can_edit_here {
                        view! {
                            <div class="ai-edit-stack">
                                <input class="pf-edit-input" placeholder="Asset name"
                                    prop:value=move || edit_name.get()
                                    on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                    on:blur=move |_| save_edit() />
                                <input class="pf-edit-input" placeholder="Description"
                                    prop:value=move || edit_desc.get()
                                    on:input=move |ev| set_edit_desc.set(event_target_value(&ev))
                                    on:blur=move |_| save_edit() />
                                <input class="pf-edit-input" placeholder="Location / Address"
                                    prop:value=move || edit_loc.get()
                                    on:input=move |ev| set_edit_loc.set(event_target_value(&ev))
                                    on:blur=move |_| save_edit() />
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                <div class="ai-list-name">{a_name.clone()}</div>
                                <div class="ai-list-addr">{a_addr.clone()}</div>
                            </div>
                        }.into_any()
                    }}
                    <div class="ai-list-docs" on:click=|ev| ev.stop_propagation()>
                        {let doc_count = docs.len();
                        view! {
                            <button class="ai-list-docs-btn"
                                on:click=move |_| set_show_doc_modal.set(true)>
                                {if doc_count == 0 {
                                    "📄 Add document".to_string()
                                } else {
                                    format!("📄 {} document{}", doc_count, if doc_count == 1 { "" } else { "s" })
                                }}
                            </button>
                        }.into_any()}
                    </div>
                </div>
                <div class="ai-list-actions" on:click=|ev| ev.stop_propagation()>
                    <span class="ai-list-arrow">{move || if expanded_detail.get() { "▲" } else { "▼" }}</span>
                </div>
            </div>

            {move || if show_doc_modal.get() {
                let mt = asset_name_for_modal.clone();
                let ac = add_cb.clone();
                let docs_snap = docs.clone();
                view! {
                    <DocModal
                        docs={docs_snap}
                        title={mt}
                        on_close=move || set_show_doc_modal.set(false)
                        can_edit={can_edit_here}
                        on_add={ac}
                    />
                }.into_any()
            } else { ().into_any() }}

            {move || {
                if expanded_detail.get() {
                    view! {
                        <div class="ai-detail-panel" on:click=|ev| ev.stop_propagation()>
                            <div class="ai-detail-tabs">
                                <button
                                    class="ai-detail-tab"
                                    class:active={move || detail_tab.get() == 0}
                                    on:click=move |_| set_detail_tab.set(0)
                                >
                                    "View"
                                </button>
                                {if can_edit_here {
                                    view! {
                                        <button
                                            class="ai-detail-tab"
                                            class:active={move || detail_tab.get() == 1}
                                            on:click=move |_| set_detail_tab.set(1)
                                        >
                                            "Edit"
                                        </button>
                                    }.into_any()
                                } else { ().into_any() }}
                            </div>

                            {move || match detail_tab.get() {
                                1 => view! {
                                    <div class="ai-edit-tab">
                                        <div class="asset-edit-form">
                                            <label class="ai-edit-label">"Name"</label>
                                            <input class="pf-edit-input" placeholder="Name"
                                                prop:value={move || edit_name.get()}
                                                on:input=move |ev| set_edit_name.set(event_target_value(&ev)) />
                                            <label class="ai-edit-label">"Description"</label>
                                            <input class="pf-edit-input" placeholder="Description"
                                                prop:value={move || edit_desc.get()}
                                                on:input=move |ev| set_edit_desc.set(event_target_value(&ev)) />
                                            <label class="ai-edit-label">"Location / Address"</label>
                                            <input class="pf-edit-input" placeholder="Location / Address"
                                                prop:value={move || edit_loc.get()}
                                                on:input=move |ev| set_edit_loc.set(event_target_value(&ev)) />
                                            <div class="asset-edit-actions">
                                                <button class="pf-edit-save" on:click=move |_| save_edit()>"✔ Save"</button>
                                                <button class="pf-edit-cancel" on:click=move |_| { set_detail_tab.set(0); }>"✕ Cancel"</button>
                                            </div>
                                            <UserAssignmentPanel assigned={get_asset_assigned_users()} users={get_org_users()} on_toggle={toggle_asset_assignment} />
                                        </div>
                                    </div>
                                }.into_any(),
                                _ => view! {
                                    <AssetDetailViewTab
                                        detail=detail
                                        a_purchase_val=a_purchase_val
                                        a_current_val=a_current_val
                                        a_pl=a_pl
                                        a_pl_pct=a_pl_pct
                                        a_revenue=a_revenue
                                        a_pl_cls=a_pl_cls
                                        doc_sort=doc_sort
                                        set_doc_sort=set_doc_sort
                                        show_doc_modal=show_doc_modal
                                        set_show_doc_modal=set_show_doc_modal
                                        can_edit=can_edit_here
                                        add_cb=add_cb
                                    />
                                }.into_any(),
                            }}
                        </div>
                    }.into_any()
                } else { ().into_any() }
            }}
        </div>
    }.into_any()
    }
}

#[component]
fn AssetDetailViewTab(
    detail: StoredValue<AssetDetailSnapshot>,
    a_purchase_val: f64,
    a_current_val: f64,
    a_pl: f64,
    a_pl_pct: f64,
    a_revenue: f64,
    a_pl_cls: &'static str,
    doc_sort: ReadSignal<u8>,
    set_doc_sort: WriteSignal<u8>,
    show_doc_modal: ReadSignal<bool>,
    set_show_doc_modal: WriteSignal<bool>,
    can_edit: bool,
    add_cb: Option<Callback<String>>,
) -> impl IntoView {
    let _ = (a_purchase_val, a_pl, a_pl_pct, a_revenue, a_pl_cls);
    let modal_title = detail.get_value().asset_name_for_modal.clone();

    view! {
        <div class="ai-view-tab">
            <div class="pf-detail-grid">
                <div class="pf-detail-cell">
                    <span class="pf-detail-label">"NAME"</span>
                    <span class="pf-detail-value">{modal_title.clone()}</span>
                </div>
                <div class="pf-detail-cell">
                    <span class="pf-detail-label">"TYPE & BUILD"</span>
                    <span class="pf-detail-value">{detail.get_value().a_type.clone()}</span>
                </div>
                <div class="pf-detail-cell">
                    <span class="pf-detail-label">"ADDRESS"</span>
                    <span class="pf-detail-value">{detail.get_value().a_addr.clone()}</span>
                </div>
                <div class="pf-detail-cell">
                    <span class="pf-detail-label">"PRICE"</span>
                    <span class="pf-detail-value">{format!("${:.2}", a_current_val)}</span>
                </div>
            </div>

            <div class="ai-docs-section">
                <div class="ai-docs-heading-row">
                    <span class="ai-detail-heading">"Documents"</span>
                    <div class="ai-docs-sort-btns">
                        <button class="ai-docs-sort-btn"
                            class:active=move || doc_sort.get() == 0
                            on:click=move |_| set_doc_sort.set(0)>
                            "Recent"
                        </button>
                        <button class="ai-docs-sort-btn"
                            class:active=move || doc_sort.get() == 1
                            on:click=move |_| set_doc_sort.set(1)>
                            "Name"
                        </button>
                        {if can_edit {
                            view! {
                                <button class="ai-docs-sort-btn ai-docs-add-btn"
                                    on:click=move |_| set_show_doc_modal.set(true)>
                                    "+ Add"
                                </button>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                </div>
                {move || {
                    let mut sorted_docs = detail.get_value().docs.clone();
                    if doc_sort.get() == 1 {
                        sorted_docs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                    }
                    // Recent order is insertion order (already newest-last in seed)
                    if sorted_docs.is_empty() {
                        view! { <div class="ai-docs-empty">"No documents attached"</div> }.into_any()
                    } else {
                        view! {
                            <div class="ai-docs-strip">
                                {sorted_docs.into_iter().map(|doc| {
                                    let icon  = document_icon(&doc.file_type);
                                    let ft    = doc.file_type.to_uppercase();
                                    let dname = doc.name.clone();
                                    let doc_for_view = doc.clone();
                                    let (viewing, set_viewing) = signal(false);
                                    view! {
                                        <div class="ai-doc-card" on:click=move |_| set_viewing.set(true)>
                                            <span class="ai-doc-card-icon">{icon}</span>
                                            <span class="ai-doc-card-name">{dname}</span>
                                            <span class="ai-doc-card-ft">{ft}</span>
                                        </div>
                                        {move || if viewing.get() {
                                            let d = doc_for_view.clone();
                                            view! {
                                                <div class="doc-modal-overlay" on:click=move |_| set_viewing.set(false)>
                                                    <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                                                        <DocumentViewer
                                                            doc={d.clone()}
                                                            on_close=move || set_viewing.set(false)
                                                            can_edit={can_edit}
                                                        />
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else { ().into_any() }}
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>

        {move || if show_doc_modal.get() {
            let mt = modal_title.clone();
            let ac = add_cb.clone();
            view! {
                <DocModal
                    docs={detail.get_value().docs.clone()}
                    title={mt}
                    on_close=move || set_show_doc_modal.set(false)
                    can_edit={can_edit}
                    on_add={ac}
                />
            }.into_any()
        } else { ().into_any() }}
    }
}

#[component]
fn AssetDetailView(asset: Asset, on_close: Callback<()>) -> impl IntoView {
    let icon = match asset.asset_type {
        AssetType::RealEstate => "🏢",
        AssetType::Vehicle => "🚗",
        AssetType::Equipment => "⚙️",
        AssetType::Stock => "📈",
        AssetType::Bond => "📜",
        AssetType::Commodity => "🌾",
        AssetType::Digital => "💻",
        AssetType::IntellectualProperty => "💡",
        AssetType::Custom(_) => "📦",
    };
    let pl_class = if asset.profit_loss >= 0.0 { "positive" } else { "negative" };

    view! {
        <div class="asset-detail-overlay" on:click=move |_| on_close.run(())>
            <div class="asset-detail" on:click=|ev| ev.stop_propagation()>
                <div class="asset-detail-header">
                    <div class="asset-detail-icon">{icon}</div>
                    <div class="asset-detail-title">{asset.name}</div>
                    <button class="asset-detail-close" on:click=move |_| on_close.run(())>"✕"</button>
                </div>
                <div class="asset-detail-body">
                    <div class="asset-detail-row">
                        <span class="asset-detail-label">"Type"</span>
                        <span class="asset-detail-value">{format!("{:?}", asset.asset_type)}</span>
                    </div>
                    <div class="asset-detail-row">
                        <span class="asset-detail-label">"Location"</span>
                        <span class="asset-detail-value">{asset.location.clone().unwrap_or_else(|| "—".to_string())}</span>
                    </div>
                    <div class="asset-detail-row">
                        <span class="asset-detail-label">"Current Value"</span>
                        <span class="asset-detail-value">{format!("${:.2}M", asset.current_value / 1000000.0)}</span>
                    </div>
                    <div class="asset-detail-row">
                        <span class="asset-detail-label">"Profit/Loss"</span>
                        <span class={format!("asset-detail-value {}", pl_class)}>{format!("${:+.0}K", asset.profit_loss / 1000.0)}</span>
                    </div>
                    <div class="asset-detail-row">
                        <span class="asset-detail-label">"Organization"</span>
                        <span class="asset-detail-value">{asset.organization_id.map(|id| id.to_string()).unwrap_or_else(|| "Unassigned".to_string())}</span>
                    </div>
                    <div class="asset-detail-row">
                        <span class="asset-detail-label">"Status"</span>
                        <span class="asset-detail-value">{format!("{:?}", asset.status)}</span>
                    </div>
                    <div class="asset-detail-images">
                        {if asset.images.is_empty() {
                            view! { <div class="asset-detail-no-image">"No images"</div> }.into_any()
                        } else {
                            asset.images.into_iter().map(|url| view! {
                                <img class="asset-detail-img" src={url} alt="Asset image" />
                            }).collect::<Vec<_>>().into_any()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}

// Helper functions to create mock data
fn create_mock_asset(name: &str, asset_type: AssetType, purchase: f64, current: f64) -> Asset {
    let id = Uuid::new_v4();
    let image_url = asset_placeholder_url(&asset_type, name);
    let docs = vec![
        ("Title Deed", "pdf"),
        ("Inspection Report", "pdf"),
        ("Valuation", "xlsx"),
        ("Photos", "zip"),
        ("Contract", "docx"),
        ("Insurance", "pdf"),
        ("Notes", "txt"),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (n, ext))| crate::models::Document {
        id: Uuid::new_v4(),
        name: format!("{} {}", n, i + 1),
        file_type: ext.to_string(),
        content: None,
        url: "#".to_string(),
        uploaded_at: chrono::Utc::now(),
        uploaded_by: Uuid::nil(),
    })
    .collect();
    Asset {
        id,
        name: name.to_string(),
        description: Some(format!("Open Rose Rental Duplex 112, Open Rose Court, Coolangatta, QLD, 4269.").to_string()),
        asset_type,
        location: Some("Coolangatta, QLD, 4269".to_string()),
        organization_id: None,
        purchase_value: purchase,
        current_value: current,
        profit_loss: current - purchase,
        profit_loss_percent: ((current - purchase) / purchase) * 100.0,
        revenue: 0.0,
        purchase_date: chrono::Utc::now(),
        images: vec![image_url],
        documents: docs,
        tags: vec![],
        status: AssetStatus::Active,
        metadata: serde_json::json!({}),
        assigned_workers: vec![],
        quick_sale_enabled: false,
        notification_settings: vec![],
        calendar_events: vec![],
    }
}

fn document_icon(file_type: &str) -> &'static str {
    match file_type.to_lowercase().as_str() {
        "pdf" => "📕",
        "doc" | "docx" => "📘",
        "xls" | "xlsx" => "📗",
        "ppt" | "pptx" => "📙",
        "txt" | "md" | "rs" | "js" | "ts" | "html" | "css" => "📄",
        "zip" | "rar" | "7z" | "tar" => "🗜️",
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" => "🖼️",
        "mp4" | "mov" | "avi" | "mkv" => "🎬",
        "mp3" | "wav" | "flac" => "🎵",
        _ => "📎",
    }
}

fn shorthand_name(name: &str) -> String {
    if name.len() <= 16 {
        name.to_string()
    } else {
        format!("{}...", &name[..13])
    }
}

#[component]
fn UserAssignmentPanel(
    assigned: Vec<Uuid>,
    users: Vec<crate::models::User>,
    on_toggle: Callback<Uuid>,
) -> impl IntoView {
    view! {
        <div class="assignment-panel">
            <div class="assignment-title">"Assigned users"</div>
            {if users.is_empty() {
                view! { <div class="assignment-empty">"No users available"</div> }.into_any()
            } else {
                users.into_iter().map(move |u| {
                    let checked = assigned.contains(&u.id);
                    let uid = u.id;
                    view! {
                        <label class="assignment-row">
                            <input type="checkbox" checked=checked on:change=move |_| on_toggle.run(uid) />
                            <span>{format!("{} ({:?})", u.name, u.role)}</span>
                        </label>
                    }
                }).collect::<Vec<_>>().into_any()
            }}
        </div>
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
        calendar_events: vec![],
        assigned_users: vec![],
    };
    group.recalculate_values();
    group
}

/// Generate mock document content for the in-app viewer based on name and type.
fn mock_doc_content(name: &str, file_type: &str) -> String {
    match file_type.to_lowercase().as_str() {
        "pdf" => format!(
"DOCUMENT: {name}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Generated: {date}
Reference: DOC-{ref_num}
Status: ACTIVE

SUMMARY
This document serves as an official record pertaining to {name}.
All details contained herein have been verified and are accurate
as of the date of generation.

CONTENT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Section 1 — Overview
This section provides a high-level summary of the subject matter
described by this document. All parties are advised to review
the complete contents before proceeding.

Section 2 — Details
Full legal description and relevant information specific to the
named subject has been recorded. Supporting evidence is appended
at the rear of this document.

Section 3 — Certification
This document has been certified and notarised. Any alterations
render this document void. Contact the issuing authority for
certified copies.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Page 1 of 1  |  CONFIDENTIAL",
            name = name,
            date = "22 Jun 2025",
            ref_num = "28471",
        ),
        "docx" => format!(
"{name}

Prepared by: Carly Asset Management
Date: 22 June 2025
Version: 1.0

──────────────────────────────────────

INTRODUCTION

This document outlines the key terms and conditions associated
with {name}. It has been prepared in accordance with applicable
regulations and internal policy.

MAIN BODY

The following details apply to the subject of this document:

  • All parties have been duly notified of their obligations.
  • The effective date is confirmed as 1 January 2025.
  • Terms are binding for a period of 12 months unless varied.
  • Renewal is subject to mutual agreement in writing.

SIGNATURE BLOCK

Authorised by: ________________________
Position:      Portfolio Manager
Date:          22 / 06 / 2025",
            name = name,
        ),
        "xlsx" => format!(
"┌─────────────────────────────────────────────────────────┐
│  {name:<55}│
│  Generated: 22 Jun 2025                                 │
├───────────────────┬──────────────┬──────────────────────┤
│  Description      │  Value       │  Notes               │
├───────────────────┼──────────────┼──────────────────────┤
│  Opening Balance  │  $1,200,000  │  FY2024              │
│  Acquisitions     │  $340,000    │  Q1-Q2               │
│  Disposals        │  -$80,000    │  Q3                  │
│  Revaluations     │  $120,000    │  Per valuer report   │
│  Closing Balance  │  $1,580,000  │  FY2025              │
├───────────────────┼──────────────┼──────────────────────┤
│  Net Change       │  +$380,000   │  +31.7%              │
└───────────────────┴──────────────┴──────────────────────┘

  Notes:
  All figures are in AUD. Subject to audit adjustment.
  Prepared by Finance — Internal Use Only.",
            name = name,
        ),
        "txt" => format!(
"Document: {name}
Date: 22 June 2025

This is a plain-text record associated with the above document.
No special formatting is required for this file type.

Key points:
- Document is current as of the date above.
- Retain for a minimum of 7 years per policy.
- Any queries should be directed to the portfolio manager.",
            name = name,
        ),
        _ => format!("Document: {name}\n\nNo preview available for this file type ({file_type}).",
            name = name, file_type = file_type),
    }
}

/// Document list modal — multi-tab viewer: open multiple docs simultaneously.
/// Tabs are pinned at the top; the list is always accessible via the "List" tab.
#[component]
pub fn DocModal(
    docs: Vec<Document>,
    title: String,
    on_close: impl Fn() + 'static,
    can_edit: bool,
    on_add: Option<Callback<String>>,
) -> impl IntoView {
    let app_store = use_app_store();
    // open_tabs: vec of (tab_id, Document); tab_id=0 is reserved for the list tab
    let (open_tabs, set_open_tabs) = signal::<Vec<(u32, Document)>>(vec![]);
    let (active_tab, set_active_tab) = signal::<u32>(0); // 0 = list view
    let (next_id, set_next_id) = signal(1u32);
    let (new_doc_name, set_new_doc_name) = signal(String::new());
    let docs_sig = StoredValue::new(docs);
    let title_stored = StoredValue::new(title);
    let on_close = std::rc::Rc::new(on_close);
    let on_close2 = on_close.clone();

    let open_doc_tab = move |doc: Document| {
        // don't duplicate — if already open, switch to it
        let existing = open_tabs.get().into_iter().find(|(_, d)| d.id == doc.id).map(|(id, _)| id);
        if let Some(id) = existing {
            set_active_tab.set(id);
            return;
        }
        let id = next_id.get();
        set_next_id.set(id + 1);
        set_open_tabs.update(|v| v.push((id, doc)));
        set_active_tab.set(id);
    };

    let close_tab = move |tid: u32| {
        set_open_tabs.update(|v| v.retain(|(id, _)| *id != tid));
        // fall back to list if this was the active tab
        set_active_tab.update(|cur| { if *cur == tid { *cur = 0; } });
    };

    view! {
        <div class="doc-modal-overlay" on:click=move |_| on_close()>
            <div class="doc-modal doc-modal-tabbed" on:click=|ev| ev.stop_propagation()>

                // ── Modal header ───────────────────────────────────────
                <div class="doc-modal-header">
                    <span class="doc-modal-title">"📄 " {title_stored.get_value()}</span>
                    <button class="doc-modal-close" on:click=move |_| on_close2()>"✕"</button>
                </div>

                // ── Tab strip (always visible at top) ──────────────────
                <div class="dv-tab-strip">
                    // List tab (always present)
                    <div class="dv-tab"
                        class:dv-tab-active=move || active_tab.get() == 0
                        on:click=move |_| set_active_tab.set(0)>
                        <span class="dv-tab-icon">"☰"</span>
                        <span class="dv-tab-name">"List"</span>
                    </div>
                    // Open document tabs
                    {move || open_tabs.get().into_iter().map(|(tid, doc)| {
                        let icon  = document_icon(&doc.file_type);
                        let dname = shorthand_name(&doc.name);
                        view! {
                            <div class="dv-tab"
                                class:dv-tab-active=move || active_tab.get() == tid
                                on:click=move |_| set_active_tab.set(tid)>
                                <span class="dv-tab-icon">{icon}</span>
                                <span class="dv-tab-name">{dname}</span>
                                <button class="dv-tab-close"
                                    on:click=move |ev| {
                                        ev.stop_propagation();
                                        close_tab(tid);
                                    }>"✕"</button>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                // ── Panel: list view (tab 0) ───────────────────────────
                {move || if active_tab.get() == 0 {
                    let on_add_cb = on_add.clone();
                    view! {
                        <div class="doc-modal-body">
                            <div class="doc-modal-list">
                                {docs_sig.get_value().into_iter().map(|doc| {
                                    let icon = document_icon(&doc.file_type);
                                    let ft   = doc.file_type.to_uppercase();
                                    let doc_for_open = doc.clone();
                                    let doc_for_tap = doc.clone();
                                    let doc_id = doc.id;
                                    let (editing_name, set_editing_name) = signal(false);
                                    let (edit_name, set_edit_name) = signal(doc.name.clone());
                                    view! {
                                        <div class="doc-modal-row">
                                            <span class="doc-modal-icon">{icon}</span>
                                            <div class="doc-modal-info"
                                                class:doc-modal-info-tap=can_edit
                                                on:click=move |ev: leptos::ev::MouseEvent| {
                                                    if can_edit && !editing_name.get() {
                                                        ev.stop_propagation();
                                                        open_doc_tab(doc_for_tap.clone());
                                                    }
                                                }
                                            >
                                                {move || if editing_name.get() {
                                                    view! {
                                                        <input
                                                            class="doc-modal-edit-input"
                                                            prop:value=move || edit_name.get()
                                                            on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                                            on:blur=move |_| {
                                                                let n = edit_name.get();
                                                                if !n.trim().is_empty() {
                                                                    app_store.update(|s| s.update_document_name(doc_id, n));
                                                                }
                                                                set_editing_name.set(false);
                                                            }
                                                        />
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="doc-modal-name">{doc.name.clone()}</span>
                                                        <span class="doc-modal-ft">{ft.clone()}</span>
                                                    }.into_any()
                                                }}
                                            </div>
                                            {move || if can_edit && !editing_name.get() {
                                                view! {
                                                    <button class="doc-modal-edit-btn"
                                                        on:click=move |_| set_editing_name.set(true)>
                                                        "✎"
                                                    </button>
                                                }.into_any()
                                            } else { ().into_any() }}
                                            <button class="doc-modal-open-btn"
                                                on:click=move |_| open_doc_tab(doc_for_open.clone())>
                                                "Open"
                                            </button>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            {if can_edit {
                                view! {
                                    <div class="doc-modal-add-row">
                                        <input class="doc-modal-add-input" type="text"
                                            placeholder="New document name…"
                                            prop:value=move || new_doc_name.get()
                                            on:input=move |ev| set_new_doc_name.set(event_target_value(&ev)) />
                                        <button class="doc-modal-add-btn"
                                            on:click=move |_| {
                                                let n = new_doc_name.get();
                                                if !n.trim().is_empty() {
                                                    if let Some(cb) = &on_add_cb { cb.run(n); }
                                                    set_new_doc_name.set(String::new());
                                                }
                                            }>
                                            "+ Add"
                                        </button>
                                    </div>
                                }.into_any()
                            } else { ().into_any() }}
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                // ── Panel: document viewer tabs ────────────────────────
                {move || {
                    let cur = active_tab.get();
                    open_tabs.get().into_iter().filter_map(|(tid, doc)| {
                        if tid != cur { return None; }
                        Some(view! {
                            <DocumentViewer
                                doc={doc}
                                on_close=move || close_tab(tid)
                                can_edit={can_edit}
                            />
                        })
                    }).collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}

/// In-app document viewer — sticky toolbar, zoom, edit mode, inline editing, image popup, why/save.
#[component]
pub fn DocumentViewer(
    doc: Document,
    on_close: impl Fn() + 'static,
    #[prop(default = false)] can_edit: bool,
) -> impl IntoView {
    let app_store = use_app_store();
    let undo_store = use_undo_redo_store();
    let initial_content = doc.content.clone().unwrap_or_else(|| mock_doc_content(&doc.name, &doc.file_type));
    let icon      = document_icon(&doc.file_type);
    let ft        = doc.file_type.to_uppercase();
    let is_sheet  = doc.file_type == "xlsx" || doc.file_type == "csv";
    let doc_name  = StoredValue::new(doc.name.clone());
    let doc_id = doc.id;

    // viewer state
    let (zoom, set_zoom)         = signal(100u32);       // percent
    let (edit_mode, set_edit_mode) = signal(can_edit);
    let (content, set_content)   = signal(initial_content);
    let (why, set_why)           = signal(String::new());
    // image popup: Some((x_px, y_px))
    let (img_popup, set_img_popup) = signal::<Option<(i32, i32)>>(None);
    let (link_val, set_link_val) = signal(String::new());

    let on_close = std::rc::Rc::new(on_close);
    let on_close_toolbar = on_close.clone();

    let save_doc = move || {
        let new_content = content.get();
        let reason = why.get();
        let reason_for_action = if reason.trim().is_empty() { None } else { Some(reason.clone()) };
        app_store.update(|s| {
            for p in s.portfolios.iter_mut() {
                for d in &mut p.documents {
                    if d.id == doc_id { d.content = Some(new_content.clone()); }
                }
                for g in &mut p.asset_groups {
                    for d in &mut g.documents {
                        if d.id == doc_id { d.content = Some(new_content.clone()); }
                    }
                    for a in &mut g.assets {
                        for d in &mut a.documents {
                            if d.id == doc_id { d.content = Some(new_content.clone()); }
                        }
                    }
                }
                for a in &mut p.assets {
                    for d in &mut a.documents {
                        if d.id == doc_id { d.content = Some(new_content.clone()); }
                    }
                }
            }
        });
        let (uid, name, role, org) = {
            let s = app_store.get();
            (s.current_user.id, s.current_user.name.clone(), format!("{:?}", s.current_user.role), s.current_user.organization_id)
        };
        undo_store.update(|u| {
            u.record_action(create_action(
                ActionType::Update,
                "Document",
                &format!("Updated content of document '{}'", doc_name.get_value()),
                uid,
                &name,
                &role,
                org,
                reason_for_action,
            ));
        });
        set_edit_mode.set(false);
    };

    view! {
        <div class="docviewer">
            // ── Sticky toolbar ────────────────────────────────────────
            <div class="docviewer-toolbar">
                <span class="docviewer-icon">{icon}</span>
                <span class="docviewer-name">{doc_name.get_value()}</span>
                <span class="docviewer-ft">{ft}</span>

                // Zoom controls
                <div class="dv-zoom-group">
                    <button class="dv-toolbar-btn"
                        on:click=move |_| set_zoom.update(|z| *z = (*z).saturating_sub(10).max(50))>
                        "−"
                    </button>
                    <span class="dv-zoom-label">{move || format!("{}%", zoom.get())}</span>
                    <button class="dv-toolbar-btn"
                        on:click=move |_| set_zoom.update(|z| *z = (*z + 10).min(300))>
                        "+"
                    </button>
                    <button class="dv-toolbar-btn"
                        on:click=move |_| set_zoom.set(100)>
                        "⟳"
                    </button>
                </div>

                // Edit toggle (only when can_edit)
                {if can_edit {
                    view! {
                        <button class="dv-toolbar-btn dv-edit-toggle"
                            class:dv-edit-active=move || edit_mode.get()
                            on:click=move |_| set_edit_mode.update(|v| *v = !*v)>
                            {move || if edit_mode.get() { "👁 Read" } else { "✎ Edit" }}
                        </button>
                        {move || if edit_mode.get() {
                            view! {
                                <button class="dv-toolbar-btn dv-save-btn" on:click=move |_| save_doc()>
                                    "✔ Save"
                                </button>
                            }.into_any()
                        } else { ().into_any() }}
                    }.into_any()
                } else { ().into_any() }}

                <button class="docviewer-back" on:click=move |_| on_close_toolbar()>"← Back"</button>
            </div>

            // ── Document body ─────────────────────────────────────────
            <div
                class={move || if is_sheet { "docviewer-body docviewer-sheet".to_string() } else { "docviewer-body".to_string() }}
                style=move || format!("font-size: {}%;", zoom.get())
                on:click=move |_| { if img_popup.get().is_some() { set_img_popup.set(None); } }
            >
                // Image area (shown for image-type docs or as a doc header image)
                {if can_edit {
                    view! {
                        <div class="dv-image-row">
                            <div
                                class="dv-doc-image-placeholder"
                                class:dv-editable=move || edit_mode.get()
                                on:click=move |ev: leptos::ev::MouseEvent| {
                                    if edit_mode.get() {
                                        ev.stop_propagation();
                                        set_img_popup.set(Some((ev.client_x(), ev.client_y())));
                                    }
                                }
                            >
                                {move || if edit_mode.get() {
                                    view! { <span class="dv-img-hint">"🖼 Click to set image"</span> }.into_any()
                                } else { view! { <span class="dv-img-hint dv-img-muted">"🖼"</span> }.into_any() }}
                            </div>
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                // Image option popup (appears at cursor position)
                {move || if let Some((cx, cy)) = img_popup.get() {
                    view! {
                        <div class="dv-img-popup"
                            style=move || format!("left:{}px;top:{}px;", cx, cy)
                            on:click=|ev| ev.stop_propagation()>
                            <div class="dv-img-popup-opt"
                                on:click=move |_| {
                                    // Simulate upload — in a real app this opens a file picker
                                    set_img_popup.set(None);
                                }
                            >
                                <span class="dv-img-opt-icon">"📁"</span>
                                <span>"Upload"</span>
                            </div>
                            <div class="dv-img-popup-opt">
                                <span class="dv-img-opt-icon">"🔗"</span>
                                <input
                                    class="dv-img-link-input"
                                    placeholder="Paste URL…"
                                    prop:value=move || link_val.get()
                                    on:input=move |ev| set_link_val.set(event_target_value(&ev))
                                    on:click=|ev| ev.stop_propagation()
                                />
                                <button class="dv-img-link-ok"
                                    on:click=move |_| { set_img_popup.set(None); }>
                                    "OK"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                // Text content — editable textarea in edit mode, pre otherwise
                {move || if edit_mode.get() {
                    view! {
                        <textarea
                            class="docviewer-content dv-editable-text"
                            prop:value=move || content.get()
                            on:input=move |ev| set_content.set(event_target_value(&ev))
                        />
                        <div class="dv-why-row">
                            <label class="dv-why-label">"Why are you making this change?"</label>
                            <textarea
                                class="dv-why-input"
                                placeholder="Optional reason for this update…"
                                prop:value=move || why.get()
                                on:input=move |ev| set_why.set(event_target_value(&ev))
                            />
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <pre class="docviewer-content">{move || content.get()}</pre>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
