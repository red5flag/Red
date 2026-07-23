use crate::models::Perm;
use crate::stores::{use_app_store, use_notification_store, use_organization_store};
use crate::types::{AssetType, ViewMode};
use leptos::prelude::*;
use uuid::Uuid;

use super::{
    detect_file_type, name_click_handlers, read_image_as_data_url, single_sentence, AssetTarget,
    AssetViewer, DocEntry, DocSlider, UserAssignmentPanel,
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
    let pid = portfolio.id;
    let name = portfolio.name.clone();
    let name_for_aria = name.clone();
    let name_for_doc_btn = name.clone();
    let name_for_slider = StoredValue::new(name.clone());
    let (show_doc_slider, set_show_doc_slider) = signal(false);
    let desc = single_sentence(&portfolio.description.clone().unwrap_or_default());
    let asset_count = portfolio.get_all_assets().len();
    let portfolio_image_url = portfolio.image_url.clone();
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
                    name_for_aria,
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

            {move || if show_doc_slider.get() {
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
    }.into_any()
}
