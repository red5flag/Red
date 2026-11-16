use crate::stores::{
    use_app_store, use_organization_store, use_search_store, use_ui_store, SearchResultItem,
};
use crate::types::{SearchFilters, TabType};
use chrono::TimeZone;
use leptos::prelude::*;
use uuid::Uuid;

fn parse_dd_mm_yyyy(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    if s.is_empty() {
        return None;
    }
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 3 {
        return None;
    }
    let day = parts[0].parse::<u32>().ok()?;
    let month = parts[1].parse::<u32>().ok()?;
    let year = parts[2].parse::<i32>().ok()?;
    let naive = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
    let dt = naive.and_hms_opt(0, 0, 0)?;
    Some(chrono::Utc.from_utc_datetime(&dt))
}

#[component]
pub fn SearchFilters() -> impl IntoView {
    let app_store = use_app_store();
    let search_store = use_search_store();
    let organization_store = use_organization_store();
    let ui_store = use_ui_store();

    let execute_search = move || {
        app_store.with(|app| {
            organization_store.with(|org| {
                search_store.update(|store| {
                    store.is_loading = true;
                    store.perform_search(app, org);
                    store.is_loading = false;
                });
            });
        });
    };

    let (adv_open, set_adv_open) = signal(false);
    let search_input_ref = NodeRef::<leptos::html::Input>::new();
    let (lineage_open, set_lineage_open) = signal(false);
    let (tree_open, set_tree_open) = signal(false);
    let (prof1, set_prof1) = signal(false);
    let (prof2, set_prof2) = signal(false);
    let (time_from, set_time_from) = signal(String::new());
    let (time_to, set_time_to) = signal(String::new());
    let (asset_p, set_asset_p) = signal(false);
    let (asset_a, set_asset_a) = signal(false);
    let (asset_addr, set_asset_addr) = signal(false);
    let (chg_add, set_chg_add) = signal(false);
    let (chg_rm, set_chg_rm) = signal(false);
    let (chg_ch, set_chg_ch) = signal(false);
    let (chg_undo, set_chg_undo) = signal(false);
    let (chg_redo, set_chg_redo) = signal(false);

    // Only re-run this effect when the search panel actually opens/closes,
    // not when unrelated ui_store fields (tabs drawer, messages, etc.) change.
    let is_search_open = Memo::new(move |_| ui_store.with(|s| s.is_search_open));

    // Run an initial search when the panel opens and sync the advanced filter
    // chips/inputs into SearchStore.filters whenever they change.
    Effect::new(move |_| {
        let is_open = is_search_open.get();
        if is_open {
            if let Some(input) = search_input_ref.get() {
                let _ = input.focus();
            }
        } else {
            return;
        }

        // Build filters from the advanced UI signals.
        let mut filters = SearchFilters::default();
        if prof1.get() {
            filters.tags.push("Stevenson 2".to_string());
        }
        if prof2.get() {
            filters.tags.push("Stevenson 3".to_string());
        }
        if asset_p.get() {
            filters.tags.push("portfolio".to_string());
        }
        if asset_a.get() {
            filters.tags.push("asset".to_string());
        }
        if asset_addr.get() {
            filters.tags.push("address".to_string());
        }
        if chg_add.get() {
            filters.tags.push("add".to_string());
        }
        if chg_rm.get() {
            filters.tags.push("remove".to_string());
        }
        if chg_ch.get() {
            filters.tags.push("change".to_string());
        }
        if chg_undo.get() {
            filters.tags.push("undo".to_string());
        }
        if chg_redo.get() {
            filters.tags.push("redo".to_string());
        }
        if let (Some(from), Some(to)) = (
            parse_dd_mm_yyyy(&time_from.get()),
            parse_dd_mm_yyyy(&time_to.get()),
        ) {
            filters.date_range = Some((from, to));
        }

        // Use untracked snapshots so this effect only re-runs when search opens
        // or the filter/time signals change, not on every app store update.
        let app_snapshot = app_store.get_untracked();
        let context_tab = app_snapshot
            .active_tabs
            .iter()
            .find(|&t| matches!(t, TabType::Networking | TabType::NetworkingAddMember))
            .cloned()
            .or_else(|| app_snapshot.active_tabs.first().cloned())
            .unwrap_or(TabType::Overview);
        let org_snapshot = organization_store.get_untracked();

        // Write back via set/get_untracked to avoid making the effect depend on
        // search_store, which would create a reactive write-read loop.
        let mut store = search_store.get_untracked();
        store.filters = filters;
        store.set_context_tab(context_tab);
        if !store.query.trim().is_empty() {
            store.is_loading = true;
            store.perform_search(&app_snapshot, &org_snapshot);
            store.is_loading = false;
        }
        search_store.set(store);
    });

    view! {
        <div class="sd-panel">
            <div class="sd-search-bar">
                <input
                    type="text"
                    class="sd-search-input"
                    placeholder="Search..."
                    node_ref=search_input_ref
                    prop:value={move || search_store.with(|s| s.query.clone())}
                    on:input=move |ev| {
                        let v = event_target_value(&ev);
                        app_store.with(|app| {
                            organization_store.with(|org| {
                                search_store.update(|store| {
                                    store.set_query(v);
                                    store.is_loading = true;
                                    store.perform_search(app, org);
                                    store.is_loading = false;
                                });
                            });
                        });
                    }
                    on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                        if ev.key() == "Enter" {
                            execute_search();
                        }
                    }
                />
                <button class="sd-search-close-btn" on:click=move |_| {
                    ui_store.update(|s| s.close_search());
                }>"✕"</button>
                <button class="sd-search-close-btn sd-search-submit-btn" on:click=move |_| {
                    execute_search();
                }>"⏎"</button>
            </div>

            {move || {
                let suggestions = search_store.with(|store| {
                    if store.query.len() >= 2 {
                        store.suggestions.clone()
                    } else {
                        store.recent_searches.clone().into_iter().rev().collect()
                    }
                });
                if suggestions.is_empty() {
                    ().into_any()
                } else {
                    view! {
                        <div class="sd-suggestions">
                            <div class="sd-suggestions-label">"Relevant searches"</div>
                            {suggestions.into_iter().map(|s| {
                                let s_clone = s.clone();
                                view! {
                                    <div
                                        class="sd-suggestion"
                                        on:click=move |_| search_store.update(|st| st.set_query(s_clone.clone()))
                                    >
                                        {s}
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }
            }}

            <div class="sd-section">
                <div class="sd-section-header" on:click=move |_| set_adv_open.update(|v| *v = !*v)>
                    <span class="sd-section-title">"ADVANCED SEARCH"</span>
                    <span class="sd-arrow">{move || if adv_open.get() {"▲"} else {"▼"}}</span>
                </div>
                {move || if adv_open.get() { view! {
                    <div class="sd-adv-body">
                        <div class="sd-filter-row">
                            <div class="sd-filter-label">"PROFILE"</div>
                            <div class="sd-filter-chips">
                                <button class="sd-chip" class:sd-chip-active=move || prof1.get()
                                    on:click=move |_| set_prof1.update(|v| *v = !*v)>"Stevenson 2"</button>
                                <button class="sd-chip sd-chip-purple" class:sd-chip-active=move || prof2.get()
                                    on:click=move |_| set_prof2.update(|v| *v = !*v)>"Stevenson 3"</button>
                            </div>
                        </div>
                        <div class="sd-filter-row">
                            <div class="sd-filter-label">"TIME"</div>
                            <div class="sd-filter-chips">
                                <input class="sd-time-input" type="text" placeholder="From: DD/MM/YYYY"
                                    prop:value=time_from
                                    on:input=move |ev| set_time_from.set(event_target_value(&ev)) />
                                <input class="sd-time-input" type="text" placeholder="To: DD/MM/YYYY"
                                    prop:value=time_to
                                    on:input=move |ev| set_time_to.set(event_target_value(&ev)) />
                            </div>
                        </div>
                        <div class="sd-filter-row">
                            <div class="sd-filter-label">"ASSET"</div>
                            <div class="sd-filter-chips">
                                <button class="sd-chip sd-chip-green" class:sd-chip-active=move || asset_p.get()
                                    on:click=move |_| set_asset_p.update(|v| *v = !*v)>"PORTFOLIO"</button>
                                <button class="sd-chip sd-chip-blue" class:sd-chip-active=move || asset_a.get()
                                    on:click=move |_| set_asset_a.update(|v| *v = !*v)>"ASSET"</button>
                                <button class="sd-chip sd-chip-purple" class:sd-chip-active=move || asset_addr.get()
                                    on:click=move |_| set_asset_addr.update(|v| *v = !*v)>"ADDRESS"</button>
                            </div>
                        </div>
                        <div class="sd-filter-row">
                            <div class="sd-filter-label">"CHANGE"</div>
                            <div class="sd-filter-chips">
                                <button class="sd-chip sd-chip-green" class:sd-chip-active=move || chg_add.get()
                                    on:click=move |_| set_chg_add.update(|v| *v = !*v)>"ADD"</button>
                                <button class="sd-chip sd-chip-red" class:sd-chip-active=move || chg_rm.get()
                                    on:click=move |_| set_chg_rm.update(|v| *v = !*v)>"REMOVE"</button>
                                <button class="sd-chip sd-chip-yellow" class:sd-chip-active=move || chg_ch.get()
                                    on:click=move |_| set_chg_ch.update(|v| *v = !*v)>"CHANGE"</button>
                                <button class="sd-chip" class:sd-chip-active=move || chg_undo.get()
                                    on:click=move |_| set_chg_undo.update(|v| *v = !*v)>"UNDO"</button>
                                <button class="sd-chip" class:sd-chip-active=move || chg_redo.get()
                                    on:click=move |_| set_chg_redo.update(|v| *v = !*v)>"REDO"</button>
                            </div>
                        </div>
                        <div class="sd-filter-row">
                            <div class="sd-filter-label">"VIEW"</div>
                            <div class="sd-filter-chips">
                                <button class="sd-chip sd-chip-blue" class:sd-chip-active=move || lineage_open.get()
                                    on:click=move |_| set_lineage_open.update(|v| *v = !*v)>"Lineage View"</button>
                                <button class="sd-chip sd-chip-green" class:sd-chip-active=move || tree_open.get()
                                    on:click=move |_| set_tree_open.update(|v| *v = !*v)>"Tree View"</button>
                            </div>
                        </div>
                        {move || if lineage_open.get() { view! {
                            <div class="sd-lineage-body">
                                <div class="sd-lineage-cols">
                                    <div class="sd-lineage-col">
                                        <div class="sd-lineage-col-header">"STEVENSON 2"</div>
                                        <div class="sd-lineage-row-label">"DATE"</div>
                                        <div class="sd-lineage-row-label">"PORT"</div>
                                        <div class="sd-lineage-row-label">"ASSET"</div>
                                        <div class="sd-lineage-row-label">"CHANGE"</div>
                                    </div>
                                    <div class="sd-lineage-col">
                                        <div class="sd-lineage-col-header">"STEVENSON 3"</div>
                                        <div class="sd-lineage-row-label">"DATE"</div>
                                        <div class="sd-lineage-row-label">"PORT"</div>
                                        <div class="sd-lineage-row-label">"ASSET"</div>
                                        <div class="sd-lineage-row-label">"CHANGE"</div>
                                    </div>
                                </div>
                            </div>
                        }.into_any()} else {().into_any()}}
                        {move || if tree_open.get() { view! {
                            <div class="sd-tree-body">
                                <table class="sd-tree-table">
                                    <thead>
                                        <tr>
                                            <th>"PAGE"</th><th>"DETAILS"</th>
                                            <th>"TIME"</th><th>"CONFIG"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <tr><td>"Login"</td><td>"Profile"</td><td>"—"</td><td>"—"</td></tr>
                                        <tr><td>"Home"</td><td>"Base"</td><td>"—"</td><td>"—"</td></tr>
                                        <tr><td>"Overview"</td><td>"Portfolio"</td><td>"—"</td><td>"—"</td></tr>
                                        <tr><td>"Group Chat"</td><td>"GroupChatLink"</td><td>"—"</td><td>"—"</td></tr>
                                        <tr><td>"Message"</td><td>"OpenLink"</td><td>"—"</td><td>"—"</td></tr>
                                    </tbody>
                                </table>
                            </div>
                        }.into_any()} else {().into_any()}}
                    </div>
                }.into_any()} else {().into_any()}}
            </div>

            <SearchResults />
        </div>
    }
}

#[component]
pub fn SearchResults() -> impl IntoView {
    let search_store = use_search_store();
    let app_store = use_app_store();
    let ui_store = use_ui_store();
    let is_loading = move || search_store.with(|s| s.is_loading);
    let has_results = move || search_store.with(|s| s.results.total_count > 0);
    let results_count = move || search_store.with(|s| s.results.total_count);
    let has_searched = move || search_store.with(|s| s.has_searched);

    fn result_icon(item: &SearchResultItem) -> &'static str {
        match item {
            SearchResultItem::Portfolio(_) => "🏢",
            SearchResultItem::AssetGroup { .. } => "📁",
            SearchResultItem::Asset { .. } => "📦",
            SearchResultItem::AssetDetail(d) => match d.detail_type {
                "Channel" => "📡",
                "Booking" => "🗓",
                _ => "📋",
            },
            SearchResultItem::Document { .. } => "📄",
            SearchResultItem::Organization(_) => "🏛",
            SearchResultItem::User(_) => "👤",
        }
    }

    fn result_snippet(item: &SearchResultItem) -> String {
        match item {
            SearchResultItem::Portfolio(p) => {
                format!(
                    "Portfolio · ${:.2} value · {} assets",
                    p.total_value,
                    p.get_all_assets().len()
                )
            }
            SearchResultItem::AssetGroup { group, .. } => {
                format!(
                    "Asset Group · ${:.2} value · {} assets",
                    group.total_value,
                    group.assets.len()
                )
            }
            SearchResultItem::Asset { asset, .. } => {
                format!("{:?} · ${:.2}", asset.asset_type, asset.current_value)
            }
            SearchResultItem::AssetDetail(d) => d.snippet.clone(),
            SearchResultItem::Document { document, .. } => {
                format!("{} · {}", document.file_type, item.entity_type())
            }
            SearchResultItem::Organization(o) => {
                format!("{} members · {} roles", o.members.len(), o.roles.len())
            }
            SearchResultItem::User(u) => u.email.clone(),
        }
    }

    let on_portfolio_click = move |id: Uuid| {
        app_store.update(|s| {
            s.touch_portfolio(id);
            s.selected_portfolio_ids.insert(id);
            s.pending_scroll_target = Some(crate::stores::ScrollTarget::Portfolio(id));
            s.expand_tab(crate::types::TabType::Portfolios);
        });
        ui_store.update(|s| s.close_search());
    };

    let on_group_click = move |portfolio_id: Uuid, group_id: Uuid| {
        app_store.update(|s| {
            s.touch_portfolio(portfolio_id);
            s.selected_portfolio_ids.insert(portfolio_id);
            s.selected_asset_group_id = Some(group_id);
            s.pending_group_expand = Some(group_id);
            s.pending_scroll_target = Some(crate::stores::ScrollTarget::AssetGroup(group_id));
            s.expand_tab(crate::types::TabType::Portfolios);
        });
        ui_store.update(|s| s.close_search());
    };

    let on_asset_click = move |portfolio_id: Uuid,
                               group_id: Option<Uuid>,
                               asset_id: Uuid,
                               detail_type: Option<&str>| {
        let detail_type = detail_type.map(|s| s.to_string());
        app_store.update(|s| {
            s.touch_portfolio(portfolio_id);
            s.selected_portfolio_ids.insert(portfolio_id);
            s.selected_asset_id = Some(asset_id);
            s.pending_group_expand = group_id;
            s.pending_asset_detail = Some(asset_id);
            s.pending_scroll_target = if let Some(dt) = detail_type {
                Some(crate::stores::ScrollTarget::AssetDetail {
                    asset_id,
                    detail_type: dt,
                })
            } else {
                Some(crate::stores::ScrollTarget::Asset(asset_id))
            };
            s.expand_tab(crate::types::TabType::Portfolios);
        });
        ui_store.update(|s| s.close_search());
    };

    let on_org_click = move |id: Uuid| {
        app_store.update(|s| {
            s.expand_tab(crate::types::TabType::Organization);
            let _ = id;
        });
        ui_store.update(|s| s.close_search());
    };

    let on_user_click = move |_id: Uuid| {
        app_store.update(|s| {
            s.expand_tab(crate::types::TabType::Networking);
        });
        ui_store.update(|s| s.close_search());
    };

    let on_document_click =
        move |portfolio_id: Uuid, group_id: Option<Uuid>, asset_id: Option<Uuid>, doc_id: Uuid| {
            app_store.update(|s| {
                s.touch_portfolio(portfolio_id);
                s.selected_portfolio_ids.insert(portfolio_id);
                s.pending_group_expand = group_id;
                if asset_id.is_none() {
                    if let Some(gid) = group_id {
                        s.pending_doc_slider = Some(gid);
                    } else {
                        s.pending_doc_slider = Some(portfolio_id);
                    }
                }
                s.pending_doc_view = Some(doc_id);
                s.pending_scroll_target = Some(crate::stores::ScrollTarget::Document(doc_id));
                s.expand_tab(crate::types::TabType::Portfolios);
            });
            ui_store.update(|s| s.close_search());
        };

    view! {
        <div class="sd-results">
            {move || if is_loading() {
                view! { <div class="loading"><span class="loading-text">"Searching..."</span></div> }.into_any()
            } else if has_results() {
                view! {
                    <div>
                        <span class="sd-result-count">{format!("{} results", results_count())}</span>
                        <div class="sd-results-list">
                            <For
                                each=move || search_store.with(|s| s.results.items.clone())
                                key=|item| item.id()
                                children=move |item| {
                                    let icon = result_icon(&item);
                                    let name = item.name();
                                    let breadcrumb = item.breadcrumb();
                                    let snippet = result_snippet(&item);
                                    match item {
                                        SearchResultItem::Portfolio(p) => {
                                            let pid = p.id;
                                            view! {
                                                <div class="sd-result-row" on:click=move |_| on_portfolio_click(pid)>
                                                    <div class="sd-result-icon">{icon}</div>
                                                    <div class="sd-result-info">
                                                        <div class="sd-result-name">{name}</div>
                                                        <div class="sd-result-breadcrumb">{breadcrumb}</div>
                                                        <div class="sd-result-snippet">{snippet}</div>
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                        SearchResultItem::AssetGroup { group, portfolio_id, .. } => {
                                            let pid = portfolio_id;
                                            let gid = group.id;
                                            view! {
                                                <div class="sd-result-row" on:click=move |_| on_group_click(pid, gid)>
                                                    <div class="sd-result-icon">{icon}</div>
                                                    <div class="sd-result-info">
                                                        <div class="sd-result-name">{name}</div>
                                                        <div class="sd-result-breadcrumb">{breadcrumb}</div>
                                                        <div class="sd-result-snippet">{snippet}</div>
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                        SearchResultItem::Asset { asset, portfolio_id, group_id, .. } => {
                                            let pid = portfolio_id;
                                            let gid = group_id;
                                            let aid = asset.id;
                                            view! {
                                                <div class="sd-result-row" on:click=move |_| on_asset_click(pid, gid, aid, None)>
                                                    <div class="sd-result-icon">{icon}</div>
                                                    <div class="sd-result-info">
                                                        <div class="sd-result-name">{name}</div>
                                                        <div class="sd-result-breadcrumb">{breadcrumb}</div>
                                                        <div class="sd-result-snippet">{snippet}</div>
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                        SearchResultItem::AssetDetail(d) => {
                                            let pid = d.portfolio_id;
                                            let gid = d.group_id;
                                            let aid = d.asset_id;
                                            let dt = Some(d.detail_type);
                                            view! {
                                                <div class="sd-result-row" on:click=move |_| on_asset_click(pid, gid, aid, dt)>
                                                    <div class="sd-result-icon">{icon}</div>
                                                    <div class="sd-result-info">
                                                        <div class="sd-result-name">{name}</div>
                                                        <div class="sd-result-breadcrumb">{breadcrumb}</div>
                                                        <div class="sd-result-snippet">{snippet}</div>
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                        SearchResultItem::Document {
                                            document,
                                            portfolio_id,
                                            group_id,
                                            asset_id,
                                            ..
                                        } => {
                                            let did = document.id;
                                            view! {
                                                <div class="sd-result-row" on:click=move |_| on_document_click(portfolio_id, group_id, asset_id, did)>
                                                    <div class="sd-result-icon">{icon}</div>
                                                    <div class="sd-result-info">
                                                        <div class="sd-result-name">{name}</div>
                                                        <div class="sd-result-breadcrumb">{breadcrumb}</div>
                                                        <div class="sd-result-snippet">{snippet}</div>
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                        SearchResultItem::Organization(o) => {
                                            let oid = o.id;
                                            view! {
                                                <div class="sd-result-row" on:click=move |_| on_org_click(oid)>
                                                    <div class="sd-result-icon">{icon}</div>
                                                    <div class="sd-result-info">
                                                        <div class="sd-result-name">{name}</div>
                                                        <div class="sd-result-breadcrumb">{breadcrumb}</div>
                                                        <div class="sd-result-snippet">{snippet}</div>
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                        SearchResultItem::User(u) => {
                                            let uid = u.id;
                                            view! {
                                                <div class="sd-result-row" on:click=move |_| on_user_click(uid)>
                                                    <div class="sd-result-icon">{icon}</div>
                                                    <div class="sd-result-info">
                                                        <div class="sd-result-name">{name}</div>
                                                        <div class="sd-result-breadcrumb">{breadcrumb}</div>
                                                        <div class="sd-result-snippet">{snippet}</div>
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                    }
                                }
                            />
                        </div>
                    </div>
                }.into_any()
            } else if has_searched() {
                view! { <div class="empty-state"><div class="empty-icon">"🔍"</div><div class="empty-text">"No results found"</div></div> }.into_any()
            } else {
                ().into_any()
            }}
        </div>
    }
}
