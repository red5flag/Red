use crate::stores::use_ui_store;
use leptos::prelude::*;
use uuid::Uuid;

use super::{document_icon, shorthand_name, DocEntry, DocModal};

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
    let ui_store = use_ui_store();

    // External triggers (e.g. context menu "Add Document") open the add modal.
    Effect::new(move |_| {
        if ui_store.get().is_doc_modal_open(entity_id) {
            set_modal_target.set(Some(ModalTarget::Add));
            ui_store.update(|s| s.close_doc_modal(entity_id));
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
                        view! {
                            <div
                                class="ai-doc-slider-item"
                                aria-label={format!("View document {}. Type {}", entry.doc.name, ft)}
                                on:click=move |_| open_view(doc_for_click.clone())
                            >
                                <div class="ai-doc-slider-thumb">{icon}</div>
                                <div class="ai-doc-slider-name">{short_name}</div>
                                <div class="ai-doc-slider-type">{ft.clone()}</div>
                            </div>
                        }
                    }
                />
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
