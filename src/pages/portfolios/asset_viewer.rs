use crate::models::{Asset, Channel, Perm, Portfolio};
use crate::stores::{use_app_store, use_organization_store, use_ui_store};
use crate::types::{AssetType, ViewMode};
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::{AssetDetailView, AssetGroupItem, AssetItem, AssetTarget};

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
        AssetSortMode::NameAsc => {
            assets.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        }
        AssetSortMode::NameDesc => {
            assets.sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase()))
        }
        AssetSortMode::ValueHigh => assets.sort_by(|a, b| {
            b.current_value
                .partial_cmp(&a.current_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        AssetSortMode::ValueLow => assets.sort_by(|a, b| {
            a.current_value
                .partial_cmp(&b.current_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
    }
    assets
}

fn sort_groups(
    mut groups: Vec<crate::models::AssetGroup>,
    mode: AssetSortMode,
) -> Vec<crate::models::AssetGroup> {
    match mode {
        AssetSortMode::Recent => groups.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
        AssetSortMode::NameAsc => {
            groups.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        }
        AssetSortMode::NameDesc => {
            groups.sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase()))
        }
        AssetSortMode::ValueHigh => groups.sort_by(|a, b| {
            b.total_value
                .partial_cmp(&a.total_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        AssetSortMode::ValueLow => groups.sort_by(|a, b| {
            a.total_value
                .partial_cmp(&b.total_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
    }
    groups
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

#[component]
pub(crate) fn AssetViewer(
    portfolio: Portfolio,
    #[prop(into)] can_edit: Signal<bool>,
    #[prop(into)] can_edit_documents: Signal<bool>,
    #[prop(into)] view_mode: Signal<ViewMode>,
    #[prop(into)] show_add_group: Signal<Option<Uuid>>,
    #[allow(unused_variables)] set_show_add_group: WriteSignal<Option<Uuid>>,
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
    on_add_asset: Callback<AssetTarget, Option<Uuid>>,
) -> impl IntoView {
    let pid = portfolio.id;
    let portfolio_name_groups = portfolio.name.clone();
    let portfolio_name_direct = portfolio.name.clone();
    let app_store_inner = use_app_store();
    let ui_store = use_ui_store();
    let current_user = app_store_inner.get().current_user.clone();
    let user_id = current_user.id;
    let can_view_all = current_user.can_view_all();
    let organization_store = use_organization_store();

    let has_org_perm = move |org_id: Option<Uuid>, perm: &Perm| -> bool {
        org_id.map_or(can_view_all, |oid| {
            organization_store
                .get()
                .user_has_perm_in_org(oid, user_id, perm)
        })
    };

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
            set_expanded_groups.update(|set| {
                set.insert(gid);
            });
            app_store_inner.update(|s| s.pending_group_expand = None);
        }
    });

    let (show_groups, set_show_groups) = signal(false);
    let (show_direct_assets, set_show_direct_assets) = signal(false);

    let (new_channel_name, set_new_channel_name) = signal(String::new());
    let (new_channel_rate, set_new_channel_rate) = signal(String::new());
    let (new_asset_id, set_new_asset_id) = signal(Option::<Uuid>::None);

    let (grid_columns, _set_grid_columns) = signal(3usize);
    let (selected_asset, set_selected_asset) = signal::<Option<Asset>>(None);

    // Asset sort state for grid view sections
    let (group_sort_open, set_group_sort_open) = signal(false);
    let (group_sort_mode, set_group_sort_mode) = signal(AssetSortMode::Recent);
    let (direct_sort_open, set_direct_sort_open) = signal(false);
    let (direct_sort_mode, set_direct_sort_mode) = signal(AssetSortMode::Recent);

    // Reactive sorted lists so the sort dropdowns actually re-order items
    let portfolio_for_group_sort = portfolio.clone();
    let sorted_groups = Memo::new(move |_| {
        sort_groups(
            portfolio_for_group_sort
                .asset_groups
                .clone()
                .into_iter()
                .filter(|g| {
                    g.is_visible_to(user_id, can_view_all)
                        || has_org_perm(g.organization_id, &Perm::ViewAssetGroups)
                })
                .collect(),
            group_sort_mode.get(),
        )
    });
    let portfolio_for_direct_sort = portfolio.clone();
    let sorted_direct_assets = Memo::new(move |_| {
        sort_assets(
            portfolio_for_direct_sort
                .assets
                .clone()
                .into_iter()
                .filter(|a| {
                    a.is_visible_to(user_id, can_view_all)
                        || has_org_perm(a.organization_id, &Perm::ViewAssets)
                })
                .collect(),
            direct_sort_mode.get(),
        )
    });

    // Per-scope visible counts for Expand View + behavior.
    // Scopes: "groups-{pid}", "direct-{pid}", "group-assets-{gid}".
    // Each scope starts at the global view count and increments by that amount.
    let (visible_counts, set_visible_counts) = signal(HashMap::<String, usize>::new());
    let group_scope = format!("groups-{pid}");
    let direct_scope = format!("direct-{pid}");

    let page_size = move || {
        ui_store
            .get()
            .portfolio_view_count(view_mode.get())
            .as_usize()
    };
    let visible_for = move |scope: &str| {
        visible_counts
            .get()
            .get(scope)
            .copied()
            .unwrap_or_else(page_size)
    };
    let expand_scope = Callback::new(move |scope: String| {
        let increment = page_size();
        set_visible_counts.update(|map| {
            let current = map.get(&scope).copied().unwrap_or_else(|| page_size());
            map.insert(scope, current.saturating_add(increment));
        });
    });

    // Auto-expand the relevant section when an add form is targeted at this portfolio
    Effect::new(move |_| {
        if show_add_asset.get() == AssetTarget::PortfolioDirect(pid) {
            set_show_direct_assets.set(true);
        }
        if show_add_group.get() == Some(pid) {
            set_show_groups.set(true);
        }
    });

    let on_select_asset = Callback::new(move |asset: Asset| {
        set_selected_asset.set(Some(asset));
    });

    let on_close_asset = Callback::new(move |_| {
        set_selected_asset.set(None);
    });

    let view_mode_groups_content = view_mode;
    let view_mode_direct_title = view_mode;
    let view_mode_direct_content = view_mode;
    let portfolio_groups_for_header = portfolio.clone();
    let portfolio_for_group_count = portfolio.clone();
    let portfolio_for_direct_count = portfolio.clone();

    let has_groups = !portfolio_for_group_count.asset_groups.is_empty();
    let has_direct = !portfolio_for_direct_count.assets.is_empty();

    let groups_view = move |standalone: bool| {
        let group_scope = group_scope.clone();
        view! {
            {let visible_groups = sorted_groups.get();
            let vmg = view_mode_groups_content.clone();
            view! {
                <div id="av-groups-content">
                    // Sort dropdown inside content area (grid mode only) only when this view is standalone
                    {let vg_for_sort = visible_groups.clone();
                    if standalone {
                        view! {
                            {move || {
                                if vmg.get() == ViewMode::Grid && !vg_for_sort.is_empty() {
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
                        }.into_any()
                    } else { ().into_any() }}

                    {move || show_add_group.get().map(|gp| {
                        if gp == pid {
                            view! {
                                <div class="add-form">
                                    <input class="login-input" type="text" placeholder="Group name"
                                        aria-label="Group name"
                                        on:input=move |ev| set_new_group_name.set(event_target_value(&ev)) />
                                    <button class="login-btn" on:click=move |_| on_add_group.run(pid)>
                                        "Add Group"
                                    </button>
                                </div>
                            }.into_any()
                        } else { ().into_any() }
                    })}

                    {let total_groups = visible_groups.len();
                    if visible_groups.is_empty() {
                        view! {
                            <div class="empty-state">
                                <div class="empty-text">"No asset groups"</div>
                            </div>
                        }.into_any()
                    } else {
                        let portfolio_name = portfolio_name_groups.clone();
                        let display_count = visible_for(&group_scope).min(total_groups);
                        let remaining = total_groups.saturating_sub(display_count);
                        let groups_to_show: Vec<_> = visible_groups.into_iter().take(display_count).collect();
                        let group_scope_btn = group_scope.clone();
                        let group_class = move || if view_mode_groups_content.get() == ViewMode::Grid { "grid-view".to_string() } else { "asset-list".to_string() };
                        view! {
                            <div class={group_class}>
                                {groups_to_show.into_iter().enumerate().map(move |(idx, group)| {
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
                                            view_mode={view_mode_groups_content.get()}
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
                                            visible_counts={visible_counts}
                                            set_visible_counts={set_visible_counts}
                                        />
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            {if remaining > 0 {
                                view! {
                                    <button class="pf-show-more-btn pf-expand-view-btn"
                                        aria-label={format!("Expand view. Currently showing {} of {} asset groups", display_count, total_groups)}
                                        on:click=move |_| expand_scope.run(group_scope_btn.clone())
                                    >
                                        {format!("Expand View + ({}/{}) ", display_count, total_groups)}
                                    </button>
                                }.into_any()
                            } else { ().into_any() }}
                        }.into_any()
                    }}
                </div>
            }.into_any()}
        }.into_any()
    };

    let direct_view = move |standalone: bool| {
        let direct_scope = direct_scope.clone();
        view! {
            {if standalone {
                view! {
                    <div class="section-title-right">
                        {move || {
                            if view_mode_direct_title.get() == ViewMode::Grid && !sorted_direct_assets.get().is_empty() {
                                view! {
                                    <div class="sort-dropdown-wrap sort-dropdown-inline">
                                        <button class="sort-btn"
                                            on:click=move |ev: leptos::ev::MouseEvent| {
                                                ev.stop_propagation();
                                                set_direct_sort_open.update(|v| *v = !*v)
                                            }
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
                }.into_any()
            } else { ().into_any() }}

            {let visible_direct_assets = sorted_direct_assets.get();
            view! {
                <div id="av-direct-content">

                    {move || {
                        if show_add_asset.get() == AssetTarget::PortfolioDirect(pid) {
                            view! {
                                <div class="add-form">
                                    <input class="login-input" type="text" placeholder="Asset name"
                                        aria-label="Asset name"
                                        on:input=move |ev| set_new_asset_name.set(event_target_value(&ev)) />
                                    <input class="login-input" type="text" list="asset-type-options-direct" placeholder="Asset type"
                                        aria-label="Asset type"
                                        prop:value={move || new_asset_type.get().to_input_string()}
                                        on:input=move |ev| set_new_asset_type.set(AssetType::from_input(&event_target_value(&ev))) />
                                    <datalist id="asset-type-options-direct">
                                        <option value="RealEstate">"Real Estate"</option>
                                        <option value="Vehicle">"Vehicle"</option>
                                        <option value="Equipment">"Equipment"</option>
                                        <option value="Stock">"Stock"</option>
                                        <option value="Bond">"Bond"</option>
                                        <option value="Commodity">"Commodity"</option>
                                        <option value="Digital">"Digital"</option>
                                        <option value="IntellectualProperty">"Intellectual Property"</option>
                                        <option value="Channel">"Channel"</option>
                                    </datalist>
                                    <input class="login-input" type="number" placeholder="Value ($)"
                                        aria-label="Value"
                                        on:input=move |ev| set_new_asset_value.set(event_target_value(&ev)) />
                                    <input class="login-input" type="text" placeholder="Channel name (optional)"
                                        aria-label="Channel name"
                                        prop:value={move || new_channel_name.get()}
                                        on:input=move |ev| set_new_channel_name.set(event_target_value(&ev)) />
                                    <input class="login-input" type="number" placeholder="Channel nightly rate (optional)"
                                        aria-label="Channel nightly rate"
                                        prop:value={move || new_channel_rate.get()}
                                        on:input=move |ev| set_new_channel_rate.set(event_target_value(&ev)) />
                                    <button class="login-btn" on:click=move |_| {
                                        if let Some(asset_id) = on_add_asset.run(AssetTarget::PortfolioDirect(pid)) {
                                            let name = new_channel_name.get();
                                            if !name.trim().is_empty() {
                                                let rate = new_channel_rate.get().parse::<f64>().ok();
                                                let mut channel = Channel::new_test_channel(name, Some(asset_id), Some(pid));
                                                channel.nightly_rate_override = rate;
                                                app_store_inner.update(|s| s.add_channel(channel));
                                            }
                                            set_new_asset_id.set(Some(asset_id));
                                            set_new_channel_name.set(String::new());
                                            set_new_channel_rate.set(String::new());
                                        }
                                    }>
                                        "Add Asset"
                                    </button>
                                </div>
                            }.into_any()
                        } else { ().into_any() }
                    }}

                    {let total_direct = visible_direct_assets.len();
                    if visible_direct_assets.is_empty() {
                        view! {
                            <div class="empty-state">
                                <div class="empty-text">"No direct assets"</div>
                            </div>
                        }.into_any()
                    } else {
                        let portfolio_name = portfolio_name_direct.clone();
                        let display_direct = visible_for(&direct_scope).min(total_direct);
                        let direct_remaining = total_direct.saturating_sub(display_direct);
                        let direct_to_show: Vec<_> = visible_direct_assets.into_iter().take(display_direct).collect();
                        let direct_scope_btn = direct_scope.clone();
                        let direct_class = move || if view_mode_direct_content.get() == ViewMode::Grid {
                            format!("grid-view-{}", grid_columns.get())
                        } else {
                            "asset-list".to_string()
                        };
                        view! {
                            <div class={direct_class}>
                                {direct_to_show.into_iter().enumerate().map(move |(idx, asset)| view! {
                                    <AssetItem asset={asset} portfolio_name={portfolio_name.clone()} portfolio_id={Some(pid)} group_id={None} view_mode={view_mode_direct_content.get()} on_select={on_select_asset} can_edit={can_edit} can_edit_documents={can_edit_documents} tint_index={idx + 1} collapsible=true highlight={Some(Signal::derive(move || new_asset_id.get()))} />
                                }).collect::<Vec<_>>()}
                            </div>
                            {if direct_remaining > 0 {
                                view! {
                                    <button class="pf-show-more-btn pf-expand-view-btn"
                                        aria-label={format!("Expand view. Currently showing {} of {} direct assets", display_direct, total_direct)}
                                        on:click=move |_| expand_scope.run(direct_scope_btn.clone())
                                    >
                                        {format!("Expand View + ({}/{}) ", display_direct, total_direct)}
                                    </button>
                                }.into_any()
                            } else { ().into_any() }}
                        }.into_any()
                    }}
                </div>
            }.into_any()}
        }.into_any()
    };

    view! {
        <div class="asset-viewer">
            {if has_groups && has_direct {
                view! {
                    <div class="asset-section">
                        <div class="asset-section-title"
                            role="button"
                            tabindex="0"
                            aria-expanded={move || show_groups.get()}
                            aria-controls="av-groups-content"
                            aria-label={move || if show_groups.get() { "Collapse Asset Groups section" } else { "Expand Asset Groups section" }}
                            on:click=move |_| set_show_groups.update(|v| *v = !*v)
                            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                if ev.key() == "Enter" || ev.key() == " " {
                                    ev.prevent_default();
                                    set_show_groups.update(|v| *v = !*v);
                                }
                            }
                        >
                            <div class="asset-section-title-left">
                                <span class="asset-section-arrow" aria-hidden="true">
                                    {move || if show_groups.get() { "▶" } else { "▼" }}
                                </span>
                                <span class="asset-section-label">"Asset Groups"</span>
                            </div>
                            <div class="section-title-right">
                                {move || {
                                    if show_groups.get() && view_mode_groups_content.get() == ViewMode::Grid && !portfolio_groups_for_header.asset_groups.is_empty() {
                                        view! {
                                            <div class="sort-dropdown-wrap sort-dropdown-inline">
                                                <button class="sort-btn"
                                                    on:click=move |ev: leptos::ev::MouseEvent| {
                                                        ev.stop_propagation();
                                                        set_group_sort_open.update(|v| *v = !*v)
                                                    }
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
                            </div>
                        </div>

                        {move || if show_groups.get() { groups_view(false) } else { ().into_any() }}
                    </div>
                }.into_any()
            } else if has_groups {
                groups_view(true)
            } else { ().into_any() }}

            {if has_direct && has_groups {
                view! {
                    <div class="asset-section">
                        <div class="asset-section-title"
                            role="button"
                            tabindex="0"
                            aria-expanded={move || show_direct_assets.get()}
                            aria-controls="av-direct-content"
                            aria-label={move || if show_direct_assets.get() { "Collapse Direct Assets section" } else { "Expand Direct Assets section" }}
                            on:click=move |_| set_show_direct_assets.update(|v| *v = !*v)
                            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                if ev.key() == "Enter" || ev.key() == " " {
                                    ev.prevent_default();
                                    set_show_direct_assets.update(|v| *v = !*v);
                                }
                            }
                        >
                            <div class="asset-section-title-left">
                                <span class="asset-section-arrow" aria-hidden="true">
                                    {move || if show_direct_assets.get() { "▶" } else { "▼" }}
                                </span>
                                <span class="asset-section-label">"Direct Assets"</span>
                            </div>
                            <div class="section-title-right">
                                {move || {
                                    if show_direct_assets.get() && view_mode_direct_title.get() == ViewMode::Grid && !sorted_direct_assets.get().is_empty() {
                                        view! {
                                            <div class="sort-dropdown-wrap sort-dropdown-inline">
                                                <button class="sort-btn"
                                                    on:click=move |ev: leptos::ev::MouseEvent| {
                                                        ev.stop_propagation();
                                                        set_direct_sort_open.update(|v| *v = !*v)
                                                    }
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

                        {move || if show_direct_assets.get() { direct_view(false) } else { ().into_any() }}
                    </div>
                }.into_any()
            } else if has_direct {
                direct_view(true)
            } else { ().into_any() }}

            {move || selected_asset.get().map(|asset| view! {
                <AssetDetailView asset={asset} portfolio_id={Some(pid)} can_edit={can_edit} on_close={on_close_asset} />
            })}
        </div>
    }
}
