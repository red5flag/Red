use crate::components::tabs::use_tab_edit_mode;
use crate::models::{Asset, AssetGroup, AssetStatus, Document, EntityNotificationSetting, Portfolio};
use crate::stores::{create_action, use_app_store, use_undo_redo_store};
use crate::types::{ActionType, AssetType, NotificationTrigger, NotificationType, SortMode, UserRole, ViewMode};
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, PartialEq)]
pub enum NotifTarget {
    Portfolio(Uuid),
    Group(Uuid, Uuid),
}

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
    let selected_id = move || app_store.get().selected_portfolio_id;
    let edit_mode = use_tab_edit_mode();
    let _ = edit_mode;
    let can_edit = move |org_id: Option<Uuid>| {
        let store = app_store.get();
        let role = org_id
            .map(|id| store.current_user_role_in_org(id))
            .unwrap_or_else(|| store.current_user.role.clone());
        role == UserRole::Owner || role == UserRole::Manager
    };

    let can_edit_documents = move |org_id: Option<Uuid>| {
        let store = app_store.get();
        let role = org_id
            .map(|id| store.current_user_role_in_org(id))
            .unwrap_or_else(|| store.current_user.role.clone());
        let mut user = store.current_user.clone();
        user.role = role;
        user.can_upload_documents()
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

    // Notification quick settings popover state
    let (notif_qs_target, set_notif_qs_target) = signal(Option::<(NotifTarget, String)>::None);

    // Context menu modal signals (Add Role, Add Organization)
    let (show_pf_add_role, set_show_pf_add_role) = signal(Option::<Uuid>::None);
    let (show_pf_add_org, set_show_pf_add_org) = signal(Option::<Uuid>::None);
    let (pf_new_role_name, set_pf_new_role_name) = signal(String::new());
    let (confirm_pf_remove, set_confirm_pf_remove) = signal(Option::<Uuid>::None);
    let (pf_new_role_desc, set_pf_new_role_desc) = signal(String::new());
    let (pf_new_org_name, set_pf_new_org_name) = signal(String::new());

    // Consume pending navigation from notification clicks — expand portfolio and open doc modal
    Effect::new(move |_| {
        if let Some(nav) = app_store.get().pending_nav_target {
            let pid = nav.portfolio_id;
            let doc_id = nav.doc_id;
            let gid = nav.group_id;
            let aid = nav.asset_id;
            app_store.update(|s| {
                s.selected_portfolio_id = Some(pid);
                s.touch_portfolio(pid);
                // Open doc modal for the entity that contains the document
                if let Some(did) = doc_id {
                    // Determine which entity to open the modal for
                    if let Some(aid) = aid {
                        s.open_doc_modal(aid);
                    } else if let Some(gid) = gid {
                        s.open_doc_modal(gid);
                    } else {
                        s.open_doc_modal(pid);
                    }
                }
                s.pending_nav_target = None;
            });
        }
    });

    let on_toggle_view = move |id: Uuid| {
        app_store.update(|s| {
            if s.selected_portfolio_id == Some(id) {
                s.selected_portfolio_id = None;
            } else {
                s.selected_portfolio_id = Some(id);
                s.touch_portfolio(id);
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
        let uploaded_by = app_store.get().current_user.id;
        let _asset = create_mock_asset(&name, new_asset_type.get(), value, value, uploaded_by);
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
        <div class="home-screen home-screen-pf">
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
                    <option value="sort_highest_value">"Sort: High Val"</option>
                    <option value="sort_lowest_value">"Sort: Low Val"</option>
                    <option value="sort_highest_profit">"Sort: High P&L"</option>
                    <option value="sort_lowest_profit">"Sort: Low P&L"</option>
                    <option value="sort_highest_revenue">"Sort: High Rev"</option>
                    <option value="sort_lowest_revenue">"Sort: Low Rev"</option>
                    <option value="sort_by_organization">"Sort: Org"</option>
                </select>
                <button
                    class="nav-portfolio-btn sort-direction-btn"
                    title={move || if app_store.get().sort_ascending { "Ascending ↑" } else { "Descending ↓" }}
                    on:click=move |_| app_store.update(|s| s.toggle_sort_direction())
                >
                    {move || if app_store.get().sort_ascending { "↑" } else { "↓" }}
                </button>
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
                                "Channel" => AssetType::Channel,
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
                        <option value="Channel">"Channel"</option>
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
                    let store = app_store.get();
                    let sort = if store.sort_ascending { store.reversed_sort_mode() } else { store.portfolio_sort_mode.clone() };
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
                        SortMode::ByOrganization => b.organization_id.cmp(&a.organization_id),
                    });
                    items.into_iter().map(move |portfolio| {
                        let portfolio_id = portfolio.id;
                        let org_id = portfolio.organization_id;
                        let is_expanded = selected_id() == Some(portfolio_id);
                        let can = can_edit(org_id);
                        let can_docs = can_edit_documents(org_id);

                        view! {
                            <PortfolioListItem
                                portfolio={portfolio}
                                can_edit={can}
                                can_edit_documents={can_docs}
                                expanded={is_expanded}
                                on_toggle=Callback::new(move |_| on_toggle_view(portfolio_id))
                                on_context=move |ev: leptos::ev::MouseEvent| {
                                    ev.prevent_default();
                                    set_context_menu.set(Some((portfolio_id, ev.client_x(), ev.client_y())));
                                }
                                on_open_notif_qs={Callback::new(move |(target, name)| set_notif_qs_target.set(Some((target, name))))}
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
                let pid_doc = pid;
                let pid_role = pid;
                let pid_org = pid;
                let pid_remove = pid;
                let org_id = app_store.get().portfolios.iter().find(|p| p.id == pid).and_then(|p| p.organization_id);
                let can = can_edit(org_id);
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
                                    on_toggle_view(pid);
                                }
                            >
                                "Overview"
                            </button>
                            {move || if can {
                                view! {
                                    <button
                                        class="context-menu-item"
                                        on:click=move |_| {
                                            set_context_menu.set(None);
                                            app_store.update(|s| s.open_doc_modal(pid_doc));
                                        }
                                    >
                                        "📄 Add Document"
                                    </button>
                                    <button
                                        class="context-menu-item"
                                        on:click=move |_| {
                                            set_context_menu.set(None);
                                            set_show_pf_add_role.set(Some(pid_role));
                                        }
                                    >
                                        "🎭 Add Role"
                                    </button>
                                    <button
                                        class="context-menu-item"
                                        on:click=move |_| {
                                            set_context_menu.set(None);
                                            set_show_pf_add_org.set(Some(pid_org));
                                        }
                                    >
                                        "🏢 Add Organization"
                                    </button>
                                    <button
                                        class="context-menu-item"
                                        on:click=move |_| {
                                            set_context_menu.set(None);
                                            set_confirm_pf_remove.set(Some(pid_remove));
                                        }
                                    >
                                        "🗑 Remove"
                                    </button>
                                }.into_any()
                            } else { ().into_any() }}
                        </div>
                    </div>
                }.into_any()
            })}

            // Notification quick settings popover
            {move || notif_qs_target.get().map(|(target, name)| {
                view! {
                    <NotificationQuickSettings
                        target={target}
                        entity_name={name}
                        on_close=move || set_notif_qs_target.set(None)
                    />
                }.into_any()
            })}

            // Add Role modal (portfolio context menu)
            {move || show_pf_add_role.get().map(|pid| {
                let org_id = app_store.get().portfolios.iter().find(|p| p.id == pid).and_then(|p| p.organization_id);
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_pf_add_role.set(None)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add Role"</span>
                                <button class="doc-modal-close" on:click=move |_| set_show_pf_add_role.set(None)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="Role name"
                                    prop:value={move || pf_new_role_name.get()}
                                    on:input=move |ev| set_pf_new_role_name.set(event_target_value(&ev)) />
                                <input class="login-input" type="text" placeholder="Description"
                                    prop:value={move || pf_new_role_desc.get()}
                                    on:input=move |ev| set_pf_new_role_desc.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| {
                                    let name = pf_new_role_name.get();
                                    let desc = pf_new_role_desc.get();
                                    if !name.trim().is_empty() {
                                        let role = crate::models::OrgRole::new(name, 0, desc, vec![]);
                                        if let Some(oid) = org_id {
                                            app_store.update(|s| s.add_role_to_org(oid, role));
                                        }
                                    }
                                    set_pf_new_role_name.set(String::new());
                                    set_pf_new_role_desc.set(String::new());
                                    set_show_pf_add_role.set(None);
                                }>"Add Role"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            })}

            // Add Organization modal (portfolio context menu)
            {move || show_pf_add_org.get().map(|pid| {
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_pf_add_org.set(None)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add Organization"</span>
                                <button class="doc-modal-close" on:click=move |_| set_show_pf_add_org.set(None)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="Organization name"
                                    prop:value={move || pf_new_org_name.get()}
                                    on:input=move |ev| set_pf_new_org_name.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| {
                                    let name = pf_new_org_name.get();
                                    if !name.trim().is_empty() {
                                        let owner_id = app_store.get().current_user.id;
                                        let org = crate::models::Organization::new(name, owner_id);
                                        let oid = org.id;
                                        app_store.update(|s| s.add_organization(org));
                                        app_store.update(|s| {
                                            if let Some(p) = s.get_portfolio_mut(pid) {
                                                p.organization_id = Some(oid);
                                            }
                                        });
                                    }
                                    set_pf_new_org_name.set(String::new());
                                    set_show_pf_add_org.set(None);
                                }>"Add Organization"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            })}
 
            // Confirm portfolio removal
            {move || confirm_pf_remove.get().map(|pid| {
                let pf_name = app_store.get().portfolios.iter()
                    .find(|p| p.id == pid)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| "this portfolio".to_string());
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_confirm_pf_remove.set(None)>
                        <div class="doc-modal confirm-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"🗑 Confirm Removal"</span>
                                <button class="doc-modal-close" on:click=move |_| set_confirm_pf_remove.set(None)>"✕"</button>
                            </div>
                            <div class="confirm-modal-body">
                                <p class="confirm-modal-msg">
                                    "Are you sure you want to remove "
                                    <strong>{pf_name.clone()}</strong>
                                    "? This action cannot be undone."
                                </p>
                                <div class="confirm-modal-actions">
                                    <button class="login-btn confirm-no"
                                        on:click=move |_| set_confirm_pf_remove.set(None)>
                                        "✕ No, Cancel"
                                    </button>
                                    <button class="login-btn sell confirm-yes"
                                        on:click=move |_| {
                                            set_confirm_pf_remove.set(None);
                                            on_delete_portfolio(pid);
                                        }>
                                        "✔ Yes, Remove"
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any()
            })}
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AssetSortMode {
    Recent,
    NameAsc,
    NameDesc,
    ValueHigh,
    ValueLow,
}

fn sort_assets(mut assets: Vec<Asset>, mode: AssetSortMode) -> Vec<Asset> {
    match mode {
        AssetSortMode::Recent => assets.sort_by(|a, b| b.last_accessed_at.cmp(&a.last_accessed_at)),
        AssetSortMode::NameAsc => assets.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        AssetSortMode::NameDesc => assets.sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase())),
        AssetSortMode::ValueHigh => assets.sort_by(|a, b| b.current_value.partial_cmp(&a.current_value).unwrap_or(std::cmp::Ordering::Equal)),
        AssetSortMode::ValueLow => assets.sort_by(|a, b| a.current_value.partial_cmp(&b.current_value).unwrap_or(std::cmp::Ordering::Equal)),
    }
    assets
}

