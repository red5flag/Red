use crate::models::{Asset, AssetGroup, Channel};
use crate::stores::{use_app_store, use_notification_store, use_organization_store, use_ui_store};
use crate::types::{AssetType, ViewMode};
use leptos::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use super::{
    detect_file_type, name_click_handlers, read_image_as_data_url, single_sentence, AssetItem,
    AssetTarget, DocModal, NotifTarget, UserAssignmentPanel,
};

#[component]
pub(crate) fn AssetGroupItem(
    group: AssetGroup,
    #[prop(into)] can_edit: Signal<bool>,
    #[prop(into)] can_edit_documents: Signal<bool>,
    pid: Uuid,
    gid: Uuid,
    expanded: Memo<bool>,
    view_mode: ViewMode,
    grid_columns: usize,
    on_toggle: Callback<Uuid>,
    show_add_asset: ReadSignal<AssetTarget>,
    #[allow(unused_variables)] set_show_add_asset: WriteSignal<AssetTarget>,
    _new_asset_name: ReadSignal<String>,
    set_new_asset_name: WriteSignal<String>,
    _new_asset_type: ReadSignal<AssetType>,
    set_new_asset_type: WriteSignal<AssetType>,
    _new_asset_value: ReadSignal<String>,
    set_new_asset_value: WriteSignal<String>,
    on_add_asset: Callback<AssetTarget, Option<Uuid>>,
    on_select_asset: Callback<Asset>,
    portfolio_name: String,
    #[prop(default = 0)] tint_index: usize,
    on_open_notif_qs: Callback<(NotifTarget, String, bool)>,
    visible_counts: ReadSignal<HashMap<String, usize>>,
    set_visible_counts: WriteSignal<HashMap<String, usize>>,
) -> impl IntoView {
    let app_store = use_app_store();
    let notification_store = use_notification_store();
    let ui_store = use_ui_store();
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
    let (show_group_move_portfolio, set_show_group_move_portfolio) = signal(false);
    let (group_target_portfolio_id, set_group_target_portfolio_id) = signal(String::new());
    let (group_org_id, set_group_org_id) = signal(String::new());
    let (confirm_group_remove, set_confirm_group_remove) = signal(false);
    let (group_role_name, set_group_role_name) = signal(String::new());
    let (group_role_desc, set_group_role_desc) = signal(String::new());
    let (group_org_name, set_group_org_name) = signal(String::new());
    let (edit_name, set_edit_name) = signal(group.name.clone());
    let (edit_desc, set_edit_desc) = signal(group.description.clone().unwrap_or_default());

    let (name_click, name_dblclick) = name_click_handlers(
        move || on_toggle.run(gid),
        move || if can_edit_here.get() { set_is_editing.set(true); },
    );
    let (desc_click, desc_dblclick) = name_click_handlers(
        move || on_toggle.run(gid),
        move || if can_edit_here.get() { set_is_editing.set(true); },
    );

    let (new_channel_name, set_new_channel_name) = signal(String::new());
    let (new_channel_rate, set_new_channel_rate) = signal(String::new());
    let (new_asset_id, set_new_asset_id) = signal(Option::<Uuid>::None);
    let group_image_input_ref = NodeRef::<leptos::html::Input>::new();
    let group_image_url = group.image_url.clone();
    let group_emoji = group.emoji.clone().unwrap_or_else(|| "📁".to_string());

    // Per-scope visible count for this group's assets.
    let assets_scope = format!("group-assets-{gid}");
    let assets_scope_visible = assets_scope.clone();
    let assets_scope_expand = assets_scope.clone();
    let page_size = move || ui_store.get().portfolio_view_count(view_mode).as_usize();
    let visible_for_scope = move || {
        visible_counts
            .get()
            .get(&assets_scope_visible)
            .copied()
            .unwrap_or_else(page_size)
    };
    let expand_scope = move |_| {
        let increment = page_size();
        set_visible_counts.update(|map| {
            let current = map
                .get(&assets_scope_expand)
                .copied()
                .unwrap_or_else(page_size);
            map.insert(
                assets_scope_expand.clone(),
                current.saturating_add(increment),
            );
        });
    };

    let g_name = group.name.clone();
    let g_desc = single_sentence(&group.description.clone().unwrap_or_default());
    let g_name_for_modal = group.name.clone();
    let g_name_for_doc_btn = group.name.clone();
    let g_name_for_confirm = group.name.clone();
    let docs = group.documents.clone();
    let doc_count = docs.len();
    let asset_count = group.assets.len();
    let assigned_users = group.assigned_users.clone();
    let organization_store = use_organization_store();
    let org_users = move || organization_store.get().organization_users.clone();
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
        if n.trim().is_empty() {
            return;
        }
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                    g.name = n.clone();
                    g.description = if d.trim().is_empty() {
                        None
                    } else {
                        Some(d.clone())
                    };
                    g.updated_at = chrono::Utc::now();
                }
            }
        });
        set_is_editing.set(false);
    };

    let add_group_doc = move |n: String| {
        if n.trim().is_empty() {
            return;
        }
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

    let group_tint_style = format!(
        "background: rgba(255,255,255,{});",
        (tint_index as f64 * 0.1).min(0.9)
    );

    view! {
        <div class="asset-group" class:expanded={move || expanded.get()} style={group_tint_style.clone()}
            on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                if can_edit_here.get() {
                    ev.prevent_default();
                    ev.stop_propagation();
                    set_group_context_menu.set(Some((ev.client_x(), ev.client_y())));
                }
            }
        >
            <div class="asset-group-header"
                role="button"
                tabindex="0"
                aria-expanded={move || expanded.get()}
                aria-controls={format!("ag-content-{}", gid)}
                aria-label={move || {
                    format!("{} group. {} asset{}. {} document{}. {}",
                        g_name,
                        asset_count,
                        if asset_count == 1 { "" } else { "s" },
                        doc_count,
                        if doc_count == 1 { "" } else { "s" },
                        if expanded.get() { "Expanded" } else { "Collapsed" }
                    )
                }}
                on:click=move |ev: leptos::ev::MouseEvent| {
                    ev.stop_propagation();
                    if !is_editing.get() { on_toggle.run(gid); }
                }
                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                    if ev.key() == "Enter" || ev.key() == " " {
                        ev.prevent_default();
                        ev.stop_propagation();
                        if !is_editing.get() { on_toggle.run(gid); }
                    }
                }
            >
                <span class="asset-group-arrow">
                    {move || if expanded.get() { "▼" } else { "▶" }}
                </span>
                <input
                    type="file"
                    accept="image/*"
                    class="pf-hidden-file-input"
                    node_ref=group_image_input_ref
                    on:change=move |ev| {
                        read_image_as_data_url(&ev, {
                            let app_store = app_store.clone();
                            move |url: String| {
                                app_store.update(|s| {
                                    if let Some(p) = s.get_portfolio_mut(pid) {
                                        if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                                            g.image_url = Some(url);
                                            g.updated_at = chrono::Utc::now();
                                        }
                                    }
                                });
                                if let Some(p) = app_store.get().get_portfolio(pid).cloned() {
                                    leptos::task::spawn_local(async move {
                                        let _ = crate::server::save_portfolio(p).await;
                                    });
                                }
                            }
                        });
                    }
                />
                <div class="asset-group-icon"
                    on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                        if can_edit_here.get() {
                            ev.prevent_default();
                            ev.stop_propagation();
                            if let Some(input) = group_image_input_ref.get() {
                                let _ = input.click();
                            }
                        }
                    }
                >
                    {if let Some(ref url) = group_image_url {
                        view! { <img class="pf-header-image" src={url.clone()} alt="Group image" /> }.into_any()
                    } else {
                        view! { <span>{group_emoji.clone()}</span> }.into_any()
                    }}
                </div>
                <div class="asset-group-info-wrap">
                    {let g_name_header = g_name.clone();
                    let g_desc_header = g_desc.clone();
                    let group_channel_ids = group.channel_ids.clone();
                    move || if is_editing.get() && can_edit_here.get() {
                        view! {
                            <div class="asset-group-edit-form">
                                <input class="pf-edit-input" placeholder="Group name"
                                    aria-label="Group name"
                                    prop:value=move || edit_name.get()
                                    on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                    on:blur=save_group_edit />
                                <input class="pf-edit-input" placeholder="Description"
                                    aria-label="Description"
                                    prop:value=move || edit_desc.get()
                                    on:input=move |ev| set_edit_desc.set(event_target_value(&ev))
                                    on:blur=save_group_edit />
                                <UserAssignmentPanel assigned={assigned_users.clone()} users={org_users()} on_toggle={toggle_group_assignment} />
                            </div>
                        }.into_any()
                    } else {
                        let channel_count = group_channel_ids.len();
                        let has_channels = channel_count > 0;
                        view! {
                            <div>
                                <div class="asset-group-name" on:click={name_click.clone()} on:dblclick={name_dblclick.clone()}>{g_name_header.clone()}</div>
                                {if !g_desc_header.is_empty() {
                                    view! { <div class="asset-group-desc" on:click={desc_click.clone()} on:dblclick={desc_dblclick.clone()}>{g_desc_header.clone()}</div> }.into_any()
                                } else { ().into_any() }}
                                <div class="asset-group-count">{
                                    if asset_count == 0 && has_channels {
                                        format!("{} channels", channel_count)
                                    } else {
                                        format!("{} assets", asset_count)
                                    }
                                }</div>
                            </div>
                            {if has_channels && asset_count > 0 {
                                view! {
                                    <div class="asset-group-channel-icon" title={format!("{} channel(s)", channel_count)}>
                                        "📡" {channel_count}
                                    </div>
                                }.into_any()
                            } else { ().into_any() }}
                        }.into_any()
                    }}
                </div>
                // Action buttons
                <div class="pf-list-actions" on:click=|ev| ev.stop_propagation()>
                    {let g_name_for_notif = g_name_for_doc_btn.clone();
                    move || {
                        let count = app_store.get().doc_notifications_for_group(pid, gid, &notification_store.get().notifications);
                        let gname = g_name_for_notif.clone();
                        let gname_click = gname.clone();
                        let gname_ctx = gname.clone();
                        let gname_keydown = gname.clone();
                        view! {
                            <span class="pf-notif-badge pf-notif-badge-clickable"
                                role="button"
                                tabindex="0"
                                aria-label={format!("Notifications for {} group. {} unread", gname, count)}
                                title="Left-click to view notifications, right-click to edit settings"
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    on_open_notif_qs.run((NotifTarget::Group(pid, gid), gname_click.clone(), false));
                                }
                                on:contextmenu=move |ev| {
                                    ev.prevent_default();
                                    ev.stop_propagation();
                                    on_open_notif_qs.run((NotifTarget::Group(pid, gid), gname_ctx.clone(), true));
                                }
                                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                    if ev.key() == "Enter" || ev.key() == " " {
                                        ev.prevent_default();
                                        ev.stop_propagation();
                                        on_open_notif_qs.run((NotifTarget::Group(pid, gid), gname_keydown.clone(), false));
                                    }
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
                        class:active=move || ui_store.get().is_doc_modal_open(gid)
                        aria-label={format!("View documents for {} group. {} document{}", g_name_for_doc_btn, doc_count, if doc_count == 1 { "" } else { "s" })}
                        on:click=move |_| ui_store.update(|s| s.toggle_doc_modal(gid))>
                        {format!("📄 {}", doc_count)}
                    </button>
                </div>
            </div>
            // Docs modal for group
            {move || if ui_store.get().is_doc_modal_open(gid) {
                let modal_title = g_name_for_modal.clone();
                let add_cb = if can_edit_documents_here.get() { Some(Callback::new(move |n: String| add_group_doc(n))) } else { None };
                view! {
                    <DocModal
                        entity_id={gid}
                        title={modal_title}
                        on_close=move || ui_store.update(|s| s.close_doc_modal(gid))
                        can_edit={can_edit_documents_here.get()}
                        on_add={add_cb}
                        portfolio_id={Some(pid)}
                        group_id={Some(gid)}
                    />
                }.into_any()
            } else { ().into_any() }}

            <div id={format!("ag-content-{}", gid)} class="asset-group-content" class:hidden={move || !expanded.get()}>
                {move || if show_add_asset.get() == AssetTarget::Group(pid, gid) {
                    view! {
                        <div class="add-form">
                            <input class="login-input" type="text" placeholder="Asset name"
                                aria-label="Asset name"
                                on:input=move |ev| set_new_asset_name.set(event_target_value(&ev)) />
                            <input class="login-input" type="text" list="asset-type-options-group" placeholder="Asset type"
                                aria-label="Asset type"
                                prop:value={move || _new_asset_type.get().to_input_string()}
                                on:input=move |ev| set_new_asset_type.set(AssetType::from_input(&event_target_value(&ev))) />
                            <datalist id="asset-type-options-group">
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
                            <button class="login-btn"
                                on:click=move |_| {
                                    if let Some(asset_id) = on_add_asset.run(AssetTarget::Group(pid, gid)) {
                                        let name = new_channel_name.get();
                                        if !name.trim().is_empty() {
                                            let rate = new_channel_rate.get().parse::<f64>().ok();
                                            let mut channel = Channel::new_test_channel(name, Some(asset_id), Some(pid));
                                            channel.nightly_rate_override = rate;
                                            app_store.update(|s| s.add_channel(channel));
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
                } else { ().into_any() }}

                {{
                    let view_mode = view_mode.clone();
                    let group_assets: Vec<_> = group.assets.into_iter().filter(|a| group_visible_to_user || a.is_visible_to(user_id, can_view_all)).collect();
                    let total_assets = group_assets.len();
                    let class_str = if view_mode == ViewMode::Grid {
                        format!("asset-group-assets grid-view-{}", grid_columns)
                    } else {
                        "asset-group-assets asset-list".to_string()
                    };
                    let display_assets = visible_for_scope().min(total_assets);
                    let remaining_assets = total_assets.saturating_sub(display_assets);
                    let assets_to_show: Vec<_> = group_assets.into_iter().take(display_assets).collect();
                    view! {
                        <div class={class_str}>
                            {assets_to_show.into_iter().enumerate().map({
                                let view_mode = view_mode.clone();
                                move |(idx, asset)| view! {
                                    <AssetItem asset={asset} portfolio_name={portfolio_name.clone()} portfolio_id={Some(pid)} group_id={Some(gid)} view_mode={view_mode.clone()} on_select={on_select_asset} can_edit={can_edit_here} can_edit_documents={can_edit_documents_here} tint_index={idx + 1} collapsible=true highlight={Some(Signal::derive(move || new_asset_id.get()))} />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                        {if remaining_assets > 0 {
                            view! {
                                <button class="pf-show-more-btn pf-expand-view-btn"
                                    aria-label={format!("Expand view. Currently showing {} of {} assets in this group", display_assets, total_assets)}
                                    on:click=expand_scope
                                >
                                    {format!("Expand View + ({}/{}) ", display_assets, total_assets)}
                                </button>
                            }.into_any()
                        } else { ().into_any() }}
                    }
                }}
            </div>

            // Context menu for group press-and-hold
            {move || group_context_menu.get().map(|(x, y)| {
                view! {
                    <div class="context-menu-overlay" on:click=move |_| set_group_context_menu.set(None)>
                        <div class="context-menu" style={format!("left: {}px; top: {}px;", x, y)}>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_group_context_menu.set(None);
                                    set_group_target_portfolio_id.set(String::new());
                                    set_show_group_move_portfolio.set(true);
                                }
                            >"➕ Add to Portfolio"</button>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_group_context_menu.set(None);
                                    app_store.with(|s| {
                                        for p in s.portfolios.iter() {
                                            if let Some(g) = p.asset_groups.iter().find(|g| g.id == gid) {
                                                set_group_org_id.set(g.organization_id.map(|id| id.to_string()).unwrap_or_default());
                                                break;
                                            }
                                        }
                                    });
                                    set_show_group_add_org.set(true);
                                }
                            >"� Add to Organization"</button>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_group_context_menu.set(None);
                                    ui_store.update(|s| s.open_doc_modal(gid));
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
                                <button class="doc-modal-close" aria-label="Close add role" on:click=move |_| set_show_group_add_role.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="Role name"
                                    aria-label="Role name"
                                    prop:value={move || group_role_name.get()}
                                    on:input=move |ev| set_group_role_name.set(event_target_value(&ev)) />
                                <input class="login-input" type="text" placeholder="Description"
                                    aria-label="Description"
                                    prop:value={move || group_role_desc.get()}
                                    on:input=move |ev| set_group_role_desc.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| {
                                    let name = group_role_name.get();
                                    let desc = group_role_desc.get();
                                    if !name.trim().is_empty() {
                                        let role = crate::models::OrgRole::new(name, 0, desc, vec![]);
                                        if let Some(oid) = org_id {
                                            organization_store.update(|s| s.add_role_to_org(oid, role));
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

            // Add to Organization modal (group context menu)
            {move || if show_group_add_org.get() {
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_group_add_org.set(false)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add to Organization"</span>
                                <button class="doc-modal-close" aria-label="Close add organization" on:click=move |_| set_show_group_add_org.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <label class="list-item-title">"Organization"</label>
                                <select
                                    class="form-select"
                                    aria-label="Organization"
                                    prop:value={move || group_org_id.get()}
                                    on:change=move |ev| set_group_org_id.set(event_target_value(&ev))
                                >
                                    <option value="">"(None)"</option>
                                    {move || organization_store.get().organizations.iter().map(|o| {
                                        let id = o.id.to_string();
                                        view! { <option value={id.clone()}>{o.name.clone()}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                                <input class="login-input" type="text" placeholder="Or create a new organization"
                                    aria-label="New organization name"
                                    prop:value={move || group_org_name.get()}
                                    on:input=move |ev| set_group_org_name.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| {
                                    let name = group_org_name.get().trim().to_string();
                                    let org_id = if name.is_empty() {
                                        let s = group_org_id.get();
                                        if s.trim().is_empty() { None } else { Uuid::parse_str(&s).ok() }
                                    } else {
                                        let owner_id = app_store.get().current_user.id;
                                        let org = crate::models::Organization::new(name, owner_id);
                                        let oid = org.id;
                                        organization_store.update(|s| s.add_organization(org));
                                        Some(oid)
                                    };
                                    app_store.update(|s| { s.set_asset_group_organization(gid, org_id); });
                                    set_group_org_name.set(String::new());
                                    set_group_org_id.set(String::new());
                                    set_show_group_add_org.set(false);
                                }>"Save Organization"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}

            // Move to Portfolio modal (group context menu)
            {move || if show_group_move_portfolio.get() {
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_group_move_portfolio.set(false)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add to Portfolio"</span>
                                <button class="doc-modal-close" aria-label="Close move to portfolio" on:click=move |_| set_show_group_move_portfolio.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <label class="list-item-title">"Target portfolio"</label>
                                <select
                                    class="form-select"
                                    aria-label="Target portfolio"
                                    prop:value={move || group_target_portfolio_id.get()}
                                    on:change=move |ev| set_group_target_portfolio_id.set(event_target_value(&ev))
                                >
                                    <option value="">"Select a portfolio"</option>
                                    {move || app_store.get().portfolios.iter().map(|p| {
                                        let id = p.id.to_string();
                                        view! { <option value={id.clone()}>{p.name.clone()}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                                <button class="login-btn" on:click=move |_| {
                                    let s = group_target_portfolio_id.get();
                                    if let Ok(target_pid) = Uuid::parse_str(&s) {
                                        app_store.update(|store| { store.move_group_to_portfolio(gid, target_pid); });
                                    }
                                    set_group_target_portfolio_id.set(String::new());
                                    set_show_group_move_portfolio.set(false);
                                }>"Move to Portfolio"</button>
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
                                <button class="doc-modal-close" aria-label={format!("Cancel removal of {} group", g_name_for_confirm)} on:click=move |_| set_confirm_group_remove.set(false)>"✕"</button>
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
