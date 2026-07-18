use crate::models::Asset;
use crate::stores::{
    use_app_store, use_notification_store, use_organization_store, use_transaction_store,
    use_ui_store,
};
use crate::types::{AssetType, ViewMode};
use leptos::prelude::*;
use uuid::Uuid;

use super::{
    detect_file_type, document_icon, read_images_as_data_urls, shorthand_name,
    AddChannelModal, AssetBookingControls, AssetChannelsSection, AssetLinkingControls, DocModal,
    DocumentViewer, UserAssignmentPanel,
};

pub(crate) fn asset_placeholder_url(asset_type: &AssetType, name: &str) -> String {
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
    format!(
        "https://placehold.co/400x400/2d3748/FFF?text={}%2B{}",
        text, seed
    )
}

#[component]
pub(crate) fn AssetItem(
    asset: Asset,
    portfolio_name: String,
    #[prop(default = None)] portfolio_id: Option<Uuid>,
    #[prop(default = None)] group_id: Option<Uuid>,
    view_mode: ViewMode,
    on_select: Callback<Asset>,
    #[prop(into)] can_edit: Signal<bool>,
    #[prop(into)] can_edit_documents: Signal<bool>,
    #[prop(default = 0)] tint_index: usize,
    #[prop(default = false)] collapsible: bool,
    #[prop(default = None)] highlight: Option<Signal<Option<Uuid>>>,
) -> impl IntoView {
    let app_store = use_app_store();
    let notification_store = use_notification_store();
    let transaction_store = use_transaction_store();
    let ui_store = use_ui_store();
    let image_url = asset
        .images
        .first()
        .cloned()
        .unwrap_or_else(|| asset_placeholder_url(&asset.asset_type, &asset.name));

    let asset_id = asset.id;
    let has_channels = Memo::new(move |_| !app_store.get().channels_for_asset(asset_id).is_empty());
    let asset_for_highlight = asset.clone();
    let (expanded_detail, set_expanded_detail) = signal(false);
    let (collapsed, set_collapsed) = signal(collapsible);
    let (editing, set_editing) = signal(false);
    let (asset_context_menu, set_asset_context_menu) = signal(Option::<(i32, i32)>::None);
    let item_ref = NodeRef::<leptos::html::Div>::new();

    // When the parent marks this asset as highlighted, open it in the view:
    // expand the list item and scroll it into view, or select it in grid view.
    Effect::new(move |_| {
        if let Some(h) = highlight {
            if let Some(id) = h.get() {
                if id == asset_id {
                    if view_mode == ViewMode::List {
                        set_collapsed.set(false);
                    } else {
                        on_select.run(asset_for_highlight.clone());
                    }
                    if let Some(el) = item_ref.get() {
                        el.scroll_into_view();
                    }
                }
            }
        }
    });
    let (show_add_user, set_show_add_user) = signal(false);
    let (show_add_role, set_show_add_role) = signal(false);
    let (show_add_org, set_show_add_org) = signal(false);
    let (show_add_transaction, set_show_add_transaction) = signal(false);
    let (confirm_asset_remove, set_confirm_asset_remove) = signal(false);
    let (show_add_channel, set_show_add_channel) = signal(false);
    let (show_add_booking, set_show_add_booking) = signal(false);
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
    let user_id = app_store.get().current_user.id;
    let assigned_workers = asset.assigned_workers.clone();
    let can_add_images = Memo::new(move |_| can_edit_here.get() || assigned_workers.contains(&user_id));
    // doc sort: 0 = recent, 1 = name
    let (_doc_sort, _set_doc_sort) = signal(0u8);
    let (_detail_tab, _set_detail_tab) = signal(0u8);

    let asset_id = asset.id;
    let pname = portfolio_name.clone();
    let docs = asset.documents.clone();
    let _doc_count = docs.len();

    // Reactive document list for this asset (read from store so it updates on add)
    let asset_docs_reactive = Memo::new(move |_| {
        app_store
            .get()
            .portfolios
            .iter()
            .flat_map(|p| {
                p.assets
                    .iter()
                    .chain(p.asset_groups.iter().flat_map(|g| g.assets.iter()))
            })
            .find(|a| a.id == asset_id)
            .map(|a| a.documents.clone())
            .unwrap_or_default()
    });

    // Reactive image list for this asset
    let default_asset_images = asset.images.clone();
    let asset_images = Memo::new(move |_| {
        app_store
            .get()
            .portfolios
            .iter()
            .flat_map(|p| {
                p.assets
                    .iter()
                    .chain(p.asset_groups.iter().flat_map(|g| g.assets.iter()))
            })
            .find(|a| a.id == asset_id)
            .map(|a| a.images.clone())
            .unwrap_or_else(|| default_asset_images.clone())
    });

    let max_images = if asset.organization_id.is_some() { 100usize } else { 50usize };

    let add_image = Callback::new(move |url: String| {
        if let Some(pid) = portfolio_id {
            app_store.update(|s| {
                if let Some(p) = s.get_portfolio_mut(pid) {
                    let all: Vec<_> = p
                        .assets
                        .iter_mut()
                        .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                        .collect();
                    for a in all {
                        if a.id == asset_id && a.images.len() < max_images {
                            a.images.push(url.clone());
                            break;
                        }
                    }
                }
            });
        }
    });

    let a_name = asset.name.clone();
    let a_addr = asset.location.clone().unwrap_or_default();
    let a_addr_grid = if a_addr.is_empty() { "\u{00A0}".to_string() } else { a_addr.clone() };
    let a_name_tx = a_name.clone();
    let a_org_id = asset.organization_id;
    let organization_store = use_organization_store();

    // Permission-based flags for linking and booking controls
    let current_user_id = app_store.get().current_user.id;
    let can_link = move || {
        if a_org_id.is_none() {
            return can_edit.get();
        }
        let oid = a_org_id.unwrap();
        organization_store
            .get()
            .user_has_perm_in_org(oid, current_user_id, &crate::models::Perm::EditDirectAssetLinking)
            || organization_store
                .get()
                .user_has_perm_in_org(oid, current_user_id, &crate::models::Perm::EditChannels)
    };
    let can_book = move || {
        if a_org_id.is_none() {
            return can_edit.get();
        }
        let oid = a_org_id.unwrap();
        organization_store
            .get()
            .user_has_perm_in_org(oid, current_user_id, &crate::models::Perm::CreateDirectAssetBookings)
            || organization_store
                .get()
                .user_has_perm_in_org(oid, current_user_id, &crate::models::Perm::CreateBookings)
    };
    let a_org_name = move || {
        organization_store
            .get()
            .organizations
            .iter()
            .find(|o| Some(o.id) == a_org_id)
            .map(|o| o.name.clone())
            .unwrap_or_else(|| "—".to_string())
    };
    let asset_name_for_modal = asset.name.clone();
    let asset_name_for_confirm = asset.name.clone();
    let (_asset_name_signal, _set_asset_name) = signal(a_name.clone());
    // snapshot values for the detail panel
    let a_type = format!("{:?}", asset.asset_type);
    let a_type_grid = a_type.clone();
    let _a_desc = asset.description.clone().unwrap_or_else(|| "—".to_string());
    let _a_status = format!("{:?}", asset.status);
    let _a_purchase_val = asset.purchase_value;
    let a_current_val = asset.current_value;
    let _a_pl = asset.profit_loss;
    let _a_pl_pct = asset.profit_loss_percent;
    let _a_revenue = asset.revenue;
    let _a_pl_cls = if asset.profit_loss >= 0.0 {
        "positive"
    } else {
        "negative"
    };
    let _a_purchase_date = asset.purchase_date.format("%d %b %Y").to_string();

    let save_edit = move || {
        let n = edit_name.get();
        let d = edit_desc.get();
        let l = edit_loc.get();
        if n.trim().is_empty() {
            return;
        }
        app_store.update(|s| {
            for p in s.portfolios.iter_mut() {
                let all: Vec<_> = p
                    .assets
                    .iter_mut()
                    .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                    .collect();
                for a in all {
                    if a.id == asset_id {
                        a.name = n.clone();
                        a.description = if d.trim().is_empty() {
                            None
                        } else {
                            Some(d.clone())
                        };
                        a.location = if l.trim().is_empty() {
                            None
                        } else {
                            Some(l.clone())
                        };
                        return;
                    }
                }
            }
        });
        set_editing.set(false);
    };

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
            for p in s.portfolios.iter_mut() {
                let all: Vec<_> = p
                    .assets
                    .iter_mut()
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
    let add_cb = if can_edit_documents_here.get() {
        Some(Callback::new(add_doc))
    } else {
        None
    };

    let asset_id_for_assign = asset_id;
    let portfolio_id_for_assign = portfolio_id;
    let toggle_asset_assignment = Callback::new(move |uid: Uuid| {
        let aid = asset_id_for_assign;
        app_store.update(|s| {
            for p in s.portfolios.iter_mut() {
                let all: Vec<_> = p
                    .assets
                    .iter_mut()
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
        app_store
            .get()
            .portfolios
            .iter()
            .flat_map(|p| {
                p.assets
                    .iter()
                    .chain(p.asset_groups.iter().flat_map(|g| g.assets.iter()))
            })
            .find(|a| a.id == asset_id)
            .map(|a| a.assigned_workers.clone())
            .unwrap_or_default()
    };
    let get_org_users = move || organization_store.get().organization_users.clone();

    let tint_style = format!(
        "background: rgba(255,255,255,{});",
        (tint_index as f64 * 0.1).min(0.9)
    );

    if view_mode == ViewMode::Grid {
        let asset_for_click = asset.clone();
        let short_name = shorthand_name(&a_name);
        view! {
            <div class="asset-grid-card" node_ref=item_ref style={tint_style.clone()} aria-label={format!("Asset {}. Type {}. In {}", a_name, a_type_grid, pname)} on:click=move |_| on_select.run(asset_for_click.clone())>
                <img class="asset-grid-image" src={image_url.clone()} alt={a_name.clone()} />
                <div class="asset-grid-name">{short_name}</div>
            </div>
        }.into_any()
    } else {
        let asset_id_for_toggle = asset_id;
        let content_id = format!("ai-content-{}", asset_id);
        let content_id_for_header = content_id.clone();
        let a_name_header = a_name.clone();
        let a_type_header = a_type_grid.clone();
        let a_val_header = a_current_val;
        let image_url_header = image_url.clone();
        let (asset_name_signal, _set_asset_name) = signal(a_name.clone());
        view! {
        <div class="ai-item"
            node_ref=item_ref
            class:ai-item-expanded={move || expanded_detail.get()}
            class:ai-item-collapsible={collapsible}
            class:ai-item-collapsed={move || collapsed.get()}
            style={tint_style.clone()}
            aria-label={format!("Asset {}. Type {}. In {}. {}", a_name, a_type_grid, pname, if a_addr.is_empty() { "No address set" } else { a_addr.as_str() })}
            on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                if can_edit_here.get() {
                    ev.prevent_default();
                    ev.stop_propagation();
                    set_asset_context_menu.set(Some((ev.client_x(), ev.client_y())));
                }
            }
        >
            {move || if collapsible {
                view! {
                    <div class="ai-collapsible-header"
                        id={format!("ai-header-{}", asset_id_for_toggle)}
                        role="button"
                        tabindex="0"
                        aria-expanded={move || !collapsed.get()}
                        aria-controls={content_id_for_header.clone()}
                        aria-label={format!("Asset {}. Type {}. {}", a_name_header, a_type_header, if collapsed.get() { "Collapsed" } else { "Expanded" })}
                        on:click=move |_| if !editing.get() { set_collapsed.update(|v| *v = !*v) }
                        on:dblclick=move |ev| { if can_edit_here.get() { ev.stop_propagation(); set_editing.set(true); set_collapsed.set(false); } }
                        on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                            if ev.key() == "Enter" || ev.key() == " " {
                                ev.prevent_default();
                                if !editing.get() { set_collapsed.update(|v| *v = !*v); }
                            }
                        }
                    >
                        <img class="ai-list-image" src={image_url_header.clone()} alt={a_name_header.clone()} />
                        <div class="ai-collapsible-summary">
                            <div class="ai-collapsible-name">{a_name_header.clone()}</div>
                            <div class="ai-collapsible-meta">{format!("{} · ${:.2}", a_type_header, a_val_header)}</div>
                            {let channel_count = asset.channel_ids.len();
                            let channel_ids = asset.channel_ids.clone();
                            move || if !channel_ids.is_empty() {
                                view! {
                                    <div class="ai-channel-badge" title={format!("{} channel(s)", channel_count)}>
                                        "📡" {channel_count}
                                    </div>
                                }.into_any()
                            } else { ().into_any() }}
                        </div>
                        {let a_name_add_btn = a_name_header.clone();
                        move || if can_add_images.get() && asset_images.get().len() < max_images { view! {
                            <label class="ai-add-image-btn" aria-label={format!("Add image to {}", a_name_add_btn)} on:click=|ev| ev.stop_propagation()>
                                <input
                                    type="file"
                                    accept="image/*"
                                    multiple
                                    class="ai-image-file-input"
                                    on:change=move |ev| {
                                        read_images_as_data_urls(&ev, {
                                            let add_image = add_image.clone();
                                            move |url| add_image.run(url)
                                        });
                                    }
                                />
                                <span>"➕"</span>
                                <span>"Image"</span>
                            </label>
                        }.into_any() } else { ().into_any() }}
                        <span class="ai-collapsible-arrow" aria-hidden="true">
                            {move || if collapsed.get() { "▶" } else { "▼" }}
                        </span>
                    </div>
                }.into_any()
            } else { ().into_any() }}
            <div class="ai-list-card" id={content_id.clone()}>
                <div class="ai-list-body">
                    {move || if can_edit_here.get() && editing.get() {
                        view! {
                            <div class="ai-edit-stack">
                                <input class="pf-edit-input" placeholder="Asset name"
                                    aria-label="Asset name"
                                    prop:value=move || edit_name.get()
                                    on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                    on:blur=move |_| save_edit() />
                                <input class="pf-edit-input" placeholder="Description"
                                    aria-label="Description"
                                    prop:value=move || edit_desc.get()
                                    on:input=move |ev| set_edit_desc.set(event_target_value(&ev))
                                    on:blur=move |_| save_edit() />
                                <input class="pf-edit-input" placeholder="Location / Address"
                                    aria-label="Location / Address"
                                    prop:value=move || edit_loc.get()
                                    on:input=move |ev| set_edit_loc.set(event_target_value(&ev))
                                    on:blur=move |_| save_edit() />
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                    // Image slider with upload outline
                    <div class="ai-image-slider" on:click=|ev| ev.stop_propagation()>
                        {{
                            let a_name_slider = a_name.clone();
                            move || {
                                let images = asset_images.get();
                                let count = images.len();
                                let can_add = can_add_images.get() && count < max_images;
                                let a_name_add = a_name_slider.clone();
                                let a_name_images = a_name_slider.clone();
                                view! {
                                    {if can_add {
                                        view! {
                                            <div class="ai-image-slider-item ai-image-add-card" aria-label={format!("Add image to {} (max {})", a_name_add, max_images)}>
                                                <input
                                                    type="file"
                                                    accept="image/*"
                                                    multiple
                                                    class="ai-image-file-input"
                                                    on:change=move |ev| {
                                                        read_images_as_data_urls(&ev, {
                                                            let add_image = add_image.clone();
                                                            move |url| add_image.run(url)
                                                        });
                                                    }
                                                />
                                                <span class="ai-image-add-icon">"➕"</span>
                                                <span class="ai-image-add-label">"Image"</span>
                                            </div>
                                        }.into_any()
                                    } else { ().into_any() }}
                                    <For
                                        each=move || asset_images.get()
                                        key=|url| url.clone()
                                        children=move |url: String| {
                                            let a_name_images = a_name_images.clone();
                                            view! {
                                                <div class="ai-image-slider-item">
                                                    <img class="ai-image-slider-img" src={url} alt={format!("Image of {}", a_name_images)} />
                                                </div>
                                            }
                                        }
                                    />
                                }.into_any()
                            }
                        }}
                    </div>
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
                    {move || if has_channels.get() { view! {
                        <div class="ai-controls-row">
                            <AssetLinkingControls
                                asset_id={asset_id}
                                asset_name={asset_name_signal.get()}
                                portfolio_id={portfolio_id}
                                can_link={can_link()}
                            />
                            <AssetBookingControls
                                asset_id={asset_id}
                                asset_name={asset_name_signal.get()}
                                portfolio_id={portfolio_id}
                                can_book={can_book()}
                            />
                        </div>
                    }.into_any() } else { ().into_any() }}
                    // Horizontal document slider with + Document card
                    <div class="ai-doc-slider" on:click=|ev| ev.stop_propagation()>
                        // + Document card (always first)
                        <div class="ai-doc-slider-item ai-doc-add-card"
                            aria-label={format!("Add document to {}", a_name)}
                            on:click=move |_| ui_store.update(|s| s.toggle_doc_modal(asset_id))>
                            <div class="ai-doc-slider-thumb">"➕"</div>
                            <div class="ai-doc-slider-name">"+ Document"</div>
                            <div class="ai-doc-slider-type">"ADD"</div>
                        </div>
                        <For
                            each=move || asset_docs_reactive.get()
                            key=|doc| doc.id
                            children=move |doc| {
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
                                        aria-label={format!("View document {}. Type {}", dname, ft)}
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
                                            let ncount = notification_store.get().notifications_for_doc(doc_id_for_notif);
                                            if ncount > 0 {
                                                view! {
                                                    <span class="pf-notif-badge pf-notif-badge-inline" role="status" aria-live="polite"
                                aria-label={format!("{} pending document review{}", ncount, if ncount == 1 { "" } else { "s" })}
                                title={format!("{} notification{}", ncount, if ncount == 1 { "" } else { "s" })}>
                                                        "🔔"
                                                        <span class="pf-notif-count" aria-hidden="true">{ncount}</span>
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
                                                        can_edit={can_edit_documents_here.get()}
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
                            }
                        />
                    </div>
                </div>
            </div>

            {move || if ui_store.get().is_doc_modal_open(asset_id) {
                let mt = asset_name_for_modal.clone();
                let ac = add_cb.clone();
                view! {
                    <DocModal
                        entity_id={asset_id}
                        title={mt}
                        on_close=move || ui_store.update(|s| s.close_doc_modal(asset_id))
                        can_edit={can_edit_documents_here.get()}
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
                                    set_show_add_channel.set(true);
                                }
                            >"📡 Add Channel"</button>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_asset_context_menu.set(None);
                                    set_show_add_booking.set(true);
                                }
                            >"Add Booking"</button>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_asset_context_menu.set(None);
                                    ui_store.update(|s| s.toggle_doc_modal(asset_id));
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
                                <button class="doc-modal-close" aria-label="Close add user" on:click=move |_| set_show_add_user.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="User name"
                                    aria-label="User name"
                                    prop:value={move || new_user_name.get()}
                                    on:input=move |ev| set_new_user_name.set(event_target_value(&ev)) />
                                <input class="login-input" type="email" placeholder="Email"
                                    aria-label="Email"
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
                                        organization_store.update(|s| {
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
                                <button class="doc-modal-close" aria-label="Close add role" on:click=move |_| set_show_add_role.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="Role name"
                                    aria-label="Role name"
                                    prop:value={move || new_role_name.get()}
                                    on:input=move |ev| set_new_role_name.set(event_target_value(&ev)) />
                                <input class="login-input" type="text" placeholder="Description"
                                    aria-label="Description"
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
                                            organization_store.update(|s| s.add_role_to_org(oid, role));
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
                                <button class="doc-modal-close" aria-label="Close add organization" on:click=move |_| set_show_add_org.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="Organization name"
                                    aria-label="Organization name"
                                    prop:value={move || new_org_name.get()}
                                    on:input=move |ev| set_new_org_name.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| {
                                    let name = new_org_name.get();
                                    if !name.trim().is_empty() {
                                        let owner_id = app_store.get().current_user.id;
                                        let org = crate::models::Organization::new(name, owner_id);
                                        let oid = org.id;
                                        organization_store.update(|s| s.add_organization(org));
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
                                <button class="doc-modal-close" aria-label="Close add transaction" on:click=move |_| set_show_add_transaction.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <select class="login-input"
                                    aria-label="Transaction type"
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
                                    aria-label="Amount"
                                    prop:value={move || new_tx_amount.get()}
                                    on:input=move |ev| set_new_tx_amount.set(event_target_value(&ev)) />
                                <input class="login-input" type="text" placeholder="Description"
                                    aria-label="Description"
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
                                    transaction_store.update(|s| s.add_transaction(tx));
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
                                <button class="doc-modal-close" aria-label={format!("Cancel removal of {} asset", asset_name_for_confirm)} on:click=move |_| set_confirm_asset_remove.set(false)>"✕"</button>
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
                if expanded_detail.get() && can_edit_here.get() {
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
                                    <AssetChannelsSection asset_id={asset_id} asset_name={asset_name_signal.get()} portfolio_id={portfolio_id} can_edit={can_edit_here.get()} />
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else { ().into_any() }
            }}

            {move || if show_add_channel.get() { view! {
                <AddChannelModal
                    asset_id={asset_id}
                    asset_name={asset_name_signal.get()}
                    portfolio_id={portfolio_id}
                    on_close={Callback::new(move |_| set_show_add_channel.set(false))}
                />
            }.into_any() } else { ().into_any() }}

            {move || if show_add_booking.get() { view! {
                <div class="doc-modal-overlay" on:click=move |_| set_show_add_booking.set(false)>
                    <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                        <div class="doc-modal-header">
                            <span>"Booking Controls"</span>
                            <button class="doc-modal-close" aria-label="Close booking controls" on:click=move |_| set_show_add_booking.set(false)>"✕"</button>
                        </div>
                        <div class="doc-modal-body">
                            <AssetBookingControls
                                asset_id={asset_id}
                                asset_name={asset_name_signal.get()}
                                portfolio_id={portfolio_id}
                                can_book={can_book()}
                            />
                        </div>
                    </div>
                </div>
            }.into_any() } else { ().into_any() }}
        </div>
    }.into_any()
    }
}

#[component]
pub(crate) fn AssetDetailView(
    asset: Asset,
    #[prop(default = None)] portfolio_id: Option<Uuid>,
    #[prop(into)] can_edit: Signal<bool>,
    on_close: Callback<()>,
) -> impl IntoView {
    let app_store = use_app_store();
    let asset_id = asset.id;
    let user_id = app_store.get().current_user.id;
    let assigned_workers = asset.assigned_workers.clone();
    let can_add_images = Memo::new(move |_| can_edit.get() || assigned_workers.contains(&user_id));
    let (local_asset, set_local_asset) = signal(asset);

    let add_image = Callback::new(move |url: String| {
        if let Some(pid) = portfolio_id {
            app_store.update(|s| {
                if let Some(p) = s.get_portfolio_mut(pid) {
                    let max = if local_asset.get().organization_id.is_some() { 100 } else { 50 };
                    let all: Vec<_> = p
                        .assets
                        .iter_mut()
                        .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                        .collect();
                    for a in all {
                        if a.id == asset_id && a.images.len() < max {
                            a.images.push(url.clone());
                            break;
                        }
                    }
                }
            });
        }
        set_local_asset.update(|a| {
            let max = if a.organization_id.is_some() { 100 } else { 50 };
            if a.images.len() < max {
                a.images.push(url.clone());
            }
        });
    });

    view! {
        {move || {
            let asset = local_asset.get();
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
            let max_images = if asset.organization_id.is_some() { 100 } else { 50 };
            let can_add = can_add_images.get() && asset.images.len() < max_images;
            let name = asset.name.clone();
            let images = asset.images;

            view! {
                <div class="asset-detail-overlay" on:click=move |_| on_close.run(())>
                    <div class="asset-detail" on:click=|ev| ev.stop_propagation()>
                        <div class="asset-detail-header">
                            <div class="asset-detail-icon">{icon}</div>
                            <div class="asset-detail-title">{name.clone()}</div>
                            <button class="asset-detail-close" aria-label={format!("Close details for {}", name)} on:click=move |_| on_close.run(())>"✕"</button>
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
                                <span class={format!("asset-detail-value {}", pl_class)}
                                    aria-label={format!("Profit/Loss is {}: ${:+.0}K", if asset.profit_loss >= 0.0 { "positive" } else { "negative" }, asset.profit_loss / 1000.0)}>
                                    {format!("${:+.0}K", asset.profit_loss / 1000.0)}
                                </span>
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
                                {if can_add { view! {
                                    <div class="asset-detail-img asset-detail-add-image" title="Add image">
                                        <input
                                            type="file"
                                            accept="image/*"
                                            multiple
                                            class="ai-image-file-input"
                                            on:change=move |ev| {
                                                read_images_as_data_urls(&ev, {
                                                    let cb = add_image.clone();
                                                    move |url| cb.run(url)
                                                });
                                            }
                                        />
                                        <span>"➕"</span>
                                    </div>
                                }.into_any() } else { ().into_any() }}
                                {if images.is_empty() && !can_add {
                                    view! { <div class="asset-detail-no-image">"No images"</div> }.into_any()
                                } else {
                                    images.into_iter().map(|url| view! {
                                        <img class="asset-detail-img" src={url} alt={format!("Image of {}", name)} />
                                    }).collect::<Vec<_>>().into_any()
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            }
        }}
    }
}
