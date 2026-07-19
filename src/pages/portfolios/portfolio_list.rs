use crate::stores::{use_app_store, use_notification_store, use_organization_store, use_ui_store};
use crate::types::{AssetType, ViewMode};
use leptos::prelude::*;
use uuid::Uuid;

use super::{
    detect_file_type, read_image_as_data_url, single_sentence, AssetTarget, AssetViewer, DocModal,
    NotifTarget, UserAssignmentPanel,
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
    on_open_notif_qs: Callback<(NotifTarget, String, bool)>,
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
    let pid = portfolio.id;
    let doc_count = portfolio.documents.len();
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

    let pf_image_input_ref = NodeRef::<leptos::html::Input>::new();

    let do_save = move || {
        let n = edit_name.get();
        let d = edit_desc.get();
        if n.trim().is_empty() {
            return;
        }
        let img = edit_image_url.get();
        let emoji = edit_emoji.get().trim().to_string();
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
                p.updated_at = chrono::Utc::now();
            }
        });
        set_is_editing_name.set(false);
        set_is_editing_desc.set(false);
    };
    let save_edit = move |_: leptos::ev::FocusEvent| do_save();
    let save_edit_now = move || do_save();
    let save_edit_callback = Callback::new(move |_| do_save());

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
                class:editing={move || is_editing_name.get() || is_editing_desc.get()}
                role="button"
                tabindex="0"
                aria-expanded={move || expanded.get()}
                aria-controls={format!("pf-content-{}", pid)}
                aria-label={move || format!("{} portfolio. {} asset{}. {} document{}. {}",
                    name,
                    asset_count,
                    if asset_count == 1 { "" } else { "s" },
                    doc_count,
                    if doc_count == 1 { "" } else { "s" },
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
                <div class="asset-group-info-wrap" on:click=|ev| ev.stop_propagation()>
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
                                    on:blur=save_edit
                                    on:keydown=move |ev| { if ev.key() == "Enter" { save_edit_now(); } }
                                />
                            }.into_any());
                        } else {
                            let set_editing = set_is_editing_name;
                            parts.push(view! {
                                <div class="asset-group-name"
                                    on:dblclick=move |ev| { if can_edit_here.get() { ev.stop_propagation(); set_editing.set(true); } }
                                >{name_header.clone()}</div>
                            }.into_any());
                        }
                        // Description
                        if is_editing_desc.get() && can_edit_here.get() {
                            parts.push(view! {
                                <input class="pf-edit-input" placeholder="Description"
                                    prop:value=move || edit_desc.get()
                                    on:input=move |ev| set_edit_desc.set(event_target_value(&ev))
                                    on:blur=save_edit
                                    on:keydown=move |ev| { if ev.key() == "Enter" { save_edit_now(); } }
                                />
                            }.into_any());
                        } else if !desc_header.is_empty() {
                            let set_editing = set_is_editing_desc;
                            parts.push(view! {
                                <div class="asset-group-desc"
                                    on:dblclick=move |ev| { if can_edit_here.get() { ev.stop_propagation(); set_editing.set(true); } }
                                >{desc_header.clone()}</div>
                            }.into_any());
                        }
                        // Image + emoji editor
                        if (is_editing_name.get() || is_editing_desc.get()) && can_edit_here.get() {
                            let save_cb = save_edit_callback.clone();
                            let save_for_file = save_edit_callback.clone();
                            parts.push(view! {
                                <input
                                    class="pf-edit-input"
                                    type="file"
                                    accept="image/*"
                                    aria-label="Portfolio image"
                                    on:change=move |ev| {
                                        read_image_as_data_url(&ev, {
                                            let save = save_for_file.clone();
                                            move |url| {
                                                set_edit_image_url.set(Some(url));
                                                save.run(());
                                            }
                                        });
                                    }
                                />
                                <select
                                    class="pf-edit-input"
                                    aria-label="Portfolio emoji"
                                    prop:value={move || edit_emoji.get()}
                                    on:change=move |ev| {
                                        set_edit_emoji.set(event_target_value(&ev));
                                        save_cb.run(());
                                    }
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
                // Action strip — notification bell above document icon, hidden when count is zero
                <div class="pf-list-actions pf-list-actions-stacked" on:click=|ev| ev.stop_propagation()>
                    {let name_for_notif = name_for_doc_btn.clone();
                    move || {
                        let count = app_store.get().doc_notifications_for_portfolio(pid, &notification_store.get().notifications);
                        let pname = name_for_notif.clone();
                        let pname_click = pname.clone();
                        let pname_ctx = pname.clone();
                        let pname_keydown = pname.clone();
                        if count > 0 {
                            view! {
                                <button
                                    class="pf-action-btn pf-notif-action-btn"
                                    type="button"
                                    aria-label={format!("Notifications for {} portfolio. {} unread", pname, count)}
                                    title="Left-click to view notifications, right-click to edit settings"
                                    on:click=move |ev| {
                                        ev.stop_propagation();
                                        on_open_notif_qs.run((NotifTarget::Portfolio(pid), pname_click.clone(), false));
                                    }
                                    on:contextmenu=move |ev| {
                                        ev.prevent_default();
                                        ev.stop_propagation();
                                        on_open_notif_qs.run((NotifTarget::Portfolio(pid), pname_ctx.clone(), true));
                                    }
                                    on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                        if ev.key() == "Enter" || ev.key() == " " {
                                            ev.prevent_default();
                                            ev.stop_propagation();
                                            on_open_notif_qs.run((NotifTarget::Portfolio(pid), pname_keydown.clone(), false));
                                        }
                                    }
                                >
                                    <div class="pf-action-stack">
                                        <span class="pf-action-icon">"🔔"</span>
                                        <span class="pf-action-count">{count}</span>
                                    </div>
                                </button>
                            }.into_any()
                        } else { ().into_any() }
                    }}
                    <button class="pf-action-btn pf-doc-action-btn"
                        class:active=move || ui_store.get().is_doc_modal_open(pid)
                        aria-label={format!("View documents for {} portfolio. {} document{}", name_for_doc_btn, doc_count, if doc_count == 1 { "" } else { "s" })}
                        on:click=move |_| ui_store.update(|s| s.toggle_doc_modal(pid))
                        on:dblclick=move |ev| { if can_edit_here.get() { ev.stop_propagation(); ui_store.update(|s| s.open_doc_modal(pid)); } }
                    >
                        <div class="pf-action-stack">
                            <span class="pf-action-icon">"📄"</span>
                            <span class="pf-action-count">{doc_count}</span>
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
                    on_open_notif_qs={on_open_notif_qs.clone()}
                />
            </div>
        </div>
    }
}
