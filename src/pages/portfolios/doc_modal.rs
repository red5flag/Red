use crate::models::Document;
use crate::stores::{
    create_action, use_app_store, use_notification_store, use_organization_store,
    use_undo_redo_store,
};
use crate::types::ActionType;
use leptos::prelude::*;
use uuid::Uuid;

use super::{document_icon, shorthand_name};

/// Generate mock document content for the in-app viewer based on name and type.
fn mock_doc_content(name: &str, file_type: &str) -> String {
    match file_type.to_lowercase().as_str() {
        "pdf" => format!(
            "DOCUMENT: {name}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Generated: {date}
Reference: DOC-{ref_num}
Status: ACTIVE

SUMMARY
This document serves as an official record pertaining to {name}.
All details contained herein have been verified and are accurate
as of the date of generation.

CONTENT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Section 1 — Overview
This section provides a high-level summary of the subject matter
described by this document. All parties are advised to review
the complete contents before proceeding.

Section 2 — Details
Full legal description and relevant information specific to the
named subject has been recorded. Supporting evidence is appended
at the rear of this document.

Section 3 — Certification
This document has been certified and notarised. Any alterations
render this document void. Contact the issuing authority for
certified copies.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Page 1 of 1  |  CONFIDENTIAL",
            name = name,
            date = "22 Jun 2025",
            ref_num = "28471",
        ),
        "docx" => format!(
            "{name}

Prepared by: Red Asset Management
Date: 22 June 2025
Version: 1.0

──────────────────────────────────────

INTRODUCTION

This document outlines the key terms and conditions associated
with {name}. It has been prepared in accordance with applicable
regulations and internal policy.

MAIN BODY

The following details apply to the subject of this document:

  • All parties have been duly notified of their obligations.
  • The effective date is confirmed as 1 January 2025.
  • Terms are binding for a period of 12 months unless varied.
  • Renewal is subject to mutual agreement in writing.

SIGNATURE BLOCK

Authorised by: ________________________
Position:      Portfolio Manager
Date:          22 / 06 / 2025",
            name = name,
        ),
        "xlsx" => format!(
            "┌─────────────────────────────────────────────────────────┐
│  {name:<55}│
│  Generated: 22 Jun 2025                                 │
├───────────────────┬──────────────┬──────────────────────┤
│  Description      │  Value       │  Notes               │
├───────────────────┼──────────────┼──────────────────────┤
│  Opening Balance  │  $1,200,000  │  FY2024              │
│  Acquisitions     │  $340,000    │  Q1-Q2               │
│  Disposals        │  -$80,000    │  Q3                  │
│  Revaluations     │  $120,000    │  Per valuer report   │
│  Closing Balance  │  $1,580,000  │  FY2025              │
├───────────────────┼──────────────┼──────────────────────┤
│  Net Change       │  +$380,000   │  +31.7%              │
└───────────────────┴──────────────┴──────────────────────┘

  Notes:
  All figures are in AUD. Subject to audit adjustment.
  Prepared by Finance — Internal Use Only.",
            name = name,
        ),
        "txt" => format!(
            "Document: {name}
Date: 22 June 2025

This is a plain-text record associated with the above document.
No special formatting is required for this file type.

Key points:
- Document is current as of the date above.
- Retain for a minimum of 7 years per policy.
- Any queries should be directed to the portfolio manager.",
            name = name,
        ),
        _ => format!(
            "Document: {name}\n\nNo preview available for this file type ({file_type}).",
            name = name,
            file_type = file_type
        ),
    }
}
#[component]
pub fn DocModal(
    entity_id: Uuid,
    title: String,
    on_close: impl Fn() + Send + Sync + 'static,
    can_edit: bool,
    on_add: Option<Callback<String>>,
    #[prop(default = None)] portfolio_id: Option<Uuid>,
    #[prop(default = None)] group_id: Option<Uuid>,
    #[prop(default = None)] asset_id: Option<Uuid>,
    #[prop(default = None)] organization_id: Option<Uuid>,
) -> impl IntoView {
    let app_store = use_app_store();
    let organization_store = use_organization_store();
    let notification_store = use_notification_store();
    // open_tabs: vec of (tab_id, Document); tab_id=0 is reserved for the list tab
    let (open_tabs, set_open_tabs) = signal::<Vec<(u32, Document)>>(vec![]);
    let (active_tab, set_active_tab) = signal::<u32>(0); // 0 = list view
    let (next_id, set_next_id) = signal(1u32);
    let (new_doc_name, set_new_doc_name) = signal(String::new());
    let title_stored = StoredValue::new(title);

    // Reactive document list read directly from the store so additions show immediately.
    let docs = Memo::new(move |_| {
        let store = app_store.get();
        let mut docs = Vec::new();
        if let Some(oid) = organization_id {
            if let Some(org) = organization_store.get().get_organization(oid) {
                docs.extend(org.documents.clone());
            }
        }
        for p in &store.portfolios {
            if p.id == entity_id {
                docs.extend(p.documents.clone());
            }
            for g in &p.asset_groups {
                if g.id == entity_id {
                    docs.extend(g.documents.clone());
                }
                for a in &g.assets {
                    if a.id == entity_id {
                        docs.extend(a.documents.clone());
                    }
                }
            }
            for a in &p.assets {
                if a.id == entity_id {
                    docs.extend(a.documents.clone());
                }
            }
        }
        docs
    });
    let on_close = std::sync::Arc::new(on_close);
    let on_close2 = on_close.clone();

    let open_doc_tab = move |doc: Document| {
        // don't duplicate — if already open, switch to it
        let existing = open_tabs
            .get()
            .into_iter()
            .find(|(_, d)| d.id == doc.id)
            .map(|(id, _)| id);
        if let Some(id) = existing {
            set_active_tab.set(id);
            return;
        }
        let id = next_id.get();
        set_next_id.set(id + 1);
        set_open_tabs.update(|v| v.push((id, doc)));
        set_active_tab.set(id);
    };

    let close_tab = move |tid: u32| {
        set_open_tabs.update(|v| v.retain(|(id, _)| *id != tid));
        // fall back to list if this was the active tab
        set_active_tab.update(|cur| {
            if *cur == tid {
                *cur = 0;
            }
        });
    };

    view! {
        <div class="doc-modal-overlay" on:click=move |_| on_close()>
            <div class="doc-modal doc-modal-tabbed" on:click=|ev| ev.stop_propagation()>

                // ── Modal header ───────────────────────────────────────
                <div class="doc-modal-header">
                    <span class="doc-modal-title" id="doc-modal-title">"📄 " {title_stored.get_value()}</span>
                    <button class="doc-modal-close" aria-label={format!("Close document viewer for {}", title_stored.get_value())} on:click=move |_| on_close2()>"✕"</button>
                </div>

                // ── Tab strip (always visible at top) ──────────────────
                <div class="dv-tab-strip">
                    // List tab (always present)
                    <div class="dv-tab"
                        class:dv-tab-active=move || active_tab.get() == 0
                        on:click=move |_| set_active_tab.set(0)>
                        <span class="dv-tab-icon">"☰"</span>
                        <span class="dv-tab-name">"List"</span>
                    </div>
                    // Open document tabs
                    <For
                        each=move || open_tabs.get()
                        key=|tab| tab.0
                        children=move |(tid, doc)| {
                            let icon = document_icon(&doc.file_type);
                            let dname = shorthand_name(&doc.name);
                            let close_tab_c = close_tab.clone();
                            view! {
                                <div class="dv-tab"
                                    class:dv-tab-active=move || active_tab.get() == tid
                                    on:click=move |_| set_active_tab.set(tid)>
                                    <span class="dv-tab-icon">{icon}</span>
                                    <span class="dv-tab-name">{dname.clone()}</span>
                                    <button class="dv-tab-close"
                                        aria-label={format!("Close tab for {}", dname)}
                                        on:click=move |ev| {
                                            ev.stop_propagation();
                                            close_tab_c(tid);
                                        }>"✕"</button>
                                </div>
                            }
                        }
                    />
                </div>

                // ── Panel: list view (tab 0) ───────────────────────────
                {move || if active_tab.get() == 0 {
                    let on_add_cb = on_add.clone();
                    view! {
                        <div class="doc-modal-body">
                            <div class="doc-modal-list">
                                <For
                                    each=move || docs.get()
                                    key=|doc| doc.id
                                    children=move |doc| {
                                        let icon = document_icon(&doc.file_type);
                                        let ft   = doc.file_type.to_uppercase();
                                        let current_user = app_store.get().current_user.clone();
                                        let can_edit_doc = can_edit && current_user.can_edit_document(&doc);
                                        let doc_for_open = doc.clone();
                                        let doc_for_tap = doc.clone();
                                        let doc_id = doc.id;
                                        let doc_name_for_label = doc.name.clone();
                                        let (editing_name, set_editing_name) = signal(false);
                                        let (edit_name, set_edit_name) = signal(doc.name.clone());
                                        view! {
                                            <div class="doc-modal-row">
                                                <div class="doc-modal-icon-wrap">
                                                    <span class="doc-modal-icon">{icon}</span>
                                                </div>
                                                <div class="doc-modal-info"
                                                    class:doc-modal-info-tap=can_edit_doc
                                                    on:click=move |ev: leptos::ev::MouseEvent| {
                                                        if can_edit_doc && !editing_name.get() {
                                                            ev.stop_propagation();
                                                            open_doc_tab(doc_for_tap.clone());
                                                        }
                                                    }
                                                >
                                                    {move || if editing_name.get() {
                                                        view! {
                                                            <input
                                                                class="doc-modal-edit-input"
                                                                aria-label="Document name"
                                                                prop:value=move || edit_name.get()
                                                                on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                                                on:blur=move |_| {
                                                                    let n = edit_name.get();
                                                                    if !n.trim().is_empty() {
                                                                        if let Some(_oid) = organization_id {
                                                                            organization_store.update(|s| s.update_document_name(doc_id, n));
                                                                        } else {
                                                                            app_store.update(|s| s.update_document_name(doc_id, n, &mut notification_store.get_untracked()));
                                                                        }
                                                                    }
                                                                    set_editing_name.set(false);
                                                                }
                                                            />
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <span class="doc-modal-name">{doc.name.clone()}</span>
                                                            <span class="doc-modal-ft">{ft.clone()}</span>
                                                        }.into_any()
                                                    }}
                                                </div>
                                                {move || if can_edit_doc && !editing_name.get() {
                                                    view! {
                                                        <button class="doc-modal-edit-btn"
                                                            aria-label={format!("Rename document {}", doc_name_for_label)}
                                                            on:click=move |_| set_editing_name.set(true)>
                                                            "✎"
                                                        </button>
                                                    }.into_any()
                                                } else { ().into_any() }}
                                                {move || {
                                                    let notifs = notification_store.get().notifications_list_for_doc(doc_id);
                                                    if notifs.is_empty() {
                                                        ().into_any()
                                                    } else {
                                                        let n = notifs[0].clone();
                                                        let _nid = n.id;
                                                        let from_user = n.from_user.clone().unwrap_or_else(|| "System".to_string());
                                                        let preview = n.content_preview.clone();
                                                        // Truncate note to less than a sentence (~60 chars)
                                                        let short_note = preview.as_ref().map(|p| {
                                                            let truncated = if p.len() > 60 {
                                                                // Find a good break point
                                                                let slice = &p[..60];
                                                                if let Some(idx) = slice.rfind(|c: char| c == ' ' || c == ',' || c == '.') {
                                                                    &p[..idx]
                                                                } else {
                                                                    slice
                                                                }
                                                            } else {
                                                                p.as_str()
                                                            };
                                                            format!("— {}", truncated)
                                                        }).unwrap_or_default();
                                                        view! {
                                                            <span class="doc-notif-label">
                                                                "Linked (Document) by " <strong>{from_user}</strong> " " {short_note}
                                                            </span>
                                                            <button class="doc-notif-view-btn"
                                                                on:click=move |ev: leptos::ev::MouseEvent| {
                                                                    ev.stop_propagation();
                                                                    app_store.update(|s| s.navigate_to_notification(&n));
                                                                    notification_store.update(|s| s.close_drawer());
                                                                }>
                                                                "View Content"
                                                            </button>
                                                        }.into_any()
                                                    }
                                                }}
                                                <button class="doc-modal-open-btn"
                                                    on:click=move |_| open_doc_tab(doc_for_open.clone())>
                                                    "Open"
                                                </button>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                            {if can_edit {
                                view! {
                                    <div class="doc-modal-add-row">
                                        <input class="doc-modal-add-input" type="text"
                                            placeholder="New document name…"
                                            aria-label="New document name"
                                            prop:value=move || new_doc_name.get()
                                            on:input=move |ev| set_new_doc_name.set(event_target_value(&ev)) />
                                        <button class="doc-modal-add-btn"
                                            on:click=move |_| {
                                                let n = new_doc_name.get();
                                                if !n.trim().is_empty() {
                                                    if let Some(cb) = &on_add_cb { cb.run(n); }
                                                    set_new_doc_name.set(String::new());
                                                }
                                            }>
                                            "+ Add"
                                        </button>
                                    </div>
                                }.into_any()
                            } else { ().into_any() }}
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                // ── Panel: document viewer tabs ────────────────────────
                {move || {
                    let cur = active_tab.get();
                    open_tabs.get().into_iter().filter_map(|(tid, doc)| {
                        if tid != cur { return None; }
                        Some(view! {
                            <DocumentViewer
                                doc={doc}
                                on_close=move || close_tab(tid)
                                can_edit={can_edit}
                                portfolio_id={portfolio_id}
                                group_id={group_id}
                                asset_id={asset_id}
                            />
                        })
                    }).collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}
/// In-app document viewer — sticky toolbar, zoom, edit mode, inline editing, image popup, why/save.
#[component]
pub fn DocumentViewer(
    doc: Document,
    on_close: impl Fn() + Send + Sync + 'static,
    #[prop(default = false)] can_edit: bool,
    #[prop(default = None)] portfolio_id: Option<Uuid>,
    #[prop(default = None)] group_id: Option<Uuid>,
    #[prop(default = None)] asset_id: Option<Uuid>,
) -> impl IntoView {
    let app_store = use_app_store();
    let organization_store = use_organization_store();
    let notification_store = use_notification_store();
    let undo_store = use_undo_redo_store();
    let initial_content = doc
        .content
        .clone()
        .unwrap_or_else(|| mock_doc_content(&doc.name, &doc.file_type));
    let doc_name = StoredValue::new(doc.name.clone());
    let doc_id = doc.id;
    let doc_url = StoredValue::new(doc.url.clone());

    let current_user = app_store.get().current_user.clone();
    let effective_can_edit = can_edit && current_user.can_edit_document(&doc);

    // viewer state
    let (zoom, set_zoom) = signal(100u32); // percent
    let (edit_mode, set_edit_mode) = signal(effective_can_edit);
    let (content, set_content) = signal(initial_content);
    let (why, set_why) = signal(String::new());
    let (notes, set_notes) = signal(String::new());
    // image popup: Some((x_px, y_px))
    let (img_popup, set_img_popup) = signal::<Option<(i32, i32)>>(None);
    let (link_val, set_link_val) = signal(doc.url.clone());
    let (file_type, set_file_type) = signal(doc.file_type.clone());
    let (show_type_dropdown, set_show_type_dropdown) = signal(false);

    let is_image = move || {
        matches!(
            file_type.get().to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "tiff" | "ico"
        )
    };
    let is_video = move || {
        matches!(
            file_type.get().to_lowercase().as_str(),
            "mp4" | "mov" | "avi" | "mkv" | "webm" | "flv"
        )
    };
    let is_sheet = move || file_type.get() == "xlsx" || file_type.get() == "csv";

    let apply_image_url = move || {
        let url = link_val.get().trim().to_string();
        if !url.is_empty() {
            app_store.update(|s| {
                for p in s.portfolios.iter_mut() {
                    for d in &mut p.documents {
                        if d.id == doc_id {
                            d.url = url.clone();
                        }
                    }
                    for g in &mut p.asset_groups {
                        for d in &mut g.documents {
                            if d.id == doc_id {
                                d.url = url.clone();
                            }
                        }
                        for a in &mut g.assets {
                            for d in &mut a.documents {
                                if d.id == doc_id {
                                    d.url = url.clone();
                                }
                            }
                        }
                    }
                    for a in &mut p.assets {
                        for d in &mut a.documents {
                            if d.id == doc_id {
                                d.url = url.clone();
                            }
                        }
                    }
                }
            });
        }
        set_img_popup.set(None);
    };

    let on_close = std::sync::Arc::new(on_close);
    let on_close_toolbar = on_close.clone();

    let save_doc = move || {
        let new_content = content.get();
        let reason = why.get();
        let notes_text = notes.get();
        let reason_for_action = if reason.trim().is_empty() {
            None
        } else {
            Some(reason.clone())
        };
        let doc_name_val = doc_name.get_value();
        let pid = portfolio_id;
        let gid = group_id;
        let aid = asset_id;
        app_store.update(|s| {
            for p in s.portfolios.iter_mut() {
                for d in &mut p.documents {
                    if d.id == doc_id {
                        d.content = Some(new_content.clone());
                    }
                }
                for g in &mut p.asset_groups {
                    for d in &mut g.documents {
                        if d.id == doc_id {
                            d.content = Some(new_content.clone());
                        }
                    }
                    for a in &mut g.assets {
                        for d in &mut a.documents {
                            if d.id == doc_id {
                                d.content = Some(new_content.clone());
                            }
                        }
                    }
                }
                for a in &mut p.assets {
                    for d in &mut a.documents {
                        if d.id == doc_id {
                            d.content = Some(new_content.clone());
                        }
                    }
                }
            }
        });
        // Send notification with notes and @username parsing
        let (updater_name, organization_users) = {
            let s = app_store.get();
            let org = organization_store.get();
            (s.current_user.name.clone(), org.organization_users.clone())
        };
        notification_store.update(|s| {
            s.add_document_update_with_notes(
                doc_id,
                &doc_name_val,
                &notes_text,
                &updater_name,
                pid,
                gid,
                aid,
                &organization_users,
            );
        });
        let (uid, name, role, org) = {
            let s = app_store.get();
            (
                s.current_user.id,
                s.current_user.name.clone(),
                format!("{:?}", s.current_user.role),
                s.current_user.organization_id,
            )
        };
        undo_store.update(|u| {
            u.record_action(create_action(
                ActionType::Update,
                "Document",
                &format!("Updated content of document '{}'", doc_name.get_value()),
                uid,
                &name,
                &role,
                org,
                reason_for_action,
            ));
        });
        set_edit_mode.set(false);
        set_notes.set(String::new());
    };

    view! {
        <div class="docviewer">
            // ── Sticky toolbar ────────────────────────────────────────
            <div class="docviewer-toolbar">
                <span class="docviewer-icon">{move || document_icon(&file_type.get())}</span>
                <span class="docviewer-name">{doc_name.get_value()}</span>
                // Document type selector
                <div class="dv-type-selector">
                    <button class="docviewer-ft dv-type-btn"
                        on:click=move |_| set_show_type_dropdown.update(|v| *v = !*v)>
                        {move || file_type.get().to_uppercase()}
                        <span class="dv-type-arrow">{move || if show_type_dropdown.get() { "▶" } else { "▼" }}</span>
                    </button>
                    {move || if show_type_dropdown.get() {
                        let type_options = ["pdf", "docx", "txt", "odt", "rtf", "xlsx", "csv", "pptx", "md",
                            "jpg", "jpeg", "png", "gif", "webp", "svg",
                            "mp4", "mov", "avi", "webm",
                            "mp3", "wav", "zip"];
                        let current_ft = file_type.get();
                        view! {
                            <div class="dv-type-dropdown-overlay" on:click=move |_| set_show_type_dropdown.set(false)>
                                <div class="dv-type-dropdown" on:click=|ev| ev.stop_propagation()>
                                    {type_options.iter().map(|opt| {
                                        let opt_str = opt.to_string();
                                        let is_active = current_ft == opt_str;
                                        let opt_for_click = opt_str.clone();
                                        view! {
                                            <button class="dv-type-option" class:dv-type-option-active={is_active}
                                                on:click=move |_| {
                                                    set_file_type.set(opt_for_click.clone());
                                                    set_show_type_dropdown.set(false);
                                                    app_store.update(|s| s.update_document_file_type(doc_id, opt_for_click.clone()));
                                                }>
                                                <span>{document_icon(opt)}</span>
                                                <span>{opt.to_uppercase()}</span>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                </div>

                // Zoom controls
                <div class="dv-zoom-group">
                    <button class="dv-toolbar-btn"
                        aria-label="Zoom out"
                        on:click=move |_| set_zoom.update(|z| *z = (*z).saturating_sub(10).max(50))>
                        "−"
                    </button>
                    <span class="dv-zoom-label">{move || format!("{}%", zoom.get())}</span>
                    <button class="dv-toolbar-btn"
                        aria-label="Zoom in"
                        on:click=move |_| set_zoom.update(|z| *z = (*z + 10).min(300))>
                        "+"
                    </button>
                    <button class="dv-toolbar-btn"
                        aria-label="Reset zoom"
                        on:click=move |_| set_zoom.set(100)>
                        "⟳"
                    </button>
                </div>

                // Edit toggle (only when effective_can_edit)
                {if effective_can_edit {
                    view! {
                        <button class="dv-toolbar-btn dv-edit-toggle"
                            class:dv-edit-active=move || edit_mode.get()
                            on:click=move |_| set_edit_mode.update(|v| *v = !*v)>
                            {move || if edit_mode.get() { "👁 Read" } else { "✎ Edit" }}
                        </button>
                        {move || if edit_mode.get() {
                            view! {
                                <button class="dv-toolbar-btn dv-save-btn" on:click=move |_| save_doc()>
                                    "✔ Save"
                                </button>
                            }.into_any()
                        } else { ().into_any() }}
                    }.into_any()
                } else { ().into_any() }}

                <button class="docviewer-back" aria-label={format!("Back to document list for {}", doc_name.get_value())} on:click=move |_| on_close_toolbar()>"← Back"</button>
            </div>

            // ── Document body ─────────────────────────────────────────
            <div
                class={move || if is_sheet() { "docviewer-body docviewer-sheet".to_string() } else { "docviewer-body".to_string() }}
                style=move || format!("font-size: {}%;", zoom.get())
                on:click=move |_| { if img_popup.get().is_some() { apply_image_url(); } }
            >
                // Media rendering area (images and videos)
                {move || {
                    let url = link_val.get();
                    if is_image() {
                        if url != "#" && !url.is_empty() {
                            view! {
                                <div class="dv-media-area">
                                    <img class="dv-media-img" src={url.clone()} alt={doc_name.get_value()} />
                                </div>
                            }.into_any()
                        } else if effective_can_edit {
                            view! {
                                <div class="dv-media-placeholder dv-media-img-placeholder"
                                    class:dv-editable=move || edit_mode.get()
                                    on:click=move |ev: leptos::ev::MouseEvent| {
                                        if edit_mode.get() {
                                            ev.stop_propagation();
                                            set_link_val.set(doc_url.get_value());
                                            set_img_popup.set(Some((ev.client_x(), ev.client_y())));
                                        }
                                    }
                                >
                                    {move || if edit_mode.get() {
                                        view! { <span class="dv-img-hint">"🖼 Click to set image URL"</span> }.into_any()
                                    } else { view! { <span class="dv-img-hint dv-img-muted">"🖼 No image set"</span> }.into_any() }}
                                </div>
                            }.into_any()
                        } else { ().into_any() }
                    } else if is_video() {
                        if url != "#" && !url.is_empty() {
                            view! {
                                <div class="dv-media-area">
                                    <video class="dv-media-video" src={url.clone()} controls=true>
                                        "Your browser does not support video playback."
                                    </video>
                                </div>
                            }.into_any()
                        } else if effective_can_edit {
                            view! {
                                <div class="dv-media-placeholder dv-media-video-placeholder"
                                    class:dv-editable=move || edit_mode.get()
                                    on:click=move |ev: leptos::ev::MouseEvent| {
                                        if edit_mode.get() {
                                            ev.stop_propagation();
                                            set_link_val.set(doc_url.get_value());
                                            set_img_popup.set(Some((ev.client_x(), ev.client_y())));
                                        }
                                    }
                                >
                                    {move || if edit_mode.get() {
                                        view! { <span class="dv-img-hint">"🎬 Click to set video URL"</span> }.into_any()
                                    } else { view! { <span class="dv-img-hint dv-img-muted">"🎬 No video set"</span> }.into_any() }}
                                </div>
                            }.into_any()
                        } else { ().into_any() }
                    } else {
                        ().into_any()
                    }
                }}

                // Image/media URL popup (appears at cursor position)
                {move || if let Some((cx, cy)) = img_popup.get() {
                    view! {
                        <div class="dv-img-popup"
                            style=move || format!("left:{}px;top:{}px;", cx, cy)
                            on:click=|ev| ev.stop_propagation()>
                            <div class="dv-img-popup-opt"
                                on:click=move |_| {
                                    set_img_popup.set(None);
                                }
                            >
                                <span class="dv-img-opt-icon">"📁"</span>
                                <span>"Upload"</span>
                            </div>
                            <div class="dv-img-popup-opt">
                                <span class="dv-img-opt-icon">"🔗"</span>
                                <input
                                    class="dv-img-link-input"
                                    placeholder="Paste media URL…"
                                    aria-label="Media URL"
                                    prop:value=move || link_val.get()
                                    on:input=move |ev| set_link_val.set(event_target_value(&ev))
                                    on:click=|ev| ev.stop_propagation()
                                    on:keydown=move |ev| { if ev.key() == "Enter" { apply_image_url(); } }
                                />
                            </div>
                            <div class="dv-img-popup-opt"
                                on:click=move |_| {
                                    apply_image_url();
                                }
                            >
                                <span class="dv-img-opt-icon">"✔"</span>
                                <span>"Apply URL"</span>
                            </div>
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                // Text content — editable textarea in edit mode, pre otherwise
                // (hidden for pure media types in read mode when URL is set)
                {move || if edit_mode.get() {
                    view! {
                        <textarea
                            class="docviewer-content dv-editable-text"
                            prop:value=move || content.get()
                            on:input=move |ev| set_content.set(event_target_value(&ev))
                        />
                        <div class="dv-why-row">
                            <label class="dv-why-label">"Why are you making this change?"</label>
                            <textarea
                                class="dv-why-input"
                                aria-label="Reason for this update"
                                placeholder="Optional reason for this update…"
                                prop:value=move || why.get()
                                on:input=move |ev| set_why.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="dv-why-row">
                            <label class="dv-why-label">"Notes — tag people with @username to notify them"</label>
                            <textarea
                                class="dv-why-input dv-notes-input"
                                aria-label="Notes for reviewers"
                                placeholder="Add notes for reviewers. Use @username to tag people (e.g. @red please review section 3)…"
                                prop:value=move || notes.get()
                                on:input=move |ev| set_notes.set(event_target_value(&ev))
                            />
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <pre class="docviewer-content">{move || content.get()}</pre>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
