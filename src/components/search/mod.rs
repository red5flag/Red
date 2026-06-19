use crate::stores::use_search_store;
use leptos::prelude::*;

#[component]
pub fn SearchFilters() -> impl IntoView {
    let search_store = use_search_store();

    let available_tags = move || search_store.get().available_tags.clone();
    let selected_tags = move || search_store.get().selected_tags.clone();

    view! {
        <div class="search-dropdown">
            <div class="search-filters">
                <span style="font-size: 12px; color: var(--text-secondary);">
                    "Filter by:"
                </span>
                {move || {
                    available_tags()
                        .into_iter()
                        .map(|tag| {
                            let is_selected = selected_tags().contains(&tag);
                            let tag_clone = tag.clone();
                            view! {
                                <button
                                    class="filter-tag"
                                    class:active=is_selected
                                    on:click=move |_| {
                                        search_store.update(|store| {
                                            store.toggle_tag(tag_clone.clone());
                                        });
                                    }
                                >
                                    {tag}
                                </button>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>

            <div style="border-top: 2px solid var(--border-color); padding-top: 12px;">
                <span style="font-size: 12px; color: var(--text-secondary);">
                    "Suggestions:"
                </span>
                <div class="tag-container" style="margin-top: 8px;">
                    {move || {
                        search_store
                            .get()
                            .get_contextual_suggestions()
                            .into_iter()
                            .map(|suggestion| {
                                view! {
                                    <span class="tag">{suggestion}</span>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn SearchResults() -> impl IntoView {
    let search_store = use_search_store();

    let is_loading = move || search_store.get().is_loading;
    let has_results = move || search_store.get().results.total_count > 0;
    let results_count = move || search_store.get().results.total_count;

    view! {
        <div style="padding: 16px; background: var(--secondary-bg); border: 2px solid var(--border-color); margin-top: -2px;">
            {move || {
                if is_loading() {
                    view! {
                        <div class="loading">
                            <span class="loading-text">"Searching..."</span>
                        </div>
                    }
                        .into_any()
                } else if has_results() {
                    view! {
                        <div>
                            <span style="font-size: 12px; color: var(--text-secondary);">
                                {format!("Found {} results", results_count())}
                            </span>
                        </div>
                    }
                    .into_any()
                } else {
                    view! {
                        <div class="empty-state">
                            <div class="empty-icon">"🔍"</div>
                            <div class="empty-text">"No results found"</div>
                        </div>
                    }
                    .into_any()
                }
            }}
        </div>
    }
}
