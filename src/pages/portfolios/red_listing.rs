use crate::stores::{use_app_store, use_ui_store, AppStore};
use leptos::prelude::*;
use uuid::Uuid;

use super::{csv_to_list, MarketplaceListing};

/// Which portfolio entity a Red Marketplace listing describes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RedEntityKind {
    Portfolio,
    AssetGroup,
    Asset,
}

impl RedEntityKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            RedEntityKind::Portfolio => "Portfolio",
            RedEntityKind::AssetGroup => "Asset Group",
            RedEntityKind::Asset => "Asset",
        }
    }

    pub(crate) fn icon(self) -> &'static str {
        match self {
            RedEntityKind::Portfolio => "📁",
            RedEntityKind::AssetGroup => "📂",
            RedEntityKind::Asset => "📦",
        }
    }
}

/// A Red Marketplace listing resolved for the browse page — the listing
/// plus the context needed to display and sort it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RedBrowseItem {
    pub kind: RedEntityKind,
    pub entity_id: Uuid,
    pub portfolio_id: Uuid,
    pub portfolio_name: String,
    pub owner_id: Uuid,
    pub listing: MarketplaceListing,
    pub image_url: Option<String>,
}

fn parse_red_listing(meta: &serde_json::Value) -> Option<MarketplaceListing> {
    serde_json::from_value(meta.get("red_listing")?.clone()).ok()
}

/// Collect every Red Marketplace listing across portfolios, groups and
/// assets. The asset card stores its Red listing at the same
/// `metadata["red_listing"]` slot, so one shape covers all three kinds.
pub(crate) fn collect_red_listings(app: &AppStore) -> Vec<RedBrowseItem> {
    let mut items = Vec::new();
    for p in &app.portfolios {
        if let Some(listing) = parse_red_listing(&p.metadata) {
            items.push(RedBrowseItem {
                kind: RedEntityKind::Portfolio,
                entity_id: p.id,
                portfolio_id: p.id,
                portfolio_name: p.name.clone(),
                owner_id: p.owner_id,
                listing,
                image_url: p.image_url.clone(),
            });
        }
        for g in &p.asset_groups {
            if let Some(listing) = parse_red_listing(&g.metadata) {
                items.push(RedBrowseItem {
                    kind: RedEntityKind::AssetGroup,
                    entity_id: g.id,
                    portfolio_id: p.id,
                    portfolio_name: p.name.clone(),
                    owner_id: p.owner_id,
                    listing,
                    image_url: g
                        .image_url
                        .clone()
                        .or_else(|| g.assets.first().and_then(|a| a.images.first().cloned())),
                });
            }
            for a in &g.assets {
                if let Some(listing) = parse_red_listing(&a.metadata) {
                    items.push(RedBrowseItem {
                        kind: RedEntityKind::Asset,
                        entity_id: a.id,
                        portfolio_id: p.id,
                        portfolio_name: p.name.clone(),
                        owner_id: p.owner_id,
                        listing,
                        image_url: a.images.first().cloned(),
                    });
                }
            }
        }
        for a in &p.assets {
            if let Some(listing) = parse_red_listing(&a.metadata) {
                items.push(RedBrowseItem {
                    kind: RedEntityKind::Asset,
                    entity_id: a.id,
                    portfolio_id: p.id,
                    portfolio_name: p.name.clone(),
                    owner_id: p.owner_id,
                    listing,
                    image_url: a.images.first().cloned(),
                });
            }
        }
    }
    items
}

/// Locate the metadata object for a group/portfolio listing target.
fn entity_metadata_mut<'a>(
    s: &'a mut AppStore,
    kind: RedEntityKind,
    entity_id: Uuid,
) -> Option<&'a mut serde_json::Value> {
    match kind {
        RedEntityKind::Portfolio => s
            .portfolios
            .iter_mut()
            .find(|p| p.id == entity_id)
            .map(|p| &mut p.metadata),
        RedEntityKind::AssetGroup => s.portfolios.iter_mut().find_map(|p| {
            p.asset_groups
                .iter_mut()
                .find(|g| g.id == entity_id)
                .map(|g| &mut g.metadata)
        }),
        // Assets are listed through the marketplace card, not this panel.
        RedEntityKind::Asset => None,
    }
}

