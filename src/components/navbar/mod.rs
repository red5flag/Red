use crate::components::messenger::MessageDrawer;
use crate::components::search::SearchFilters;
use crate::stores::{create_action, use_app_store, use_search_store, use_undo_redo_store};
use crate::types::{ActionType, SortMode, TabType, ViewMode};
use leptos::prelude::*;

#[component]
pub fn Navbar() -> impl IntoView {
    let app_store = use_app_store();
    let undo_store = use_undo_redo_store();
    let search_store = use_search_store();

    // Derived signals
    let can_undo = move || undo_store.get().can_undo();
    let can_redo = move || undo_store.get().can_redo();
    let is_search_open = move || app_store.get().is_search_open;
    let profile_name = move || app_store.get().current_user.name.clone();
    let message_count = move || app_store.get().unread_message_count();
    let is_message_drawer_open = move || app_store.get().message_drawer_open;

    // Helper to get user info tuple
    fn user_info(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>) -> (uuid::Uuid, String, String, Option<uuid::Uuid>) {
        let store = app_store.get();
        (store.current_user.id, store.current_user.name.clone(), format!("{:?}", store.current_user.role), store.current_user.organization_id)
    }

    // Handlers
    let on_home = move |_| {
        let (uid, name, role, org) = user_info(&app_store);
        let action = create_action(
            ActionType::Navigate,
            "App",
            "Returned home",
            uid,
            &name,
            &role,
            org,
            None,
        );
        undo_store.update(|u| u.record_action(action));
        app_store.update(|store| {
            store.collapse_all_tabs();
            store.expand_tab(TabType::Overview);
            store.close_search();
        });
    };

    let on_redo = move |_| {
        if let Some(redone) = undo_store.get().redo() {
            let (uid, name, role, org) = user_info(&app_store);
            let action = create_action(
                ActionType::Redo,
                "Action",
                &format!("Redid: {}", redone.description),
                uid,
                &name,
                &role,
                org,
                None,
            );
            undo_store.update(|u| u.record_action(action));
            tracing::info!("Redo: {:?}", redone);
        }
    };

    let on_undo = move |_| {
        if let Some(undone) = undo_store.get().undo() {
            let (uid, name, role, org) = user_info(&app_store);
            let action = create_action(
                ActionType::Undo,
                "Action",
                &format!("Undid: {}", undone.description),
                uid,
                &name,
                &role,
                org,
                None,
            );
            undo_store.update(|u| u.record_action(action));
            tracing::info!("Undo: {:?}", undone);

            if let Some(ref from) = undone.navigated_from {
                tracing::info!("Navigating back to: {}", from);
            }
        }
    };

    let on_search_click = move |_| {
        let (uid, name, role, org) = user_info(&app_store);
        app_store.update(|store| {
            if store.is_search_open {
                store.close_search();
                let action = create_action(
                    ActionType::Search,
                    "Search",
                    "Closed search",
                    uid,
                    &name,
                    &role,
                    org,
                    None,
                );
                undo_store.update(|u| u.record_action(action));
            } else {
                store.open_search();
                let action = create_action(
                    ActionType::Search,
                    "Search",
                    "Opened search",
                    uid,
                    &name,
                    &role,
                    org,
                    None,
                );
                undo_store.update(|u| u.record_action(action));
            }
        });
        search_store.update(|store| {
            store.set_context_tab(
                app_store
                    .get()
                    .active_tabs
                    .first()
                    .cloned()
                    .unwrap_or(TabType::Overview),
            );
        });
    };

    let on_message_click = move |_| {
        app_store.update(|store| store.toggle_message_drawer());
    };

    let on_pen = move |_| {
        app_store.update(|store| {
            if let Some(tab) = store.active_tabs.first().cloned() {
                store.toggle_tab_edit_mode(&tab);
            }
        });
    };

    let on_drawer_toggle = move |_| {
        app_store.update(|store| store.toggle_drawer());
    };

    let drawer_open = move || app_store.get().drawer_open;
    let drawer_icon = move || if drawer_open() { "◀" } else { "▶" };

    // Contextual portfolio controls
    let is_portfolios_tab = move || {
        app_store.get().active_tabs.first().map(|t| *t == TabType::Portfolios).unwrap_or(false)
    };
    let portfolio_view = move || app_store.get().portfolio_view_mode.clone();
    let portfolio_edit_mode = move || {
        app_store.get().active_tabs.first()
            .map(|t| app_store.get().is_tab_edit_mode(t))
            .unwrap_or(false)
    };
    let can_edit_portfolio = move || {
        let role = app_store.get().current_user.role.clone();
        portfolio_edit_mode() && (role == crate::types::UserRole::Owner || role == crate::types::UserRole::Manager)
    };

    let on_list_view = move |_| {
        app_store.update(|s| s.portfolio_view_mode = ViewMode::List);
    };
    let on_grid_view = move |_| {
        app_store.update(|s| s.portfolio_view_mode = ViewMode::Grid);
    };
    let on_toggle_add_portfolio = move |_| {
        app_store.update(|s| s.show_add_portfolio = !s.show_add_portfolio);
        app_store.update(|s| s.show_add_modal = false);
    };
    let on_toggle_add_group = move |_| {
        app_store.update(|s| s.show_top_add_group = !s.show_top_add_group);
        app_store.update(|s| s.show_add_modal = false);
    };
    let on_toggle_add_asset = move |_| {
        app_store.update(|s| s.show_top_add_asset = !s.show_top_add_asset);
        app_store.update(|s| s.show_add_modal = false);
    };
    let on_toggle_add_modal = move |_| {
        app_store.update(|s| s.show_add_modal = !s.show_add_modal);
    };

    view! {
        // Main Navbar - Fixed at top, single row
        <nav class="navbar">
            // ROW 1: buttons always visible
            <div class="navbar-row navbar-row-1">
                <div class="nav-row1-left">
                    <button class="nav-btn" on:click=on_drawer_toggle title="Toggle sidebar" class:nav-btn-active=drawer_open>{drawer_icon}</button>
                    <button class="nav-btn" on:click=on_home title="Home">"⌂"</button>
                    <button class="nav-btn" on:click=on_redo
                        disabled={move || !can_redo()} title="Redo">"↻"</button>
                </div>
                <div class="nav-row1-centre">
                    <span class="nav-profile-name-top">{profile_name}</span>
                </div>
                <div class="nav-row1-right">
                    <button class="nav-btn" on:click=on_undo
                        disabled={move || !can_undo()} title="Undo">"↺"</button>
                    <button class="nav-btn nav-search-btn" on:click=on_search_click title="Search">"🔍"</button>
                    <div class="nav-message-wrap" on:click=on_message_click title="Open messages">
                        <div class="nav-message-icon">"💬"</div>
                        {move || {
                            let count = message_count();
                            if count > 0 {
                                view! { <div class="nav-message-badge">{count}</div> }.into_any()
                            } else { ().into_any() }
                        }}
                    </div>
                    <button
                        class="nav-btn nav-pen-btn"
                        title="Toggle edit mode"
                        on:click=on_pen
                        class:nav-pen-active={move || app_store.get().active_tabs.first().map(|t| app_store.get().is_tab_edit_mode(t)).unwrap_or(false)}
                    >
                        "✎"
                    </button>
                </div>
            </div>

            // Contextual portfolio controls row (shown when Portfolios tab is active)
            {move || if is_portfolios_tab() {
                view! {
                    <div class="navbar-row navbar-row-portfolio">
                        <div class="nav-portfolio-controls">
                            <button
                                class="nav-portfolio-btn"
                                class:active={move || portfolio_view() == ViewMode::List}
                                on:click=on_list_view
                            >
                                "☰ List"
                            </button>
                            <button
                                class="nav-portfolio-btn"
                                class:active={move || portfolio_view() == ViewMode::Grid}
                                on:click=on_grid_view
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
                        {move || if can_edit_portfolio() {
                            view! {
                                <div class="nav-portfolio-edit-controls">
                                    <button
                                        class="nav-portfolio-btn"
                                        class:active={move || app_store.get().show_add_modal}
                                        on:click=on_toggle_add_modal
                                    >
                                        "+"
                                    </button>
                                    {move || if app_store.get().show_add_modal {
                                        view! {
                                            <div class="pf-nav-add-dropdown" on:click=|ev| ev.stop_propagation()>
                                                <button class="pf-nav-add-item" on:click=on_toggle_add_portfolio>
                                                    "🏢 Portfolio"
                                                </button>
                                                <button class="pf-nav-add-item" on:click=on_toggle_add_group>
                                                    "📁 Asset Group"
                                                </button>
                                                <button class="pf-nav-add-item" on:click=on_toggle_add_asset>
                                                    "📦 Asset"
                                                </button>
                                            </div>
                                        }.into_any()
                                    } else { ().into_any() }}
                                </div>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </nav>

        // Message drawer overlay
        {move || if is_message_drawer_open() {
            view! { <MessageDrawer /> }.into_any()
        } else { ().into_any() }}

        // Search panel - drops below navbar when open
        {move || if is_search_open() {
            view! {
                <div class="search-drop-panel">
                    <SearchFilters />
                </div>
            }.into_any()
        } else { ().into_any() }}
    }
}
