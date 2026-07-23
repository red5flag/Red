use crate::components::tabs::use_tab_edit_mode;
use crate::models::{
    Asset, AssetGroup, AvailabilityStatus, Channel, CommercialStatus, ConditionStatus, Document,
    LifecycleStatus, Organization, Perm, Portfolio,
};
use crate::stores::{use_app_store, use_notification_store, use_organization_store, use_ui_store};
use crate::types::{AssetType, SortMode, UserRole, ViewCount, ViewMode};
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::{
    asset_placeholder_url, detect_file_type, name_click_handlers, read_image_as_data_url,
    AddPortfolioModal, AssetTarget, DocEntry, DocSlider, PortfolioListItem,
};

#[derive(Clone, PartialEq)]
struct PortfolioGroup {
    key: Uuid,
    first_index: usize,
    organization: Option<Organization>,
    portfolios: Vec<Portfolio>,
}

/// Compare two portfolio lists by their ids in order.
/// Used by the list memos so that `selected_portfolio_ids` changes
/// (which do not affect filtering or ordering) do not re-render the list.
fn pf_list_changed(old: Option<&Vec<Portfolio>>, new: Option<&Vec<Portfolio>>) -> bool {
    let old = match old {
        Some(old) => old,
        None => return new.is_some(),
    };
    let new = match new {
        Some(new) => new,
        None => return true,
    };
    if old.len() != new.len() {
        return true;
    }
    old.iter().zip(new.iter()).any(|(a, b)| a.id != b.id)
}

