use crate::components::tabs::use_tab_edit_mode;
use crate::models::{
    Asset, AssetGroup, AvailabilityStatus, Channel, CommercialStatus, ConditionStatus, Document,
    LifecycleStatus, Organization, Perm, Portfolio,
};
use crate::stores::{
    use_app_store, use_notification_store, use_organization_store, use_ui_store, ScrollTarget,
};
use crate::types::{AssetType, SortMode, UserRole, ViewCount, ViewMode};
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::{
    asset_placeholder_url, clear_pf_drag_kind, clamp_context_menu, detect_file_type,
    mobile_tab_gestures, name_click_handlers, nav_tab_from_drag_event, pf_drop_target,
    phantom_edit_scope, read_image_as_data_url, set_drag_tab_data, sort_portfolios_for_ui,
    AddPortfolioModal, AssetTarget, AssetViewer, DocEntry, DocSlider, NavTab, NavTabKind,
    PortfolioListItem,
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
    // Compare structure + cheap content signals, not just ids — pushing
    // an asset/group/doc into an existing portfolio changes no ids, and
    // without this the memo would keep serving the stale clone.
    old.iter().zip(new.iter()).any(|(a, b)| {
        a.id != b.id
            || a.name != b.name
            || a.position != b.position
            || a.organization_id != b.organization_id
            || a.assets.len() != b.assets.len()
            || a.asset_groups.len() != b.asset_groups.len()
            || a.documents.len() != b.documents.len()
            || a.total_value.to_bits() != b.total_value.to_bits()
            || a.purchase_value.to_bits() != b.purchase_value.to_bits()
            || a.asset_groups
                .iter()
                .zip(b.asset_groups.iter())
                .any(|(ga, gb)| {
                    ga.id != gb.id
                        || ga.name != gb.name
                        || ga.assets.len() != gb.assets.len()
                        || ga.documents.len() != gb.documents.len()
                })
    })
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
            app_store.with(|app| {
                organization_store.with(|orgs| {
                    let user = app.current_user.clone();
                    let can_view_all = user.can_view_all();
                    let user_id = user.id;
                    app.portfolios
                        .iter()
                        .filter(|p| {
                            p.organization_id.map_or(true, |oid| {
                                orgs.can_view_org_content(oid, user_id, can_view_all)
                            }) && (p.is_visible_to(user_id, can_view_all)
                                || p.organization_id.map_or(can_view_all, |oid| {
                                    orgs.user_has_perm_in_org(
                                        oid,
                                        user_id,
                                        &Perm::ViewPortfolios,
                                    )
                                }))
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                })
            })
        },
        pf_list_changed,
    );

    let sorted_portfolios = Memo::new_with_compare(
        move |_| {
            ui_store.with(|ui| sort_portfolios_for_ui(filtered_portfolios.get(), ui))
        },
        pf_list_changed,
    );

    let view_mode = Memo::new(move |_| ui_store.with(|s| s.portfolio_view_mode.clone()));
    let selected_ids = move || app_store.with(|s| s.selected_portfolio_ids.clone());
    let (expanded_orgs, set_expanded_orgs) = signal(HashSet::<Uuid>::new());

    // Drag-and-drop navigation tabs on the portfolio controls bar.
    let (nav_tabs, set_nav_tabs) = signal(Vec::<NavTab>::new());
    let (active_tab_id, set_active_tab_id) = signal(None::<Uuid>);
    let add_nav_tab = {
        let set_nav_tabs = set_nav_tabs.clone();
        let set_active = set_active_tab_id.clone();
        move |tab: NavTab| {
            let id = tab.id;
            let kind = tab.kind;
            let already = nav_tabs.with(|tabs| tabs.iter().any(|t| t.id == id && t.kind == kind));
            if !already {
                set_nav_tabs.update(|tabs| tabs.push(tab));
            } else {
                drop(tab);
            }
            set_active.set(Some(id));
        }
    };
    let remove_nav_tab = {
        let set_nav_tabs = set_nav_tabs.clone();
        let set_active = set_active_tab_id.clone();
        move |id: Uuid| {
            set_nav_tabs.update(|tabs| {
                if let Some(idx) = tabs.iter().position(|t| t.id == id) {
                    tabs.remove(idx);
                }
            });
            if active_tab_id.get() == Some(id) {
                set_active.set(None);
            }
        }
    };

    let navigate_from_tab = move |kind: NavTabKind, id: Uuid| {
        set_active_tab_id.set(Some(id));
        match kind {
            NavTabKind::Organization => {
                set_expanded_orgs.update(|s| {
                    s.insert(id);
                });
                crate::utils::dom::scroll_to_id(&format!("org-group-{}", id));
            }
            NavTabKind::Portfolio => {
                app_store.update(|s| {
                    s.touch_portfolio(id);
                    s.selected_portfolio_ids.insert(id);
                    s.pending_scroll_target = Some(ScrollTarget::Portfolio(id));
                });
            }
            NavTabKind::AssetGroup => {
                if let Some((portfolio_id, group_id)) = app_store.with(|s| {
                    s.portfolios.iter().find_map(|p| {
                        p.asset_groups
                            .iter()
                            .find(|g| g.id == id)
                            .map(|g| (p.id, g.id))
                    })
                }) {
                    app_store.update(|s| {
                        s.touch_portfolio(portfolio_id);
                        s.selected_portfolio_ids.insert(portfolio_id);
                        s.selected_asset_group_id = Some(group_id);
                        s.pending_group_expand = Some(group_id);
                        s.pending_scroll_target = Some(ScrollTarget::AssetGroup(group_id));
                    });
                }
            }
            NavTabKind::Asset => {
                if let Some((portfolio_id, group_id, asset_id)) = app_store.with(|s| {
                    s.portfolios.iter().find_map(|p| {
                        if let Some(a) = p.assets.iter().find(|a| a.id == id) {
                            Some((p.id, None, a.id))
                        } else {
                            p.asset_groups.iter().find_map(|g| {
                                g.assets
                                    .iter()
                                    .find(|a| a.id == id)
                                    .map(|a| (p.id, Some(g.id), a.id))
                            })
                        }
                    })
                }) {
                    app_store.update(|s| {
                        s.touch_portfolio(portfolio_id);
                        s.selected_portfolio_ids.insert(portfolio_id);
                        s.selected_asset_id = Some(asset_id);
                        s.pending_group_expand = group_id;
                        s.pending_asset_detail = Some(asset_id);
                        s.pending_scroll_target = Some(ScrollTarget::Asset(asset_id));
                    });
                }
            }
            NavTabKind::Document => {
                if let Some((portfolio_id, group_id, asset_id)) = app_store.with(|s| {
                    s.portfolios.iter().find_map(|p| {
                        if p.documents.iter().any(|d| d.id == id) {
                            return Some((p.id, None, None));
                        }
                        if let Some(g) = p
                            .asset_groups
                            .iter()
                            .find(|g| g.documents.iter().any(|d| d.id == id))
                        {
                            return Some((p.id, Some(g.id), None));
                        }
                        for g in &p.asset_groups {
                            if let Some(a) = g
                                .assets
                                .iter()
                                .find(|a| a.documents.iter().any(|d| d.id == id))
                            {
                                return Some((p.id, Some(g.id), Some(a.id)));
                            }
                        }
                        if let Some(a) = p
                            .assets
                            .iter()
                            .find(|a| a.documents.iter().any(|d| d.id == id))
                        {
                            return Some((p.id, None, Some(a.id)));
                        }
                        None
                    })
                }) {
                    app_store.update(|s| {
                        s.touch_portfolio(portfolio_id);
                        s.selected_portfolio_ids.insert(portfolio_id);
                        s.pending_group_expand = group_id;
                        s.selected_asset_group_id = group_id;
                        if asset_id.is_none() {
                            if let Some(gid) = group_id {
                                s.pending_doc_slider = Some(gid);
                            } else {
                                s.pending_doc_slider = Some(portfolio_id);
                            }
                        }
                        s.pending_doc_view = Some(id);
                        s.pending_scroll_target = Some(ScrollTarget::Document(id));
                    });
                }
            }
        }
    };

    // Consume pending "Open in new tab" requests from context menus / tap-hold.
    Effect::new(move |_| {
        let pending = ui_store.with(|s| s.pending_open_tab.clone());
        if let Some(pending) = pending {
            ui_store.update(|s| s.pending_open_tab = None);
            if let Some(kind) = NavTabKind::from_str(&pending.kind) {
                if matches!(
                    kind,
                    NavTabKind::Portfolio
                        | NavTabKind::AssetGroup
                        | NavTabKind::Asset
                        | NavTabKind::Document
                ) {
                    ui_store.update(|s| s.portfolio_view_mode = ViewMode::List);
                }
                let tab = NavTab {
                    kind,
                    id: pending.id,
                    name: pending.name,
                };
                add_nav_tab(tab);
                navigate_from_tab(kind, pending.id);
            }
        }
    });

    let render_nav_tabs = move || {
        view! {
            {move || if nav_tabs.get().is_empty() { ().into_any() } else {
                view! {
                    <For
                        each=move || nav_tabs.get().clone()
                        key=|tab| tab.id
                        children=move |tab: NavTab| {
                            let tab_id = tab.id;
                            let kind = tab.kind;
                            let name = tab.name.clone();
                            view! {
                                <div class="pf-nav-tab"
                                    class:active={move || active_tab_id.get() == Some(tab_id)}
                                    title={format!("{} {}", kind.icon(), name)}
                                    on:click=move |_| {
                                        navigate_from_tab(kind, tab_id);
                                    }
                                >
                                    <span class="pf-nav-tab-icon">{kind.icon()}</span>
                                    <span class="pf-nav-tab-name">{name.clone()}</span>
                                    <button class="pf-nav-tab-close" type="button" aria-label="Close tab"
                                        on:click=move |ev: leptos::ev::MouseEvent| {
                                            ev.stop_propagation();
                                            remove_nav_tab(tab_id);
                                        }
                                    >"×"</button>
                                </div>
                            }
                        }
                    />
                }.into_any()
            }}
        }.into_any()
    };

    let grouped_portfolios = Memo::new(move |_| {
        let orgs = organization_store.with(|s| s.organizations.clone());
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
        app_store.with(|store| {
            organization_store.with(|orgs| {
                let role = org_id
                    .map(|id| {
                        orgs.current_user_role_in_org(
                            id,
                            store.current_user.id,
                            store.current_user.role.clone(),
                        )
                    })
                    .unwrap_or_else(|| store.current_user.role.clone());
                role == UserRole::Owner || role == UserRole::Manager
            })
        })
    };

    let can_edit_documents = move |org_id: Option<Uuid>| {
        app_store.with(|store| {
            organization_store.with(|orgs| {
                let role = org_id
                    .map(|id| {
                        orgs.current_user_role_in_org(
                            id,
                            store.current_user.id,
                            store.current_user.role.clone(),
                        )
                    })
                    .unwrap_or_else(|| store.current_user.role.clone());
                let mut user = store.current_user.clone();
                user.role = role;
                user.can_upload_documents()
            })
        })
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
    let (add_type_menu, set_add_type_menu) = signal(Option::<(i32, i32)>::None);
    let (_org_context_menu, set_org_context_menu) = signal(Option::<(Uuid, i32, i32)>::None);
    let (pf_image_upload_target, set_pf_image_upload_target) = signal(Option::<Uuid>::None);
    let pf_image_input_ref = NodeRef::<leptos::html::Input>::new();
    let (new_asset_name, set_new_asset_name) = signal(String::new());
    let (new_asset_type, set_new_asset_type) = signal(AssetType::RealEstate);
    let (new_asset_value, set_new_asset_value) = signal(String::new());
    let (new_channel_name, set_new_channel_name) = signal(String::new());
    let (new_channel_rate, set_new_channel_rate) = signal(String::new());

    // Portfolio add-form fields live at page level so switching between
    // Portfolio / Asset Group / Asset keeps the last entered contents.
    let new_pf_name = RwSignal::new(String::new());
    let new_pf_desc = RwSignal::new(String::new());
    let new_pf_img = RwSignal::new(Option::<String>::None);
    let new_pf_org = RwSignal::new(String::new());

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
        if let Some(nav) = app_store.with(|s| s.pending_nav_target.clone()) {
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
        let in_grid = view_mode.get() == ViewMode::Grid;
        app_store.update(|s| {
            if s.selected_portfolio_ids.contains(&id) && !in_grid {
                s.selected_portfolio_ids.remove(&id);
            } else {
                s.selected_portfolio_ids.insert(id);
                s.touch_portfolio(id);
            }
        });
        if in_grid {
            ui_store.update(|s| s.portfolio_view_mode = ViewMode::List);
        }
    };

    let on_delete_portfolio = move |id: Uuid| {
        // The built-in Unassigned portfolio can't be deleted — its
        // assets would be lost. Move them out first.
        if crate::stores::app_store::is_unassigned_portfolio(id) {
            return;
        }
        app_store.update(|s| {
            s.remove_portfolio(id);
            s.selected_portfolio_ids.remove(&id);
        });
        set_edit_portfolio_id.set(None);
        leptos::task::spawn_local(async move {
            let _ = crate::server::delete_portfolio(id).await;
        });
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
        // Keep the entered name — the add form preserves its contents
        // for quick repeat creation; the user edits it for the next item.
        set_show_add_group.set(None);
    });

    // The built-in "Unassigned" portfolio holds assets created without
    // a target; keep it present so it always renders as a drop target.
    app_store.update(|s| s.ensure_unassigned_portfolio());

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
            AssetTarget::None => {
                // No portfolio/group chosen — park the asset in the
                // built-in "Unassigned" portfolio so it can be dragged
                // into place afterwards.
                s.ensure_unassigned_portfolio();
                if let Some(p) = s
                    .portfolios
                    .iter_mut()
                    .find(|p| p.id == crate::stores::app_store::UNASSIGNED_PORTFOLIO_ID)
                {
                    p.assets.push(asset);
                    p.recalculate_values();
                }
            }
        });
        // Keep name/value/type/etc. — the add form preserves its last
        // contents so creating a series of similar items is quick.
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
        // Keep the selected portfolio — contents persist for repeat adds.
    };

    let on_top_add_asset = move |_| {
        let pid = top_add_asset_pid.get();
        // No portfolio selected → create the asset unassigned so it can
        // be dragged into a portfolio/group afterwards.
        let target = match (pid, top_add_asset_gid.get()) {
            (Some(p), Some(g)) => AssetTarget::Group(p, g),
            (Some(p), None) => AssetTarget::PortfolioDirect(p),
            (None, _) => AssetTarget::None,
        };
        if let Some(asset_id) = on_add_asset.run(target) {
            let name = new_channel_name.get();
            if !name.trim().is_empty() {
                let rate = new_channel_rate.get().parse::<f64>().ok();
                let mut channel = Channel::new_test_channel(name, Some(asset_id), pid);
                channel.nightly_rate_override = rate;
                app_store.update(|s| s.add_channel(channel));
            }
        }
        ui_store.update(|s| s.show_top_add_asset = false);
        // Keep portfolio/group/channel selections — contents persist.
    };

    // Open exactly one top-level add form — the Portfolio / Asset Group /
    // Asset forms share the same spot and replace one another.
    let open_add_form = move |which: u8| {
        ui_store.update(|s| {
            s.show_add_portfolio = which == 0;
            s.show_top_add_group = which == 1;
            s.show_top_add_asset = which == 2;
        });
    };

    let _selected_portfolio = move || {
        let ids = selected_ids();
        filtered_portfolios
            .get()
            .into_iter()
            .find(|p| ids.contains(&p.id))
    };

    let render_portfolio = move |(tint_idx, portfolio): (usize, Portfolio)| -> AnyView {
        let portfolio_id = portfolio.id;
        let org_id = portfolio.organization_id;
        let can = Memo::new(move |_| can_edit(org_id));
        let can_docs = Memo::new(move |_| can_edit_documents(org_id));
        // The built-in Unassigned bucket gets no portfolio wrapper — its
        // assets render freely at the same level as the portfolio cards.
        if crate::stores::app_store::is_unassigned_portfolio(portfolio_id) {
            if portfolio.assets.is_empty() && portfolio.asset_groups.is_empty() {
                return ().into_any();
            }
            return view! {
                <div class="pf-unassigned">
                    <AssetViewer
                        portfolio={portfolio}
                        can_edit={can}
                        can_edit_documents={can_docs}
                        view_mode={view_mode}
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
                        child_depth={tint_idx}
                    />
                </div>
            }
            .into_any();
        }
        let is_expanded = Memo::new(move |_| selected_ids().contains(&portfolio_id));
        view! {
            <PortfolioListItem
                portfolio={portfolio}
                tint_index={tint_idx}
                can_edit={can}
                can_edit_documents={can_docs}
                expanded={is_expanded}
                on_toggle=Callback::new(move |_| on_toggle_view(portfolio_id))
                on_context=move |x, y| {
                    let (x, y) = clamp_context_menu(x, y);
                    set_org_context_menu.set(None);
                    set_context_menu.set(Some((portfolio_id, x, y)));
                }
                on_context_close=move || {
                    set_context_menu.set(None);
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
        <div class="home-screen home-screen-pf"
            on:dragend=move |_| clear_pf_drag_kind()
        >
            // Portfolio controls bar (attached below navbar)
            <div class="portfolio-controls-bar"
                on:dragover=move |ev: leptos::ev::DragEvent| {
                    ev.prevent_default();
                    if let Some(dt) = ev.data_transfer() { let _ = dt.set_drop_effect("copy"); }
                }
                on:drop=move |ev: leptos::ev::DragEvent| {
                    ev.prevent_default();
                    match nav_tab_from_drag_event(&ev) {
                        Some(tab) => {
                            leptos::logging::log!("portfolio bar drop: {:?} {}", tab.kind, tab.id);
                            add_nav_tab(tab);
                        }
                        None => leptos::logging::log!("portfolio bar drop: no tab data"),
                    }
                }
            >
                <div class="pf-nav-controls"
                    on:dragover=move |ev: leptos::ev::DragEvent| {
                        ev.prevent_default();
                        if let Some(dt) = ev.data_transfer() { let _ = dt.set_drop_effect("copy"); }
                    }
                >
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
                            SortMode::Relevance => "sort_relevance",
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
                            "sort_relevance" => SortMode::Relevance,
                            "sort_oldest" => SortMode::Oldest,
                            "sort_highest_value" => SortMode::HighestValue,
                            "sort_lowest_value" => SortMode::LowestValue,
                            "sort_highest_profit" => SortMode::HighestProfit,
                            "sort_lowest_profit" => SortMode::LowestProfit,
                            "sort_highest_revenue" => SortMode::HighestRevenue,
                            "sort_lowest_revenue" => SortMode::LowestRevenue,
                            "sort_by_organization" => SortMode::ByOrganization,
                            _ => SortMode::Relevance,
                        };
                        ui_store.update(|s| s.portfolio_sort_mode = mode);
                    }
                >
                    <option value="sort_relevance">"Sort: Relevance"</option>
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
                <span class="nav-portfolio-divider"></span>
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
            {move || if nav_tabs.get().is_empty() { ().into_any() } else { view! {
                <div class="pf-nav-tabs-scroll"
                    on:dragover=move |ev: leptos::ev::DragEvent| {
                        ev.prevent_default();
                        if let Some(dt) = ev.data_transfer() { let _ = dt.set_drop_effect("copy"); }
                    }
                >
                    {render_nav_tabs}
                </div>
            }.into_any() }}
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
                                    {(!crate::stores::app_store::is_unassigned_portfolio(pid_delete)).then(|| view! {
                                        <button
                                            class="login-btn sell"
                                            on:click=move |_| {
                                                on_delete_portfolio(pid_delete);
                                            }
                                        >
                                            "🗑 Delete Portfolio"
                                        </button>
                                    })}
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any()
            })}

            // Portfolios List
            <div class={move || {
                if view_mode.get() == ViewMode::Grid {
                    format!("grid-view grid-cols-{}", ui_store.get().portfolio_grid_columns)
                } else { "pf-accordion".to_string() }
            }}
            on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                ev.prevent_default();
                set_org_context_menu.set(None);
                let (x, y) = clamp_context_menu(ev.client_x(), ev.client_y());
                set_context_menu.set(Some((Uuid::new_v4(), x, y)));
            }>
                {move || if grouped_portfolios.get().is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-state-icon">"📊"</div>
                            <div class="empty-state-title">"No portfolios yet"</div>
                            <div class="empty-state-desc">"Create your first portfolio to get started"</div>
                            <button class="login-btn" on:click=move |_| ui_store.update(|s| {
                                s.show_add_portfolio = !s.show_add_portfolio;
                                if s.show_add_portfolio {
                                    s.show_top_add_group = false;
                                    s.show_top_add_asset = false;
                                }
                            })>"Create Portfolio"</button>
                            {move || ui_store.get().show_add_portfolio.then(|| view! {
                                <AddPortfolioModal name={new_pf_name} desc={new_pf_desc} image_url={new_pf_img} org_id={new_pf_org} />
                            })}
                        </div>
                    }.into_any()
                } else if view_mode.get() == ViewMode::Grid {
                    view! {
                        <For
                            each=move || sorted_portfolios.get()
                            key=|portfolio| portfolio.id
                            children=move |p| render_portfolio((0, p))
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
                                    let indexed_portfolios: Vec<(usize, Portfolio)> =
                                        portfolios.iter().cloned().enumerate().collect();
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
                                    let current_user_id = app_store.with(|s| s.current_user.id);
                                    let org_can_view_docs = Memo::new(move |_| {
                                        organization_store.with(|s| s.user_has_perm_in_org(
                                            org_id,
                                            current_user_id,
                                            &Perm::ViewDocuments,
                                        ))
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
                                    // Phantom-click dismissal for the org name/desc
                                    // inline editors.
                                    let org_scope_token: &'static str =
                                        Box::leak(format!("org-{}", org_id).into_boxed_str());
                                    {
                                        let save = save_org_edit;
                                        let alive = phantom_edit_scope(
                                            org_scope_token.to_string(),
                                            move || {
                                                is_editing_org_name.get_untracked()
                                                    || is_editing_org_desc.get_untracked()
                                            },
                                            move || save(),
                                        );
                                        on_cleanup(move || {
                                            alive.store(false, std::sync::atomic::Ordering::Relaxed)
                                        });
                                    }
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
                                    let (org_touch_start, org_touch_move, org_touch_end) = mobile_tab_gestures(
                                        NavTabKind::Organization,
                                        org_id,
                                        org_name.clone(),
                                        move |_x, _y| {},
                                        move || {},
                                    );
                                    // Drop target: portfolios/groups/assets into this org
                                    let (org_drop_hover, set_org_drop_hover) = signal(false);
                                    let (org_d_over, org_d_leave, org_d_drop) = pf_drop_target(
                                        set_org_drop_hover,
                                        |k| matches!(k, NavTabKind::Portfolio | NavTabKind::AssetGroup | NavTabKind::Asset),
                                        move |tab, _rel_y| match tab.kind {
                                            NavTabKind::Portfolio => {
                                                app_store.update(|s| s.set_portfolio_organization(tab.id, Some(org_id)));
                                            }
                                            NavTabKind::AssetGroup => {
                                                app_store.update(|s| s.set_asset_group_organization(tab.id, Some(org_id)));
                                            }
                                            NavTabKind::Asset => {
                                                app_store.update(|s| s.set_asset_organization(tab.id, Some(org_id)));
                                            }
                                            _ => {}
                                        },
                                    );
                                    view! {
                                        <div class="pf-org-group" id={format!("org-group-{}", org_id)} class:expanded={move || is_expanded()}
                                            class:drop-hover={move || org_drop_hover.get()}
                                            draggable="true"
                                            on:dragover=org_d_over
                                            on:dragleave=org_d_leave
                                            on:drop=org_d_drop
                                            on:dragend=move |_| clear_pf_drag_kind()
                                            on:dragstart=move |ev: leptos::ev::DragEvent| {
                                                set_drag_tab_data(&ev, &NavTabKind::Organization, org_id, &org_name);
                                            }
                                            on:touchstart=org_touch_start
                                            on:touchmove=org_touch_move
                                            on:touchend=org_touch_end
                                        >
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
                                                        ev.prevent_default();
                                                        ev.stop_propagation();
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
                                                                <input class="pf-edit-input pf-edit-name-md" placeholder="Organization name"
                                                                    attr:data-pf-edit-scope=org_scope_token
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
                                                                <input class="pf-edit-input pf-edit-desc" placeholder="Description"
                                                                    attr:data-pf-edit-scope=org_scope_token
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
                                                    each=move || indexed_portfolios.clone()
                                                    key=|item| item.1.id
                                                    children=move |(idx, p)| render_portfolio((idx + 1, p))
                                                />
                                            </div>
                                        </div>
                                    }.into_any()
                                } else if let Some(p) = portfolios.into_iter().next() {
                                    render_portfolio((0, p))
                                } else {
                                    ().into_any()
                                }
                            }
                        />
                    }.into_any()
                }}
                    <div class="add-type-dropdown">
                        <button
                            class="pf-empty-outline org-empty-outline"
                            type="button"
                            aria-label="Add a new item"
                            aria-expanded={move || add_type_menu.get().is_some().to_string()}
                            on:click=move |ev: leptos::ev::MouseEvent| {
                                ev.stop_propagation();
                                set_add_type_menu.update(|v| *v = if v.is_some() { None } else { Some((0, 0)) });
                            }
                        >
                            <span class="org-empty-outline-icon">"+"</span>
                            <span class="org-empty-outline-text">"Add ▾"</span>
                        </button>
                        {move || add_type_menu.get().map(|_| view! {
                            <div class="context-menu-overlay" on:click=move |_| set_add_type_menu.set(None)></div>
                            <div class="add-type-menu" role="menu">
                                <button class="add-type-item" role="menuitem"
                                    on:click=move |_| {
                                        set_add_type_menu.set(None);
                                        open_add_form(0);
                                    }
                                >"📁 Portfolio"</button>
                                <button class="add-type-item" role="menuitem"
                                    on:click=move |_| {
                                        set_add_type_menu.set(None);
                                        open_add_form(1);
                                    }
                                >"📂 Asset Group"</button>
                                <button class="add-type-item" role="menuitem"
                                    on:click=move |_| {
                                        set_add_type_menu.set(None);
                                        open_add_form(2);
                                    }
                                >"📦 Asset"</button>
                            </div>
                        }.into_any())}
                    </div>
                    {move || ui_store.get().show_add_portfolio.then(|| view! {
                        <AddPortfolioModal name={new_pf_name} desc={new_pf_desc} image_url={new_pf_img} org_id={new_pf_org} />
                    })}
                    {move || ui_store.get().show_top_add_group.then(|| view! {
                        <div class="add-form">
                            <select
                                class="login-input"
                                aria-label="Select portfolio for group"
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
                                prop:value={move || new_group_name.get()}
                                on:input=move |ev| set_new_group_name.set(event_target_value(&ev))
                            />
                            <button class="login-btn" on:click=on_top_add_group>"Create Group"</button>
                        </div>
                    })}
                    {move || ui_store.get().show_top_add_asset.then(|| view! {
                        <div class="add-form">
                            <select
                                class="login-input"
                                aria-label="Select portfolio for asset"
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
                                <option value="">"Unassigned — no portfolio"</option>
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
                                        aria-label="Select group for asset"
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
                                prop:value={move || new_asset_name.get()}
                                on:input=move |ev| set_new_asset_name.set(event_target_value(&ev))
                            />
                            <input
                                class="login-input"
                                type="text"
                                placeholder="Value"
                                aria-label="Value"
                                prop:value={move || new_asset_value.get()}
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
            </div>

            // Context menu for portfolio press-and-hold or whitespace
            {move || context_menu.get().map(|(pid, x, y)| {
                let pid_doc = pid;
                let pid_role = pid;
                let pid_org = pid;
                let pid_remove = pid;
                let pid_assign = pid;
                let (is_whitespace, org_id, pf_name) = app_store.with(|s| {
                    let p = s.portfolios.iter().find(|p| p.id == pid);
                    (
                        p.is_none(),
                        p.and_then(|p| p.organization_id),
                        p.map(|p| p.name.clone()).unwrap_or_default(),
                    )
                });
                let open_name = pf_name.clone();
                let can = can_edit(org_id);
                let content = if is_whitespace {
                    view! {
                        <button
                            class="context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                open_add_form(0);
                            }
                        >
                            "➕ Create Portfolio"
                        </button>
                        <button
                            class="context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                open_add_form(1);
                            }
                        >
                            "➕ Create Asset Group"
                        </button>
                        <button
                            class="context-menu-item"
                            on:click=move |_| {
                                set_context_menu.set(None);
                                open_add_form(2);
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
                                    ui_store.update(|s| s.toggle_assignment_panel(pid_assign));
                                }
                            >
                                "👥 Assign Users"
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
                                    ui_store.update(|s| s.toggle_red_listing_panel(pid_org));
                                }
                            >
                                "🛒 List on Red Marketplace"
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
                                ui_store.update(|s| s.open_nav_tab("portfolio", pid, &open_name));
                            }
                        >
                            "🗔 Open in New Tab"
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
                        {admin_items}
                    }.into_any()
                };
                view! {
                    <div
                        class="context-menu-overlay"
                        on:click=move |_| set_context_menu.set(None)
                        on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                            ev.prevent_default();
                            set_context_menu.set(None);
                        }
                    >
                        <div
                            class="context-menu"
                            style={format!("left: {}px; top: {}px;", x, y)}
                            on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                                ev.prevent_default();
                                ev.stop_propagation();
                            }
                        >
                            {content}
                        </div>
                    </div>
                }.into_any()
            })}

            // Add Role modal (portfolio context menu)
            {move || show_pf_add_role.get().map(|pid| {
                let org_id = app_store.with(|s| s.portfolios.iter().find(|p| p.id == pid).and_then(|p| p.organization_id));
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
                                    {move || organization_store.with(|s| s.organizations.iter().map(|o| {
                                        let id = o.id.to_string();
                                        view! { <option value={id.clone()}>{o.name.clone()}</option> }
                                    }).collect::<Vec<_>>())}
                                </select>
                                <input class="login-input" type="text" placeholder="Or create a new organization"
                                    aria-label="New organization name"
                                    prop:value={move || pf_new_org_name.get()}
                                    on:input=move |ev| set_pf_new_org_name.set(event_target_value(&ev)) />
                                <label class="list-item-title">"Secondary / tertiary organizations"</label>
                                {move || organization_store.with(|s| s.organizations.iter().map(|o| {
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
                                }).collect::<Vec<_>>())}
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
                let pf_name = app_store.with(|s| s.portfolios.iter()
                    .find(|p| p.id == pid)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| "this portfolio".to_string()));
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
