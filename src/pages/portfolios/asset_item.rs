use crate::models::Asset;
use crate::stores::{use_app_store, use_organization_store, use_transaction_store, use_ui_store};
use crate::types::{AssetType, ViewMode};
use leptos::prelude::*;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use uuid::Uuid;

use super::{
    clamp_context_menu, detect_file_type, document_icon, download_document, name_click_handlers,
    read_images_as_data_urls, shorthand_name, AddChannelModal, AssetChannelsSection, DocModal,
    DocumentViewer, LinkingBookingModal, UserAssignmentPanel,
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
    let transaction_store = use_transaction_store();
    let ui_store = use_ui_store();
    let asset_id = asset.id;
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
    let asset_type_for_placeholder = asset.asset_type.clone();
    let asset_name_for_placeholder = asset.name.clone();
    let image_url: Signal<String> = Signal::derive(move || {
        asset_images.get().first().cloned().unwrap_or_else(|| {
            asset_placeholder_url(&asset_type_for_placeholder, &asset_name_for_placeholder)
        })
    });
    let max_images = if asset.organization_id.is_some() {
        100usize
    } else {
        50usize
    };
    let asset_for_highlight = asset.clone();
    let (expanded_detail, set_expanded_detail) = signal(false);
    let (collapsed, set_collapsed) = signal(collapsible);
    let (editing, set_editing) = signal(false);
    let (asset_context_menu, set_asset_context_menu) = signal(Option::<(i32, i32)>::None);
    let (dragged_idx, set_dragged_idx) = signal(None::<usize>);
    let (drag_over_idx, set_drag_over_idx) = signal(None::<usize>);
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
    let (show_move_to_portfolio, set_show_move_to_portfolio) = signal(false);
    let (show_move_to_group, set_show_move_to_group) = signal(false);
    let (show_add_transaction, set_show_add_transaction) = signal(false);
    let (confirm_asset_remove, set_confirm_asset_remove) = signal(false);
    let (show_add_channel, set_show_add_channel) = signal(false);
    let (show_linking_booking, set_show_linking_booking) = signal(false);
    let (asset_target_portfolio_id, set_asset_target_portfolio_id) = signal(String::new());
    let (asset_target_group_id, set_asset_target_group_id) = signal(String::new());
    let (asset_org_id, set_asset_org_id) = signal(String::new());
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
    let (edit_asset_type, set_edit_asset_type) = signal(asset.asset_type.clone());
    let (edit_asset_subtype, set_edit_asset_subtype) =
        signal(asset.asset_subtype.clone().unwrap_or_default());
    let (edit_org_id, set_edit_org_id) = signal(
        asset
            .organization_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
    );
    let (editing_type_build, set_editing_type_build) = signal(false);
    let (editing_address, set_editing_address) = signal(false);
    let (editing_org, set_editing_org) = signal(false);
    let (address_suggestions, set_address_suggestions) = signal(Vec::<String>::new());

    let can_edit_here = can_edit;
    let can_edit_documents_here = can_edit_documents;

    let (name_click, name_dblclick) = name_click_handlers(
        move || set_collapsed.update(|v| *v = !*v),
        move || {
            if can_edit_here.get() {
                set_editing_type_build.set(false);
                set_editing_address.set(false);
                set_editing_org.set(false);
                set_editing.set(true);
                set_collapsed.set(false);
            }
        },
    );
    let user_id = app_store.get().current_user.id;
    let assigned_workers = asset.assigned_workers.clone();
    let can_reorder_images = Memo::new(move |_| {
        can_edit_here.get() || can_edit_documents_here.get() || assigned_workers.contains(&user_id)
    });
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

    let reorder_images = Callback::new(move |args: (usize, usize)| {
        let (from, to) = args;
        if from == to {
            return;
        }
        if let Some(pid) = portfolio_id {
            app_store.update(|s| {
                if let Some(p) = s.get_portfolio_mut(pid) {
                    let all: Vec<_> = p
                        .assets
                        .iter_mut()
                        .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                        .collect();
                    for a in all {
                        if a.id == asset_id && from < a.images.len() && to < a.images.len() {
                            let item = a.images.remove(from);
                            let to_index = if from < to { to - 1 } else { to };
                            a.images.insert(to_index, item);
                        }
                    }
                }
            });
        }
    });

    let a_name = asset.name.clone();
    let a_addr = Memo::new(move |_| {
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
            .and_then(|a| a.location.clone())
            .unwrap_or_default()
    });
    let a_addr_grid = Memo::new(move |_| {
        let addr = a_addr.get();
        if addr.is_empty() {
            "\u{00A0}".to_string()
        } else {
            addr
        }
    });
    let a_name_tx = a_name.clone();
    let a_org_id = asset.organization_id;
    let addr_datalist_id = format!("addr-datalist-{}", asset_id);
    let type_datalist_id = format!("type-datalist-{}", asset_id);
    let subtype_datalist_id = format!("subtype-datalist-{}", asset_id);
    let organization_store = use_organization_store();

    // Permission-based flags for linking and booking controls
    let current_user_id = app_store.get().current_user.id;
    let can_link = move || {
        if a_org_id.is_none() {
            return can_edit.get();
        }
        let oid = a_org_id.unwrap();
        organization_store.get().user_has_perm_in_org(
            oid,
            current_user_id,
            &crate::models::Perm::EditDirectAssetLinking,
        ) || organization_store.get().user_has_perm_in_org(
            oid,
            current_user_id,
            &crate::models::Perm::EditChannels,
        )
    };
    let can_book = move || {
        if a_org_id.is_none() {
            return can_edit.get();
        }
        let oid = a_org_id.unwrap();
        organization_store.get().user_has_perm_in_org(
            oid,
            current_user_id,
            &crate::models::Perm::CreateDirectAssetBookings,
        ) || organization_store.get().user_has_perm_in_org(
            oid,
            current_user_id,
            &crate::models::Perm::CreateBookings,
        )
    };

    let can_view_images = move || {
        if a_org_id.is_none() {
            return true;
        }
        let oid = a_org_id.unwrap();
        organization_store.get().user_has_perm_in_org(
            oid,
            current_user_id,
            &crate::models::Perm::ViewAssetImages,
        )
    };
    let can_upload_images = move || {
        if a_org_id.is_none() {
            return can_edit.get();
        }
        let oid = a_org_id.unwrap();
        organization_store.get().user_has_perm_in_org(
            oid,
            current_user_id,
            &crate::models::Perm::UploadAssetImages,
        )
    };
    let can_view_documents = move || {
        if a_org_id.is_none() {
            return true;
        }
        let oid = a_org_id.unwrap();
        let store = organization_store.get();
        store.user_has_perm_in_org(
            oid,
            current_user_id,
            &crate::models::Perm::ViewAssetDocuments,
        ) || store.user_has_perm_in_org(oid, current_user_id, &crate::models::Perm::ViewDocuments)
    };
    let can_upload_documents = move || {
        if a_org_id.is_none() {
            return can_edit_documents.get();
        }
        let oid = a_org_id.unwrap();
        let store = organization_store.get();
        store.user_has_perm_in_org(
            oid,
            current_user_id,
            &crate::models::Perm::UploadAssetDocuments,
        ) || store.user_has_perm_in_org(oid, current_user_id, &crate::models::Perm::UploadDocuments)
    };
    let can_view_linking = move || {
        if a_org_id.is_none() {
            return can_link();
        }
        let oid = a_org_id.unwrap();
        let store = organization_store.get();
        store.user_has_perm_in_org(
            oid,
            current_user_id,
            &crate::models::Perm::ViewDirectAssetLinking,
        ) || store.user_has_perm_in_org(
            oid,
            current_user_id,
            &crate::models::Perm::EditDirectAssetLinking,
        ) || store.user_has_perm_in_org(oid, current_user_id, &crate::models::Perm::ViewChannels)
            || store.user_has_perm_in_org(oid, current_user_id, &crate::models::Perm::EditChannels)
            || can_link()
    };
    let can_view_booking = move || {
        if a_org_id.is_none() {
            return can_book();
        }
        let oid = a_org_id.unwrap();
        let store = organization_store.get();
        store.user_has_perm_in_org(
            oid,
            current_user_id,
            &crate::models::Perm::ViewDirectAssetBookings,
        ) || store.user_has_perm_in_org(oid, current_user_id, &crate::models::Perm::ViewBookings)
            || store.user_has_perm_in_org(
                oid,
                current_user_id,
                &crate::models::Perm::CreateDirectAssetBookings,
            )
            || store.user_has_perm_in_org(
                oid,
                current_user_id,
                &crate::models::Perm::CreateBookings,
            )
            || can_book()
    };
    let can_view_general_info = move || {
        if a_org_id.is_none() {
            return true;
        }
        let oid = a_org_id.unwrap();
        organization_store.get().user_has_perm_in_org(
            oid,
            current_user_id,
            &crate::models::Perm::ViewAssetGeneralInformation,
        )
    };
    let can_view_content =
        move || can_view_images() || can_view_documents() || can_view_general_info();
    if !can_view_content() {
        return ().into_any();
    }

    let a_org_name = move || {
        organization_store
            .get()
            .organizations
            .iter()
            .find(|o| Some(o.id) == a_org_id)
            .map(|o| o.name.clone())
            .unwrap_or_else(|| "—".to_string())
    };
    let available_orgs = Memo::new(move |_| {
        organization_store
            .get()
            .organizations
            .iter()
            .filter(|o| o.members.contains(&current_user_id) || o.owner_id == current_user_id)
            .cloned()
            .collect::<Vec<_>>()
    });
    let asset_name_for_confirm = asset.name.clone();
    let (_asset_name_signal, _set_asset_name) = signal(a_name.clone());
    // snapshot values for the detail panel
    let a_type = format!("{:?}", asset.asset_type);
    let a_type_grid = a_type.clone();
    let a_subtype_grid = asset
        .asset_subtype
        .clone()
        .unwrap_or_else(|| "—".to_string());
    let _a_desc = asset.description.clone().unwrap_or_else(|| "—".to_string());
    let a_status_badges = asset.status_badges();
    let a_status_label = a_status_badges
        .iter()
        .map(|(label, value)| format!("{}: {}", label, value))
        .collect::<Vec<_>>()
        .join(", ");
    let _a_purchase_val = asset.purchase_value;
    let a_current_val = asset.current_value;
    let a_uuid = asset.id;
    let child_asset_count = Memo::new(move |_| {
        app_store
            .get()
            .portfolios
            .iter()
            .flat_map(|p| {
                p.assets
                    .iter()
                    .chain(p.asset_groups.iter().flat_map(|g| g.assets.iter()))
            })
            .filter(|a| a.parent_asset_id == Some(asset_id))
            .count()
    });
    let _a_pl = asset.profit_loss;
    let _a_pl_pct = asset.profit_loss_percent;
    let _a_revenue = asset.revenue;
    let _a_pl_cls = if asset.profit_loss >= 0.0 {
        "positive"
    } else {
        "negative"
    };
    let _a_purchase_date = asset.purchase_date.format("%d %b %Y").to_string();

    // OpenStreetMap/Nominatim address autocomplete generation counter.
    let address_gen = Arc::new(AtomicU64::new(0));

    // Persist the owning portfolio to RocksDB after asset field changes.
    let persist_portfolio = move || {
        if let Some(pid) = portfolio_id {
            let portfolio_clone = app_store.get().get_portfolio(pid).cloned();
            if let Some(p) = portfolio_clone {
                leptos::task::spawn_local(async move {
                    let _ = crate::server::save_portfolio(p).await;
                });
            }
        }
    };

    let save_type_build = move || {
        let t = edit_asset_type.get();
        let st = edit_asset_subtype.get();
        app_store.update(|s| {
            for p in s.portfolios.iter_mut() {
                let all: Vec<_> = p
                    .assets
                    .iter_mut()
                    .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                    .collect();
                for a in all {
                    if a.id == asset_id {
                        a.asset_type = t;
                        a.asset_subtype = if st.trim().is_empty() {
                            None
                        } else {
                            Some(st.clone())
                        };
                        a.updated_at = chrono::Utc::now();
                        return;
                    }
                }
            }
        });
        persist_portfolio();
        set_editing_type_build.set(false);
    };

    let save_address = move || {
        let l = edit_loc.get();
        app_store.update(|s| {
            for p in s.portfolios.iter_mut() {
                let all: Vec<_> = p
                    .assets
                    .iter_mut()
                    .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                    .collect();
                for a in all {
                    if a.id == asset_id {
                        a.location = if l.trim().is_empty() {
                            None
                        } else {
                            Some(l.clone())
                        };
                        a.updated_at = chrono::Utc::now();
                        return;
                    }
                }
            }
        });
        persist_portfolio();
        set_editing_address.set(false);
        set_address_suggestions.set(Vec::new());
    };

    let save_org = move || {
        let oid_str = edit_org_id.get();
        let oid = if oid_str.trim().is_empty() {
            None
        } else {
            Uuid::parse_str(&oid_str).ok()
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
                        a.organization_id = oid;
                        a.updated_at = chrono::Utc::now();
                        return;
                    }
                }
            }
        });
        persist_portfolio();
        set_editing_org.set(false);
    };

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
        persist_portfolio();
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
    let add_cb = if can_upload_documents() {
        Some(Callback::new(add_doc))
    } else {
        None
    };
    let (show_batch_doc_modal, set_show_batch_doc_modal) = signal(false);

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
                <img class="asset-grid-image" src={image_url} alt={a_name.clone()} />
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
        let (asset_name_signal, _set_asset_name) = signal(a_name.clone());
        view! {
        <div class="ai-item"
            node_ref=item_ref
            class:ai-item-expanded={move || expanded_detail.get()}
            class:ai-item-collapsible={collapsible}
            class:ai-item-collapsed={move || collapsed.get()}
            style={tint_style.clone()}
            aria-label=move || { let addr = a_addr.get(); format!("Asset {}. Type {}. In {}. {}", a_name, a_type_grid, pname, if addr.is_empty() { "No address set" } else { addr.as_str() }) }
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
                        on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                            if ev.key() == "Enter" || ev.key() == " " {
                                ev.prevent_default();
                                if !editing.get() { set_collapsed.update(|v| *v = !*v); }
                            }
                        }
                    >
                        <img class="ai-list-image" src={image_url} alt={a_name_header.clone()} />
                        <div class="ai-collapsible-summary">
                            <div class="ai-collapsible-name" on:click={name_click.clone()} on:dblclick={name_dblclick.clone()}>{a_name_header.clone()}</div>
                            <div class="ai-collapsible-meta">{format!("{} · ${:.2} · {}", a_type_header, a_val_header, a_status_label)}</div>
                            {let channel_count = asset.channel_ids.len();
                            let channel_ids = asset.channel_ids.clone();
                            move || if !channel_ids.is_empty() {
                                view! {
                                    <div class="ai-channel-count" title={format!("{} channel(s)", channel_count)}>
                                        {format!("{} channel{}", channel_count, if channel_count == 1 { "" } else { "s" })}
                                    </div>
                                }.into_any()
                            } else { ().into_any() }}
                        </div>
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
                    <div class="ai-image-slider" class:ai-perm-hidden={move || !can_view_images()} on:click=|ev| ev.stop_propagation()>
                        {{
                            let a_name_slider = a_name.clone();
                            move || {
                                let images = asset_images.get();
                                let count = images.len();
                                let can_add = count < max_images;
                                let a_name_add = a_name_slider.clone();
                                let a_name_images = a_name_slider.clone();
                                view! {
                                    {if can_add {
                                        view! {
                                            <div class="ai-image-slider-item ai-image-add-card" class:ai-perm-hidden={move || !can_upload_images()} aria-label={format!("Add image to {} (max {})", a_name_add, max_images)}>
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
                                        each=move || { asset_images.get().into_iter().enumerate().collect::<Vec<_>>() }
                                        key=|(_, url)| url.clone()
                                        children=move |(idx, url): (usize, String)| {
                                            let a_name_images = a_name_images.clone();
                                            let is_dragged = move || dragged_idx.get() == Some(idx);
                                            let is_drag_over = move || drag_over_idx.get() == Some(idx) && dragged_idx.get() != Some(idx);
                                            view! {
                                                <div class="ai-image-slider-item"
                                                    class:ai-image-dragging={is_dragged}
                                                    class:ai-image-drag-over={is_drag_over}
                                                    draggable={move || if can_reorder_images.get() { "true" } else { "false" }}
                                                    on:dragstart=move |_| { set_dragged_idx.set(Some(idx)); }
                                                    on:dragover=move |ev: leptos::ev::DragEvent| {
                                                        ev.prevent_default();
                                                        set_drag_over_idx.set(Some(idx));
                                                    }
                                                    on:dragleave=move |_| { set_drag_over_idx.set(None); }
                                                    on:drop=move |ev: leptos::ev::DragEvent| {
                                                        ev.prevent_default();
                                                        if let Some(from) = dragged_idx.get() {
                                                            if from != idx {
                                                                reorder_images.run((from, idx));
                                                            }
                                                        }
                                                        set_drag_over_idx.set(None);
                                                        set_dragged_idx.set(None);
                                                    }
                                                    on:dragend=move |_| {
                                                        set_drag_over_idx.set(None);
                                                        set_dragged_idx.set(None);
                                                    }
                                                >
                                                    <img class="ai-image-slider-img" src={url} alt={format!("Image of {}", a_name_images)} />
                                                </div>
                                            }
                                        }
                                    />
                                }.into_any()
                            }
                        }}
                    </div>
                    // Horizontal document slider with + Document card
                    <div class="ai-doc-slider" class:ai-perm-hidden={move || !can_view_documents()} on:click=|ev| ev.stop_propagation()>
                        // + Document card (always first)
                        <div class="ai-doc-slider-item ai-doc-add-card" class:ai-perm-hidden={move || !can_upload_documents()}
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
                                let (doc_ctx_menu_x, set_doc_ctx_menu_x) = signal(0i32);
                                let (doc_ctx_menu_y, set_doc_ctx_menu_y) = signal(0i32);
                                let (show_doc_ctx_menu, set_show_doc_ctx_menu) = signal(false);
                                let (viewing, set_viewing) = signal(false);
                                let (show_tooltip, set_show_tooltip) = signal(false);
                                let (tooltip_x, set_tooltip_x) = signal(0i32);
                                let (tooltip_y, set_tooltip_y) = signal(0i32);
                                let tooltip_text = dname.clone();
                                let update_tooltip_pos = {
                                    let set_x = set_tooltip_x.clone();
                                    let set_y = set_tooltip_y.clone();
                                    let text = tooltip_text.clone();
                                    move |ev: leptos::ev::MouseEvent| {
                                        let Some(window) = web_sys::window() else { return; };
                                        let Some(vw) = window.inner_width().ok().and_then(|v| v.as_f64()) else { return; };
                                        let Some(vh) = window.inner_height().ok().and_then(|v| v.as_f64()) else { return; };
                                        let offset = 12.0;
                                        let (mut tx, mut ty) = (ev.client_x() as f64 + offset, ev.client_y() as f64 + offset);
                                        let char_count = text.chars().count();
                                        let lines = ((char_count + 29) / 30).max(1).min(5);
                                        let tw = ((char_count.min(30) as f64 * 7.0 + 16.0)).min(200.0);
                                        let th = lines as f64 * 14.0 + 8.0;
                                        if tx + tw > vw { tx = (ev.client_x() as f64 - tw - offset).max(4.0); }
                                        if ty + th > vh { ty = (ev.client_y() as f64 - th - offset).max(4.0); }
                                        set_x.set(tx as i32);
                                        set_y.set(ty as i32);
                                    }
                                };
                                view! {
                                    <div class="ai-doc-slider-item"
                                        aria-label={format!("View document {}. Type {}", dname, ft)}
                                        on:click=move |_| set_viewing.set(true)
                                        on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                                            ev.prevent_default();
                                            ev.stop_propagation();
                                            let (x, y) = clamp_context_menu(ev.client_x(), ev.client_y());
                                            set_doc_ctx_menu_x.set(x);
                                            set_doc_ctx_menu_y.set(y);
                                            set_show_doc_ctx_menu.set(true);
                                        }
                                        on:mouseenter={
                                            let update_tooltip_pos = update_tooltip_pos.clone();
                                            move |ev: leptos::ev::MouseEvent| {
                                                set_show_tooltip.set(true);
                                                update_tooltip_pos(ev);
                                            }
                                        }
                                        on:mousemove=update_tooltip_pos
                                        on:mouseleave=move |_| set_show_tooltip.set(false)
                                    >
                                        <div class="ai-doc-slider-thumb">{icon}</div>
                                        <div class="ai-doc-slider-name">{short_name}</div>
                                        <div class="ai-doc-slider-type">{ft.clone()}</div>
                                        {move || if show_tooltip.get() {
                                            let text = tooltip_text.clone();
                                            view! {
                                                <div class="ai-doc-tooltip"
                                                    style=move || format!("left:{}px;top:{}px", tooltip_x.get(), tooltip_y.get())>
                                                    {text}
                                                </div>
                                            }.into_any()
                                        } else { ().into_any() }}
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
                                        let doc_for_download = doc.clone();
                                        view! {
                                            <div class="context-menu-overlay" on:click=move |_| set_show_doc_ctx_menu.set(false)>
                                                <div class="context-menu" style={format!("left: {}px; top: {}px;", dx, dy)}>
                                                    <button class="context-menu-item"
                                                        on:click=move |_| {
                                                            set_show_doc_ctx_menu.set(false);
                                                            download_document(&doc_for_download);
                                                        }
                                                    >"📥 Export / Download"</button>
                                                    {move || if can_upload_documents() {
                                                        view! {
                                                            <button class="context-menu-item"
                                                                on:click=move |_| {
                                                                    set_show_doc_ctx_menu.set(false);
                                                                    set_show_batch_doc_modal.set(true);
                                                                }
                                                            >"➕ Batch Add"</button>
                                                        }.into_any()
                                                    } else { ().into_any() }}
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
                    // Detail grid inline (always visible)
                    <div class="pf-detail-grid pf-detail-grid-inline" class:ai-perm-hidden={move || !can_view_general_info()}>
                        <div class="pf-detail-cell" on:dblclick=move |_| { if can_edit_here.get() { set_editing.set(false); set_editing_address.set(false); set_editing_org.set(false); set_editing_type_build.set(true); } }>
                            <span class="pf-detail-label">"TYPE & BUILD"</span>
                            {let type_grid = a_type_grid.clone();
                            let subtype_grid = a_subtype_grid.clone();
                            move || if can_edit_here.get() && editing_type_build.get() {
                                view! {
                                    <div style="display:flex;flex-direction:column;gap:4px;">
                                        <input class="pf-edit-input" type="text" list={type_datalist_id.clone()}
                                            placeholder="Type"
                                            prop:value=move || edit_asset_type.get().to_input_string()
                                            on:input=move |ev| {
                                                let new_type = AssetType::from_input(&event_target_value(&ev));
                                                let st = edit_asset_subtype.get();
                                                let subs = new_type.common_subtypes();
                                                if !subs.iter().any(|s| s.to_lowercase() == st.trim().to_lowercase()) {
                                                    set_edit_asset_subtype.set(subs.first().copied().unwrap_or("").to_string());
                                                }
                                                set_edit_asset_type.set(new_type);
                                            }
                                            on:blur=move |_| save_type_build()
                                            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                                if ev.key() == "Enter" { save_type_build(); }
                                                if ev.key() == "Escape" { set_editing_type_build.set(false); }
                                            }
                                        />
                                        <datalist id={type_datalist_id.clone()}>
                                            {AssetType::all_labels().into_iter().map(|l| view! { <option value={l.to_string()}>{l.to_string()}</option> }).collect::<Vec<_>>()}
                                        </datalist>
                                        <input class="pf-edit-input" type="text" list={subtype_datalist_id.clone()}
                                            placeholder="Build"
                                            prop:value=move || edit_asset_subtype.get()
                                            on:input=move |ev| set_edit_asset_subtype.set(event_target_value(&ev))
                                            on:blur=move |_| save_type_build()
                                            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                                if ev.key() == "Enter" { save_type_build(); }
                                                if ev.key() == "Escape" { set_editing_type_build.set(false); }
                                            }
                                        />
                                        <datalist id={subtype_datalist_id.clone()}>
                                            {move || {
                                                let t = edit_asset_type.get();
                                                let st = edit_asset_subtype.get();
                                                let mut opts: Vec<String> = t.common_subtypes().into_iter().map(|s| s.to_string()).collect();
                                                if !st.trim().is_empty() && !opts.iter().any(|o| o.to_lowercase() == st.trim().to_lowercase()) {
                                                    opts.push(st.clone());
                                                }
                                                opts.into_iter().map(|s| view! { <option value={s.clone()}>{s.clone()}</option> }).collect::<Vec<_>>()
                                            }}
                                        </datalist>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span class="pf-detail-value">{format!("{} / {}", type_grid, subtype_grid)}</span> }.into_any()
                            }}
                        </div>
                        <div class="pf-detail-cell" on:dblclick=move |_| { if can_edit_here.get() { set_editing.set(false); set_editing_type_build.set(false); set_editing_org.set(false); set_editing_address.set(true); } }>
                            <span class="pf-detail-label">"ADDRESS"</span>
                            {move || if can_edit_here.get() && editing_address.get() {
                                view! {
                                    <div style="display:flex;flex-direction:column;gap:4px;">
                                        <input class="pf-edit-input" type="text" list={addr_datalist_id.clone()}
                                            prop:value=move || edit_loc.get()
                                            on:input={
                                                let address_gen = address_gen.clone();
                                                move |ev| {
                                                    let v = event_target_value(&ev);
                                                    set_edit_loc.set(v.clone());
                                                    if v.trim().len() < 3 {
                                                        set_address_suggestions.set(Vec::new());
                                                        return;
                                                    }
                                                    let gen = address_gen.fetch_add(1, Ordering::SeqCst) + 1;
                                                    let address_gen = address_gen.clone();
                                                    let query = v.clone();
                                                    leptos::task::spawn_local(async move {
                                                        gloo_timers::future::TimeoutFuture::new(300).await;
                                                        if address_gen.load(Ordering::SeqCst) != gen {
                                                            return;
                                                        }
                                                        match crate::server::openmaps_autocomplete(query).await {
                                                            Ok(sugs) => set_address_suggestions.set(sugs),
                                                            Err(_) => set_address_suggestions.set(Vec::new()),
                                                        }
                                                    });
                                                }
                                            }
                                            on:change=move |_| save_address()
                                            on:keydown=move |ev: leptos::ev::KeyboardEvent| { if ev.key() == "Escape" { set_editing_address.set(false); } }
                                        />
                                        <datalist id={addr_datalist_id.clone()}>
                                            {move || address_suggestions.get().into_iter().map(|s| view! { <option value={s.clone()}>{s.clone()}</option> }).collect::<Vec<_>>()}
                                        </datalist>
                                        <button class="view-btn" on:click=move |_| { set_editing_address.set(false); set_address_suggestions.set(Vec::new()); }>"Cancel"</button>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span class="pf-detail-value">{move || a_addr_grid.get()}</span> }.into_any()
                            }}
                        </div>
                        <div class="pf-detail-cell" on:dblclick=move |_| { if can_edit_here.get() { set_editing.set(false); set_editing_type_build.set(false); set_editing_address.set(false); set_editing_org.set(true); } }>
                            <span class="pf-detail-label">"ORGANIZATION"</span>
                            {move || if can_edit_here.get() && editing_org.get() {
                                view! {
                                    <select class="pf-edit-input"
                                        prop:value=move || edit_org_id.get()
                                        on:change=move |ev| { set_edit_org_id.set(event_target_value(&ev)); save_org(); }
                                        on:focusout=move |_| set_editing_org.set(false)>
                                        <option value="">"(None)"</option>
                                        {move || available_orgs.get().into_iter().map(|o| view! {
                                            <option value={o.id.to_string()} selected=move || edit_org_id.get() == o.id.to_string()>{o.name.clone()}</option>
                                        }).collect::<Vec<_>>()}
                                    </select>
                                }.into_any()
                            } else {
                                view! { <span class="pf-detail-value">{a_org_name()}</span> }.into_any()
                            }}
                        </div>
                        <div class="pf-detail-cell">
                            <span class="pf-detail-label">"PRICE"</span>
                            <span class="pf-detail-value">{format!("${:.2}", a_current_val)}</span>
                        </div>
                        <div class="pf-detail-cell">
                            <span class="pf-detail-label">"# ASSETS"</span>
                            <span class="pf-detail-value">{move || child_asset_count.get().to_string()}</span>
                        </div>
                        <div class="pf-detail-cell" title={a_uuid.to_string()}>
                            <span class="pf-detail-label">"UUID"</span>
                            <span class="pf-detail-value">{a_uuid.to_string().chars().take(8).collect::<String>()}</span>
                        </div>
                    </div>
                    <div class="ai-controls-row" class:ai-perm-hidden={move || !(can_view_linking() || can_view_booking())}>
                        <button class="pf-small-btn" on:click=move |_| set_show_linking_booking.set(true)>
                            "Channels & Bookings"
                        </button>
                    </div>
                </div>
            </div>

            {move || if ui_store.get().is_doc_modal_open(asset_id) {
                let mt = asset_name_signal.get();
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

            {move || if show_batch_doc_modal.get() {
                let mt = asset_name_signal.get();
                let ac = add_cb.clone();
                view! {
                    <DocModal
                        entity_id={asset_id}
                        title={format!("Batch Add - {}", mt)}
                        on_close=move || set_show_batch_doc_modal.set(false)
                        can_edit={can_edit_documents_here.get()}
                        on_add={ac}
                        portfolio_id={portfolio_id}
                        group_id={group_id}
                        asset_id={Some(asset_id)}
                        batch={true}
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
                                    set_asset_target_portfolio_id.set(String::new());
                                    set_asset_target_group_id.set(String::new());
                                    set_show_move_to_group.set(true);
                                }
                            >"➕ Add to Asset Group"</button>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_asset_context_menu.set(None);
                                    set_asset_target_portfolio_id.set(String::new());
                                    set_show_move_to_portfolio.set(true);
                                }
                            >"➕ Add to Portfolio"</button>
                            <button class="context-menu-item"
                                on:click=move |_| {
                                    set_asset_context_menu.set(None);
                                    app_store.with(|s| {
                                        for p in s.portfolios.iter() {
                                            let found = p.assets.iter().find(|a| a.id == asset_id)
                                                .or_else(|| p.asset_groups.iter().flat_map(|g| g.assets.iter()).find(|a| a.id == asset_id));
                                            if let Some(a) = found {
                                                set_asset_org_id.set(a.organization_id.map(|id| id.to_string()).unwrap_or_default());
                                                break;
                                            }
                                        }
                                    });
                                    set_show_add_org.set(true);
                                }
                            >"🏢 Add to Organization"</button>
                            <button class="context-menu-item" class:ai-perm-hidden={move || !can_upload_documents()}
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
                            >"� Add Role"</button>
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

            // Add to Organization modal (asset)
            {move || if show_add_org.get() {
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_add_org.set(false)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add to Organization"</span>
                                <button class="doc-modal-close" aria-label="Close add organization" on:click=move |_| set_show_add_org.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <label class="list-item-title">"Organization"</label>
                                <select
                                    class="form-select"
                                    aria-label="Organization"
                                    prop:value={move || asset_org_id.get()}
                                    on:change=move |ev| set_asset_org_id.set(event_target_value(&ev))
                                >
                                    <option value="">"(None)"</option>
                                    {move || organization_store.get().organizations.iter().map(|o| {
                                        let id = o.id.to_string();
                                        view! { <option value={id.clone()}>{o.name.clone()}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                                <input class="login-input" type="text" placeholder="Or create a new organization"
                                    aria-label="New organization name"
                                    prop:value={move || new_org_name.get()}
                                    on:input=move |ev| set_new_org_name.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| {
                                    let name = new_org_name.get().trim().to_string();
                                    let org_id = if name.is_empty() {
                                        let s = asset_org_id.get();
                                        if s.trim().is_empty() { None } else { Uuid::parse_str(&s).ok() }
                                    } else {
                                        let owner_id = app_store.get().current_user.id;
                                        let org = crate::models::Organization::new(name, owner_id);
                                        let oid = org.id;
                                        organization_store.update(|s| s.add_organization(org));
                                        Some(oid)
                                    };
                                    app_store.update(|s| { s.set_asset_organization(asset_id, org_id); });
                                    set_new_org_name.set(String::new());
                                    set_asset_org_id.set(String::new());
                                    set_show_add_org.set(false);
                                }>"Save Organization"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}

            // Move to Portfolio modal (asset)
            {move || if show_move_to_portfolio.get() {
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_move_to_portfolio.set(false)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add to Portfolio"</span>
                                <button class="doc-modal-close" aria-label="Close move to portfolio" on:click=move |_| set_show_move_to_portfolio.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <label class="list-item-title">"Target portfolio"</label>
                                <select
                                    class="form-select"
                                    aria-label="Target portfolio"
                                    prop:value={move || asset_target_portfolio_id.get()}
                                    on:change=move |ev| set_asset_target_portfolio_id.set(event_target_value(&ev))
                                >
                                    <option value="">"Select a portfolio"</option>
                                    {move || app_store.get().portfolios.iter().map(|p| {
                                        let id = p.id.to_string();
                                        view! { <option value={id.clone()}>{p.name.clone()}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                                <button class="login-btn" on:click=move |_| {
                                    let s = asset_target_portfolio_id.get();
                                    if let Ok(target_pid) = Uuid::parse_str(&s) {
                                        app_store.update(|store| { store.move_asset_to_portfolio(asset_id, target_pid); });
                                    }
                                    set_asset_target_portfolio_id.set(String::new());
                                    set_show_move_to_portfolio.set(false);
                                }>"Move to Portfolio"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}

            // Move to Asset Group modal (asset)
            {move || if show_move_to_group.get() {
                view! {
                    <div class="doc-modal-overlay" on:click=move |_| set_show_move_to_group.set(false)>
                        <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="doc-modal-header">
                                <span>"Add to Asset Group"</span>
                                <button class="doc-modal-close" aria-label="Close move to group" on:click=move |_| set_show_move_to_group.set(false)>"✕"</button>
                            </div>
                            <div class="add-form">
                                <label class="list-item-title">"Target portfolio"</label>
                                <select
                                    class="form-select"
                                    aria-label="Target portfolio"
                                    prop:value={move || asset_target_portfolio_id.get()}
                                    on:change=move |ev| set_asset_target_portfolio_id.set(event_target_value(&ev))
                                >
                                    <option value="">"Select a portfolio"</option>
                                    {move || app_store.get().portfolios.iter().map(|p| {
                                        let id = p.id.to_string();
                                        view! { <option value={id.clone()}>{p.name.clone()}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                                <label class="list-item-title">"Target asset group"</label>
                                <select
                                    class="form-select"
                                    aria-label="Target asset group"
                                    prop:value={move || asset_target_group_id.get()}
                                    on:change=move |ev| set_asset_target_group_id.set(event_target_value(&ev))
                                >
                                    <option value="">"Select an asset group"</option>
                                    {move || {
                                        let s = asset_target_portfolio_id.get();
                                        if let Ok(target_pid) = Uuid::parse_str(&s) {
                                            app_store.get().portfolios.iter()
                                                .find(|p| p.id == target_pid)
                                                .map(|p| p.asset_groups.iter().map(|g| {
                                                    let id = g.id.to_string();
                                                    view! { <option value={id.clone()}>{g.name.clone()}</option> }
                                                }).collect::<Vec<_>>())
                                                .unwrap_or_default()
                                        } else {
                                            Vec::new()
                                        }
                                    }}
                                </select>
                                <button class="login-btn" on:click=move |_| {
                                    let p = asset_target_portfolio_id.get();
                                    let g = asset_target_group_id.get();
                                    if let (Ok(target_pid), Ok(target_gid)) = (Uuid::parse_str(&p), Uuid::parse_str(&g)) {
                                        app_store.update(|store| { store.move_asset_to_group(asset_id, target_pid, target_gid); });
                                    }
                                    set_asset_target_portfolio_id.set(String::new());
                                    set_asset_target_group_id.set(String::new());
                                    set_show_move_to_group.set(false);
                                }>"Move to Group"</button>
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

            {move || if show_linking_booking.get() { view! {
                <LinkingBookingModal
                    asset_id={asset_id}
                    asset_name={asset_name_signal.get()}
                    portfolio_id={portfolio_id}
                    can_link={can_link()}
                    can_book={can_book()}
                    on_close={Callback::new(move |_| set_show_linking_booking.set(false))}
                />
            }.into_any() } else { ().into_any() }}
        </div>
    }.into_any()
    }
}
