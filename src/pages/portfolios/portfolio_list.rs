use crate::models::Perm;
use crate::stores::{use_app_store, use_notification_store, use_organization_store, use_ui_store};
use crate::types::{AssetType, ViewMode};
use leptos::prelude::*;
use uuid::Uuid;

use super::{
    detect_file_type, mobile_tab_gestures, name_click_handlers, pf_drop_target,
    phantom_edit_scope, read_image_as_data_url, set_drag_tab_data, single_sentence, AssetTarget,
    AssetViewer, DocEntry, DocSlider, NavTabKind, RedEntityKind, RedListingPanel,
    UserAssignmentPanel,
};

/// Portfolio list row — accordion style matching AssetGroupItem.
#[component]
pub(crate) fn PortfolioListItem(
    portfolio: crate::models::Portfolio,
    #[prop(into)] can_edit: Signal<bool>,
    #[prop(into)] can_edit_documents: Signal<bool>,
    #[prop(into)] expanded: Signal<bool>,
    on_toggle: Callback<()>,
    on_context: impl Fn(i32, i32) + Clone + 'static,
    on_context_close: impl Fn() + Clone + 'static,
    // AssetViewer props forwarded for expanded content
    #[prop(into)] show_add_group: Signal<Option<Uuid>>,
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
    on_add_asset: Callback<AssetTarget, Option<Uuid>>,
    #[prop(into)] view_mode: Signal<ViewMode>,
    #[prop(default = 0)] tint_index: usize,
) -> impl IntoView {
    let app_store = use_app_store();
    let notification_store = use_notification_store();
    let ui_store = use_ui_store();
    let pf_tint_style = format!(
        "background: rgba(255,255,255,{});",
        (tint_index as f64 * 0.1).min(0.9)
    );
    let (is_editing_name, set_is_editing_name) = signal(false);
    let (is_editing_desc, set_is_editing_desc) = signal(false);
    let (edit_name, set_edit_name) = signal(portfolio.name.clone());
    let (edit_desc, set_edit_desc) = signal(portfolio.description.clone().unwrap_or_default());
    let pid = portfolio.id;
    let name = portfolio.name.clone();
    let name_for_aria = name.clone();
    let name_for_doc_btn = name.clone();
    let name_for_slider = StoredValue::new(name.clone());
    let (show_doc_slider, set_show_doc_slider) = signal(false);
    let desc = single_sentence(&portfolio.description.clone().unwrap_or_default());
    let asset_count = portfolio.get_all_assets().len();
    let portfolio_image_url = portfolio.image_url.clone();
    // Values for the Red Marketplace listing panel (StoredValue keeps the
    // render closure Copy-safe).
    let red_name = StoredValue::new(name.clone());
    let red_desc = StoredValue::new(portfolio.description.clone());
    let red_value = portfolio.total_value;
    let red_img = StoredValue::new(portfolio_image_url.clone());
    let portfolio_emoji = portfolio.emoji.clone().unwrap_or_else(|| "🏢".to_string());
    let can_edit_here = can_edit;
    let can_edit_documents_here = can_edit_documents;
    let organization_store = use_organization_store();
    let current_user = app_store.get().current_user.clone();
    let current_user_id = current_user.id;
    let can_view_portfolio = portfolio.is_visible_to(current_user_id, current_user.can_view_all())
        || portfolio
            .organization_id
            .map_or(current_user.can_view_all(), |oid| {
                organization_store.get().user_has_perm_in_org(
                    oid,
                    current_user_id,
                    &Perm::ViewPortfolios,
                )
            });
    if !can_view_portfolio {
        return ().into_any();
    }
    // Snapshots used to re-sync the edit buffers when an editor opens, so a
    // cancelled edit doesn't leave stale text behind.
    let sync_name = portfolio.name.clone();
    let sync_desc = portfolio.description.clone().unwrap_or_default();
    let begin_text_edit = {
        let sync_name = sync_name.clone();
        let sync_desc = sync_desc.clone();
        move |which: u8| {
            if !can_edit_here.get() {
                return;
            }
            set_edit_name.set(sync_name.clone());
            set_edit_desc.set(sync_desc.clone());
            set_is_editing_name.set(which == 0);
            set_is_editing_desc.set(which == 1);
        }
    };

    let (name_click, name_dblclick) = name_click_handlers(
        move || on_toggle.run(()),
        {
            let begin = begin_text_edit.clone();
            move || begin(0)
        },
    );
    let (desc_click, desc_dblclick) = name_click_handlers(
        move || on_toggle.run(()),
        {
            let begin = begin_text_edit.clone();
            move || begin(1)
        },
    );

    let pf_image_input_ref = NodeRef::<leptos::html::Input>::new();
    let pf_name_input_ref = NodeRef::<leptos::html::Input>::new();
    let pf_desc_input_ref = NodeRef::<leptos::html::Input>::new();

    let do_save = move || {
        let n = edit_name.get();
        let d = edit_desc.get();
        if n.trim().is_empty() {
            return;
        }
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                p.name = n.clone();
                p.description = if d.trim().is_empty() {
                    None
                } else {
                    Some(d.clone())
                };
                p.updated_at = chrono::Utc::now();
            }
        });
        set_is_editing_name.set(false);
        set_is_editing_desc.set(false);
    };
    let save_edit_now = move || do_save();

    // Phantom-click dismissal: a press outside the editing inputs is
    // consumed and saves+closes the inline editor.
    let scope_token: &'static str =
        Box::leak(format!("pl-{}", pid).into_boxed_str());
    {
        let alive = phantom_edit_scope(
            scope_token.to_string(),
            move || {
                is_editing_name.get_untracked() || is_editing_desc.get_untracked()
            },
            move || save_edit_now(),
        );
        on_cleanup(move || alive.store(false, std::sync::atomic::Ordering::Relaxed));
    }

    // Focus the inline header editors as soon as they mount.
    Effect::new(move |_| {
        if is_editing_name.get() {
            if let Some(el) = pf_name_input_ref.get() {
                let _ = el.focus();
            }
        }
    });
    Effect::new(move |_| {
        if is_editing_desc.get() {
            if let Some(el) = pf_desc_input_ref.get() {
                let _ = el.focus();
            }
        }
    });

    // Deep-link scrolling from search / notifications.
    Effect::new(move |_| {
        let target = app_store.get().pending_scroll_target.clone();
        let slider_target = app_store.get().pending_doc_slider.clone();
        if let Some(crate::stores::ScrollTarget::Portfolio(id)) = target {
            if id == pid {
                crate::utils::dom::scroll_to_id(&format!("portfolio-{}", pid));
                app_store.update(|s| s.pending_scroll_target = None);
            }
        }
        if let Some(sid) = slider_target {
            if sid == pid {
                set_show_doc_slider.set(true);
                app_store.update(|s| {
                    s.pending_doc_slider = None;
                    if matches!(
                        s.pending_scroll_target,
                        Some(crate::stores::ScrollTarget::Document(_))
                    ) {
                        s.pending_scroll_target = None;
                    }
                });
            }
        }
    });

    let add_doc = move |n: String| {
        if n.trim().is_empty() {
            return;
        }
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
            s.add_document_to_portfolio(pid, doc, &mut notification_store.get_untracked());
        });
    };

    let portfolio_for_viewer = portfolio.clone();
    let assigned_users = portfolio.assigned_users.clone();

    // Reactive combined document list for the portfolio dropdown, including asset docs.
    let doc_entries = Memo::new(move |_| {
        let store = app_store.get();
        let mut entries = Vec::new();
        if let Some(p) = store.get_portfolio(pid) {
            for d in &p.documents {
                entries.push((
                    p.id,
                    p.name.clone(),
                    "Portfolio".to_string(),
                    d.clone(),
                    Some(p.id),
                    None,
                    None,
                ));
            }
            for g in &p.asset_groups {
                for d in &g.documents {
                    entries.push((
                        g.id,
                        g.name.clone(),
                        "Group".to_string(),
                        d.clone(),
                        None,
                        Some(g.id),
                        None,
                    ));
                }
                for a in &g.assets {
                    for d in &a.documents {
                        entries.push((
                            a.id,
                            a.name.clone(),
                            "Asset".to_string(),
                            d.clone(),
                            None,
                            None,
                            Some(a.id),
                        ));
                    }
                }
            }
            for a in &p.assets {
                for d in &a.documents {
                    entries.push((
                        a.id,
                        a.name.clone(),
                        "Asset".to_string(),
                        d.clone(),
                        None,
                        None,
                        Some(a.id),
                    ));
                }
            }
        }
        entries
    });
    let dropdown_entries = Memo::new(move |_| {
        doc_entries
            .get()
            .into_iter()
            .map(
                |(_, _, _, doc, portfolio_id, group_id, asset_id)| DocEntry {
                    doc,
                    portfolio_id: portfolio_id.or(Some(pid)),
                    group_id,
                    asset_id,
                    organization_id: None,
                },
            )
            .collect::<Vec<_>>()
    });
    let doc_count = Memo::new(move |_| dropdown_entries.get().len());
    let org_users = move || organization_store.get().organization_users.clone();

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

    // Drop target: assets/groups move in; portfolio+portfolio reorders the
    // dragged portfolio to the target's slot (Relevance position); org→portfolio
    // assigns the portfolio to that org
    let (drop_hover, set_drop_hover) = signal(false);
    let (d_over, d_leave, d_drop) = pf_drop_target(
        set_drop_hover,
        |k| matches!(
            k,
            NavTabKind::Asset
                | NavTabKind::AssetGroup
                | NavTabKind::Portfolio
                | NavTabKind::Organization
        ),
        move |tab, rel_y| match tab.kind {
            NavTabKind::Asset => {
                app_store.update(|s| {
                    s.move_asset_to_portfolio(tab.id, pid);
                });
            }
            NavTabKind::AssetGroup => {
                app_store.update(|s| {
                    s.move_group_to_portfolio(tab.id, pid);
                });
            }
            NavTabKind::Portfolio if tab.id != pid => {
                // Reorder in the currently displayed order: the dragged
                // portfolio takes the slot just above (top half) or below
                // (bottom half) the drop target, then positions are
                // rewritten densely and persisted.
                let all = app_store.get_untracked().portfolios.clone();
                let old_positions: std::collections::HashMap<Uuid, f64> = all
                    .iter()
                    .map(|p| (p.id, p.position))
                    .collect();
                let ui = ui_store.get_untracked();
                let ordered = super::sort_portfolios_for_ui(all, &ui);
                let mut ids: Vec<Uuid> = ordered.iter().map(|p| p.id).collect();
                ids.retain(|id| *id != tab.id);
                if let Some(tidx) = ids.iter().position(|id| *id == pid) {
                    let insert_at = if rel_y < 0.5 { tidx } else { tidx + 1 };
                    ids.insert(insert_at, tab.id);
                    app_store.update(|s| s.reorder_portfolios(&ids));
                    let changed: Vec<crate::models::Portfolio> = app_store
                        .get_untracked()
                        .portfolios
                        .iter()
                        .filter(|p| {
                            old_positions
                                .get(&p.id)
                                .map(|op| (*op - p.position).abs() > f64::EPSILON)
                                .unwrap_or(false)
                        })
                        .cloned()
                        .collect();
                    for p in changed {
                        leptos::task::spawn_local(async move {
                            let _ = crate::server::save_portfolio(p).await;
                        });
                    }
                }
            }
            NavTabKind::Organization => {
                app_store.update(|s| {
                    s.set_portfolio_organization(pid, Some(tab.id));
                });
            }
            _ => {}
        },
    );

    let on_context_touch = on_context.clone();
    let on_context_close_touch = on_context_close.clone();
    let (touch_start, touch_move, touch_end) = mobile_tab_gestures(
        NavTabKind::Portfolio,
        pid,
        name.clone(),
        move |x, y| on_context_touch(x, y),
        move || on_context_close_touch(),
    );

    view! {
        <div class="asset-group pf-portfolio" id={format!("portfolio-{}", pid)} class:expanded={expanded}
            style={pf_tint_style}
            class:drop-hover={move || drop_hover.get()}
            on:dragover=d_over
            on:dragleave=d_leave
            on:drop=d_drop
            on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                ev.prevent_default();
                ev.stop_propagation();
                on_context(ev.client_x(), ev.client_y());
            }
            on:touchstart=touch_start
            on:touchmove=touch_move
            on:touchend=touch_end
            draggable="true"
            on:dragstart=move |ev: leptos::ev::DragEvent| {
                set_drag_tab_data(&ev, &NavTabKind::Portfolio, pid, &name);
            }
        >
            // Header row — same structure as asset-group-header
            <div class="asset-group-header pf-accordion-header"
                class:editing={move || is_editing_name.get() || is_editing_desc.get()}
                role="button"
                tabindex="0"
                aria-expanded={move || expanded.get()}
                aria-controls={format!("pf-content-{}", pid)}
                aria-label={move || format!("{} portfolio. {} asset{}. {} document{}. {}",
                    name_for_aria,
                    asset_count,
                    if asset_count == 1 { "" } else { "s" },
                    doc_count.get(),
                    if doc_count.get() == 1 { "" } else { "s" },
                    if expanded.get() { "Expanded" } else { "Collapsed" }
                )}
                on:click=move |ev: leptos::ev::MouseEvent| {
                    ev.stop_propagation();
                    if !is_editing_name.get() && !is_editing_desc.get() {
                        on_toggle.run(());
                    }
                }
                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                    if ev.key() == "Enter" || ev.key() == " " {
                        ev.prevent_default();
                        ev.stop_propagation();
                        if !is_editing_name.get() && !is_editing_desc.get() {
                            on_toggle.run(());
                        }
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
                    node_ref=pf_image_input_ref
                    on:change=move |ev| {
                        read_image_as_data_url(&ev, {
                            let app_store = app_store.clone();
                            move |url: String| {
                                app_store.update(|s| {
                                    if let Some(p) = s.get_portfolio_mut(pid) {
                                        p.image_url = Some(url);
                                        p.updated_at = chrono::Utc::now();
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
                            if let Some(input) = pf_image_input_ref.get() {
                                let _ = input.click();
                            }
                        }
                    }
                >
                    {if let Some(ref url) = portfolio_image_url {
                        view! { <img class="pf-header-image" src={url.clone()} alt="Portfolio image" /> }.into_any()
                    } else {
                        view! { <span>{portfolio_emoji.clone()}</span> }.into_any()
                    }}
                </div>
                <div class="asset-group-info-wrap">
                    {let name_header = name.clone();
                    let desc_header = desc.clone();
                    move || {
                        let mut parts: Vec<leptos::prelude::AnyView> = Vec::new();
                        // Name
                        if is_editing_name.get() && can_edit_here.get() {
                            parts.push(view! {
                                <input class="pf-edit-input pf-edit-name-md" placeholder="Portfolio name"
                                    aria-label="Portfolio name"
                                    attr:data-pf-edit-scope=scope_token
                                    node_ref=pf_name_input_ref
                                    prop:value=move || edit_name.get()
                                    on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                    on:keydown=move |ev| {
                                        if ev.key() == "Enter" { save_edit_now(); }
                                        else if ev.key() == "Escape" { set_is_editing_name.set(false); set_is_editing_desc.set(false); }
                                    }
                                />
                            }.into_any());
                        } else {
                            parts.push(view! {
                                <div class="asset-group-name"
                                    on:click={name_click.clone()}
                                    on:dblclick={name_dblclick.clone()}
                                >{name_header.clone()}</div>
                            }.into_any());
                        }
                        // Description
                        if is_editing_desc.get() && can_edit_here.get() {
                            parts.push(view! {
                                <input class="pf-edit-input pf-edit-desc" placeholder="Description"
                                    aria-label="Description"
                                    attr:data-pf-edit-scope=scope_token
                                    node_ref=pf_desc_input_ref
                                    prop:value=move || edit_desc.get()
                                    on:input=move |ev| set_edit_desc.set(event_target_value(&ev))
                                    on:keydown=move |ev| {
                                        if ev.key() == "Enter" { save_edit_now(); }
                                        else if ev.key() == "Escape" { set_is_editing_name.set(false); set_is_editing_desc.set(false); }
                                    }
                                />
                            }.into_any());
                        } else if !desc_header.is_empty() {
                            parts.push(view! {
                                <div class="asset-group-desc"
                                    on:click={desc_click.clone()}
                                    on:dblclick={desc_dblclick.clone()}
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
                // Document icon for portfolio — toggles inline slider under the parent
                <div class="pf-list-actions" on:click=|ev| ev.stop_propagation()>
                    <button
                        class="pf-action-btn pf-doc-action-btn"
                        class:active=move || show_doc_slider.get()
                        aria-label={move || {
                            let count = dropdown_entries.get().len();
                            format!("View documents for {}. {} document{}", name_for_doc_btn, count, if count == 1 { "" } else { "s" })
                        }}
                        on:click=move |_| set_show_doc_slider.update(|v| *v = !*v)
                    >
                        <div class="pf-action-stack">
                            <span class="pf-action-icon">"📄"</span>
                            <span class="pf-action-count">{move || dropdown_entries.get().len()}</span>
                        </div>
                    </button>
                </div>
            </div>

            {move || if view_mode.get() == ViewMode::List
                && ui_store.get().is_red_listing_panel_open(pid)
                && can_edit_here.get()
            {
                view! {
                    <div class="pf-doc-slider-panel" on:click=|ev| ev.stop_propagation()>
                        <RedListingPanel
                            kind={RedEntityKind::Portfolio}
                            entity_id={pid}
                            entity_name={red_name.get_value()}
                            entity_desc={red_desc.get_value()}
                            entity_value={red_value}
                            image_url={red_img.get_value()}
                            can_edit={can_edit_here}
                        />
                    </div>
                }.into_any()
            } else { ().into_any() }}

            {move || if view_mode.get() == ViewMode::List && show_doc_slider.get() {
                view! {
                    <div class="pf-doc-slider-panel" on:click=|ev| ev.stop_propagation()>
                        <DocSlider
                            entity_id={pid}
                            title={name_for_slider.get_value()}
                            entity_name={name_for_slider.get_value()}
                            can_edit_documents={can_edit_documents_here}
                            entries={dropdown_entries}
                            on_add={Some(Callback::new(move |n: String| add_doc(n)))}
                            portfolio_id={Some(pid)}
                        />
                    </div>
                }.into_any()
            } else { ().into_any() }}

            {move || if view_mode.get() == ViewMode::List && ui_store.get().is_assignment_panel_open(pid) && can_edit_here.get() {
                let users = org_users();
                let assigned = assigned_users.clone();
                view! {
                    <UserAssignmentPanel assigned={assigned} users={users} on_toggle={toggle_portfolio_assignment} />
                }.into_any()
            } else { ().into_any() }}

            // Expanded content — AssetViewer
            {move || if view_mode.get() == ViewMode::List {
                let portfolio_for_viewer = portfolio_for_viewer.clone();
                view! {
                    <div id={format!("pf-content-{}", pid)} class="asset-group-content" class:hidden={move || !expanded.get()}>
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
                    child_depth={tint_index + 1}
                    />
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </div>
    }.into_any()
}