fn entity_metadata<'a>(
    s: &'a AppStore,
    kind: RedEntityKind,
    entity_id: Uuid,
) -> Option<&'a serde_json::Value> {
    match kind {
        RedEntityKind::Portfolio => s
            .portfolios
            .iter()
            .find(|p| p.id == entity_id)
            .map(|p| &p.metadata),
        RedEntityKind::AssetGroup => s.portfolios.iter().find_map(|p| {
            p.asset_groups
                .iter()
                .find(|g| g.id == entity_id)
                .map(|g| &g.metadata)
        }),
        RedEntityKind::Asset => None,
    }
}

/// Inline listing editor for asset groups and portfolios — the same
/// `metadata["red_listing"]` slot the asset marketplace card and the
/// Marketplace tab read, so all three entity kinds share one listing shape.
#[component]
pub(crate) fn RedListingPanel(
    kind: RedEntityKind,
    entity_id: Uuid,
    entity_name: String,
    entity_desc: Option<String>,
    entity_value: f64,
    image_url: Option<String>,
    #[prop(into)]
    can_edit: Signal<bool>,
) -> impl IntoView {
    let app_store = use_app_store();
    let ui_store = use_ui_store();

    let existing = move || {
        entity_metadata(&app_store.get(), kind, entity_id)
            .and_then(parse_red_listing)
    };

    let (title, set_title) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (price, set_price) = signal(String::new());
    let (condition, set_condition) = signal(String::new());
    let (status, set_status) = signal("published".to_string());
    let (tags, set_tags) = signal(String::new());
    let (location, set_location) = signal(String::new());
    let (photos, set_photos) = signal(String::new());
    let (docs, set_docs) = signal(String::new());
    let (form_err, set_form_err) = signal(Option::<String>::None);
    let (initialized, set_initialized) = signal(false);

    // Seed the form once from the saved listing, or from entity defaults.
    Effect::new(move |_| {
        if initialized.get() {
            return;
        }
        if let Some(l) = existing() {
            set_title.set(l.title.clone());
            set_description.set(l.description.clone());
            set_price.set(l.price.clone());
            set_condition.set(l.condition.clone());
            set_status.set(if l.status.is_empty() {
                "published".to_string()
            } else {
                l.status.clone()
            });
            set_tags.set(l.tags.join(", "));
            set_location.set(l.location.clone());
            set_photos.set(l.photos.join(", "));
            set_docs.set(l.docs.join(", "));
        } else {
            set_title.set(entity_name.clone());
            set_description.set(entity_desc.clone().unwrap_or_default());
            if entity_value > 0.0 {
                set_price.set(format!("{:.2}", entity_value));
            }
        }
        set_initialized.set(true);
    });

    let persist = move |listing: Option<MarketplaceListing>| {
        app_store.update(|s| {
            let Some(meta) = entity_metadata_mut(s, kind, entity_id) else {
                return;
            };
            if !meta.is_object() {
                *meta = serde_json::json!({});
            }
            if let Some(l) = listing {
                if let Ok(v) = serde_json::to_value(&l) {
                    meta.as_object_mut()
                        .unwrap()
                        .insert("red_listing".to_string(), v);
                }
            } else if let Some(obj) = meta.as_object_mut() {
                obj.remove("red_listing");
            }
        });
        if let Some(p) = app_store.get().portfolios.iter().find_map(|p| {
            let touches = match kind {
                RedEntityKind::Portfolio => p.id == entity_id,
                _ => p.asset_groups.iter().any(|g| g.id == entity_id),
            };
            touches.then(|| p.clone())
        }) {
            leptos::task::spawn_local(async move {
                let _ = crate::server::save_portfolio(p).await;
            });
        }
    };

    let save = move |_| {
        if title.get().trim().is_empty() {
            set_form_err.set(Some("Title is required".to_string()));
            return;
        }
        set_form_err.set(None);
        let mut l = existing().unwrap_or_default();
        l.title = title.get();
        l.description = description.get();
        l.price = price.get();
        l.condition = condition.get();
        l.status = status.get();
        l.tags = csv_to_list(&tags.get());
        l.location = location.get();
        l.photos = csv_to_list(&photos.get());
        l.docs = csv_to_list(&docs.get());
        l.saved_at = chrono::Utc::now().to_rfc3339();
        persist(Some(l));
    };

    let unlist = move |_| persist(None);

    let kind_label = kind.label();
    view! {
        <div class="mp-listing-form red-listing-panel">
            <p class="mp-hint">
                {format!("{} listing — shown in the Marketplace tab once saved.", kind_label)}
            </p>
            {move || image_url.clone().map(|u| view! {
                <div class="mp-af-photos">
                    <img class="mp-af-photo" src={u} alt="Listing image" />
                </div>
            })}
            <div class="mp-included">
                <span class="mp-included-label">"Listing details:"</span>
                <div class="mp-af-grid">
                    <div class="mp-af-row">
                        <span class="mp-af-label">"TITLE"</span>
                        <input class="mp-af-input" type="text" prop:value=move || title.get()
                            on:input=move |ev| set_title.set(event_target_value(&ev)) />
                    </div>
                    <div class="mp-af-row">
                        <span class="mp-af-label">"DESCRIPTION"</span>
                        <input class="mp-af-input" type="text" prop:value=move || description.get()
                            on:input=move |ev| set_description.set(event_target_value(&ev)) />
                    </div>
                    <div class="mp-af-row">
                        <span class="mp-af-label">"PRICE"</span>
                        <input class="mp-af-input" type="text" prop:value=move || price.get()
                            on:input=move |ev| set_price.set(event_target_value(&ev)) />
                    </div>
                    <div class="mp-af-row">
                        <span class="mp-af-label">"CONDITION"</span>
                        <select class="mp-af-input" prop:value=move || condition.get()
                            on:change=move |ev| set_condition.set(event_target_value(&ev))>
                            <option value="">"—"</option>
                            <option value="New">"New"</option>
                            <option value="Used">"Used"</option>
                            <option value="For parts">"For parts"</option>
                            <option value="Refurbished">"Refurbished"</option>
                        </select>
                    </div>
                    <div class="mp-af-row">
                        <span class="mp-af-label">"STATUS"</span>
                        <select class="mp-af-input" prop:value=move || status.get()
                            on:change=move |ev| set_status.set(event_target_value(&ev))>
                            <option value="published">"Listed"</option>
                            <option value="draft">"Draft"</option>
                            <option value="sold">"Sold"</option>
                        </select>
                    </div>
                    <div class="mp-af-row">
                        <span class="mp-af-label">"ADDRESS"</span>
                        <input class="mp-af-input" type="text" placeholder="Location / address"
                            prop:value=move || location.get()
                            on:input=move |ev| set_location.set(event_target_value(&ev)) />
                    </div>
                    <div class="mp-af-row">
                        <span class="mp-af-label">"TAGS"</span>
                        <input class="mp-af-input" type="text" placeholder="comma separated"
                            prop:value=move || tags.get()
                            on:input=move |ev| set_tags.set(event_target_value(&ev)) />
                    </div>
                    <div class="mp-af-row">
                        <span class="mp-af-label">"PHOTOS"</span>
                        <input class="mp-af-input" type="text" placeholder="comma separated image URLs"
                            prop:value=move || photos.get()
                            on:input=move |ev| set_photos.set(event_target_value(&ev)) />
                    </div>
                    <div class="mp-af-row">
                        <span class="mp-af-label">"DOCS"</span>
                        <input class="mp-af-input" type="text" placeholder="comma separated document names"
                            prop:value=move || docs.get()
                            on:input=move |ev| set_docs.set(event_target_value(&ev)) />
                    </div>
                </div>
            </div>
            {move || form_err.get().map(|e| view! { <p class="mp-error">{e}</p> })}
            <div class="mp-form-actions">
                <button type="button" class="pf-small-btn"
                    on:click=move |_| ui_store.update(|s| s.close_red_listing_panel(entity_id))>
                    "Close"
                </button>
                <button type="button" class="pf-small-btn" disabled={move || !can_edit.get()}
                    on:click=save>"Save listing"</button>
                {move || if existing().is_some() {
                    view! {
                        <button type="button" class="pf-small-btn mp-disconnect"
                            disabled={move || !can_edit.get()}
                            on:click=unlist>"Remove listing"</button>
                    }.into_any()
                } else { ().into_any() }}
            </div>
        </div>
    }
}
