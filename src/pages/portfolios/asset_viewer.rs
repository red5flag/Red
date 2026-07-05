use crate::models::{Asset, Portfolio};
use crate::stores::use_app_store;
use crate::types::{AssetType, ViewMode};
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

use super::{AssetDetailView, AssetGroupItem, AssetItem, AssetTarget, NotifTarget};

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
    can_edit: bool,
    can_edit_documents: bool,
    view_mode: ViewMode,
    show_add_group: Option<Uuid>,
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
    on_add_asset: Callback<AssetTarget>,
    on_open_notif_qs: Callback<(NotifTarget, String, bool)>,
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
            set_expanded_groups.update(|set| {
                set.insert(gid);
            });
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
                        <span class="asset-section-arrow" aria-hidden="true">
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
                        <div id="av-groups-content">
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
                                                aria-label="Group name"
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
                                                aria-label="Asset name"
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
                                                aria-label="Value"
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
