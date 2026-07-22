use crate::stores::{use_app_store, use_notification_store, use_organization_store, use_ui_store};
use crate::types::{AssetType, ViewMode};
use leptos::prelude::*;
use uuid::Uuid;

use super::{
    detect_file_type, name_click_handlers, read_image_as_data_url, single_sentence, AssetTarget,
    AssetViewer, DocModal, UserAssignmentPanel,
};

/// Portfolio list row — accordion style matching AssetGroupItem.
#[component]
pub(crate) fn PortfolioListItem(
    portfolio: crate::models::Portfolio,
    #[prop(into)] can_edit: Signal<bool>,
    #[prop(into)] can_edit_documents: Signal<bool>,
    #[prop(into)] expanded: Signal<bool>,
    on_toggle: Callback<()>,
    on_context: impl Fn(leptos::ev::MouseEvent) + 'static,
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
) -> impl IntoView {
    let app_store = use_app_store();
    let notification_store = use_notification_store();
    let ui_store = use_ui_store();
    let (is_editing_name, set_is_editing_name) = signal(false);
    let (is_editing_desc, set_is_editing_desc) = signal(false);
    let (edit_name, set_edit_name) = signal(portfolio.name.clone());
    let (edit_desc, set_edit_desc) = signal(portfolio.description.clone().unwrap_or_default());
    let (edit_image_url, set_edit_image_url) = signal(portfolio.image_url.clone());
    let (edit_emoji, set_edit_emoji) = signal(portfolio.emoji.clone().unwrap_or_default());
    let (is_editing_org, set_is_editing_org) = signal(false);
    let (edit_org_id, set_edit_org_id) = signal(
        portfolio
            .organization_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
    );
    let (show_doc_dropdown, set_show_doc_dropdown) = signal(false);
    let (doc_modal_target, set_doc_modal_target) =
        signal(Option::<(Uuid, String, Option<Uuid>, Option<Uuid>, Option<Uuid>)>::None);
    let (new_doc_name_input, set_new_doc_name_input) = signal(String::new());
    let pid = portfolio.id;
    let doc_count = Memo::new(move |_| {
        app_store
            .get()
            .get_portfolio(pid)
            .map(|p| p.documents.len())
            .unwrap_or(0)
    });
    let name = portfolio.name.clone();
    let name_for_modal = portfolio.name.clone();
    let name_for_doc_btn = portfolio.name.clone();
    let desc = single_sentence(&portfolio.description.clone().unwrap_or_default());
    let asset_count = portfolio.get_all_assets().len();
    let portfolio_image_url = portfolio.image_url.clone();
    let portfolio_emoji = portfolio.emoji.clone().unwrap_or_else(|| "🏢".to_string());
    let can_edit_here = can_edit;
    let can_edit_documents_here = can_edit_documents;
    let organization_store = use_organization_store();
    let current_user_id = app_store.get().current_user.id;
    let available_orgs = Memo::new(move |_| {
        organization_store
            .get()
            .organizations
            .iter()
            .filter(|o| o.members.contains(&current_user_id) || o.owner_id == current_user_id)
            .cloned()
            .collect::<Vec<_>>()
    });

    let (name_click, name_dblclick) = name_click_handlers(
        move || on_toggle.run(()),
        move || {
            if can_edit_here.get() {
                set_is_editing_name.set(true);
            }
        },
    );
    let (desc_click, desc_dblclick) = name_click_handlers(
        move || on_toggle.run(()),
        move || {
            if can_edit_here.get() {
                set_is_editing_desc.set(true);
            }
        },
    );

    let pf_image_input_ref = NodeRef::<leptos::html::Input>::new();

    let do_save = move || {
        let n = edit_name.get();
        let d = edit_desc.get();
        if n.trim().is_empty() {
            return;
        }
        let img = edit_image_url.get();
        let emoji = edit_emoji.get().trim().to_string();
        let org_id_str = edit_org_id.get();
        let org_id = if org_id_str.trim().is_empty() {
            None
        } else {
            Uuid::parse_str(&org_id_str).ok()
        };
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                p.name = n.clone();
                p.description = if d.trim().is_empty() {
                    None
                } else {
                    Some(d.clone())
                };
                p.image_url = img;
                p.emoji = if emoji.is_empty() { None } else { Some(emoji) };
                p.organization_id = org_id;
                p.updated_at = chrono::Utc::now();
            }
        });
        set_is_editing_name.set(false);
        set_is_editing_desc.set(false);
        set_is_editing_org.set(false);
    };
    let save_edit_now = move || do_save();

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

    view! {
        <div class="asset-group pf-portfolio" class:expanded={expanded} on:contextmenu=on_context>
            // Header row — same structure as asset-group-header
            <div class="asset-group-header pf-accordion-header"
                class:editing={move || is_editing_name.get() || is_editing_desc.get() || is_editing_org.get()}
                role="button"
                tabindex="0"
                aria-expanded={move || expanded.get()}
                aria-controls={format!("pf-content-{}", pid)}
                aria-label={move || format!("{} portfolio. {} asset{}. {} document{}. {}",
                    name,
                    asset_count,
                    if asset_count == 1 { "" } else { "s" },
                    doc_count.get(),
                    if doc_count.get() == 1 { "" } else { "s" },
                    if expanded.get() { "Expanded" } else { "Collapsed" }
                )}
                on:click=move |ev: leptos::ev::MouseEvent| {
                    ev.stop_propagation();
                    if !is_editing_name.get() && !is_editing_desc.get() && !is_editing_org.get() {
                        on_toggle.run(());
                    }
                }
                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                    if ev.key() == "Enter" || ev.key() == " " {
                        ev.prevent_default();
                        ev.stop_propagation();
                        if !is_editing_name.get() && !is_editing_desc.get() && !is_editing_org.get() {
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
                                <input class="pf-edit-input" placeholder="Portfolio name"
                                    prop:value=move || edit_name.get()
                                    on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                    on:keydown=move |ev| {
                                        if ev.key() == "Enter" { save_edit_now(); }
                                        else if ev.key() == "Escape" { set_is_editing_name.set(false); set_is_editing_desc.set(false); set_is_editing_org.set(false); }
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
                                <input class="pf-edit-input" placeholder="Description"
                                    prop:value=move || edit_desc.get()
                                    on:input=move |ev| set_edit_desc.set(event_target_value(&ev))
                                    on:keydown=move |ev| {
                                        if ev.key() == "Enter" { save_edit_now(); }
                                        else if ev.key() == "Escape" { set_is_editing_name.set(false); set_is_editing_desc.set(false); set_is_editing_org.set(false); }
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
                        // Image + emoji + organization editor
                        if (is_editing_name.get() || is_editing_desc.get() || is_editing_org.get()) && can_edit_here.get() {
                            parts.push(view! {
                                <input
                                    class="pf-edit-input"
                                    type="file"
                                    accept="image/*"
                                    aria-label="Portfolio image"
                                    on:change=move |ev| {
                                        read_image_as_data_url(&ev, move |url| {
                                            set_edit_image_url.set(Some(url));
                                        });
                                    }
                                />
                                <select
                                    class="pf-edit-input"
                                    aria-label="Portfolio emoji"
                                    prop:value={move || edit_emoji.get()}
                                    on:change=move |ev| set_edit_emoji.set(event_target_value(&ev))
                                >
                                    <option value="">"Default 🏢"</option>
                                    <option value="🏢">"🏢 Office"</option>
                                    <option value="🏠">"🏠 Property"</option>
                                    <option value="🚗">"🚗 Vehicle"</option>
                                    <option value="💼">"💼 Business"</option>
                                    <option value="💰">"💰 Finance"</option>
                                    <option value="📈">"📈 Growth"</option>
                                    <option value="🏭">"🏭 Industrial"</option>
                                    <option value="🌐">"🌐 Global"</option>
                                    <option value="🎨">"🎨 Creative"</option>
                                    <option value="🔬">"🔬 Research"</option>
                                    <option value="⚡">"⚡ Energy"</option>
                                </select>
                                <select
                                    class="pf-edit-input"
                                    aria-label="Organization"
                                    prop:value={move || edit_org_id.get()}
                                    on:change=move |ev| set_edit_org_id.set(event_target_value(&ev))
                                >
                                    <option value="">"(None)"</option>
                                    {move || available_orgs.get().into_iter().map(|o| view! {
                                        <option value={o.id.to_string()} selected=move || edit_org_id.get() == o.id.to_string()>{o.name.clone()}</option>
                                    }).collect::<Vec<_>>()}
                                </select>
                                <div style="display:flex;gap:6px;margin-top:4px;">
                                    <button class="login-btn" on:click=move |_| save_edit_now()>"Save"</button>
                                    <button class="view-btn" on:click=move |_| {
                                        set_is_editing_name.set(false);
                                        set_is_editing_desc.set(false);
                                        set_is_editing_org.set(false);
                                    }>"Cancel"</button>
                                </div>
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
                // Document icon for portfolio
                <div class="pf-list-actions" on:click=|ev| ev.stop_propagation()>
                    <button class="pf-action-btn pf-doc-action-btn"
                        class:active=move || show_doc_dropdown.get()
                        aria-label={move || format!("View documents for {} portfolio. {} document{}", name_for_doc_btn, doc_count.get(), if doc_count.get() == 1 { "" } else { "s" })}
                        on:click=move |_| set_show_doc_dropdown.update(|s| *s = !*s)
                    >
                        <div class="pf-action-stack">
                            <span class="pf-action-icon">"📄"</span>
                            <span class="pf-action-count">{move || doc_count.get()}</span>
                        </div>
                    </button>
                </div>
            </div>

            // Docs modal for portfolio
            {move || if ui_store.get().is_doc_modal_open(pid) {
                let modal_title = name_for_modal.clone();
                let add_cb = if can_edit_documents_here.get() { Some(Callback::new(move |n: String| add_doc(n))) } else { None };
                view! {
                    <DocModal
                        entity_id={pid}
                        title={modal_title}
                        on_close=move || ui_store.update(|s| s.close_doc_modal(pid))
                        can_edit={can_edit_documents_here.get()}
                        on_add={add_cb}
                        portfolio_id={Some(pid)}
                    />
                }.into_any()
            } else { ().into_any() }}

            // Portfolio documents dropdown (portfolio + asset/group docs)
            {move || if show_doc_dropdown.get() {
                let total = doc_entries.get().len();
                view! {
                    <div class="pf-doc-dropdown" on:click=|ev| ev.stop_propagation()>
                        <div class="pf-doc-dropdown-header">
                            <span class="pf-doc-dropdown-title">"Documents"</span>
                            <button class="pf-doc-dropdown-close" aria-label="Close documents" on:click=move |_| set_show_doc_dropdown.set(false)>"✕"</button>
                        </div>
                        <div class="pf-doc-dropdown-body">
                            {if total == 0 {
                                view! { <div class="pf-doc-dropdown-empty">"No documents yet"</div> }.into_any()
                            } else { ().into_any() }}
                            <For
                                each=move || doc_entries.get()
                                key=|(_, _, _, doc, _, _, _)| doc.id
                                children=move |(entity_id, source_name, source_kind, doc, portfolio_id, group_id, asset_id): (Uuid, String, String, crate::models::Document, Option<Uuid>, Option<Uuid>, Option<Uuid>)| {
                                    let dname = doc.name.clone();
                                    let ft = doc.file_type.to_uppercase();
                                    let source_name_for_view = source_name.clone();
                                    view! {
                                        <div class="pf-doc-dropdown-row">
                                            <span class="pf-doc-dropdown-icon">"📄"</span>
                                            <span class="pf-doc-dropdown-name" title={dname.clone()}>{dname.clone()}</span>
                                            <span class="pf-doc-dropdown-kind">{format!("{} • {}", source_name_for_view, source_kind)}</span>
                                            <span class="pf-doc-dropdown-type">{ft}</span>
                                            <button class="pf-doc-dropdown-view" on:click=move |_| {
                                                set_doc_modal_target.set(Some((entity_id, source_name_for_view.clone(), portfolio_id, group_id, asset_id)));
                                                set_show_doc_dropdown.set(false);
                                            }>"View"</button>
                                        </div>
                                    }
                                }
                            />
                        </div>
                        {if can_edit_documents_here.get() {
                            view! {
                                <div class="pf-doc-dropdown-footer">
                                    <input class="pf-edit-input" type="text" placeholder="New document name" prop:value={move || new_doc_name_input.get()} on:input=move |ev| set_new_doc_name_input.set(event_target_value(&ev)) on:keydown=move |ev: leptos::ev::KeyboardEvent| { if ev.key() == "Enter" { let n = new_doc_name_input.get(); if !n.trim().is_empty() { add_doc(n); set_new_doc_name_input.set(String::new()); } } } />
                                    <button class="login-btn" on:click=move |_| { let n = new_doc_name_input.get(); if !n.trim().is_empty() { add_doc(n); set_new_doc_name_input.set(String::new()); } }>"Add"</button>
                                </div>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                }.into_any()
            } else { ().into_any() }}

            // Doc viewer for a document selected from the dropdown (supports portfolio, group or asset docs)
            {move || if let Some((entity_id, title, portfolio_id, group_id, asset_id)) = doc_modal_target.get() {
                let modal_title = title;
                let can_edit_here_docs = portfolio_id == Some(pid) && can_edit_documents_here.get();
                let add_cb = if can_edit_here_docs { Some(Callback::new(move |n: String| add_doc(n))) } else { None };
                view! {
                    <DocModal
                        entity_id={entity_id}
                        title={modal_title}
                        on_close=move || set_doc_modal_target.set(None)
                        can_edit={can_edit_here_docs}
                        on_add={add_cb}
                        portfolio_id={portfolio_id}
                        group_id={group_id}
                        asset_id={asset_id}
                    />
                }.into_any()
            } else { ().into_any() }}

            {move || if is_editing_name.get() && can_edit_here.get() {
                let users = org_users();
                let assigned = assigned_users.clone();
                view! {
                    <UserAssignmentPanel assigned={assigned} users={users} on_toggle={toggle_portfolio_assignment} />
                }.into_any()
            } else { ().into_any() }}

            // Expanded content — AssetViewer
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
                />
            </div>
        </div>
    }
}
