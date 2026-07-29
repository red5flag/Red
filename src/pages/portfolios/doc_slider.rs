use crate::stores::{use_app_store, use_ui_store};
use leptos::prelude::*;
use uuid::Uuid;

use super::{
    clamp_context_menu, document_icon, download_document, mobile_tab_gestures, set_drag_tab_data,
    NavTabKind, shorthand_name, DocEntry, DocModal,
};

#[derive(Clone)]
enum ModalTarget {
    Add,
    View(DocEntry),
}

/// Inline horizontal document slider for portfolio/group/asset parents.
/// Appears under the parent row and replicates the asset-style slider:
/// +Document card, scrollable doc cards, and a DocModal for add/view.
#[component]
pub fn DocSlider(
    entity_id: Uuid,
    title: String,
    entity_name: String,
    #[prop(into)] can_edit_documents: Signal<bool>,
    #[prop(into)] entries: Signal<Vec<DocEntry>>,
    on_add: Option<Callback<String>>,
    #[prop(default = None)] portfolio_id: Option<Uuid>,
    #[prop(default = None)] group_id: Option<Uuid>,
    #[prop(default = None)] asset_id: Option<Uuid>,
    #[prop(default = None)] organization_id: Option<Uuid>,
) -> impl IntoView {
    let (modal_target, set_modal_target) = signal(Option::<ModalTarget>::None);
    let (doc_ctx, set_doc_ctx) = signal(Option::<(i32, i32, DocEntry)>::None);
    let ui_store = use_ui_store();
    let app_store = use_app_store();

    // External triggers (e.g. context menu "Add Document") open the add modal.
    Effect::new(move |_| {
        if ui_store.get().is_doc_modal_open(entity_id) {
            set_modal_target.set(Some(ModalTarget::Add));
            ui_store.update(|s| s.close_doc_modal(entity_id));
        }
    });

    // Deep-link from search: if a specific document is requested, open its viewer
    // and scroll the card into view.
    Effect::new(move |_| {
        let doc_id = app_store.get().pending_doc_view;
        if let Some(did) = doc_id {
            let entries = entries.get();
            if let Some(entry) = entries.iter().find(|e| e.doc.id == did).cloned() {
                set_modal_target.set(Some(ModalTarget::View(entry)));
                app_store.update(|s| s.pending_doc_view = None);
                crate::utils::dom::scroll_to_id(&format!("doc-card-{}", did));
            }
        }
    });

    let open_add = move || set_modal_target.set(Some(ModalTarget::Add));

    let open_view = move |entry: DocEntry| {
        set_modal_target.set(Some(ModalTarget::View(entry)));
    };

    let close_modal = move || {
        set_modal_target.set(None);
    };

    view! {
        <div class="pf-doc-slider" on:click=|ev| ev.stop_propagation()>
            <div class="ai-doc-slider">
                {move || if can_edit_documents.get() {
                    view! {
                        <div
                            class="ai-doc-slider-item ai-doc-add-card"
                            aria-label={format!("Add document to {}", entity_name)}
                            on:click=move |_| open_add()
                        >
                            <div class="ai-doc-slider-thumb">"➕"</div>
                            <div class="ai-doc-slider-name">"+ Document"</div>
                            <div class="ai-doc-slider-type">"ADD"</div>
                        </div>
                    }.into_any()
                } else { ().into_any() }}
                <For
                    each=move || entries.get()
                    key=|entry| entry.doc.id
                    children=move |entry: DocEntry| {
                        let icon = document_icon(&entry.doc.file_type);
                        let ft = entry.doc.file_type.to_uppercase();
                        let short_name = shorthand_name(&entry.doc.name);
                        let doc_for_click = entry.clone();
                        let doc_for_ctx = entry.clone();
                        let doc_for_ctx_touch = doc_for_ctx.clone();
                        let (touch_start, touch_move, touch_end) = mobile_tab_gestures(
                            NavTabKind::Document,
                            entry.doc.id,
                            entry.doc.name.clone(),
                            move |x, y| {
                                set_doc_ctx.set(Some((x, y, doc_for_ctx_touch.clone())));
                            },
                            move || set_doc_ctx.set(None),
                        );
                        view! {
                            <div
                                class="ai-doc-slider-item"
                                id={format!("doc-card-{}", entry.doc.id)}
                                aria-label={format!("View document {}. Type {}", entry.doc.name, ft)}
                                draggable="true"
                                on:dragstart=move |ev: leptos::ev::DragEvent| {
                                    set_drag_tab_data(&ev, &NavTabKind::Document, entry.doc.id, &entry.doc.name);
                                }
                                on:click=move |_| open_view(doc_for_click.clone())
                                on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                                    ev.prevent_default();
                                    ev.stop_propagation();
                                    let (x, y) = clamp_context_menu(ev.client_x(), ev.client_y());
                                    set_doc_ctx.set(Some((x, y, doc_for_ctx.clone())));
                                }
                                on:touchstart=touch_start
                                on:touchmove=touch_move
                                on:touchend=touch_end
                            >
                                <div class="ai-doc-slider-thumb">{icon}</div>
                                <div class="ai-doc-slider-name">{short_name}</div>
                                <div class="ai-doc-slider-type">{ft.clone()}</div>
                            </div>
                        }
                    }
                />

                {move || doc_ctx.get().map(|(x, y, entry)| {
                    let entry_for_download = entry.doc.clone();
                    let open_name = entry.doc.name.clone();
                    view! {
                        <div
                            class="context-menu-overlay"
                            on:click=move |_| set_doc_ctx.set(None)
                        >
                            <div class="context-menu" style={format!("left: {}px; top: {}px;", x, y)}>
                                <button
                                    class="context-menu-item"
                                    on:click=move |_| {
                                        set_doc_ctx.set(None);
                                        ui_store.update(|s| s.open_nav_tab("document", entry.doc.id, &open_name));
                                    }
                                >"🗔 Open in New Tab"</button>
                                <button
                                    class="context-menu-item"
                                    on:click=move |_| {
                                        set_doc_ctx.set(None);
                                        download_document(&entry_for_download);
                                    }
                                >"📥 Export / Download"</button>
                            </div>
                        </div>
                    }.into_any()
                })}
            </div>
        </div>

        {move || modal_target.get().map(|target| {
            match target {
                ModalTarget::Add => view! {
                    <DocModal
                        entity_id={entity_id}
                        title={title.clone()}
                        on_close=close_modal
                        can_edit={can_edit_documents.get()}
                        on_add={on_add}
                        portfolio_id={portfolio_id}
                        group_id={group_id}
                        asset_id={asset_id}
                        organization_id={organization_id}
                    />
                }.into_any(),
                ModalTarget::View(entry) => view! {
                    <DocModal
                        entity_id={entry
                            .organization_id
                            .or(entry.asset_id)
                            .or(entry.group_id)
                            .or(entry.portfolio_id)
                            .unwrap_or(entry.doc.id)}
                        title={entry.doc.name.clone()}
                        on_close=close_modal
                        can_edit={can_edit_documents.get()}
                        on_add={None}
                        portfolio_id={entry.portfolio_id}
                        group_id={entry.group_id}
                        asset_id={entry.asset_id}
                        organization_id={entry.organization_id}
                        initial_doc={Some(entry.doc)}
                    />
                }.into_any(),
            }
        })}
    }
}