fn sort_mode_label(m: AssetSortMode) -> &'static str {
    match m {
        AssetSortMode::Recent => "Recent",
        AssetSortMode::NameAsc => "Name A→Z",
        AssetSortMode::NameDesc => "Name Z→A",
        AssetSortMode::ValueHigh => "High Value",
        AssetSortMode::ValueLow => "Low Value",
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
    can_edit_documents: bool,
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
    on_open_notif_qs: Callback<(NotifTarget, String)>,
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

    // Auto-expand a group when notification navigation requests it
    Effect::new(move |_| {
        if let Some(gid) = app_store_inner.get().pending_group_expand {
            set_expanded_groups.update(|set| { set.insert(gid); });
            app_store_inner.update(|s| s.pending_group_expand = None);
        }
    });

    let (show_groups, set_show_groups) = signal(true);

    let (grid_columns, _set_grid_columns) = signal(3usize);
    let (selected_asset, set_selected_asset) = signal::<Option<Asset>>(None);

    // Asset sort state for grid view sections
    let (group_sort_open, set_group_sort_open) = signal(false);
    let (group_sort_mode, set_group_sort_mode) = signal(AssetSortMode::Recent);
    let (direct_sort_open, set_direct_sort_open) = signal(false);
    let (direct_sort_mode, set_direct_sort_mode) = signal(AssetSortMode::Recent);

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
    let portfolio_direct_sort = portfolio.clone();

    view! {
        <div class="asset-viewer">
            // Asset Groups section
            {if !portfolio_groups.asset_groups.is_empty() {
                view! {
            <div class="asset-section">
                <div class="asset-section-title">
                    <div class="asset-section-title-left"
                        on:click=move |_| set_show_groups.update(|v| *v = !*v)
                    >
                        <span class="asset-section-arrow">
                            {move || if show_groups.get() { "▼" } else { "▶" }}
                        </span>
                        <span class="asset-section-label">"Asset Groups"</span>
                    </div>
                    <div class="section-title-right">
                        {{
                            let view_mode_groups_title = view_mode_groups_title.clone();
                            move || if show_groups.get() && view_mode_groups_title == ViewMode::Grid {
                                ().into_any()
                            } else { ().into_any() }
                        }}
                    </div>
                </div>

                {move || if show_groups.get() {
                    let visible_groups: Vec<_> = portfolio_groups.asset_groups.clone().into_iter().filter(|g| portfolio_visible_to_user || g.is_visible_to(user_id, can_view_all)).collect();
                    let vmg = view_mode_groups_content.clone();
                    view! {
                        <div>
                            // Sort dropdown inside content area (grid mode only)
                            {let vg_for_sort = visible_groups.clone();
                            move || {
                                if vmg == ViewMode::Grid && !vg_for_sort.is_empty() {
                                    view! {
                                        <div class="sort-dropdown-wrap sort-dropdown-inline">
                                            <button class="sort-btn"
                                                on:click=move |_| set_group_sort_open.update(|v| *v = !*v)
                                            >{format!("Sort: {} ↕", sort_mode_label(group_sort_mode.get()))}</button>
                                            {move || if group_sort_open.get() {
                                                view! {
                                                    <div class="sort-dropdown" on:click=|ev| ev.stop_propagation()>
                                                        {[
                                                            AssetSortMode::Recent,
                                                            AssetSortMode::NameAsc,
                                                            AssetSortMode::NameDesc,
                                                            AssetSortMode::ValueHigh,
                                                            AssetSortMode::ValueLow,
                                                        ].iter().map(|&m| {
                                                            let set_m = set_group_sort_mode;
                                                            let close = set_group_sort_open;
                                                            view! {
                                                                <button class="sort-dropdown-item"
                                                                    class:active={move || group_sort_mode.get() == m}
                                                                    on:click=move |_| {
                                                                        set_m.set(m);
                                                                        close.set(false);
                                                                    }
                                                                >{sort_mode_label(m)}</button>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                }.into_any()
                                            } else { ().into_any() }}
                                        </div>
                                    }.into_any()
                                } else { ().into_any() }
                            }}

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
                                        {visible_groups.into_iter().enumerate().map(move |(idx, group)| {
                                            let gid = group.id;
                                            let pid2 = pid;
                                            let is_expanded = Memo::new(move |_| expanded_groups.get().contains(&gid));
                                            view! {
                                                <AssetGroupItem
                                                    group={group}
                                                    can_edit={can_edit}
                                                    can_edit_documents={can_edit_documents}
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
                                                    tint_index={idx + 1}
                                                    on_open_notif_qs={on_open_notif_qs.clone()}
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
                }.into_any()
            } else { ().into_any() }}

            // Direct Assets section — always visible, no dropdown toggle
            <div class="asset-section">
                <div class="asset-section-title">
                    <div class="asset-section-title-left">
                        <span class="asset-section-label">"Direct Assets"</span>
                    </div>
                    <div class="section-title-right">
                        {move || {
                            let vmd = view_mode_direct_title.clone();
                            if vmd == ViewMode::Grid && !portfolio_direct_sort.assets.is_empty() {
                                view! {
                                    <div class="sort-dropdown-wrap sort-dropdown-inline">
                                        <button class="sort-btn"
                                            on:click=move |_| set_direct_sort_open.update(|v| *v = !*v)
                                        >{format!("Sort: {} ↕", sort_mode_label(direct_sort_mode.get()))}</button>
                                        {move || if direct_sort_open.get() {
                                            view! {
                                                <div class="sort-dropdown" on:click=|ev| ev.stop_propagation()>
                                                    {[
                                                        AssetSortMode::Recent,
                                                        AssetSortMode::NameAsc,
                                                        AssetSortMode::NameDesc,
                                                        AssetSortMode::ValueHigh,
                                                        AssetSortMode::ValueLow,
                                                    ].iter().map(|&m| {
                                                        let set_m = set_direct_sort_mode;
                                                        let close = set_direct_sort_open;
                                                        view! {
                                                            <button class="sort-dropdown-item"
                                                                class:active={move || direct_sort_mode.get() == m}
                                                                on:click=move |_| {
                                                                    set_m.set(m);
                                                                    close.set(false);
                                                                }
                                                            >{sort_mode_label(m)}</button>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            }.into_any()
                                        } else { ().into_any() }}
                                    </div>
                                }.into_any()
                            } else { ().into_any() }
                        }}
                    </div>
                </div>

                {move || {
                    let visible_direct_assets: Vec<_> = portfolio_direct.assets.clone().into_iter().filter(|a| portfolio_visible_to_user || a.is_visible_to(user_id, can_view_all)).collect();
                    let visible_direct_assets = sort_assets(visible_direct_assets, direct_sort_mode.get());
                    let _vmd = view_mode_direct_content.clone();
                    view! {
                        <div>

                            {move || {
                                if show_add_asset.get() == AssetTarget::PortfolioDirect(pid) {
                                    view! {
                                        <div class="add-form">
                                            <input class="login-input" type="text" placeholder="Asset name"
                                                on:input=move |ev| set_new_asset_name.set(event_target_value(&ev)) />
                                            <select class="login-input"
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
                                                        "Channel" => AssetType::Channel,
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
                                                <option value="Channel">"Channel"</option>
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
                                        {visible_direct_assets.into_iter().enumerate().map(move |(idx, asset)| view! {
                                            <AssetItem asset={asset} portfolio_name={portfolio_name.clone()} portfolio_id={Some(pid)} group_id={None} view_mode={view_mode_clone.clone()} on_select={on_select_asset} can_edit={can_edit} can_edit_documents={can_edit_documents} tint_index={idx + 1} />
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    }.into_any()
                }}
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
    #[prop(default = false)] can_edit_documents: bool,
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
    #[prop(default = 0)] tint_index: usize,
    on_open_notif_qs: Callback<(NotifTarget, String)>,
) -> impl IntoView {
    let app_store = use_app_store();
    let _ = view_mode;

    let current_user = app_store.get().current_user.clone();
    let user_id = current_user.id;
    let can_view_all = current_user.can_view_all();
    let group_visible_to_user = group.is_visible_to(user_id, can_view_all);

    let can_edit_here = can_edit;
    let can_edit_documents_here = can_edit_documents;

    let (is_editing, set_is_editing) = signal(false);
    let (group_context_menu, set_group_context_menu) = signal(Option::<(i32, i32)>::None);
    let (show_group_add_role, set_show_group_add_role) = signal(false);
    let (show_group_add_org, set_show_group_add_org) = signal(false);
    let (confirm_group_remove, set_confirm_group_remove) = signal(false);
    let (group_role_name, set_group_role_name) = signal(String::new());
    let (group_role_desc, set_group_role_desc) = signal(String::new());
    let (group_org_name, set_group_org_name) = signal(String::new());
    let (edit_name, set_edit_name) = signal(group.name.clone());
    let (edit_desc, set_edit_desc) = signal(group.description.clone().unwrap_or_default());

    let g_name = group.name.clone();
    let g_desc = group.description.clone().unwrap_or_default();
    let g_name_for_modal = group.name.clone();
    let g_name_for_confirm = group.name.clone();
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
        set_is_editing.set(false);
    };

    let add_group_doc = move |n: String| {
        if n.trim().is_empty() { return; }
        let uploaded_by = app_store.get().current_user.id;
        let ft = detect_file_type(&n);
        let doc = crate::models::Document {
            id: Uuid::new_v4(),
            name: n.clone(),
            file_type: ft,
            content: None,
            url: "#".to_string(),
            uploaded_at: chrono::Utc::now(),
            uploaded_by,
        };
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                    g.documents.push(doc);
                }
            }
        });
    };

    let group_tint_style = format!("background: rgba(255,255,255,{});", (tint_index as f64 * 0.1).min(0.9));

    view! {
        <div class="asset-group" class:expanded={move || expanded.get()} style={group_tint_style.clone()}
            on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                if can_edit_here {
                    ev.prevent_default();
                    set_group_context_menu.set(Some((ev.client_x(), ev.client_y())));
                }
            }
        >
            <div class="asset-group-header"
                on:click=move |_| if !is_editing.get() { on_toggle.run(gid) }
                on:dblclick=move |ev| { if can_edit_here { ev.stop_propagation(); set_is_editing.set(true); } }
            >
                <span class="asset-group-arrow">
                    {move || if expanded.get() { "▲" } else { "▼" }}
                </span>
                <div class="asset-group-icon">"📁"</div>
                <div class="asset-group-info-wrap" on:click=|ev| ev.stop_propagation()>
                    {let asset_count = group.assets.len();
                    let g_name_header = g_name.clone();
                    let g_desc_header = g_desc.clone();
                    move || if is_editing.get() && can_edit_here {
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
                    {move || {
                        let count = app_store.get().doc_notifications_for_group(pid, gid);
                        let gname = g_name.clone();
                        view! {
                            <span class="pf-notif-badge pf-notif-badge-clickable"
                                title="Notification settings"
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    on_open_notif_qs.run((NotifTarget::Group(pid, gid), gname.clone()));
                                }>
                                "🔔"
                                {move || if count > 0 {
                                    Some(view! { <span class="pf-notif-count">{count}</span> })
                                } else {
                                    None
                                }}
                            </span>
                        }.into_any()
                    }}
                    <button class="pf-action-btn"
                        class:active=move || app_store.get().is_doc_modal_open(gid)
                        on:click=move |_| app_store.update(|s| s.toggle_doc_modal(gid))>
                        {format!("📄 {}", doc_count)}
                    </button>
                </div>
            </div>
            // Docs modal for group
            {move || if app_store.get().is_doc_modal_open(gid) {
                let modal_title = g_name_for_modal.clone();
                let add_cb = if can_edit_documents_here { Some(Callback::new(move |n: String| add_group_doc(n))) } else { None };
                view! {
                    <DocModal
                        entity_id={gid}
                        title={modal_title}
                        on_close=move || app_store.update(|s| s.close_doc_modal(gid))
                        can_edit={can_edit_documents_here}
                        on_add={add_cb}
                        portfolio_id={Some(pid)}
                        group_id={Some(gid)}
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
                                prop:value={move || format!("{:?}", _new_asset_type.get())}
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
                                        "Channel" => AssetType::Channel,
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
                                <option value="Channel">"Channel"</option>
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
                            {group_assets.into_iter().enumerate().map({
                                let view_mode = view_mode.clone();
                                move |(idx, asset)| view! {
                                    <AssetItem asset={asset} portfolio_name={portfolio_name.clone()} portfolio_id={Some(pid)} group_id={Some(gid)} view_mode={view_mode.clone()} on_select={on_select_asset} can_edit={can_edit_here} can_edit_documents={can_edit_documents_here} tint_index={idx + 1} />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }
                }}
            </div>

            // Context menu for group press-and-hold
            {move || group_context_menu.get().map(|(x, y)| {
                let pid2 = pid;
                let gid2 = gid;
                view! {
                    <div class="context-menu-overlay" on:click=move |_| set_group_context_menu.set(None)>
                        <div class="context-menu" style={format!("left: {}px; top: {}px;", x, y)}>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_group_context_menu.set(None);
                                    app_store.update(|s| s.open_doc_modal(gid));
                                }
                            >"📄 Add Document"</button>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_group_context_menu.set(None);
                                    set_show_group_add_role.set(true);
                                }
                            >"🎭 Add Role"</button>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_group_context_menu.set(None);
                                    set_show_group_add_org.set(true);
                                }
                            >"🏢 Add Organization"</button>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_group_context_menu.set(None);
                                    set_confirm_group_remove.set(true);
                                }
                            >"🗑 Remove"</button>
                        </div>
                    </div>
                }.into_any()
            })}

            // Add Role modal (group context menu)
            {move || if show_group_add_role.get() {
                let org_id = app_store.get().portfolios.iter()
                    .find(|p| p.id == pid)
                    .and_then(|p| p.organization_id);
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_group_add_role.set(false)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add Role"</span>
                                <button class="doc-modal-close" on:click=move |_| set_show_group_add_role.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="Role name"
                                    prop:value={move || group_role_name.get()}
                                    on:input=move |ev| set_group_role_name.set(event_target_value(&ev)) />
                                <input class="login-input" type="text" placeholder="Description"
                                    prop:value={move || group_role_desc.get()}
                                    on:input=move |ev| set_group_role_desc.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| {
                                    let name = group_role_name.get();
                                    let desc = group_role_desc.get();
                                    if !name.trim().is_empty() {
                                        let role = crate::models::OrgRole::new(name, 0, desc, vec![]);
                                        if let Some(oid) = org_id {
                                            app_store.update(|s| s.add_role_to_org(oid, role));
                                        }
                                    }
                                    set_group_role_name.set(String::new());
                                    set_group_role_desc.set(String::new());
                                    set_show_group_add_role.set(false);
                                }>"Add Role"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}

            // Add Organization modal (group context menu)
            {move || if show_group_add_org.get() {
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_group_add_org.set(false)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add Organization"</span>
                                <button class="doc-modal-close" on:click=move |_| set_show_group_add_org.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="Organization name"
                                    prop:value={move || group_org_name.get()}
                                    on:input=move |ev| set_group_org_name.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| {
                                    let name = group_org_name.get();
                                    if !name.trim().is_empty() {
                                        let owner_id = app_store.get().current_user.id;
                                        let org = crate::models::Organization::new(name, owner_id);
                                        let oid = org.id;
                                        app_store.update(|s| s.add_organization(org));
                                        // Link group's portfolio to org if not already linked
                                        app_store.update(|s| {
                                            if let Some(p) = s.get_portfolio_mut(pid) {
                                                if p.organization_id.is_none() {
                                                    p.organization_id = Some(oid);
                                                }
                                            }
                                        });
                                    }
                                    set_group_org_name.set(String::new());
                                    set_show_group_add_org.set(false);
                                }>"Add Organization"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}

            // Confirm group removal
            {move || if confirm_group_remove.get() {
                let gname = g_name_for_confirm.clone();
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_confirm_group_remove.set(false)>
                        <div class="doc-modal confirm-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"🗑 Confirm Removal"</span>
                                <button class="doc-modal-close" on:click=move |_| set_confirm_group_remove.set(false)>"✕"</button>
                            </div>
                            <div class="confirm-modal-body">
                                <p class="confirm-modal-msg">
                                    "Are you sure you want to remove "
                                    <strong>{gname.clone()}</strong>
                                    "? This action cannot be undone."
                                </p>
                                <div class="confirm-modal-actions">
                                    <button class="login-btn confirm-no"
                                        on:click=move |_| set_confirm_group_remove.set(false)>
                                        "✕ No, Cancel"
                                    </button>
                                    <button class="login-btn sell confirm-yes"
                                        on:click=move |_| {
                                            set_confirm_group_remove.set(false);
                                            app_store.update(|s| { s.remove_asset_group(pid, gid); });
                                        }>
                                        "✔ Yes, Remove"
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </div>
    }
}

/// Portfolio list row — accordion style matching AssetGroupItem.
#[component]
fn PortfolioListItem(
    portfolio: crate::models::Portfolio,
    #[prop(default = false)] can_edit: bool,
    #[prop(default = false)] can_edit_documents: bool,
    expanded: bool,
    on_toggle: Callback<()>,
    on_context: impl Fn(leptos::ev::MouseEvent) + 'static,
    on_open_notif_qs: Callback<(NotifTarget, String)>,
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
    let (is_editing_name, set_is_editing_name) = signal(false);
    let (is_editing_desc, set_is_editing_desc) = signal(false);
    let (is_editing_org, set_is_editing_org) = signal(false);
    let (edit_name, set_edit_name) = signal(portfolio.name.clone());
    let (edit_desc, set_edit_desc) = signal(portfolio.description.clone().unwrap_or_default());
    let pid = portfolio.id;
    let doc_count = portfolio.documents.len();
    let name = portfolio.name.clone();
    let name_for_modal = portfolio.name.clone();
    let desc = portfolio.description.clone().unwrap_or_default();
    let asset_count = portfolio.get_all_assets().len();
    let can_edit_here = can_edit;
    let can_edit_documents_here = can_edit_documents;
    let org_name = portfolio.organization_id.and_then(|oid| {
        app_store.get().organizations.iter().find(|o| o.id == oid).map(|o| o.name.clone())
    });
    let org_color = portfolio.organization_id.and_then(|oid| {
        app_store.get().organizations.iter().find(|o| o.id == oid)
            .and_then(|o| o.settings.color.clone())
    });
    let current_org_id = portfolio.organization_id;
    let orgs = app_store.get().organizations.clone();

    let save_edit = move |_: leptos::ev::FocusEvent| {
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
        set_is_editing_name.set(false);
        set_is_editing_desc.set(false);
    };

    let save_edit_now = move || {
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
        set_is_editing_name.set(false);
        set_is_editing_desc.set(false);
    };

    let save_org_edit = move |ev: leptos::ev::Event| {
        let v = event_target_value(&ev);
        let new_org_id = if v == "none" {
            None
        } else {
            Uuid::parse_str(&v).ok()
        };
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                p.organization_id = new_org_id;
                p.updated_at = chrono::Utc::now();
            }
        });
        set_is_editing_org.set(false);
    };

    let add_doc = move |n: String| {
        if n.trim().is_empty() { return; }
        let uploaded_by = app_store.get().current_user.id;
        let ft = detect_file_type(&n);
        let doc = crate::models::Document {
            id: Uuid::new_v4(),
            name: n.clone(),
            file_type: ft,
            url: "#".to_string(),
            uploaded_at: chrono::Utc::now(),
            uploaded_by,
            content: None,
        };
        app_store.update(|s| {
            s.add_document_to_portfolio(pid, doc);
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
            // Org color strip — left-side color coding for organization identification
            {org_color.as_ref().map(|c| view! {
                <div class="pf-org-color-strip" style={format!("background: {}", c)}></div>
            })}
            // Header row — same structure as asset-group-header
            <div class="asset-group-header"
                style={org_color.as_ref().map(|c| format!("border-left: 3px solid {}", c)).unwrap_or_default()}
                on:click=move |_| {
                    if !is_editing_name.get() && !is_editing_desc.get() && !is_editing_org.get() {
                        on_toggle.run(());
                    }
                }
            >
                <span class="asset-group-arrow">
                    {if expanded { "▲" } else { "▼" }}
                </span>
                <div class="asset-group-icon">"🏢"</div>
                <div class="asset-group-info-wrap" on:click=|ev| ev.stop_propagation()>
                    {let name_header = name.clone();
                    let desc_header = desc.clone();
                    move || {
                        let mut parts: Vec<leptos::prelude::AnyView> = Vec::new();
                        // Organization label / editor
                        if can_edit_here {
                            if is_editing_org.get() {
                                parts.push(view! {
                                    <select class="pf-edit-input pf-org-select"
                                        prop:value={move || current_org_id.map(|id| id.to_string()).unwrap_or_else(|| "none".to_string())}
                                        on:change=save_org_edit
                                        on:blur=move |_| set_is_editing_org.set(false)
                                    >
                                        <option value="none">"No Organization"</option>
                                        {orgs.iter().map(|o| {
                                            let oid = o.id.to_string();
                                            let oname = o.name.clone();
                                            view! {
                                                <option value={oid.clone()}>{oname}</option>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </select>
                                }.into_any());
                            } else if let Some(on) = &org_name {
                                parts.push(view! {
                                    <div class="pf-org-label"
                                        on:dblclick=move |ev| { ev.stop_propagation(); set_is_editing_org.set(true); }
                                    >{on.clone()}</div>
                                }.into_any());
                            }
                        } else if let Some(on) = &org_name {
                            parts.push(view! { <div class="pf-org-label">{on.clone()}</div> }.into_any());
                        }
                        // Name
                        if is_editing_name.get() && can_edit_here {
                            parts.push(view! {
                                <input class="pf-edit-input" placeholder="Portfolio name"
                                    prop:value=move || edit_name.get()
                                    on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                    on:blur=save_edit
                                    on:keydown=move |ev| { if ev.key() == "Enter" { save_edit_now(); } }
                                />
                            }.into_any());
                        } else {
                            let set_editing = set_is_editing_name;
                            parts.push(view! {
                                <div class="asset-group-name"
                                    on:dblclick=move |ev| { if can_edit_here { ev.stop_propagation(); set_editing.set(true); } }
                                >{name_header.clone()}</div>
                            }.into_any());
                        }
                        // Description
                        if is_editing_desc.get() && can_edit_here {
                            parts.push(view! {
                                <input class="pf-edit-input" placeholder="Description"
                                    prop:value=move || edit_desc.get()
                                    on:input=move |ev| set_edit_desc.set(event_target_value(&ev))
                                    on:blur=save_edit
                                    on:keydown=move |ev| { if ev.key() == "Enter" { save_edit_now(); } }
                                />
                            }.into_any());
                        } else if !desc_header.is_empty() {
                            let set_editing = set_is_editing_desc;
                            parts.push(view! {
                                <div class="asset-group-desc"
                                    on:dblclick=move |ev| { if can_edit_here { ev.stop_propagation(); set_editing.set(true); } }
                                >{desc_header.clone()}</div>
                            }.into_any());
                        }
                        // Asset count — double-click to expand
                        parts.push(view! {
                            <div class="asset-group-count"
                                on:dblclick=move |ev| { ev.stop_propagation(); on_toggle.run(()); }
                            >
                                {format!("{} asset{}", asset_count, if asset_count == 1 { "" } else { "s" })}
                            </div>
                        }.into_any());
                        parts.collect_view().into_any()
                    }}
                </div>
                // Action strip — double-click on docs opens doc modal
                <div class="pf-list-actions" on:click=|ev| ev.stop_propagation()>
                    {move || {
                        let count = app_store.get().doc_notifications_for_portfolio(pid);
                        let pname = name.clone();
                        view! {
                            <span class="pf-notif-badge pf-notif-badge-clickable"
                                title="Notification settings"
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    on_open_notif_qs.run((NotifTarget::Portfolio(pid), pname.clone()));
                                }>
                                "🔔"
                                {move || if count > 0 {
                                    Some(view! { <span class="pf-notif-count">{count}</span> })
                                } else {
                                    None
                                }}
                            </span>
                        }.into_any()
                    }}
                    <button class="pf-action-btn"
                        class:active=move || app_store.get().is_doc_modal_open(pid)
                        on:click=move |_| app_store.update(|s| s.toggle_doc_modal(pid))
                        on:dblclick=move |ev| { if can_edit_here { ev.stop_propagation(); app_store.update(|s| s.open_doc_modal(pid)); } }
                    >
                        {format!("📄 {}", doc_count)}
                    </button>
                </div>
            </div>

            // Docs modal for portfolio
            {move || if app_store.get().is_doc_modal_open(pid) {
                let modal_title = name_for_modal.clone();
                let add_cb = if can_edit_documents_here { Some(Callback::new(move |n: String| add_doc(n))) } else { None };
                view! {
                    <DocModal
                        entity_id={pid}
                        title={modal_title}
                        on_close=move || app_store.update(|s| s.close_doc_modal(pid))
                        can_edit={can_edit_documents_here}
                        on_add={add_cb}
                        portfolio_id={Some(pid)}
                    />
                }.into_any()
            } else { ().into_any() }}

            {move || if is_editing_org.get() && can_edit_here {
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
                    can_edit_documents={can_edit_documents_here}
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
                    on_open_notif_qs={on_open_notif_qs.clone()}
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
        AssetType::Channel => "Channel",
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
}

#[component]
fn AssetItem(
    asset: Asset,
    portfolio_name: String,
    #[prop(default = None)] portfolio_id: Option<Uuid>,
    #[prop(default = None)] group_id: Option<Uuid>,
    view_mode: ViewMode,
    on_select: Callback<Asset>,
    #[prop(default = false)] can_edit: bool,
    #[prop(default = false)] can_edit_documents: bool,
    #[prop(default = 0)] tint_index: usize,
) -> impl IntoView {
    let app_store = use_app_store();
    let image_url = asset
        .images
        .first()
        .cloned()
        .unwrap_or_else(|| asset_placeholder_url(&asset.asset_type, &asset.name));

    let (expanded_detail, set_expanded_detail) = signal(false);
    let (_editing, set_editing) = signal(false);
    let (asset_context_menu, set_asset_context_menu) = signal(Option::<(i32, i32)>::None);
    let (show_add_user, set_show_add_user) = signal(false);
    let (show_add_role, set_show_add_role) = signal(false);
    let (show_add_org, set_show_add_org) = signal(false);
    let (show_add_transaction, set_show_add_transaction) = signal(false);
    let (confirm_asset_remove, set_confirm_asset_remove) = signal(false);
    // Form fields for add user
    let (new_user_name, set_new_user_name) = signal(String::new());
    let (new_user_email, set_new_user_email) = signal(String::new());
    // Form fields for add role
    let (new_role_name, set_new_role_name) = signal(String::new());
    let (new_role_desc, set_new_role_desc) = signal(String::new());
    // Form fields for add org
    let (new_org_name, set_new_org_name) = signal(String::new());
    // Form fields for add transaction
    let (new_tx_amount, set_new_tx_amount) = signal(String::new());
    let (new_tx_desc, set_new_tx_desc) = signal(String::new());
    let (new_tx_type, set_new_tx_type) = signal(crate::types::TransactionType::Purchase);
    let (edit_name, set_edit_name) = signal(asset.name.clone());
    let (edit_desc, set_edit_desc) = signal(asset.description.clone().unwrap_or_default());
    let (edit_loc, set_edit_loc) = signal(asset.location.clone().unwrap_or_default());

    let can_edit_here = can_edit;
    let can_edit_documents_here = can_edit_documents;
    // doc sort: 0 = recent, 1 = name
    let (doc_sort, set_doc_sort) = signal(0u8);
    let (detail_tab, set_detail_tab) = signal(0u8);

    let asset_id = asset.id;
    let pname = portfolio_name.clone();
    let docs = asset.documents.clone();
    let _doc_count = docs.len();

    // Reactive document list for this asset (read from store so it updates on add)
    let asset_docs_reactive = Memo::new(move |_| {
        app_store.get().portfolios.iter()
            .flat_map(|p| p.assets.iter().chain(p.asset_groups.iter().flat_map(|g| g.assets.iter())))
            .find(|a| a.id == asset_id)
            .map(|a| a.documents.clone())
            .unwrap_or_default()
    });
    let a_name = asset.name.clone();
    let a_addr = asset.location.clone().unwrap_or_default();
    let a_addr_grid = a_addr.clone();
    let a_name_tx = a_name.clone();
    let a_org_id = asset.organization_id;
    let a_org_name = move || {
        app_store.get().organizations.iter()
            .find(|o| Some(o.id) == a_org_id)
            .map(|o| o.name.clone())
            .unwrap_or_else(|| "—".to_string())
    };
    let asset_name_for_modal = asset.name.clone();
    let asset_name_for_confirm = asset.name.clone();
    let (_asset_name_signal, _set_asset_name) = signal(a_name.clone());
    // snapshot values for the detail panel
    let a_type     = format!("{:?}", asset.asset_type);
    let a_type_grid = a_type.clone();
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
        let uploaded_by = app_store.get().current_user.id;
        let ft = detect_file_type(&n);
        let doc = crate::models::Document {
            id: Uuid::new_v4(),
            name: n.clone(),
            file_type: ft,
            url: "#".to_string(),
            uploaded_at: chrono::Utc::now(),
            uploaded_by,
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
    let add_cb = if can_edit_documents_here { Some(Callback::new(add_doc)) } else { None };

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

    let tint_style = format!("background: rgba(255,255,255,{});", (tint_index as f64 * 0.1).min(0.9));

    if view_mode == ViewMode::Grid {
        let asset_for_click = asset.clone();
        let short_name = shorthand_name(&a_name);
        view! {
            <div class="asset-grid-card" style={tint_style.clone()} on:click=move |_| on_select.run(asset_for_click.clone())>
                <img class="asset-grid-image" src={image_url.clone()} alt={a_name.clone()} />
                <div class="asset-grid-name">{short_name}</div>
            </div>
        }.into_any()
    } else {
    view! {
        <div class="ai-item" class:ai-item-expanded={move || expanded_detail.get()} style={tint_style.clone()}
            on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                if can_edit_here {
                    ev.prevent_default();
                    set_asset_context_menu.set(Some((ev.client_x(), ev.client_y())));
                }
            }
        >
            <div class="ai-list-card">
                <img class="ai-list-image" src={image_url.clone()} alt={a_name.clone()} />
                <div class="ai-list-body">
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
                    } else { ().into_any() }}
                    // Detail grid inline (always visible)
                    <div class="pf-detail-grid pf-detail-grid-inline">
                        <div class="pf-detail-cell">
                            <span class="pf-detail-label">"TYPE & BUILD"</span>
                            <span class="pf-detail-value">{a_type_grid.clone()}</span>
                        </div>
                        <div class="pf-detail-cell">
                            <span class="pf-detail-label">"ADDRESS"</span>
                            <span class="pf-detail-value">{a_addr_grid.clone()}</span>
                        </div>
                        <div class="pf-detail-cell">
                            <span class="pf-detail-label">"ORGANIZATION"</span>
                            <span class="pf-detail-value">{a_org_name()}</span>
                        </div>
                        <div class="pf-detail-cell">
                            <span class="pf-detail-label">"PRICE"</span>
                            <span class="pf-detail-value">{format!("${:.2}", a_current_val)}</span>
                        </div>
                    </div>
                    // Horizontal document slider with + Document card
                    <div class="ai-doc-slider" on:click=|ev| ev.stop_propagation()>
                        // + Document card (always first)
                        <div class="ai-doc-slider-item ai-doc-add-card"
                            on:click=move |_| app_store.update(|s| s.toggle_doc_modal(asset_id))>
                            <div class="ai-doc-slider-thumb">"➕"</div>
                            <div class="ai-doc-slider-name">"+ Document"</div>
                            <div class="ai-doc-slider-type">"ADD"</div>
                        </div>
                        {move || {
                            let asset_docs = asset_docs_reactive.get();
                            asset_docs.into_iter().map(|doc| {
                                let icon = document_icon(&doc.file_type);
                                let ft = doc.file_type.to_uppercase();
                                let dname = doc.name.clone();
                                let short_name = if dname.len() > 18 {
                                    format!("{}...", &dname[..15])
                                } else {
                                    dname.clone()
                                };
                                let doc_for_view = doc.clone();
                                let doc_id = doc.id;
                                let doc_id_for_notif = doc.id;
                                let (doc_ctx_menu_x, set_doc_ctx_menu_x) = signal(0i32);
                                let (doc_ctx_menu_y, set_doc_ctx_menu_y) = signal(0i32);
                                let (show_doc_ctx_menu, set_show_doc_ctx_menu) = signal(false);
                                let (viewing, set_viewing) = signal(false);
                                view! {
                                    <div class="ai-doc-slider-item"
                                        on:click=move |_| set_viewing.set(true)
                                        on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                                            ev.prevent_default();
                                            ev.stop_propagation();
                                            set_doc_ctx_menu_x.set(ev.client_x());
                                            set_doc_ctx_menu_y.set(ev.client_y());
                                            set_show_doc_ctx_menu.set(true);
                                        }
                                    >
                                        <div class="ai-doc-slider-thumb">{icon}</div>
                                        <div class="ai-doc-slider-name">{short_name}</div>
                                        <div class="ai-doc-slider-type">{ft.clone()}</div>
                                        {move || {
                                            let ncount = app_store.get().notifications_for_doc(doc_id_for_notif);
                                            if ncount > 0 {
                                                view! {
                                                    <span class="pf-notif-badge pf-notif-badge-inline" title={format!("{} notification{}", ncount, if ncount == 1 { "" } else { "s" })}>
                                                        "🔔"
                                                        <span class="pf-notif-count">{ncount}</span>
                                                    </span>
                                                }.into_any()
                                            } else { ().into_any() }
                                            }}
                                    </div>
                                    {move || if viewing.get() {
                                        let d = doc_for_view.clone();
                                        view! {
                                            <div class="doc-modal-overlay" on:click=move |_| set_viewing.set(false)>
                                                <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                                                    <DocumentViewer
                                                        doc={d.clone()}
                                                        on_close=move || set_viewing.set(false)
                                                        can_edit={can_edit_documents_here}
                                                    />
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else { ().into_any() }}
                                    // Document context menu
                                    {move || if show_doc_ctx_menu.get() {
                                        let dx = doc_ctx_menu_x.get();
                                        let dy = doc_ctx_menu_y.get();
                                        view! {
                                            <div class="context-menu-overlay" on:click=move |_| set_show_doc_ctx_menu.set(false)>
                                                <div class="context-menu" style={format!("left: {}px; top: {}px;", dx, dy)}>
                                                    <button class="context-menu-item"
                                                        on:click=move |_| {
                                                            set_show_doc_ctx_menu.set(false);
                                                            set_show_add_role.set(true);
                                                        }
                                                    >"🎭 Add Role"</button>
                                                    <button class="context-menu-item"
                                                        on:click=move |_| {
                                                            set_show_doc_ctx_menu.set(false);
                                                            set_show_add_org.set(true);
                                                        }
                                                    >"🏢 Add Organization"</button>
                                                    <button class="context-menu-item"
                                                        on:click=move |_| {
                                                            set_show_doc_ctx_menu.set(false);
                                                            if let Some(pid) = portfolio_id {
                                                                app_store.update(|s| { s.remove_document_from_asset(pid, asset_id, doc_id); });
                                                            }
                                                        }
                                                    >"🗑 Remove"</button>
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else { ().into_any() }}
                                }
                            }).collect::<Vec<_>>().into_any()
                        }}
                    </div>
                </div>
            </div>

            {move || if app_store.get().is_doc_modal_open(asset_id) {
                let mt = asset_name_for_modal.clone();
                let ac = add_cb.clone();
                view! {
                    <DocModal
                        entity_id={asset_id}
                        title={mt}
                        on_close=move || app_store.update(|s| s.close_doc_modal(asset_id))
                        can_edit={can_edit_documents_here}
                        on_add={ac}
                        portfolio_id={portfolio_id}
                        group_id={group_id}
                        asset_id={Some(asset_id)}
                    />
                }.into_any()
            } else { ().into_any() }}

            // Asset context menu (right-click / tap-and-hold)
            {move || asset_context_menu.get().map(|(x, y)| {
                view! {
                    <div class="context-menu-overlay" on:click=move |_| set_asset_context_menu.set(None)>
                        <div class="context-menu" style={format!("left: {}px; top: {}px;", x, y)}>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_asset_context_menu.set(None);
                                    app_store.update(|s| s.toggle_doc_modal(asset_id));
                                }
                            >"📄 Add Document"</button>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_asset_context_menu.set(None);
                                    set_show_add_role.set(true);
                                }
                            >"🎭 Add Role"</button>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_asset_context_menu.set(None);
                                    set_show_add_org.set(true);
                                }
                            >"🏢 Add Organization"</button>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_asset_context_menu.set(None);
                                    set_confirm_asset_remove.set(true);
                                }
                            >"🗑 Remove"</button>
                        </div>
                    </div>
                }.into_any()
            })}

            // Add User modal
            {move || if show_add_user.get() {
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_add_user.set(false)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add User to Asset"</span>
                                <button class="doc-modal-close" on:click=move |_| set_show_add_user.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="User name"
                                    prop:value={move || new_user_name.get()}
                                    on:input=move |ev| set_new_user_name.set(event_target_value(&ev)) />
                                <input class="login-input" type="email" placeholder="Email"
                                    prop:value={move || new_user_email.get()}
                                    on:input=move |ev| set_new_user_email.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| {
                                    let name = new_user_name.get();
                                    let email = new_user_email.get();
                                    if !name.trim().is_empty() {
                                        let now = chrono::Utc::now();
                                        let user = crate::models::User {
                                            id: Uuid::new_v4(),
                                            name: name.clone(),
                                            username: None,
                                            email,
                                            role: crate::types::UserRole::Worker,
                                            organization_id: None,
                                            department: None,
                                            phone: None,
                                            address: None,
                                            hire_date: None,
                                            base_salary: None,
                                            avatar_url: None,
                                            payment_settings: Default::default(),
                                            notification_preferences: vec![],
                                            permissions: vec![],
                                            assignments: vec![],
                                            activity_log: vec![],
                                            documents: vec![],
                                            created_at: now,
                                            updated_at: now,
                                            last_login: None,
                                            is_active: true,
                                        };
                                        let uid = user.id;
                                        app_store.update(|s| {
                                            s.add_organization_user(user);
                                        });
                                        app_store.update(|s| {
                                            for p in s.portfolios.iter_mut() {
                                                let all: Vec<_> = p.assets.iter_mut()
                                                    .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                                                    .collect();
                                                for a in all {
                                                    if a.id == asset_id {
                                                        if !a.assigned_workers.contains(&uid) {
                                                            a.assigned_workers.push(uid);
                                                        }
                                                        return;
                                                    }
                                                }
                                            }
                                        });
                                    }
                                    set_new_user_name.set(String::new());
                                    set_new_user_email.set(String::new());
                                    set_show_add_user.set(false);
                                }>"Add User"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}

            // Add Role modal
            {move || if show_add_role.get() {
                let org_id = app_store.get().portfolios.iter()
                    .find(|p| p.id == portfolio_id.unwrap_or_default())
                    .and_then(|p| p.organization_id);
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_add_role.set(false)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add Role"</span>
                                <button class="doc-modal-close" on:click=move |_| set_show_add_role.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="Role name"
                                    prop:value={move || new_role_name.get()}
                                    on:input=move |ev| set_new_role_name.set(event_target_value(&ev)) />
                                <input class="login-input" type="text" placeholder="Description"
                                    prop:value={move || new_role_desc.get()}
                                    on:input=move |ev| set_new_role_desc.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| {
                                    let name = new_role_name.get();
                                    let desc = new_role_desc.get();
                                    if !name.trim().is_empty() {
                                        let role = crate::models::OrgRole::new(
                                            name.clone(),
                                            0,
                                            desc,
                                            vec![],
                                        );
                                        if let Some(oid) = org_id {
                                            app_store.update(|s| s.add_role_to_org(oid, role));
                                        }
                                    }
                                    set_new_role_name.set(String::new());
                                    set_new_role_desc.set(String::new());
                                    set_show_add_role.set(false);
                                }>"Add Role"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}

            // Add Organization modal
            {move || if show_add_org.get() {
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_add_org.set(false)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add Organization"</span>
                                <button class="doc-modal-close" on:click=move |_| set_show_add_org.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="Organization name"
                                    prop:value={move || new_org_name.get()}
                                    on:input=move |ev| set_new_org_name.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| {
                                    let name = new_org_name.get();
                                    if !name.trim().is_empty() {
                                        let owner_id = app_store.get().current_user.id;
                                        let org = crate::models::Organization::new(name, owner_id);
                                        let oid = org.id;
                                        app_store.update(|s| s.add_organization(org));
                                        // Link asset to the new organization
                                        app_store.update(|s| {
                                            for p in s.portfolios.iter_mut() {
                                                let all: Vec<_> = p.assets.iter_mut()
                                                    .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                                                    .collect();
                                                for a in all {
                                                    if a.id == asset_id {
                                                        a.organization_id = Some(oid);
                                                        return;
                                                    }
                                                }
                                            }
                                        });
                                    }
                                    set_new_org_name.set(String::new());
                                    set_show_add_org.set(false);
                                }>"Add Organization"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}

            // Add Transaction modal
            {move || if show_add_transaction.get() {
                let asset_name = a_name_tx.clone();
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_add_transaction.set(false)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add Transaction"</span>
                                <button class="doc-modal-close" on:click=move |_| set_show_add_transaction.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <select class="login-input"
                                    prop:value={move || format!("{:?}", new_tx_type.get())}
                                    on:change=move |ev| {
                                        let v = event_target_value(&ev);
                                        let t = match v.as_str() {
                                            "Sale" => crate::types::TransactionType::Sale,
                                            "Rent" => crate::types::TransactionType::Rent,
                                            "Lease" => crate::types::TransactionType::Lease,
                                            "Payout" => crate::types::TransactionType::Payout,
                                            "Dividend" => crate::types::TransactionType::Dividend,
                                            "Fee" => crate::types::TransactionType::Fee,
                                            "Tax" => crate::types::TransactionType::Tax,
                                            "Transfer" => crate::types::TransactionType::Transfer,
                                            "Adjustment" => crate::types::TransactionType::Adjustment,
                                            _ => crate::types::TransactionType::Purchase,
                                        };
                                        set_new_tx_type.set(t);
                                    }
                                >
                                    <option value="Purchase">"Purchase"</option>
                                    <option value="Sale">"Sale"</option>
                                    <option value="Rent">"Rent"</option>
                                    <option value="Lease">"Lease"</option>
                                    <option value="Payout">"Payout"</option>
                                    <option value="Dividend">"Dividend"</option>
                                    <option value="Fee">"Fee"</option>
                                    <option value="Tax">"Tax"</option>
                                    <option value="Transfer">"Transfer"</option>
                                    <option value="Adjustment">"Adjustment"</option>
                                </select>
                                <input class="login-input" type="number" placeholder="Amount ($)"
                                    prop:value={move || new_tx_amount.get()}
                                    on:input=move |ev| set_new_tx_amount.set(event_target_value(&ev)) />
                                <input class="login-input" type="text" placeholder="Description"
                                    prop:value={move || new_tx_desc.get()}
                                    on:input=move |ev| set_new_tx_desc.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| {
                                    let amount = new_tx_amount.get().parse::<f64>().unwrap_or(0.0);
                                    let desc = new_tx_desc.get();
                                    let tx_type = new_tx_type.get();
                                    let user_id = app_store.get().current_user.id;
                                    let user_name = app_store.get().current_user.name.clone();
                                    let mut tx = crate::models::Transaction::new(
                                        tx_type,
                                        amount,
                                        crate::types::Currency::USD,
                                        crate::models::EntityReference {
                                            entity_type: crate::models::EntityType::External,
                                            entity_id: Uuid::new_v4(),
                                            name: asset_name.clone(),
                                        },
                                        crate::models::EntityReference {
                                            entity_type: crate::models::EntityType::User,
                                            entity_id: user_id,
                                            name: user_name,
                                        },
                                        user_id,
                                    );
                                    tx.related_asset_id = Some(asset_id);
                                    tx.related_portfolio_id = portfolio_id;
                                    tx.description = if desc.trim().is_empty() { None } else { Some(desc) };
                                    app_store.update(|s| s.transactions.push(tx));
                                    set_new_tx_amount.set(String::new());
                                    set_new_tx_desc.set(String::new());
                                    set_show_add_transaction.set(false);
                                }>"Add Transaction"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}

            // Confirm asset removal
            {move || if confirm_asset_remove.get() {
                let aname = asset_name_for_confirm.clone();
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_confirm_asset_remove.set(false)>
                        <div class="doc-modal confirm-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"🗑 Confirm Removal"</span>
                                <button class="doc-modal-close" on:click=move |_| set_confirm_asset_remove.set(false)>"✕"</button>
                            </div>
                            <div class="confirm-modal-body">
                                <p class="confirm-modal-msg">
                                    "Are you sure you want to remove "
                                    <strong>{aname.clone()}</strong>
                                    "? This action cannot be undone."
                                </p>
                                <div class="confirm-modal-actions">
                                    <button class="login-btn confirm-no"
                                        on:click=move |_| set_confirm_asset_remove.set(false)>
                                        "✕ No, Cancel"
                                    </button>
                                    <button class="login-btn sell confirm-yes"
                                        on:click=move |_| {
                                            set_confirm_asset_remove.set(false);
                                            if let Some(pid) = portfolio_id {
                                                app_store.update(|s| { s.remove_asset(pid, asset_id); });
                                            }
                                        }>
                                        "✔ Yes, Remove"
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}

            {move || {
                if expanded_detail.get() && can_edit_here {
                    view! {
                        <div class="ai-detail-panel" on:click=|ev| ev.stop_propagation()>
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
                                        <button class="pf-edit-cancel" on:click=move |_| { set_expanded_detail.set(false); }>"✕ Cancel"</button>
                                    </div>
                                    <UserAssignmentPanel assigned={get_asset_assigned_users()} users={get_org_users()} on_toggle={toggle_asset_assignment} />
                                </div>
                            </div>
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
    asset_id: Uuid,
    a_purchase_val: f64,
    a_current_val: f64,
    a_pl: f64,
    a_pl_pct: f64,
    a_revenue: f64,
    a_pl_cls: &'static str,
    doc_sort: ReadSignal<u8>,
    set_doc_sort: WriteSignal<u8>,
    can_edit: bool,
    can_edit_documents: bool,
    add_cb: Option<Callback<String>>,
    #[prop(default = None)] portfolio_id: Option<Uuid>,
    #[prop(default = None)] group_id: Option<Uuid>,
) -> impl IntoView {
    let app_store = use_app_store();
    let _ = (a_purchase_val, a_pl, a_pl_pct, a_revenue, a_pl_cls);
    let modal_title = detail.get_value().asset_name_for_modal.clone();

    // Reactive document list for this asset read directly from the store.
    let asset_docs = Memo::new(move |_| {
        app_store.get().portfolios.iter()
            .flat_map(|p| p.assets.iter().chain(p.asset_groups.iter().flat_map(|g| g.assets.iter())))
            .find(|a| a.id == asset_id)
            .map(|a| a.documents.clone())
            .unwrap_or_default()
    });

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
                        {if can_edit_documents {
                            view! {
                                <button class="ai-docs-sort-btn ai-docs-add-btn"
                                    on:click=move |_| app_store.update(|s| s.toggle_doc_modal(asset_id))>
                                    "+ Add"
                                </button>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                </div>
                {move || {
                    let mut sorted_docs = asset_docs.get();
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

        {move || if app_store.get().is_doc_modal_open(asset_id) {
            let mt = modal_title.clone();
            let ac = add_cb.clone();
            view! {
                <DocModal
                    entity_id={asset_id}
                    title={mt}
                    on_close=move || app_store.update(|s| s.close_doc_modal(asset_id))
                    can_edit={can_edit_documents}
                    on_add={ac}
                    portfolio_id={portfolio_id}
                    group_id={group_id}
                    asset_id={Some(asset_id)}
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
        AssetType::Channel => "📡",
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
fn create_mock_asset(name: &str, asset_type: AssetType, purchase: f64, current: f64, uploaded_by: Uuid) -> Asset {
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
        uploaded_by,
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
        last_accessed_at: chrono::Utc::now(),
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
        "odt" => "📘",
        "xls" | "xlsx" => "📗",
        "ppt" | "pptx" => "📙",
        "txt" | "md" | "rs" | "js" | "ts" | "html" | "css" => "📄",
        "zip" | "rar" | "7z" | "tar" => "🗜️",
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" => "🖼️",
        "mp4" | "mov" | "avi" | "mkv" | "webm" => "🎬",
        "mp3" | "wav" | "flac" => "🎵",
        _ => "📎",
    }
}

fn detect_file_type(name: &str) -> String {
    if let Some(idx) = name.rfind('.') {
        let ext = &name[idx + 1..];
        let ext_lower = ext.to_lowercase();
        match ext_lower.as_str() {
            "pdf" | "docx" | "doc" | "txt" | "odt" | "rtf" |
            "xlsx" | "xls" | "csv" |
            "pptx" | "ppt" |
            "md" | "json" | "xml" | "html" | "css" | "js" | "ts" | "rs" | "py" | "go" |
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "tiff" | "ico" |
            "mp4" | "mov" | "avi" | "mkv" | "webm" | "flv" |
            "mp3" | "wav" | "flac" | "aac" | "ogg" |
            "zip" | "rar" | "7z" | "tar" | "gz"
            => ext_lower,
            _ => "txt".to_string(),
        }
    } else {
        "txt".to_string()
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
        notification_settings: vec![],
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
    entity_id: Uuid,
    title: String,
    on_close: impl Fn() + 'static,
    can_edit: bool,
    on_add: Option<Callback<String>>,
    #[prop(default = None)] portfolio_id: Option<Uuid>,
    #[prop(default = None)] group_id: Option<Uuid>,
    #[prop(default = None)] asset_id: Option<Uuid>,
) -> impl IntoView {
    let app_store = use_app_store();
    // open_tabs: vec of (tab_id, Document); tab_id=0 is reserved for the list tab
    let (open_tabs, set_open_tabs) = signal::<Vec<(u32, Document)>>(vec![]);
    let (active_tab, set_active_tab) = signal::<u32>(0); // 0 = list view
    let (next_id, set_next_id) = signal(1u32);
    let (new_doc_name, set_new_doc_name) = signal(String::new());
    let title_stored = StoredValue::new(title);

    // Reactive document list read directly from the store so additions show immediately.
    let docs = Memo::new(move |_| {
        let store = app_store.get();
        let mut docs = Vec::new();
        for p in &store.portfolios {
            if p.id == entity_id {
                docs.extend(p.documents.clone());
            }
            for g in &p.asset_groups {
                if g.id == entity_id {
                    docs.extend(g.documents.clone());
                }
                for a in &g.assets {
                    if a.id == entity_id {
                        docs.extend(a.documents.clone());
                    }
                }
            }
            for a in &p.assets {
                if a.id == entity_id {
                    docs.extend(a.documents.clone());
                }
            }
        }
        docs
    });
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
                                {docs.get().into_iter().map(|doc| {
                                    let icon = document_icon(&doc.file_type);
                                    let ft   = doc.file_type.to_uppercase();
                                    let current_user = app_store.get().current_user.clone();
                                    let can_edit_doc = can_edit && current_user.can_edit_document(&doc);
                                    let doc_for_open = doc.clone();
                                    let doc_for_tap = doc.clone();
                                    let doc_id = doc.id;
                                    let (editing_name, set_editing_name) = signal(false);
                                    let (edit_name, set_edit_name) = signal(doc.name.clone());
                                    view! {
                                        <div class="doc-modal-row">
                                            <div class="doc-modal-icon-wrap">
                                                <span class="doc-modal-icon">{icon}</span>
                                            </div>
                                            <div class="doc-modal-info"
                                                class:doc-modal-info-tap=can_edit_doc
                                                on:click=move |ev: leptos::ev::MouseEvent| {
                                                    if can_edit_doc && !editing_name.get() {
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
                                            {move || if can_edit_doc && !editing_name.get() {
                                                view! {
                                                    <button class="doc-modal-edit-btn"
                                                        on:click=move |_| set_editing_name.set(true)>
                                                        "✎"
                                                    </button>
                                                }.into_any()
                                            } else { ().into_any() }}
                                            {move || {
                                                let notifs = app_store.get().notifications_list_for_doc(doc_id);
                                                if notifs.is_empty() {
                                                    ().into_any()
                                                } else {
                                                    let n = notifs[0].clone();
                                                    let nid = n.id;
                                                    let from_user = n.from_user.clone().unwrap_or_else(|| "System".to_string());
                                                    let preview = n.content_preview.clone();
                                                    // Truncate note to less than a sentence (~60 chars)
                                                    let short_note = preview.as_ref().map(|p| {
                                                        let truncated = if p.len() > 60 {
                                                            // Find a good break point
                                                            let slice = &p[..60];
                                                            if let Some(idx) = slice.rfind(|c: char| c == ' ' || c == ',' || c == '.') {
                                                                &p[..idx]
                                                            } else {
                                                                slice
                                                            }
                                                        } else {
                                                            p.as_str()
                                                        };
                                                        format!("— {}", truncated)
                                                    }).unwrap_or_default();
                                                    view! {
                                                        <span class="doc-notif-label">
                                                            "Linked (Document) by " <strong>{from_user}</strong> " " {short_note}
                                                        </span>
                                                        <button class="doc-notif-view-btn"
                                                            on:click=move |ev: leptos::ev::MouseEvent| {
                                                                ev.stop_propagation();
                                                                app_store.update(|s| s.navigate_to_notification(nid));
                                                            }>
                                                            "View Content"
                                                        </button>
                                                    }.into_any()
                                                }
                                            }}
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
                                portfolio_id={portfolio_id}
                                group_id={group_id}
                                asset_id={asset_id}
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
    #[prop(default = None)] portfolio_id: Option<Uuid>,
    #[prop(default = None)] group_id: Option<Uuid>,
    #[prop(default = None)] asset_id: Option<Uuid>,
) -> impl IntoView {
    let app_store = use_app_store();
    let undo_store = use_undo_redo_store();
    let initial_content = doc.content.clone().unwrap_or_else(|| mock_doc_content(&doc.name, &doc.file_type));
    let doc_name  = StoredValue::new(doc.name.clone());
    let doc_id = doc.id;
    let doc_url = StoredValue::new(doc.url.clone());

    let current_user = app_store.get().current_user.clone();
    let effective_can_edit = can_edit && current_user.can_edit_document(&doc);

    // viewer state
    let (zoom, set_zoom)         = signal(100u32);       // percent
    let (edit_mode, set_edit_mode) = signal(effective_can_edit);
    let (content, set_content)   = signal(initial_content);
    let (why, set_why)           = signal(String::new());
    let (notes, set_notes)       = signal(String::new());
    // image popup: Some((x_px, y_px))
    let (img_popup, set_img_popup) = signal::<Option<(i32, i32)>>(None);
    let (link_val, set_link_val) = signal(doc.url.clone());
    let (file_type, set_file_type) = signal(doc.file_type.clone());
    let (show_type_dropdown, set_show_type_dropdown) = signal(false);

    let is_image = move || {
        matches!(file_type.get().to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "tiff" | "ico"
        )
    };
    let is_video = move || {
        matches!(file_type.get().to_lowercase().as_str(),
            "mp4" | "mov" | "avi" | "mkv" | "webm" | "flv"
        )
    };
    let is_sheet = move || file_type.get() == "xlsx" || file_type.get() == "csv";

    let apply_image_url = move || {
        let url = link_val.get().trim().to_string();
        if !url.is_empty() {
            app_store.update(|s| {
                for p in s.portfolios.iter_mut() {
                    for d in &mut p.documents {
                        if d.id == doc_id { d.url = url.clone(); }
                    }
                    for g in &mut p.asset_groups {
                        for d in &mut g.documents {
                            if d.id == doc_id { d.url = url.clone(); }
                        }
                        for a in &mut g.assets {
                            for d in &mut a.documents {
                                if d.id == doc_id { d.url = url.clone(); }
                            }
                        }
                    }
                    for a in &mut p.assets {
                        for d in &mut a.documents {
                            if d.id == doc_id { d.url = url.clone(); }
                        }
                    }
                }
            });
        }
        set_img_popup.set(None);
    };

    let on_close = std::rc::Rc::new(on_close);
    let on_close_toolbar = on_close.clone();

    let save_doc = move || {
        let new_content = content.get();
        let reason = why.get();
        let notes_text = notes.get();
        let reason_for_action = if reason.trim().is_empty() { None } else { Some(reason.clone()) };
        let doc_name_val = doc_name.get_value();
        let pid = portfolio_id;
        let gid = group_id;
        let aid = asset_id;
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
            // Send notification with notes and @username parsing
            let updater_name = s.current_user.name.clone();
            s.add_document_update_with_notes(
                doc_id,
                &doc_name_val,
                &notes_text,
                &updater_name,
                pid,
                gid,
                aid,
            );
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
        set_notes.set(String::new());
    };

    view! {
        <div class="docviewer">
            // ── Sticky toolbar ────────────────────────────────────────
            <div class="docviewer-toolbar">
                <span class="docviewer-icon">{move || document_icon(&file_type.get())}</span>
                <span class="docviewer-name">{doc_name.get_value()}</span>
                // Document type selector
                <div class="dv-type-selector">
                    <button class="docviewer-ft dv-type-btn"
                        on:click=move |_| set_show_type_dropdown.update(|v| *v = !*v)>
                        {move || file_type.get().to_uppercase()}
                        <span class="dv-type-arrow">{move || if show_type_dropdown.get() { "▲" } else { "▼" }}</span>
                    </button>
                    {move || if show_type_dropdown.get() {
                        let type_options = ["pdf", "docx", "txt", "odt", "rtf", "xlsx", "csv", "pptx", "md",
                            "jpg", "jpeg", "png", "gif", "webp", "svg",
                            "mp4", "mov", "avi", "webm",
                            "mp3", "wav", "zip"];
                        let current_ft = file_type.get();
                        view! {
                            <div class="dv-type-dropdown-overlay" on:click=move |_| set_show_type_dropdown.set(false)>
                                <div class="dv-type-dropdown" on:click=|ev| ev.stop_propagation()>
                                    {type_options.iter().map(|opt| {
                                        let opt_str = opt.to_string();
                                        let is_active = current_ft == opt_str;
                                        let opt_for_click = opt_str.clone();
                                        view! {
                                            <button class="dv-type-option" class:dv-type-option-active={is_active}
                                                on:click=move |_| {
                                                    set_file_type.set(opt_for_click.clone());
                                                    set_show_type_dropdown.set(false);
                                                    app_store.update(|s| s.update_document_file_type(doc_id, opt_for_click.clone()));
                                                }>
                                                <span>{document_icon(opt)}</span>
                                                <span>{opt.to_uppercase()}</span>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                </div>

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

                // Edit toggle (only when effective_can_edit)
                {if effective_can_edit {
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
                class={move || if is_sheet() { "docviewer-body docviewer-sheet".to_string() } else { "docviewer-body".to_string() }}
                style=move || format!("font-size: {}%;", zoom.get())
                on:click=move |_| { if img_popup.get().is_some() { apply_image_url(); } }
            >
                // Media rendering area (images and videos)
                {move || {
                    let url = link_val.get();
                    if is_image() {
                        if url != "#" && !url.is_empty() {
                            view! {
                                <div class="dv-media-area">
                                    <img class="dv-media-img" src={url.clone()} alt={doc_name.get_value()} />
                                </div>
                            }.into_any()
                        } else if effective_can_edit {
                            view! {
                                <div class="dv-media-placeholder dv-media-img-placeholder"
                                    class:dv-editable=move || edit_mode.get()
                                    on:click=move |ev: leptos::ev::MouseEvent| {
                                        if edit_mode.get() {
                                            ev.stop_propagation();
                                            set_link_val.set(doc_url.get_value());
                                            set_img_popup.set(Some((ev.client_x(), ev.client_y())));
                                        }
                                    }
                                >
                                    {move || if edit_mode.get() {
                                        view! { <span class="dv-img-hint">"🖼 Click to set image URL"</span> }.into_any()
                                    } else { view! { <span class="dv-img-hint dv-img-muted">"🖼 No image set"</span> }.into_any() }}
                                </div>
                            }.into_any()
                        } else { ().into_any() }
                    } else if is_video() {
                        if url != "#" && !url.is_empty() {
                            view! {
                                <div class="dv-media-area">
                                    <video class="dv-media-video" src={url.clone()} controls=true>
                                        "Your browser does not support video playback."
                                    </video>
                                </div>
                            }.into_any()
                        } else if effective_can_edit {
                            view! {
                                <div class="dv-media-placeholder dv-media-video-placeholder"
                                    class:dv-editable=move || edit_mode.get()
                                    on:click=move |ev: leptos::ev::MouseEvent| {
                                        if edit_mode.get() {
                                            ev.stop_propagation();
                                            set_link_val.set(doc_url.get_value());
                                            set_img_popup.set(Some((ev.client_x(), ev.client_y())));
                                        }
                                    }
                                >
                                    {move || if edit_mode.get() {
                                        view! { <span class="dv-img-hint">"🎬 Click to set video URL"</span> }.into_any()
                                    } else { view! { <span class="dv-img-hint dv-img-muted">"🎬 No video set"</span> }.into_any() }}
                                </div>
                            }.into_any()
                        } else { ().into_any() }
                    } else {
                        ().into_any()
                    }
                }}

                // Image/media URL popup (appears at cursor position)
                {move || if let Some((cx, cy)) = img_popup.get() {
                    view! {
                        <div class="dv-img-popup"
                            style=move || format!("left:{}px;top:{}px;", cx, cy)
                            on:click=|ev| ev.stop_propagation()>
                            <div class="dv-img-popup-opt"
                                on:click=move |_| {
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
                                    placeholder="Paste media URL…"
                                    prop:value=move || link_val.get()
                                    on:input=move |ev| set_link_val.set(event_target_value(&ev))
                                    on:click=|ev| ev.stop_propagation()
                                    on:keydown=move |ev| { if ev.key() == "Enter" { apply_image_url(); } }
                                />
                            </div>
                            <div class="dv-img-popup-opt"
                                on:click=move |_| {
                                    apply_image_url();
                                }
                            >
                                <span class="dv-img-opt-icon">"✔"</span>
                                <span>"Apply URL"</span>
                            </div>
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                // Text content — editable textarea in edit mode, pre otherwise
                // (hidden for pure media types in read mode when URL is set)
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
                        <div class="dv-why-row">
                            <label class="dv-why-label">"Notes — tag people with @username to notify them"</label>
                            <textarea
                                class="dv-why-input dv-notes-input"
                                placeholder="Add notes for reviewers. Use @username to tag people (e.g. @red please review section 3)…"
                                prop:value=move || notes.get()
                                on:input=move |ev| set_notes.set(event_target_value(&ev))
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

fn trigger_label(t: &NotificationTrigger) -> String {
    match t {
        NotificationTrigger::PriceChange { percentage } => format!("Price change > {}%", percentage),
        NotificationTrigger::Sale => "Sale".to_string(),
        NotificationTrigger::Auction => "Auction".to_string(),
        NotificationTrigger::Rent => "Rent".to_string(),
        NotificationTrigger::Unrent => "Unrent".to_string(),
        NotificationTrigger::NoSales { days } => format!("No sales for {} days", days),
        NotificationTrigger::Custom(s) => s.clone(),
    }
}

fn trigger_short(t: &NotificationTrigger) -> &'static str {
    match t {
        NotificationTrigger::PriceChange { .. } => "Price Change",
        NotificationTrigger::Sale => "Sale",
        NotificationTrigger::Auction => "Auction",
        NotificationTrigger::Rent => "Rent",
        NotificationTrigger::Unrent => "Unrent",
        NotificationTrigger::NoSales { .. } => "No Sales",
        NotificationTrigger::Custom(_) => "Custom",
    }
}

fn notif_type_label(t: &NotificationType) -> &'static str {
    match t {
        NotificationType::Push => "Push",
        NotificationType::Email => "Email",
        NotificationType::Sms => "SMS",
        NotificationType::InApp => "In-App",
    }
}

fn role_label(r: &UserRole) -> &'static str {
    match r {
        UserRole::Owner => "Owner",
        UserRole::Director => "Director",
        UserRole::SeniorManager => "Senior Manager",
        UserRole::Manager => "Manager",
        UserRole::Worker => "Worker",
        UserRole::DocumentWorker => "Document Worker",
        UserRole::Contractor => "Contractor",
        UserRole::Guest => "Guest",
    }
}

/// Quick notification settings popover for a portfolio or group.
/// Opens when the 🔔 badge is clicked.
#[component]
fn NotificationQuickSettings(
    target: NotifTarget,
    entity_name: String,
    on_close: impl Fn() + 'static,
) -> impl IntoView {
    let app_store = use_app_store();
    let on_close = std::rc::Rc::new(on_close);
    let on_close2 = on_close.clone();

    let can_manage_recipients = app_store.get().can_manage_notification_recipients();
    let org_users = app_store.get().organization_users.clone();

    let target_for_settings = target.clone();
    let settings = Memo::new(move |_| {
        match &target_for_settings {
            NotifTarget::Portfolio(pid) => app_store.get().portfolio_notification_settings(*pid),
            NotifTarget::Group(pid, gid) => app_store.get().group_notification_settings(*pid, *gid),
        }
    });

    let (selected_trigger, set_selected_trigger) = signal(NotificationTrigger::Sale);
    let (new_condition, set_new_condition) = signal(String::new());

    let target_for_add = target.clone();
    let add_setting = move |_| {
        let mut setting = EntityNotificationSetting::new(selected_trigger.get());
        if !new_condition.get().trim().is_empty() {
            setting.condition = Some(new_condition.get().trim().to_string());
        }
        match &target_for_add {
            NotifTarget::Portfolio(pid) => {
                app_store.update(|s| s.add_portfolio_notification_setting(*pid, setting));
            }
            NotifTarget::Group(pid, gid) => {
                app_store.update(|s| s.add_group_notification_setting(*pid, *gid, setting));
            }
        }
        set_new_condition.set(String::new());
    };

    let target_for_recipients_section = target.clone();
    let org_users_for_recipients = org_users.clone();

    let all_triggers = vec![
        NotificationTrigger::Sale,
        NotificationTrigger::Auction,
        NotificationTrigger::Rent,
        NotificationTrigger::Unrent,
        NotificationTrigger::PriceChange { percentage: 10.0 },
        NotificationTrigger::NoSales { days: 30 },
        NotificationTrigger::Custom("Document Added".to_string()),
        NotificationTrigger::Custom("Document Updated".to_string()),
    ];

    let all_notif_types = vec![
        NotificationType::InApp,
        NotificationType::Push,
        NotificationType::Email,
        NotificationType::Sms,
    ];

    let all_roles = vec![
        UserRole::Owner,
        UserRole::Director,
        UserRole::SeniorManager,
        UserRole::Manager,
        UserRole::Worker,
        UserRole::DocumentWorker,
    ];

    view! {
        <div class="notif-qs-overlay" on:click=move |_| on_close2()>
            <div class="notif-qs-popover" on:click=|ev| ev.stop_propagation()>
                <div class="notif-qs-header">
                    <span class="notif-qs-title">"🔔 Notification Settings"</span>
                    <span class="notif-qs-entity">{entity_name.clone()}</span>
                    <button class="notif-qs-close" on:click=move |_| on_close()>"✕"</button>
                </div>

                // Existing settings
                <div class="notif-qs-section">
                    <div class="notif-qs-section-label">"Current Rules"</div>
                    {move || {
                        let items = settings.get();
                        if items.is_empty() {
                            view! {
                                <div class="notif-qs-empty">"No notification rules yet. Add one below."</div>
                            }.into_any()
                        } else {
                            items.into_iter().map(|s| {
                                let sid = s.id;
                                let sid_toggle = sid;
                                let sid_remove = sid;
                                let s_enabled = s.enabled;
                                let s_label = trigger_label(&s.trigger);
                                let s_types = s.notification_types.clone();
                                let s_recipients = s.recipients.clone();
                                let s_roles = s.recipient_roles.clone();
                                let s_condition = s.condition.clone();
                                let target_toggle = target.clone();
                                let target_remove = target.clone();

                                let all_nt = all_notif_types.clone();
                                let target_for_nt = target.clone();
                                let sid_for_nt = sid;

                                view! {
                                    <div class="notif-qs-rule" class:disabled={!s_enabled}>
                                        <div class="notif-qs-rule-top">
                                            <label class="notif-qs-toggle">
                                                <input type="checkbox" checked=s_enabled
                                                    on:change=move |_| {
                                                        match &target_toggle {
                                                            NotifTarget::Portfolio(pid) => app_store.update(|s| s.toggle_portfolio_notification_setting(*pid, sid_toggle)),
                                                            NotifTarget::Group(pid, gid) => app_store.update(|s| s.toggle_group_notification_setting(*pid, *gid, sid_toggle)),
                                                        }
                                                    } />
                                                <span class="notif-qs-rule-name">{s_label}</span>
                                            </label>
                                            <button class="notif-qs-rule-remove"
                                                on:click=move |_| {
                                                    match &target_remove {
                                                        NotifTarget::Portfolio(pid) => app_store.update(|s| s.remove_portfolio_notification_setting(*pid, sid_remove)),
                                                        NotifTarget::Group(pid, gid) => app_store.update(|s| s.remove_group_notification_setting(*pid, *gid, sid_remove)),
                                                    }
                                                }>"🗑"</button>
                                        </div>
                                        // Notification type badges
                                        <div class="notif-qs-rule-types">
                                            {all_nt.iter().map(|nt| {
                                                let nt_label = notif_type_label(nt);
                                                let is_on = s_types.contains(nt);
                                                let target_nt = target_for_nt.clone();
                                                let sid_nt = sid_for_nt;
                                                let nt_clone = nt.clone();
                                                view! {
                                                    <button class="notif-qs-type-chip"
                                                        class:active=is_on
                                                        on:click=move |_| {
                                                            match &target_nt {
                                                                NotifTarget::Portfolio(pid) => app_store.update(|s| {
                                                                    if let Some(p) = s.get_portfolio_mut(*pid) {
                                                                        if let Some(st) = p.notification_settings.iter_mut().find(|st| st.id == sid_nt) {
                                                                            if st.notification_types.contains(&nt_clone) {
                                                                                st.notification_types.retain(|t| t != &nt_clone);
                                                                            } else {
                                                                                st.notification_types.push(nt_clone.clone());
                                                                            }
                                                                        }
                                                                    }
                                                                }),
                                                                NotifTarget::Group(pid, gid) => app_store.update(|s| {
                                                                    if let Some(p) = s.get_portfolio_mut(*pid) {
                                                                        if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == *gid) {
                                                                            if let Some(st) = g.notification_settings.iter_mut().find(|st| st.id == sid_nt) {
                                                                                if st.notification_types.contains(&nt_clone) {
                                                                                    st.notification_types.retain(|t| t != &nt_clone);
                                                                                } else {
                                                                                    st.notification_types.push(nt_clone.clone());
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }),
                                                            }
                                                        }>
                                                        {nt_label}
                                                    </button>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                        // Recipients (if can manage)
                                        {if can_manage_recipients {
                                            let recipient_names: Vec<String> = s_recipients.iter().filter_map(|uid| {
                                                org_users.iter().find(|u| u.id == *uid).map(|u| u.name.clone())
                                            }).collect();
                                            let role_names: Vec<&'static str> = s_roles.iter().map(role_label).collect();
                                            let info = if recipient_names.is_empty() && role_names.is_empty() {
                                                "Just me".to_string()
                                            } else {
                                                let mut parts = Vec::new();
                                                if !recipient_names.is_empty() {
                                                    parts.push(format!("Users: {}", recipient_names.join(", ")));
                                                }
                                                if !role_names.is_empty() {
                                                    parts.push(format!("Roles: {}", role_names.join(", ")));
                                                }
                                                parts.join(" · ")
                                            };
                                            view! {
                                                <div class="notif-qs-rule-recipients">{info}</div>
                                            }.into_any()
                                        } else { ().into_any() }}
                                        // Condition
                                        {s_condition.map(|c| view! {
                                            <div class="notif-qs-rule-condition">"Condition: " {c}</div>
                                        })}
                                    </div>
                                }
                            }).collect::<Vec<_>>().into_any()
                        }
                    }}
                </div>

                // Add new rule
                <div class="notif-qs-section">
                    <div class="notif-qs-section-label">"Add Notification Rule"</div>
                    <div class="notif-qs-add-row">
                        <select class="notif-qs-select"
                            on:change=move |ev| {
                                let v = event_target_value(&ev);
                                let t = match v.as_str() {
                                    "Sale" => NotificationTrigger::Sale,
                                    "Auction" => NotificationTrigger::Auction,
                                    "Rent" => NotificationTrigger::Rent,
                                    "Unrent" => NotificationTrigger::Unrent,
                                    "PriceChange" => NotificationTrigger::PriceChange { percentage: 10.0 },
                                    "NoSales" => NotificationTrigger::NoSales { days: 30 },
                                    "DocumentAdded" => NotificationTrigger::Custom("Document Added".to_string()),
                                    "DocumentUpdated" => NotificationTrigger::Custom("Document Updated".to_string()),
                                    _ => NotificationTrigger::Sale,
                                };
                                set_selected_trigger.set(t);
                            }>
                            {all_triggers.iter().map(|t| {
                                let val = match t {
                                    NotificationTrigger::Sale => "Sale",
                                    NotificationTrigger::Auction => "Auction",
                                    NotificationTrigger::Rent => "Rent",
                                    NotificationTrigger::Unrent => "Unrent",
                                    NotificationTrigger::PriceChange { .. } => "PriceChange",
                                    NotificationTrigger::NoSales { .. } => "NoSales",
                                    NotificationTrigger::Custom(s) if s == "Document Added" => "DocumentAdded",
                                    NotificationTrigger::Custom(s) if s == "Document Updated" => "DocumentUpdated",
                                    _ => "Custom",
                                };
                                view! {
                                    <option value={val}>{trigger_short(t)}</option>
                                }
                            }).collect::<Vec<_>>()}
                        </select>
                        <input class="notif-qs-input" type="text" placeholder="Condition (optional, e.g. 'Only PDF docs')"
                            prop:value=move || new_condition.get()
                            on:input=move |ev| set_new_condition.set(event_target_value(&ev)) />
                        <button class="notif-qs-add-btn" on:click=add_setting>"+ Add"</button>
                    </div>
                </div>

                // Recipient configuration (role-gated)
                {can_manage_recipients.then(|| {
                    let target_for_recipients = target_for_recipients_section.clone();
                    let users_for_select = org_users_for_recipients.clone();
                    view! {
                        <div class="notif-qs-section">
                            <div class="notif-qs-section-label">"Recipient Roles (applies to most recent rule)"</div>
                            <div class="notif-qs-roles-row">
                                {all_roles.iter().map(|r| {
                                    let r_label = role_label(r);
                                    let r_clone = r.clone();
                                    let target_r = target_for_recipients.clone();
                                    view! {
                                        <button class="notif-qs-role-chip"
                                            on:click=move |_| {
                                                match &target_r {
                                                    NotifTarget::Portfolio(pid) => app_store.update(|s| {
                                                        if let Some(p) = s.get_portfolio_mut(*pid) {
                                                            if let Some(last) = p.notification_settings.last_mut() {
                                                                if last.recipient_roles.contains(&r_clone) {
                                                                    last.recipient_roles.retain(|r| r != &r_clone);
                                                                } else {
                                                                    last.recipient_roles.push(r_clone.clone());
                                                                }
                                                            }
                                                        }
                                                    }),
                                                    NotifTarget::Group(pid, gid) => app_store.update(|s| {
                                                        if let Some(p) = s.get_portfolio_mut(*pid) {
                                                            if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == *gid) {
                                                                if let Some(last) = g.notification_settings.last_mut() {
                                                                    if last.recipient_roles.contains(&r_clone) {
                                                                        last.recipient_roles.retain(|r| r != &r_clone);
                                                                    } else {
                                                                        last.recipient_roles.push(r_clone.clone());
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }),
                                                }
                                            }>
                                            {r_label}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            <div class="notif-qs-section-label" style="margin-top: 8px;">"Recipient Users (applies to most recent rule)"</div>
                            <div class="notif-qs-users-row">
                                {users_for_select.iter().map(|u| {
                                    let uname = u.name.clone();
                                    let uid = u.id;
                                    let target_u = target_for_recipients.clone();
                                    view! {
                                        <button class="notif-qs-user-chip"
                                            on:click=move |_| {
                                                match &target_u {
                                                    NotifTarget::Portfolio(pid) => app_store.update(|s| {
                                                        if let Some(p) = s.get_portfolio_mut(*pid) {
                                                            if let Some(last) = p.notification_settings.last_mut() {
                                                                if last.recipients.contains(&uid) {
                                                                    last.recipients.retain(|id| id != &uid);
                                                                } else {
                                                                    last.recipients.push(uid);
                                                                }
                                                            }
                                                        }
                                                    }),
                                                    NotifTarget::Group(pid, gid) => app_store.update(|s| {
                                                        if let Some(p) = s.get_portfolio_mut(*pid) {
                                                            if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == *gid) {
                                                                if let Some(last) = g.notification_settings.last_mut() {
                                                                    if last.recipients.contains(&uid) {
                                                                        last.recipients.retain(|id| id != &uid);
                                                                    } else {
                                                                        last.recipients.push(uid);
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }),
                                                }
                                            }>
                                            {uname}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
