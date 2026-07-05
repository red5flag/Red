use crate::stores::{use_app_store, use_notification_store, use_organization_store, use_ui_store};
use crate::types::{AssetType, ViewMode};
use leptos::prelude::*;
use uuid::Uuid;

use super::{
    detect_file_type, AssetTarget, AssetViewer, DocModal, NotifTarget, UserAssignmentPanel,
};

/// Portfolio list row — accordion style matching AssetGroupItem.
#[component]
pub(crate) fn PortfolioListItem(
    portfolio: crate::models::Portfolio,
    #[prop(default = false)] can_edit: bool,
    #[prop(default = false)] can_edit_documents: bool,
    expanded: bool,
    on_toggle: Callback<()>,
    on_context: impl Fn(leptos::ev::MouseEvent) + 'static,
    on_open_notif_qs: Callback<(NotifTarget, String, bool)>,
    // AssetViewer props forwarded for expanded content
    show_add_group: Option<Uuid>,
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
    on_add_asset: Callback<AssetTarget>,
    view_mode: ViewMode,
) -> impl IntoView {
    let app_store = use_app_store();
    let notification_store = use_notification_store();
    let ui_store = use_ui_store();
    let (is_editing_name, set_is_editing_name) = signal(false);
    let (is_editing_desc, set_is_editing_desc) = signal(false);
    let (is_editing_org, set_is_editing_org) = signal(false);
    let (edit_name, set_edit_name) = signal(portfolio.name.clone());
    let (edit_desc, set_edit_desc) = signal(portfolio.description.clone().unwrap_or_default());
    let pid = portfolio.id;
    let doc_count = portfolio.documents.len();
    let name = portfolio.name.clone();
    let name_for_modal = portfolio.name.clone();
    let name_for_doc_btn = portfolio.name.clone();
    let desc = portfolio.description.clone().unwrap_or_default();
    let asset_count = portfolio.get_all_assets().len();
    let can_edit_here = can_edit;
    let can_edit_documents_here = can_edit_documents;
    let organization_store = use_organization_store();
    let org_name = portfolio.organization_id.and_then(|oid| {
        organization_store
            .get()
            .organizations
            .iter()
            .find(|o| o.id == oid)
            .map(|o| o.name.clone())
    });
    let org_name_for_label = org_name.clone();
    let org_color = portfolio.organization_id.and_then(|oid| {
        organization_store
            .get()
            .organizations
            .iter()
            .find(|o| o.id == oid)
            .and_then(|o| o.settings.color.clone())
    });
    let current_org_id = portfolio.organization_id;
    let orgs = organization_store.get().organizations.clone();

    let save_edit = move |_: leptos::ev::FocusEvent| {
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

    let save_edit_now = move || {
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

    let save_org_edit = move |ev: leptos::ev::Event| {
        let v = event_target_value(&ev);
        let new_org_id = if v == "none" {
            None
        } else {
            Uuid::parse_str(&v).ok()
        };
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                p.organization_id = new_org_id;
                p.updated_at = chrono::Utc::now();
            }
        });
        set_is_editing_org.set(false);
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
        <div class="asset-group" class:expanded={expanded} on:contextmenu=on_context>
            // Org color strip — left-side color coding for organization identification
            {org_color.as_ref().map(|c| view! {
                <div class="pf-org-color-strip" style={format!("background: {}", c)} aria-hidden="true"></div>
            })}
            // Header row — same structure as asset-group-header
            <div class="asset-group-header"
                style={org_color.as_ref().map(|c| format!("border-left: 3px solid {}", c)).unwrap_or_default()}
                role="button"
                tabindex="0"
                aria-expanded={expanded}
                aria-controls={format!("pf-content-{}", pid)}
                aria-label={format!("{} portfolio. {}. {} asset{}. {} document{}. {}",
                    name,
                    org_name_for_label.as_deref().unwrap_or("No organization"),
                    asset_count,
                    if asset_count == 1 { "" } else { "s" },
                    doc_count,
                    if doc_count == 1 { "" } else { "s" },
                    if expanded { "Expanded" } else { "Collapsed" }
                )}
                on:click=move |_| {
                    if !is_editing_name.get() && !is_editing_desc.get() && !is_editing_org.get() {
                        on_toggle.run(());
                    }
                }
                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                    if ev.key() == "Enter" || ev.key() == " " {
                        ev.prevent_default();
                        if !is_editing_name.get() && !is_editing_desc.get() && !is_editing_org.get() {
                            on_toggle.run(());
                        }
                    }
                }
            >
                <span class="asset-group-arrow">
                    {if expanded { "▲" } else { "▼" }}
                </span>
                <div class="asset-group-icon">"🏢"</div>
                <div class="asset-group-info-wrap" on:click=|ev| ev.stop_propagation()>
                    {let name_header = name.clone();
                    let desc_header = desc.clone();
                    move || {
                        let mut parts: Vec<leptos::prelude::AnyView> = Vec::new();
                        // Organization label / editor
                        if can_edit_here {
                            if is_editing_org.get() {
                                parts.push(view! {
                                    <select class="pf-edit-input pf-org-select"
                                        prop:value={move || current_org_id.map(|id| id.to_string()).unwrap_or_else(|| "none".to_string())}
                                        on:change=save_org_edit
                                        on:blur=move |_| set_is_editing_org.set(false)
                                    >
                                        <option value="none">"No Organization"</option>
                                        {orgs.iter().map(|o| {
                                            let oid = o.id.to_string();
                                            let oname = o.name.clone();
                                            view! {
                                                <option value={oid.clone()}>{oname}</option>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </select>
                                }.into_any());
                            } else if let Some(on) = &org_name {
                                parts.push(view! {
                                    <div class="pf-org-label"
                                        on:dblclick=move |ev| { ev.stop_propagation(); set_is_editing_org.set(true); }
                                    >{on.clone()}</div>
                                }.into_any());
                            }
                        } else if let Some(on) = &org_name {
                            parts.push(view! { <div class="pf-org-label">{on.clone()}</div> }.into_any());
                        }
                        // Name
                        if is_editing_name.get() && can_edit_here {
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
                                    on:dblclick=move |ev| { if can_edit_here { ev.stop_propagation(); set_editing.set(true); } }
                                >{name_header.clone()}</div>
                            }.into_any());
                        }
                        // Description
                        if is_editing_desc.get() && can_edit_here {
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
                                    on:dblclick=move |ev| { if can_edit_here { ev.stop_propagation(); set_editing.set(true); } }
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
                // Action strip — double-click on docs opens doc modal
                <div class="pf-list-actions" on:click=|ev| ev.stop_propagation()>
                    {let name_for_notif = name_for_doc_btn.clone();
                    move || {
                        let count = app_store.get().doc_notifications_for_portfolio(pid, &notification_store.get().notifications);
                        let pname = name_for_notif.clone();
                        let pname_click = pname.clone();
                        let pname_ctx = pname.clone();
                        let pname_keydown = pname.clone();
                        view! {
                            <span class="pf-notif-badge pf-notif-badge-clickable"
                                role="button"
                                tabindex="0"
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
                        class:active=move || ui_store.get().is_doc_modal_open(pid)
                        aria-label={format!("View documents for {} portfolio. {} document{}", name_for_doc_btn, doc_count, if doc_count == 1 { "" } else { "s" })}
                        on:click=move |_| ui_store.update(|s| s.toggle_doc_modal(pid))
                        on:dblclick=move |ev| { if can_edit_here { ev.stop_propagation(); ui_store.update(|s| s.open_doc_modal(pid)); } }
                    >
                        {format!("📄 {}", doc_count)}
                    </button>
                </div>
            </div>

            // Docs modal for portfolio
            {move || if ui_store.get().is_doc_modal_open(pid) {
                let modal_title = name_for_modal.clone();
                let add_cb = if can_edit_documents_here { Some(Callback::new(move |n: String| add_doc(n))) } else { None };
                view! {
                    <DocModal
                        entity_id={pid}
                        title={modal_title}
                        on_close=move || ui_store.update(|s| s.close_doc_modal(pid))
                        can_edit={can_edit_documents_here}
                        on_add={add_cb}
                        portfolio_id={Some(pid)}
                    />
                }.into_any()
            } else { ().into_any() }}

            {move || if is_editing_org.get() && can_edit_here {
                let users = org_users();
                let assigned = assigned_users.clone();
                view! {
                    <UserAssignmentPanel assigned={assigned} users={users} on_toggle={toggle_portfolio_assignment} />
                }.into_any()
            } else { ().into_any() }}

            // Expanded content — AssetViewer
            <div id={format!("pf-content-{}", pid)} class="asset-group-content" class:hidden={!expanded}>
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