#[component]
pub fn PortfoliosPage() -> impl IntoView {
    let app_store = use_app_store();
    let organization_store = use_organization_store();
    let notification_store = use_notification_store();
    let ui_store = use_ui_store();

    // Read portfolios from AppStore and filter by current user visibility
    let filtered_portfolios = Memo::new_with_compare(
        move |_| {
            let user = app_store.get().current_user.clone();
            let can_view_all = user.can_view_all();
            let user_id = user.id;
            app_store
                .get()
                .portfolios
                .iter()
                .filter(|p| {
                    p.is_visible_to(user_id, can_view_all)
                        || p.organization_id.map_or(can_view_all, |oid| {
                            organization_store.get().user_has_perm_in_org(
                                oid,
                                user_id,
                                &Perm::ViewPortfolios,
                            )
                        })
                })
                .cloned()
                .collect::<Vec<_>>()
        },
        pf_list_changed,
    );

    let sorted_portfolios = Memo::new_with_compare(
        move |_| {
            let ui = ui_store.get();
            let sort = if ui.sort_ascending {
                ui.reversed_sort_mode()
            } else {
                ui.portfolio_sort_mode.clone()
            };
            let mut items: Vec<_> = filtered_portfolios.get().into_iter().collect();
            items.sort_by(|a, b| match sort {
                SortMode::Recent => b.created_at.cmp(&a.created_at),
                SortMode::Oldest => a.created_at.cmp(&b.created_at),
                SortMode::HighestValue => b
                    .total_value
                    .partial_cmp(&a.total_value)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortMode::LowestValue => a
                    .total_value
                    .partial_cmp(&b.total_value)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortMode::HighestProfit => b
                    .profit_loss
                    .partial_cmp(&a.profit_loss)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortMode::LowestProfit => a
                    .profit_loss
                    .partial_cmp(&b.profit_loss)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortMode::HighestRevenue => b
                    .revenue
                    .partial_cmp(&a.revenue)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortMode::LowestRevenue => a
                    .revenue
                    .partial_cmp(&b.revenue)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortMode::ByOrganization => b.organization_id.cmp(&a.organization_id),
            });
            items
        },
        pf_list_changed,
    );

    let view_mode = Memo::new(move |_| ui_store.get().portfolio_view_mode.clone());
    let selected_ids = move || app_store.get().selected_portfolio_ids;
    let (expanded_orgs, set_expanded_orgs) = signal(HashSet::<Uuid>::new());

    let grouped_portfolios = Memo::new(move |_| {
        let orgs = organization_store.get().organizations.clone();
        let mut groups: Vec<PortfolioGroup> = Vec::new();
        let mut org_index: HashMap<Uuid, usize> = HashMap::new();
        for (idx, p) in sorted_portfolios.get().into_iter().enumerate() {
            if let Some(oid) = p.organization_id {
                if let Some(&group_idx) = org_index.get(&oid) {
                    groups[group_idx].portfolios.push(p);
                } else if let Some(org) = orgs.iter().find(|o| o.id == oid).cloned() {
                    org_index.insert(oid, groups.len());
                    groups.push(PortfolioGroup {
                        key: org.id,
                        first_index: idx,
                        organization: Some(org),
                        portfolios: vec![p],
                    });
                } else {
                    groups.push(PortfolioGroup {
                        key: p.id,
                        first_index: idx,
                        organization: None,
                        portfolios: vec![p],
                    });
                }
            } else {
                groups.push(PortfolioGroup {
                    key: p.id,
                    first_index: idx,
                    organization: None,
                    portfolios: vec![p],
                });
            }
        }
        groups.sort_by(|a, b| {
            let a_is_org = a.organization.is_some();
            let b_is_org = b.organization.is_some();
            if a_is_org && !b_is_org {
                std::cmp::Ordering::Less
            } else if !a_is_org && b_is_org {
                std::cmp::Ordering::Greater
            } else if let (Some(ao), Some(bo)) = (&a.organization, &b.organization) {
                ao.name
                    .cmp(&bo.name)
                    .then(a.first_index.cmp(&b.first_index))
            } else {
                a.first_index.cmp(&b.first_index)
            }
        });
        groups
    });
    let edit_mode = use_tab_edit_mode();
    let _ = edit_mode;
    let can_edit = move |org_id: Option<Uuid>| {
        let store = app_store.get();
        let role = org_id
            .map(|id| {
                organization_store.get().current_user_role_in_org(
                    id,
                    store.current_user.id,
                    store.current_user.role.clone(),
                )
            })
            .unwrap_or_else(|| store.current_user.role.clone());
        role == UserRole::Owner || role == UserRole::Manager
    };

    let can_edit_documents = move |org_id: Option<Uuid>| {
        let store = app_store.get();
        let role = org_id
            .map(|id| {
                organization_store.get().current_user_role_in_org(
                    id,
                    store.current_user.id,
                    store.current_user.role.clone(),
                )
            })
            .unwrap_or_else(|| store.current_user.role.clone());
        let mut user = store.current_user.clone();
        user.role = role;
        user.can_upload_documents()
    };

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
    let (pf_image_upload_target, set_pf_image_upload_target) = signal(Option::<Uuid>::None);
    let pf_image_input_ref = NodeRef::<leptos::html::Input>::new();
    let (new_asset_name, set_new_asset_name) = signal(String::new());
    let (new_asset_type, set_new_asset_type) = signal(AssetType::RealEstate);
    let (new_asset_value, set_new_asset_value) = signal(String::new());
    let (new_channel_name, set_new_channel_name) = signal(String::new());
    let (new_channel_rate, set_new_channel_rate) = signal(String::new());

    // Context menu modal signals (Add Role, Add Organization)
    let (show_pf_add_role, set_show_pf_add_role) = signal(Option::<Uuid>::None);
    let (show_pf_add_org, set_show_pf_add_org) = signal(Option::<Uuid>::None);
    let (pf_new_role_name, set_pf_new_role_name) = signal(String::new());
    let (confirm_pf_remove, set_confirm_pf_remove) = signal(Option::<Uuid>::None);
    let (pf_new_role_desc, set_pf_new_role_desc) = signal(String::new());
    let (pf_new_org_name, set_pf_new_org_name) = signal(String::new());
    let (pf_org_parent_id, set_pf_org_parent_id) = signal(String::new());
    let (pf_secondary_org_ids, set_pf_secondary_org_ids) = signal(Vec::<Uuid>::new());

    // Consume pending navigation from notification clicks — expand portfolio and open doc modal
    Effect::new(move |_| {
        if let Some(nav) = app_store.get().pending_nav_target {
            let pid = nav.portfolio_id;
            let doc_id = nav.doc_id;
            let gid = nav.group_id;
            let aid = nav.asset_id;
            app_store.update(|s| {
                s.selected_portfolio_ids.insert(pid);
                s.touch_portfolio(pid);
                s.pending_nav_target = None;
            });
            // Open doc modal for the entity that contains the document
            ui_store.update(|ui| {
                if let Some(did) = doc_id {
                    let _ = did;
                    if let Some(aid) = aid {
                        ui.open_doc_modal(aid);
                    } else if let Some(gid) = gid {
                        ui.open_doc_modal(gid);
                    } else {
                        ui.open_doc_modal(pid);
                    }
                }
            });
        }
    });

    let on_toggle_view = move |id: Uuid| {
        app_store.update(|s| {
            if s.selected_portfolio_ids.contains(&id) {
                s.selected_portfolio_ids.remove(&id);
            } else {
                s.selected_portfolio_ids.insert(id);
                s.touch_portfolio(id);
            }
        });
    };

    let on_delete_portfolio = move |id: Uuid| {
        app_store.update(|s| {
            s.remove_portfolio(id);
            s.selected_portfolio_ids.remove(&id);
        });
        set_edit_portfolio_id.set(None);
    };

    let _on_open_edit = move |id: Uuid| {
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

    let on_add_asset = Callback::new(move |target: AssetTarget| -> Option<Uuid> {
        let name = new_asset_name.get();
        if name.trim().is_empty() {
            return None;
        }
        let value: f64 = new_asset_value.get().parse().unwrap_or(0.0);
        let uploaded_by = app_store.get().current_user.id;
        let asset = create_mock_asset(&name, new_asset_type.get(), value, value, uploaded_by);
        let asset_id = asset.id;
        app_store.update(|s| match target {
            AssetTarget::PortfolioDirect(pid) => {
                if let Some(p) = s.get_portfolio_mut(pid) {
                    p.assets.push(asset);
                    p.recalculate_values();
                }
            }
            AssetTarget::Group(pid, gid) => {
                if let Some(p) = s.get_portfolio_mut(pid) {
                    if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                        g.assets.push(asset);
                        g.recalculate_values();
                    }
                    p.recalculate_values();
                }
            }
            AssetTarget::None => {}
        });
        set_new_asset_name.set(String::new());
        set_new_asset_value.set(String::new());
        set_show_add_asset.set(AssetTarget::default());
        Some(asset_id)
    });

    let on_top_add_group = move |_| {
        let pid = top_add_group_pid.get();
        if pid.is_none() {
            return;
        }
        on_add_group.run(pid.unwrap());
        ui_store.update(|s| s.show_top_add_group = false);
        set_top_add_group_pid.set(None);
    };

    let on_top_add_asset = move |_| {
        let pid = top_add_asset_pid.get();
        if pid.is_none() {
            return;
        }
        let pid = pid.unwrap();
        let target = match top_add_asset_gid.get() {
            Some(gid) => AssetTarget::Group(pid, gid),
            None => AssetTarget::PortfolioDirect(pid),
        };
        if let Some(asset_id) = on_add_asset.run(target) {
            let name = new_channel_name.get();
            if !name.trim().is_empty() {
                let rate = new_channel_rate.get().parse::<f64>().ok();
                let mut channel = Channel::new_test_channel(name, Some(asset_id), Some(pid));
                channel.nightly_rate_override = rate;
                app_store.update(|s| s.add_channel(channel));
            }
        }
        ui_store.update(|s| s.show_top_add_asset = false);
        set_top_add_asset_pid.set(None);
        set_top_add_asset_gid.set(None);
        set_new_channel_name.set(String::new());
        set_new_channel_rate.set(String::new());
    };

    let _selected_portfolio = move || {
        let ids = selected_ids();
        filtered_portfolios
            .get()
            .into_iter()
            .find(|p| ids.contains(&p.id))
    };

    let render_portfolio = move |portfolio: Portfolio| -> AnyView {
        let portfolio_id = portfolio.id;
        let org_id = portfolio.organization_id;
        let is_expanded = Memo::new(move |_| selected_ids().contains(&portfolio_id));
        let can = Memo::new(move |_| can_edit(org_id));
        let can_docs = Memo::new(move |_| can_edit_documents(org_id));
        view! {
            <PortfolioListItem
                portfolio={portfolio}
                can_edit={can}
                can_edit_documents={can_docs}
                expanded={is_expanded}
                on_toggle=Callback::new(move |_| on_toggle_view(portfolio_id))
                on_context=move |ev: leptos::ev::MouseEvent| {
                    ev.prevent_default();
                    ev.stop_propagation();
                    set_context_menu.set(Some((portfolio_id, ev.client_x(), ev.client_y())));
                }
                show_add_group={show_add_group}
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
                view_mode={view_mode}
            />
        }
        .into_any()
    };

    view! {
        <div class="home-screen home-screen-pf">
            // Portfolio controls bar (attached below navbar)
            <div class="portfolio-controls-bar">
                <button
                    class="nav-portfolio-btn"
                    class:active={move || view_mode.get() == ViewMode::List}
                    on:click=move |_| ui_store.update(|s| s.portfolio_view_mode = ViewMode::List)
                >
                    "☰ List"
                </button>
                <button
                    class="nav-portfolio-btn"
                    class:active={move || view_mode.get() == ViewMode::Grid}
                    on:click=move |_| ui_store.update(|s| s.portfolio_view_mode = ViewMode::Grid)
                >
                    "⊞ Grid"
                </button>
                <select
                    class="nav-portfolio-btn nav-portfolio-sort"
                    aria-label="Sort portfolios"
                    prop:value={move || {
                        match ui_store.get().portfolio_sort_mode {
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
                        ui_store.update(|s| s.portfolio_sort_mode = mode);
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
                    aria-label={move || if ui_store.get().sort_ascending { "Sort ascending" } else { "Sort descending" }}
                    title={move || if ui_store.get().sort_ascending { "Ascending ↑" } else { "Descending ↓" }}
                    on:click=move |_| ui_store.update(|s| s.toggle_sort_direction())
                >
                    {move || if ui_store.get().sort_ascending { "↑" } else { "↓" }}
                </button>
                <select
                    class="nav-portfolio-btn nav-portfolio-sort nav-portfolio-view"
                    aria-label="View amount"
                    prop:value={move || {
                        match ui_store.get().portfolio_view_count(view_mode.get()) {
                            ViewCount::V1 => "view_1",
                            ViewCount::V10 => "view_10",
                            ViewCount::V20 => "view_20",
                            ViewCount::V50 => "view_50",
                            ViewCount::V100 => "view_100",
                            ViewCount::Custom(_) => "view_custom",
                        }.to_string()
                    }}
                    on:change=move |ev| {
                        let v = event_target_value(&ev);
                        let vc = match v.as_str() {
                            "view_1" => ViewCount::V1,
                            "view_10" => ViewCount::V10,
                            "view_20" => ViewCount::V20,
                            "view_50" => ViewCount::V50,
                            "view_100" => ViewCount::V100,
                            "view_custom" => ViewCount::Custom(10),
                            _ => ViewCount::V10,
                        };
                        let mode = view_mode.get();
                        ui_store.update(|s| s.set_portfolio_view_count(mode, vc));
                    }
                >
                    <option value="view_1">"View: 1"</option>
                    <option value="view_10">"View: 10"</option>
                    <option value="view_20">"View: 20"</option>
                    <option value="view_50">"View: 50"</option>
                    <option value="view_100">"View: 100"</option>
                    <option value="view_custom">"..."</option>
                </select>
                {move || if matches!(ui_store.get().portfolio_view_count(view_mode.get()), ViewCount::Custom(_)) {
                    view! {
                        <input
                            class="nav-portfolio-view-input"
                            type="number"
                            min="1"
                            step="1"
                            aria-label="Custom view count"
                            prop:value={move || match ui_store.get().portfolio_view_count(view_mode.get()) {
                                ViewCount::Custom(n) => n.to_string(),
                                _ => "10".to_string(),
                            }}
                            on:input=move |ev| {
                                let val = event_target_value(&ev);
                                if let Ok(n) = val.parse::<usize>() {
                                    let n = n.max(1);
                                    let mode = view_mode.get();
                                    ui_store.update(|s| s.set_portfolio_view_count(mode, ViewCount::Custom(n)));
                                }
                            }
                        />
                    }.into_any()
                } else { ().into_any() }}
            </div>

            // Hidden portfolio image uploader
            <input
                type="file"
                accept="image/*"
                style="display:none"
                node_ref=pf_image_input_ref
                on:change=move |ev| {
                    read_image_as_data_url(&ev, {
                        move |url| {
                            if let Some(pid) = pf_image_upload_target.get() {
                                app_store.update(|s| {
                                    if let Some(p) = s.get_portfolio_mut(pid) {
                                        p.image_url = Some(url);
                                        p.updated_at = chrono::Utc::now();
                                    }
                                });
                            }
                            set_pf_image_upload_target.set(None);
                        }
                    });
                }
            />

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
                                <button class="modal-close" aria-label="Close edit portfolio assets" on:click=move |_| on_close_edit(())>"×"</button>
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

            // Top-level Add Group Form (toggled from navbar)
            {move || ui_store.get().show_top_add_group.then(|| view! {
                <div class="add-form">
                    <select
                        class="login-input"
                        aria-label="Select portfolio"
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
                        <For
                            each=move || filtered_portfolios.get()
                            key=|portfolio| portfolio.id
                            children=move |portfolio| view! {
                                <option value={portfolio.id.to_string()}>{portfolio.name.clone()}</option>
                            }
                        />
                    </select>
                    <input
                        class="login-input"
                        type="text"
                        placeholder="Group name"
                        aria-label="Group name"
                        on:input=move |ev| set_new_group_name.set(event_target_value(&ev))
                    />
                    <button class="login-btn" on:click=on_top_add_group>"Create Group"</button>
                </div>
            })}

            // Top-level Add Asset Form (toggled from navbar)
            {move || ui_store.get().show_top_add_asset.then(|| view! {
                <div class="add-form">
                    <select
                        class="login-input"
                        aria-label="Select portfolio"
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
                        <For
                            each=move || filtered_portfolios.get()
                            key=|portfolio| portfolio.id
                            children=move |portfolio| view! {
                                <option value={portfolio.id.to_string()}>{portfolio.name.clone()}</option>
                            }
                        />
                    </select>
                    {move || {
                        let pid = top_add_asset_pid.get();
                        if pid.is_none() { return ().into_any(); }
                        let pid = pid.unwrap();
                        let groups = filtered_portfolios.get().into_iter().find(|p| p.id == pid).map(|p| p.asset_groups).unwrap_or_default();
                        view! {
                            <select
                                class="login-input"
                                aria-label="Select group"
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
                                <For
                                    each=move || groups.clone()
                                    key=|group| group.id
                                    children=move |group| view! {
                                        <option value={group.id.to_string()}>{group.name.clone()}</option>
                                    }
                                />
                            </select>
                        }.into_any()
                    }}
                    <input
                        class="login-input"
                        type="text"
                        placeholder="Asset name"
                        aria-label="Asset name"
                        on:input=move |ev| set_new_asset_name.set(event_target_value(&ev))
                    />
                    <input
                        class="login-input"
                        type="text"
                        placeholder="Value"
                        aria-label="Value"
                        on:input=move |ev| set_new_asset_value.set(event_target_value(&ev))
                    />
                    <input class="login-input" type="text" list="asset-type-options-top" placeholder="Asset type"
                        aria-label="Asset type"
                        prop:value={move || new_asset_type.get().to_input_string()}
                        on:input=move |ev| set_new_asset_type.set(AssetType::from_input(&event_target_value(&ev))) />
                    <datalist id="asset-type-options-top">
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
                    <input class="login-input" type="text" placeholder="Channel name (optional)"
                        aria-label="Channel name"
                        prop:value={move || new_channel_name.get()}
                        on:input=move |ev| set_new_channel_name.set(event_target_value(&ev)) />
                    <input class="login-input" type="number" placeholder="Channel nightly rate (optional)"
                        aria-label="Channel nightly rate"
                        prop:value={move || new_channel_rate.get()}
                        on:input=move |ev| set_new_channel_rate.set(event_target_value(&ev)) />
                    <button class="login-btn" on:click=on_top_add_asset>"Create Asset"</button>
                </div>
            })}

            // Portfolios List
            <div class={move || {
                if view_mode.get() == ViewMode::Grid {
                    format!("grid-view grid-cols-{}", ui_store.get().portfolio_grid_columns)
                } else { "pf-accordion".to_string() }
            }}
            on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                ev.prevent_default();
                set_context_menu.set(Some((Uuid::new_v4(), ev.client_x(), ev.client_y())));
            }>
                {move || if grouped_portfolios.get().is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-state-icon">"📊"</div>
                            <div class="empty-state-title">"No portfolios yet"</div>
                            <div class="empty-state-desc">"Create your first portfolio to get started"</div>
                            <button class="login-btn" on:click=move |_| ui_store.update(|s| s.show_add_portfolio = !s.show_add_portfolio)>"Create Portfolio"</button>
                            {move || ui_store.get().show_add_portfolio.then(|| view! {
                                <AddPortfolioModal />
                            })}
                        </div>
                    }.into_any()
                } else if view_mode.get() == ViewMode::Grid {
                    view! {
                        <For
                            each=move || sorted_portfolios.get()
                            key=|portfolio| portfolio.id
                            children=render_portfolio
                        />
                    }.into_any()
                } else {
                    view! {
                        <For
                            each=move || grouped_portfolios.get()
                            key=|group| group.key
                            children=move |group: PortfolioGroup| {
                                let PortfolioGroup { organization, portfolios, .. } = group;
                                if let Some(org) = organization {
                                    let org_id = org.id;
                                    let org_name = org.name.clone();
                                    let org_name_for_info = org_name.clone();
                                    let org_desc = org.description.clone().unwrap_or_default();
                                    let org_color = org.settings.color.clone();
                                    let org_image_url = org.image_url.clone();
                                    let org_image_ref = NodeRef::<leptos::html::Input>::new();
                                    let (is_editing_org_name, set_is_editing_org_name) = signal(false);
                                    let (is_editing_org_desc, set_is_editing_org_desc) = signal(false);
                                    let (edit_org_name, set_edit_org_name) = signal(org.name.clone());
                                    let (edit_org_desc, set_edit_org_desc) = signal(org.description.clone().unwrap_or_default());
                                    let can_edit_org = move || can_edit(Some(org_id));
                                    let dropdown_entries = Memo::new(move |_| {
                                        organization_store
                                            .get()
                                            .get_organization(org_id)
                                            .map(|o| {
                                                o.documents
                                                    .iter()
                                                    .map(|d| DocEntry {
                                                        doc: d.clone(),
                                                        portfolio_id: None,
                                                        group_id: None,
                                                        asset_id: None,
                                                        organization_id: Some(org_id),
                                                    })
                                                    .collect::<Vec<_>>()
                                            })
                                            .unwrap_or_default()
                                    });
                                    let org_can_edit_docs = move || can_edit_documents(Some(org_id));
                                    let org_can_edit_docs_signal = Signal::derive(move || org_can_edit_docs());
                                    let current_user_id = app_store.get().current_user.id;
                                    let org_can_view_docs = Memo::new(move |_| {
                                        organization_store.get().user_has_perm_in_org(
                                            org_id,
                                            current_user_id,
                                            &Perm::ViewDocuments,
                                        )
                                    });
                                    let add_org_doc = {
                                        let app_store = app_store;
                                        let organization_store = organization_store;
                                        let notification_store = notification_store;
                                        move |n: String| {
                                            if n.trim().is_empty() {
                                                return;
                                            }
                                            let ft = detect_file_type(&n);
                                            let doc = Document {
                                                id: Uuid::new_v4(),
                                                name: n.clone(),
                                                file_type: ft,
                                                url: String::new(),
                                                uploaded_at: chrono::Utc::now(),
                                                uploaded_by: app_store.get().current_user.id,
                                                content: None,
                                            };
                                            let doc_id = doc.id;
                                            let uploader = app_store.get().current_user.name.clone();
                                            organization_store.update(|s| {
                                                if let Some(o) = s.get_organization_mut(org_id) {
                                                    o.documents.push(doc);
                                                    o.updated_at = chrono::Utc::now();
                                                }
                                            });
                                            notification_store.update(|ns| {
                                                ns.add_document_notification(
                                                    doc_id,
                                                    &n,
                                                    &uploader,
                                                    &format!("Document \"{}\" added to organization — pending review.", n),
                                                    crate::stores::NotificationType::Info,
                                                    None,
                                                    Some(uploader.clone()),
                                                    None,
                                                    None,
                                                    None,
                                                );
                                            });
                                        }
                                    };
                                    let save_org_edit = move || {
                                        let n = edit_org_name.get();
                                        let d = edit_org_desc.get();
                                        if n.trim().is_empty() {
                                            return;
                                        }
                                        organization_store.update(|s| {
                                            if let Some(o) = s.get_organization_mut(org_id) {
                                                o.name = n;
                                                o.description = if d.trim().is_empty() { None } else { Some(d) };
                                                o.updated_at = chrono::Utc::now();
                                            }
                                        });
                                        set_is_editing_org_name.set(false);
                                        set_is_editing_org_desc.set(false);
                                    };
                                    let toggle_org = move || {
                                        set_expanded_orgs.update(|s| {
                                            if s.contains(&org_id) {
                                                s.remove(&org_id);
                                            } else {
                                                s.insert(org_id);
                                            }
                                        });
                                    };
                                    let (org_name_click, org_name_dblclick) = name_click_handlers(
                                        toggle_org.clone(),
                                        move || if can_edit_org() { set_is_editing_org_name.set(true); },
                                    );
                                    let (org_desc_click, org_desc_dblclick) = name_click_handlers(
                                        toggle_org,
                                        move || if can_edit_org() { set_is_editing_org_desc.set(true); },
                                    );
                                    let is_expanded = move || expanded_orgs.get().contains(&org_id);
                                    let (show_doc_slider, set_show_doc_slider) = signal(false);
                                    let org_name_for_slider = StoredValue::new(org.name.clone());
                                    view! {
                                        <div class="pf-org-group" class:expanded={move || is_expanded()}>
                                            <div class="asset-group-header pf-org-group-header pf-accordion-header" role="button" tabindex="0"
                                                aria-expanded={move || is_expanded()}
                                                on:click=move |ev: leptos::ev::MouseEvent| {
                                                    ev.stop_propagation();
                                                    if !is_editing_org_name.get() && !is_editing_org_desc.get() {
                                                        set_expanded_orgs.update(|s| { if s.contains(&org_id) { s.remove(&org_id); } else { s.insert(org_id); }});
                                                    }
                                                }
                                                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                                    if ev.key() == "Enter" || ev.key() == " " {
                                                        ev.prevent_default();
                                                        ev.stop_propagation();
                                                        if !is_editing_org_name.get() && !is_editing_org_desc.get() {
                                                            set_expanded_orgs.update(|s| { if s.contains(&org_id) { s.remove(&org_id); } else { s.insert(org_id); }});
                                                        }
                                                    }
                                                }
                                            >
                                                <span class="asset-group-arrow">{move || if is_expanded() { "▼" } else { "▶" }}</span>
                                                <input
                                                    type="file"
                                                    accept="image/*"
                                                    class="pf-hidden-file-input"
                                                    node_ref=org_image_ref
                                                    on:change=move |ev| {
                                                        read_image_as_data_url(&ev, {
                                                            let organization_store = organization_store;
                                                            move |url: String| {
                                                                organization_store.update(|s| {
                                                                    if let Some(o) = s.get_organization_mut(org_id) {
                                                                        o.image_url = Some(url);
                                                                        o.updated_at = chrono::Utc::now();
                                                                    }
                                                                });
                                                            }
                                                        });
                                                    }
                                                />
                                                <div class="asset-group-icon"
                                                    on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                                                        if can_edit(Some(org_id)) {
                                                            ev.prevent_default();
                                                            ev.stop_propagation();
                                                            if let Some(input) = org_image_ref.get() {
                                                                let _ = input.click();
                                                            }
                                                        }
                                                    }
                                                >
                                                    {if let Some(ref url) = org_image_url {
                                                        view! { <img class="pf-header-image" src={url.clone()} alt={format!("{} logo", org_name)} /> }.into_any()
                                                    } else {
                                                        org_color.map(|c| view! { <span class="pf-org-color-tag" style={format!("background: {}", c)} aria-hidden="true"></span> }).unwrap_or_else(|| view! { <span class="pf-org-color-tag" style={String::new()} aria-hidden="true"></span> }).into_any()
                                                    }}
                                                </div>
                                                <div class="asset-group-info-wrap">
                                                    {move || {
                                                        let mut parts: Vec<leptos::prelude::AnyView> = Vec::new();
                                                        if is_editing_org_name.get() && can_edit_org() {
                                                            parts.push(view! {
                                                                <input class="pf-edit-input" placeholder="Organization name"
                                                                    prop:value=move || edit_org_name.get()
                                                                    on:input=move |ev| set_edit_org_name.set(event_target_value(&ev))
                                                                    on:blur=move |_| save_org_edit()
                                                                    on:keydown=move |ev| { if ev.key() == "Enter" { save_org_edit(); } }
                                                                />
                                                            }.into_any());
                                                        } else {
                                                            parts.push(view! {
                                                                <div class="asset-group-name"
                                                                    on:click={org_name_click.clone()}
                                                                    on:dblclick={org_name_dblclick.clone()}
                                                                >{org_name_for_info.clone()}</div>
                                                            }.into_any());
                                                        }
                                                        if is_editing_org_desc.get() && can_edit_org() {
                                                            parts.push(view! {
                                                                <input class="pf-edit-input" placeholder="Description"
                                                                    prop:value=move || edit_org_desc.get()
                                                                    on:input=move |ev| set_edit_org_desc.set(event_target_value(&ev))
                                                                    on:blur=move |_| save_org_edit()
                                                                    on:keydown=move |ev| { if ev.key() == "Enter" { save_org_edit(); } }
                                                                />
                                                            }.into_any());
                                                        } else if !org_desc.is_empty() {
                                                            parts.push(view! {
                                                                <div class="asset-group-desc"
                                                                    on:click={org_desc_click.clone()}
                                                                    on:dblclick={org_desc_dblclick.clone()}
                                                                >{org_desc.clone()}</div>
                                                            }.into_any());
                                                        }
                                                        parts.into_iter().collect_view().into_any()
                                                    }}
                                                </div>
                                                {move || if org_can_view_docs.get() {
                                                    view! {
                                                        <div class="pf-list-actions" on:click=|ev| ev.stop_propagation()>
                                                            <button
                                                                class="pf-action-btn pf-doc-action-btn"
                                                                class:active=move || show_doc_slider.get()
                                                                aria-label={move || {
                                                                    let count = dropdown_entries.get().len();
                                                                    format!("View documents for {}. {} document{}", org_name_for_slider.get_value(), count, if count == 1 { "" } else { "s" })
                                                                }}
                                                                on:click=move |_| set_show_doc_slider.update(|v| *v = !*v)
                                                            >
                                                                <div class="pf-action-stack">
                                                                    <span class="pf-action-icon">"📄"</span>
                                                                    <span class="pf-action-count">{move || dropdown_entries.get().len()}</span>
                                                                </div>
                                                            </button>
                                                        </div>
                                                    }.into_any()
                                                } else { ().into_any() }}
                                            </div>

                                            {move || if show_doc_slider.get() && org_can_view_docs.get() {
                                                view! {
                                                    <div class="pf-doc-slider-panel" on:click=|ev| ev.stop_propagation()>
                                                        <DocSlider
                                                            entity_id={org_id}
                                                            title={org_name_for_slider.get_value()}
                                                            entity_name={org_name_for_slider.get_value()}
                                                            can_edit_documents={org_can_edit_docs_signal}
                                                            entries={dropdown_entries}
                                                            on_add={Some(Callback::new(add_org_doc))}
                                                            organization_id={Some(org_id)}
                                                        />
                                                    </div>
                                                }.into_any()
                                            } else { ().into_any() }}

                                            <div class="pf-org-group-content" class:hidden={move || !is_expanded()}>
                                                <For
                                                    each=move || portfolios.clone()
                                                    key=|p| p.id
                                                    children=render_portfolio
                                                />
                                            </div>
                                        </div>
                                    }.into_any()
                                } else if let Some(p) = portfolios.into_iter().next() {
                                    render_portfolio(p)
                                } else {
                                    ().into_any()
                                }
                            }
                        />
                    }.into_any()
                }}
                    <button
                        class="pf-empty-outline org-empty-outline"
                        type="button"
                        aria-label="Add a new portfolio"
                        on:click=move |_| ui_store.update(|s| s.show_add_portfolio = !s.show_add_portfolio)
                        on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                            ev.prevent_default();
                            ev.stop_propagation();
                            ui_store.update(|s| s.show_add_portfolio = true);
                        }
                    >
                        <span class="org-empty-outline-icon">"+"</span>
                        <span class="org-empty-outline-text">"Add Portfolio"</span>
                    </button>
                    {move || ui_store.get().show_add_portfolio.then(|| view! {
                        <AddPortfolioModal />
                    })}
            </div>

            // Context menu for portfolio press-and-hold or whitespace
            {move || context_menu.get().map(|(pid, x, y)| {
                let is_whitespace = app_store.get().portfolios.iter().find(|p| p.id == pid).is_none();
                let pid_doc = pid;
                let pid_role = pid;
                let pid_org = pid;
                let pid_remove = pid;
                let org_id = app_store.get().portfolios.iter().find(|p| p.id == pid).and_then(|p| p.organization_id);
                let can = can_edit(org_id);
                let content = if is_whitespace {
                    view! {
                        <button
                            class="context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                ui_store.update(|s| s.show_add_portfolio = true);
                            }
                        >
                            "➕ Create Portfolio"
                        </button>
                        <button
                            class="context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                ui_store.update(|s| s.show_top_add_group = true);
                            }
                        >
                            "➕ Create Asset Group"
                        </button>
                        <button
                            class="context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                ui_store.update(|s| s.show_top_add_asset = true);
                            }
                        >
                            "➕ Create Asset"
                        </button>
                    }.into_any()
                } else {
                    let admin_items = if can {
                        view! {
                            <button
                                class="context-menu-item"
                                on:click=move |_| {
                                    set_context_menu.set(None);
                                    app_store.update(|s| {
                                        s.selected_portfolio_ids.insert(pid);
                                    });
                                    set_show_add_group.set(Some(pid));
                                }
                            >
                                "➕ Add Asset Group"
                            </button>
                            <button
                                class="context-menu-item"
                                on:click=move |_| {
                                    set_context_menu.set(None);
                                    app_store.update(|s| {
                                        s.selected_portfolio_ids.insert(pid);
                                    });
                                    set_show_add_asset.set(AssetTarget::PortfolioDirect(pid));
                                }
                            >
                                "➕ Add Asset"
                            </button>
                            <button
                                class="context-menu-item"
                                on:click=move |_| {
                                    set_context_menu.set(None);
                                    set_pf_image_upload_target.set(Some(pid_doc));
                                    if let Some(input) = pf_image_input_ref.get() {
                                        let _ = input.click();
                                    }
                                }
                            >
                                "🖼 Add Image"
                            </button>
                            <button
                                class="context-menu-item"
                                on:click=move |_| {
                                    set_context_menu.set(None);
                                    ui_store.update(|s| s.open_doc_modal(pid_doc));
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
                                    app_store.with(|s| {
                                        if let Some(p) = s.get_portfolio(pid_org) {
                                            set_pf_org_parent_id.set(p.organization_id.map(|id| id.to_string()).unwrap_or_default());
                                            set_pf_secondary_org_ids.set(p.secondary_organization_ids.clone());
                                        }
                                    });
                                    set_show_pf_add_org.set(Some(pid_org));
                                }
                            >
                                "🏢 Add to Organization"
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
                    } else {
                        ().into_any()
                    };
                    view! {
                        <button
                            class="context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                on_toggle_view(pid);
                            }
                        >
                            "Overview"
                        </button>
                        {admin_items}
                    }.into_any()
                };
                view! {
                    <div
                        class="context-menu-overlay"
                        on:click=move |_| set_context_menu.set(None)
                    >
                        <div
                            class="context-menu"
                            style={format!("left: {}px; top: {}px;", x, y)}
                        >
                            {content}
                        </div>
                    </div>
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
                                <button class="doc-modal-close" aria-label="Close add role" on:click=move |_| set_show_pf_add_role.set(None)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="Role name"
                                    aria-label="Role name"
                                    prop:value={move || pf_new_role_name.get()}
                                    on:input=move |ev| set_pf_new_role_name.set(event_target_value(&ev)) />
                                <input class="login-input" type="text" placeholder="Description"
                                    aria-label="Description"
                                    prop:value={move || pf_new_role_desc.get()}
                                    on:input=move |ev| set_pf_new_role_desc.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| {
                                    let name = pf_new_role_name.get();
                                    let desc = pf_new_role_desc.get();
                                    if !name.trim().is_empty() {
                                        let role = crate::models::OrgRole::new(name, 0, desc, vec![]);
                                        if let Some(oid) = org_id {
                                            organization_store.update(|s| s.add_role_to_org(oid, role));
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

            // Add to Organization modal (portfolio context menu)
            {move || show_pf_add_org.get().map(|pid| {
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_pf_add_org.set(None)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add to Organization"</span>
                                <button class="doc-modal-close" aria-label="Close add organization" on:click=move |_| set_show_pf_add_org.set(None)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <label class="list-item-title">"Parent (primary) organization"</label>
                                <select
                                    class="form-select"
                                    aria-label="Parent organization"
                                    prop:value={move || pf_org_parent_id.get()}
                                    on:change=move |ev| set_pf_org_parent_id.set(event_target_value(&ev))
                                >
                                    <option value="">"(None)"</option>
                                    {move || organization_store.get().organizations.iter().map(|o| {
                                        let id = o.id.to_string();
                                        view! { <option value={id.clone()}>{o.name.clone()}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                                <input class="login-input" type="text" placeholder="Or create a new organization"
                                    aria-label="New organization name"
                                    prop:value={move || pf_new_org_name.get()}
                                    on:input=move |ev| set_pf_new_org_name.set(event_target_value(&ev)) />
                                <label class="list-item-title">"Secondary / tertiary organizations"</label>
                                {move || organization_store.get().organizations.iter().map(|o| {
                                    let oid = o.id;
                                    let name = o.name.clone();
                                    view! {
                                        <label class="list-item">
                                            <input
                                                type="checkbox"
                                                prop:checked={move || pf_secondary_org_ids.get().contains(&oid)}
                                                on:change=move |ev| {
                                                    let checked = event_target_checked(&ev);
                                                    set_pf_secondary_org_ids.update(|v| {
                                                        if checked {
                                                            if !v.contains(&oid) { v.push(oid); }
                                                        } else {
                                                            v.retain(|&id| id != oid);
                                                        }
                                                    });
                                                }
                                            />
                                            {name}
                                        </label>
                                    }
                                }).collect::<Vec<_>>()}
                                <button class="login-btn" on:click=move |_| {
                                    let mut name = pf_new_org_name.get();
                                    name = name.trim().to_string();
                                    let parent_id = if name.is_empty() {
                                        let s = pf_org_parent_id.get();
                                        if s.trim().is_empty() { None } else { Uuid::parse_str(&s).ok() }
                                    } else {
                                        let owner_id = app_store.get().current_user.id;
                                        let org = crate::models::Organization::new(name, owner_id);
                                        let oid = org.id;
                                        organization_store.update(|s| s.add_organization(org));
                                        Some(oid)
                                    };
                                    app_store.update(|s| {
                                        if let Some(p) = s.get_portfolio_mut(pid) {
                                            p.organization_id = parent_id;
                                            p.secondary_organization_ids = pf_secondary_org_ids.get();
                                        }
                                    });
                                    set_pf_new_org_name.set(String::new());
                                    set_pf_org_parent_id.set(String::new());
                                    set_pf_secondary_org_ids.set(Vec::new());
                                    set_show_pf_add_org.set(None);
                                }>"Save Organizations"</button>
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
                                <button class="doc-modal-close" aria-label={format!("Cancel removal of {} portfolio", pf_name)} on:click=move |_| set_confirm_pf_remove.set(None)>"✕"</button>
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
fn create_mock_asset(
    name: &str,
    asset_type: AssetType,
    purchase: f64,
    current: f64,
    uploaded_by: Uuid,
) -> Asset {
    let mut asset = Asset::new(name.to_string(), asset_type.clone(), purchase);
    asset.update_value(current);
    asset.description =
        Some("Open Rose Rental Duplex 112, Open Rose Court, Coolangatta, QLD, 4269.".to_string());
    asset.location = Some("Coolangatta, QLD, 4269".to_string());
    asset.lifecycle_status = LifecycleStatus::Active;
    asset.availability_status = AvailabilityStatus::Available;
    asset.condition_status = ConditionStatus::New;
    asset.commercial_status = CommercialStatus::NotOffered;
    asset.classification.manufacturer = Some("Mock Manufacturer".to_string());
    asset.classification.model = Some("Mock Model".to_string());
    asset.classification.serial_number = Some(format!("SN-{}", Uuid::new_v4()));
    asset.lifecycle.warranty_start_date = Some(chrono::Utc::now());
    asset.lifecycle.warranty_expiry_date = Some(chrono::Utc::now() + chrono::Duration::days(365));
    asset.images = vec![asset_placeholder_url(&asset_type, name)];
    asset.documents = vec![
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
    asset.initialize_with_creator(uploaded_by, None);
    asset
}
fn create_mock_asset_group(name: &str, assets: Vec<Asset>) -> AssetGroup {
    let mut group = AssetGroup::new(name.to_string());
    group.assets = assets;
    group.recalculate_values();
    group
}
