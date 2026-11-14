use crate::pages::portfolios::{
    collect_red_listings, save_red_listing, ListingComment, MarketplaceListing, RedBrowseItem,
    RedEntityKind,
};
use crate::stores::{use_app_store, use_organization_store, AppStore, OrganizationStore};
use crate::types::TabType;
use leptos::prelude::*;
use uuid::Uuid;

/// Sort options for the Red Marketplace browse view.
#[derive(Clone, Copy, PartialEq, Eq)]
enum MarketSort {
    Newest,
    PriceLow,
    PriceHigh,
    Kind,
    Owner,
    Portfolio,
}

impl MarketSort {
    fn label(self) -> &'static str {
        match self {
            MarketSort::Newest => "Newest first",
            MarketSort::PriceLow => "Price: low to high",
            MarketSort::PriceHigh => "Price: high to low",
            MarketSort::Kind => "Kind: asset → group → portfolio",
            MarketSort::Owner => "Owner: A–Z",
            MarketSort::Portfolio => "Portfolio: A–Z",
        }
    }
}

fn owner_name(app: &AppStore, orgs: &OrganizationStore, owner_id: Uuid) -> String {
    if owner_id == app.current_user.id {
        return app.current_user.name.clone();
    }
    orgs
        .organization_users
        .iter()
        .find(|u| u.id == owner_id)
        .map(|u| u.name.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn status_label(status: &str) -> &'static str {
    match status {
        "published" => "Listed",
        "bidding" => "Bidding",
        "draft" => "Draft",
        "sold" => "Sold",
        _ => "Listed",
    }
}

fn price_num(price: &str) -> f64 {
    price
        .trim()
        .trim_start_matches('$')
        .replace(',', "")
        .parse::<f64>()
        .unwrap_or(0.0)
}

fn kind_rank(kind: RedEntityKind) -> u8 {
    match kind {
        RedEntityKind::Asset => 0,
        RedEntityKind::AssetGroup => 1,
        RedEntityKind::Portfolio => 2,
    }
}

#[component]
pub fn MarketplacePage() -> impl IntoView {
    let app_store = use_app_store();
    let organization_store = use_organization_store();
    let (kind_filter, set_kind_filter) = signal("All".to_string());
    let (sort_mode, set_sort_mode) = signal(MarketSort::Newest);
    let (status_filter, set_status_filter) = signal("All".to_string());
    let (kind_open, set_kind_open) = signal(false);
    let (status_open, set_status_open) = signal(false);
    let (sort_open, set_sort_open) = signal(false);
    let (selected, set_selected) = signal(Option::<RedBrowseItem>::None);
    let (comment_text, set_comment_text) = signal(String::new());
    // Owner inline-editing: which listing field is being edited + its buffer.
    let (edit_field, set_edit_field) = signal(Option::<String>::None);
    let (edit_buf, set_edit_buf) = signal(String::new());

    let items = Memo::new(move |_| collect_red_listings(&app_store.get()));

    // The open item view re-resolves against the live items memo so
    // comment/status edits reflect immediately.
    let detail_item = Memo::new(move |_| {
        let sel = selected.get()?;
        items
            .get()
            .into_iter()
            .find(|i| i.entity_id == sel.entity_id && i.kind == sel.kind)
            .or(Some(sel))
    });

    let filtered = Memo::new(move |_| {
        let kf = kind_filter.get();
        let sf = status_filter.get();
        items
            .get()
            .into_iter()
            .filter(|i| {
                (kf == "All" || i.kind.label() == kf)
                    && (sf == "All" || status_label(&i.listing.status) == sf)
            })
            .collect::<Vec<_>>()
    });

    let sorted = Memo::new(move |_| {
        let app = app_store.get();
        let orgs = organization_store.get();
        let mut v = filtered.get();
        let owner_of = |i: &RedBrowseItem| owner_name(&app, &orgs, i.owner_id);
        match sort_mode.get() {
            MarketSort::Newest => {
                v.sort_by(|a, b| b.listing.saved_at.cmp(&a.listing.saved_at));
            }
            MarketSort::PriceLow => {
                v.sort_by(|a, b| {
                    price_num(&a.listing.price)
                        .partial_cmp(&price_num(&b.listing.price))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            MarketSort::PriceHigh => {
                v.sort_by(|a, b| {
                    price_num(&b.listing.price)
                        .partial_cmp(&price_num(&a.listing.price))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            MarketSort::Kind => {
                v.sort_by(|a, b| {
                    kind_rank(a.kind)
                        .cmp(&kind_rank(b.kind))
                        .then_with(|| a.listing.title.cmp(&b.listing.title))
                });
            }
            MarketSort::Owner => {
                v.sort_by(|a, b| owner_of(a).cmp(&owner_of(b)));
            }
            MarketSort::Portfolio => {
                v.sort_by(|a, b| {
                    a.portfolio_name
                        .cmp(&b.portfolio_name)
                        .then_with(|| a.listing.title.cmp(&b.listing.title))
                });
            }
        }
        v
    });

    // Jump to the listing's entity inside Portfolios: prime the same
    // pending-navigation state search results use (expand portfolio,
    // reveal group, scroll into view, open asset detail), then switch tabs.
    let open_in_portfolios = move |kind: RedEntityKind,
                                   entity_id: Uuid,
                                   portfolio_id: Uuid| {
        // Expansion/selection state persists once applied, so it can be
        // set while the tab is still hidden. The scroll target is consumed
        // by already-mounted portfolio components, so it is applied after
        // the tab switch has painted or scrollIntoView hits display:none.
        let scroll_target = match kind {
            RedEntityKind::Portfolio => {
                app_store.update(|s| {
                    s.touch_portfolio(entity_id);
                    s.selected_portfolio_ids.insert(entity_id);
                });
                crate::stores::ScrollTarget::Portfolio(entity_id)
            }
            RedEntityKind::AssetGroup => {
                app_store.update(|s| {
                    s.touch_portfolio(portfolio_id);
                    s.selected_portfolio_ids.insert(portfolio_id);
                    s.selected_asset_group_id = Some(entity_id);
                    s.pending_group_expand = Some(entity_id);
                });
                crate::stores::ScrollTarget::AssetGroup(entity_id)
            }
            RedEntityKind::Asset => {
                let group_id = app_store.with(|s| {
                    s.portfolios.iter().find_map(|p| {
                        p.asset_groups
                            .iter()
                            .find(|g| g.assets.iter().any(|a| a.id == entity_id))
                            .map(|g| g.id)
                    })
                });
                app_store.update(|s| {
                    s.touch_portfolio(portfolio_id);
                    s.selected_portfolio_ids.insert(portfolio_id);
                    s.selected_asset_id = Some(entity_id);
                    s.pending_group_expand = group_id;
                    s.pending_asset_detail = Some(entity_id);
                });
                crate::stores::ScrollTarget::Asset(entity_id)
            }
        };
        app_store.update(|s| s.expand_tab(TabType::Portfolios));
        #[cfg(not(feature = "hydrate"))]
        let _ = &scroll_target;
        #[cfg(feature = "hydrate")]
        {
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(120).await;
                app_store.update(|s| s.pending_scroll_target = Some(scroll_target));
            });
        }
    };

    // Persist an edited listing back through the entity's red_listing slot.
    let persist_item_listing = move |item: &RedBrowseItem, l: MarketplaceListing| {
        save_red_listing(
            app_store,
            item.kind,
            item.entity_id,
            item.portfolio_id,
            Some(l),
        );
    };

    // Commit an inline field edit: apply it to a clone of the open
    // listing, then persist through the shared red_listing path.
    let commit_edit = Callback::new(move |field: String| {
        let value = edit_buf.get();
        set_edit_field.set(None);
        if let Some(item) = detail_item.get_untracked() {
            let mut l = item.listing.clone();
            apply_field_edit(&mut l, &field, value);
            persist_item_listing(&item, l);
        }
    });

    // Owner-only listing status changes (Listed / Bidding / Sold / Draft).
    let set_listing_status = Callback::new(move |status: String| {
        let Some(item) = detail_item.get_untracked() else {
            return;
        };
        let mut l = item.listing.clone();
        l.status = status;
        l.saved_at = chrono::Utc::now().to_rfc3339();
        persist_item_listing(&item, l);
    });

    let post_comment = Callback::new(move |_: ()| {
        let text = comment_text.get().trim().to_string();
        if text.is_empty() {
            return;
        }
        let Some(item) = detail_item.get_untracked() else {
            return;
        };
        let mut l = item.listing.clone();
        l.comments.push(ListingComment {
            author: app_store.get_untracked().current_user.name.clone(),
            text,
            at: chrono::Utc::now().to_rfc3339(),
        });
        set_comment_text.set(String::new());
        persist_item_listing(&item, l);
    });

    // Full item view: listing info + status control + comments.
    let detail_view = move |item: RedBrowseItem| -> AnyView {
        let app = app_store.get();
        let orgs = organization_store.get();
        let owner = owner_name(&app, &orgs, item.owner_id);
        let is_owner = item.owner_id == app.current_user.id;
        let kind = item.kind;
        let portfolio_name = item.portfolio_name.clone();
        let listing = item.listing.clone();
        let comments = listing.comments.clone();
        let photos: Vec<String> = if !listing.photos.is_empty() {
            listing.photos.clone()
        } else {
            item.image_url.iter().cloned().collect()
        };
        let status = listing.status.clone();
        let saved = listing.saved_at.clone();
        let nav_kind = item.kind;
        let nav_entity = item.entity_id;
        let nav_portfolio = item.portfolio_id;

        // One label/value row that owners can click to edit inline.
        // `display` is what viewers see; `raw` is the editable buffer
        // content (CSV for list fields). Non-owners get plain text.
        // The field check lives inside a reactive closure so the row
        // switches to its input the moment edit_field changes.
        let field_row = move |label: &'static str,
                              field: &'static str,
                              display: String,
                              raw: String|
              -> AnyView {
            view! {
                <div class="mp-af-row">
                    <span class="mp-af-label">{label}</span>
                    {move || {
                        if is_owner && edit_field.get() == Some(field.to_string()) {
                            let f_blur = field.to_string();
                            let f_enter = field.to_string();
                            let f_esc = field.to_string();
                            view! {
                                <input class="mp-af-input" type="text"
                                    prop:value=move || edit_buf.get()
                                    on:input=move |ev| set_edit_buf.set(event_target_value(&ev))
                                    on:blur=move |_| commit_edit.run(f_blur.clone())
                                    on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                        if ev.key() == "Enter" {
                                            commit_edit.run(f_enter.clone());
                                        } else if ev.key() == "Escape" {
                                            set_edit_field.set(None);
                                        }
                                    } />
                            }
                            .into_any()
                        } else {
                            let f_click = field.to_string();
                            let disp = if display.is_empty() {
                                "—".to_string()
                            } else {
                                display.clone()
                            };
                            if is_owner {
                                let raw_click = raw.clone();
                                view! {
                                    <span class="mp-af-value mp-editable"
                                        on:click=move |_| {
                                            set_edit_buf.set(raw_click.clone());
                                            set_edit_field.set(Some(f_click.clone()));
                                        }>
                                        {disp}
                                    </span>
                                }
                                .into_any()
                            } else {
                                view! { <span class="mp-af-value">{disp}</span> }.into_any()
                            }
                        }
                    }}
                </div>
            }
            .into_any()
        };
        view! {
            <div class="listing-detail">
                <div class="listing-detail-head">
                    <button type="button" class="mp-head-btn"
                        on:click=move |_| { set_selected.set(None); set_edit_field.set(None); }>
                        "← Back"
                    </button>
                    <span class="marketplace-kind-badge">{format!("{} {}", kind.icon(), kind.label())}</span>
                    <span class={format!("listing-status-badge listing-status-{}", status)}>
                        {status_label(&status)}
                    </span>
                    <button type="button" class="mp-head-btn"
                        on:click=move |_| open_in_portfolios(nav_kind, nav_entity, nav_portfolio)>
                        "View in Portfolios"
                    </button>
                </div>
                {move || {
                    if is_owner && edit_field.get() == Some("title".to_string()) {
                        view! {
                            <input class="listing-detail-title listing-detail-title-input" type="text"
                                prop:value=move || edit_buf.get()
                                on:input=move |ev| set_edit_buf.set(event_target_value(&ev))
                                on:blur=move |_| commit_edit.run("title".to_string())
                                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                    if ev.key() == "Enter" { commit_edit.run("title".to_string()); }
                                    else if ev.key() == "Escape" { set_edit_field.set(None); }
                                } />
                        }.into_any()
                    } else {
                        let t = listing.title.clone();
                        let t_click = t.clone();
                        view! {
                            <h2 class={if is_owner { "listing-detail-title mp-editable" } else { "listing-detail-title" }}
                                on:click=move |_| {
                                    if is_owner {
                                        set_edit_buf.set(t_click.clone());
                                        set_edit_field.set(Some("title".to_string()));
                                    }
                                }>
                                {t}
                            </h2>
                        }.into_any()
                    }
                }}
                {move || {
                    if is_owner && edit_field.get() == Some("price".to_string()) {
                        view! {
                            <input class="listing-detail-price listing-detail-price-input" type="text"
                                prop:value=move || edit_buf.get()
                                on:input=move |ev| set_edit_buf.set(event_target_value(&ev))
                                on:blur=move |_| commit_edit.run("price".to_string())
                                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                    if ev.key() == "Enter" { commit_edit.run("price".to_string()); }
                                    else if ev.key() == "Escape" { set_edit_field.set(None); }
                                } />
                        }.into_any()
                    } else {
                        let p = listing.price.clone();
                        view! {
                            <div class={if is_owner { "listing-detail-price mp-editable" } else { "listing-detail-price" }}
                                on:click=move |_| {
                                    if is_owner {
                                        set_edit_buf.set(p.clone());
                                        set_edit_field.set(Some("price".to_string()));
                                    }
                                }>
                                {if p.is_empty() { "Price on request".to_string() } else { format!("${}", p) }}
                            </div>
                        }.into_any()
                    }
                }}
                {move || {
                    if is_owner && edit_field.get() == Some("description".to_string()) {
                        view! {
                            <textarea class="listing-detail-desc listing-detail-desc-input" rows="3"
                                prop:value=move || edit_buf.get()
                                on:input=move |ev| set_edit_buf.set(event_target_value(&ev))
                                on:blur=move |_| commit_edit.run("description".to_string())
                                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                    if ev.key() == "Enter" && !ev.shift_key() {
                                        ev.prevent_default();
                                        commit_edit.run("description".to_string());
                                    } else if ev.key() == "Escape" {
                                        set_edit_field.set(None);
                                    }
                                } />
                        }.into_any()
                    } else if !listing.description.trim().is_empty() || is_owner {
                        let d = listing.description.clone();
                        let d_click = d.clone();
                        view! {
                            <p class={if is_owner { "listing-detail-desc mp-editable" } else { "listing-detail-desc" }}
                                on:click=move |_| {
                                    if is_owner {
                                        set_edit_buf.set(d_click.clone());
                                        set_edit_field.set(Some("description".to_string()));
                                    }
                                }>
                                {if d.trim().is_empty() { "Add a description…".to_string() } else { d }}
                            </p>
                        }.into_any()
                    } else { ().into_any() }
                }}
                {if photos.is_empty() { ().into_any() } else {
                    view! {
                        <div class="listing-detail-photos">
                            {photos.iter().take(8).map(|u| view! {
                                <img class="listing-detail-photo" src={u.clone()} alt="Listing photo" />
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }}
                <div class="mp-included">
                    <span class="mp-included-label">"Listing details:"</span>
                    <div class="mp-af-grid">
                        <div class="mp-af-row"><span class="mp-af-label">"OWNER"</span><span class="mp-af-value">{owner}</span></div>
                        <div class="mp-af-row"><span class="mp-af-label">"PORTFOLIO"</span><span class="mp-af-value">{portfolio_name}</span></div>
                        {if is_owner || !listing.category.is_empty() {
                            field_row("TYPE & BUILD", "category", listing.category.clone(), listing.category.clone())
                        } else { ().into_any() }}
                        {if is_owner || !listing.condition.is_empty() {
                            field_row("CONDITION", "condition", listing.condition.clone(), listing.condition.clone())
                        } else { ().into_any() }}
                        {if is_owner || !listing.asset_condition.is_empty() {
                            field_row("ASSET CONDITION", "asset_condition", listing.asset_condition.clone(), listing.asset_condition.clone())
                        } else { ().into_any() }}
                        {if is_owner || !listing.location.is_empty() {
                            field_row("ADDRESS", "location", listing.location.clone(), listing.location.clone())
                        } else { ().into_any() }}
                        {if is_owner || !listing.sku.is_empty() {
                            let v = if listing.sku.is_empty() { String::new() } else { format!("SKU {}", listing.sku) };
                            field_row("REF / SKU", "sku", v, listing.sku.clone())
                        } else { ().into_any() }}
                        {if saved.is_empty() { ().into_any() } else {
                            let v = saved.get(..10).unwrap_or("").to_string();
                            view! { <div class="mp-af-row"><span class="mp-af-label">"SAVED"</span><span class="mp-af-value">{v}</span></div> }.into_any()
                        }}
                        {if is_owner || !listing.docs.is_empty() || listing.doc_count > 0 {
                            let n = if !listing.docs.is_empty() { listing.docs.len() } else { listing.doc_count };
                            let v = if listing.docs.is_empty() && n == 0 { String::new() } else { format!("{} doc{}", n, if n == 1 { "" } else { "s" }) };
                            field_row("DOCS", "docs", v, listing.docs.join(", "))
                        } else { ().into_any() }}
                        {if is_owner || !listing.photos.is_empty() {
                            let n = listing.photos.len();
                            let v = if n == 0 { String::new() } else { format!("{} photo{}", n, if n == 1 { "" } else { "s" }) };
                            field_row("PHOTOS", "photos", v, listing.photos.join(", "))
                        } else { ().into_any() }}
                        {if is_owner || !listing.tags.is_empty() {
                            field_row("TAGS", "tags", listing.tags.join(", "), listing.tags.join(", "))
                        } else { ().into_any() }}
                        {if is_owner {
                            view! {
                                <div class="mp-af-row">
                                    <span class="mp-af-label">"LISTING STATUS"</span>
                                    <select class="mp-af-input"
                                        prop:value=status.clone()
                                        on:change=move |ev| set_listing_status.run(event_target_value(&ev))>
                                        <option value="published">"Listed"</option>
                                        <option value="bidding">"Bidding"</option>
                                        <option value="sold">"Sold"</option>
                                        <option value="draft">"Draft"</option>
                                    </select>
                                </div>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                </div>
                <div class="listing-comments">
                    <div class="listing-comments-head">{format!("Comments ({})", comments.len())}</div>
                    {if comments.is_empty() {
                        view! { <p class="mp-hint">"No comments yet — start the conversation."</p> }.into_any()
                    } else {
                        view! {
                            <div class="listing-comment-list">
                                {comments.into_iter().map(|c| {
                                    let at = chrono::DateTime::parse_from_rfc3339(&c.at)
                                        .map(|d| {
                                            d.with_timezone(&chrono::Local)
                                                .format("%b %d · %H:%M")
                                                .to_string()
                                        })
                                        .unwrap_or_else(|_| c.at.chars().take(16).collect());
                                    view! {
                                        <div class="listing-comment">
                                            <div class="listing-comment-meta">
                                                <span class="listing-comment-author">{c.author}</span>
                                                <span class="listing-comment-at">{at}</span>
                                            </div>
                                            <div class="listing-comment-text">{c.text}</div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }}
                    <div class="listing-comment-form">
                        <input class="mp-af-input" type="text" placeholder="Add a comment…"
                            prop:value=move || comment_text.get()
                            on:input=move |ev| set_comment_text.set(event_target_value(&ev))
                            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                if ev.key() == "Enter" { post_comment.run(()); }
                            } />
                        <button type="button" class="pf-small-btn"
                            on:click=move |_| post_comment.run(())>"Post"</button>
                    </div>
                </div>
            </div>
        }
        .into_any()
    };

    view! {
        <div class="marketplace-page">
            <div class="marketplace-controls">
                <div class="sort-dropdown-wrap sort-dropdown-inline">
                    <button class="sort-btn"
                        on:click=move |_| {
                            set_status_open.set(false);
                            set_sort_open.set(false);
                            set_kind_open.update(|v| *v = !*v);
                        }
                    >{move || format!("Kind: {}", kind_filter.get())}</button>
                    {move || if kind_open.get() {
                        view! {
                            <div class="sort-dropdown" on:click=|ev| ev.stop_propagation()>
                                {["All", "Asset", "Asset Group", "Portfolio"].iter().map(|k| {
                                    let k = *k;
                                    view! {
                                        <button class="sort-dropdown-item"
                                            class:active=move || kind_filter.get() == k
                                            on:click=move |_| {
                                                set_kind_filter.set(k.to_string());
                                                set_kind_open.set(false);
                                            }
                                        >{k}</button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
                <div class="sort-dropdown-wrap sort-dropdown-inline">
                    <button class="sort-btn"
                        on:click=move |_| {
                            set_kind_open.set(false);
                            set_sort_open.set(false);
                            set_status_open.update(|v| *v = !*v);
                        }
                    >{move || format!("Status: {}", status_filter.get())}</button>
                    {move || if status_open.get() {
                        view! {
                            <div class="sort-dropdown" on:click=|ev| ev.stop_propagation()>
                                {["All", "Listed", "Bidding", "Sold", "Draft"].iter().map(|s| {
                                    let s = *s;
                                    view! {
                                        <button class="sort-dropdown-item"
                                            class:active=move || status_filter.get() == s
                                            on:click=move |_| {
                                                set_status_filter.set(s.to_string());
                                                set_status_open.set(false);
                                            }
                                        >{s}</button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
                <div class="sort-dropdown-wrap sort-dropdown-inline">
                    <button class="sort-btn"
                        on:click=move |_| {
                            set_kind_open.set(false);
                            set_status_open.set(false);
                            set_sort_open.update(|v| *v = !*v);
                        }
                    >{move || format!("Sort: {} ↕", sort_mode.get().label())}</button>
                    {move || if sort_open.get() {
                        view! {
                            <div class="sort-dropdown" on:click=|ev| ev.stop_propagation()>
                                {[MarketSort::Newest, MarketSort::PriceLow, MarketSort::PriceHigh,
                                  MarketSort::Kind, MarketSort::Owner, MarketSort::Portfolio]
                                    .iter().map(|m| {
                                        let m = *m;
                                        let label = m.label();
                                        view! {
                                            <button class="sort-dropdown-item"
                                                class:active=move || sort_mode.get() == m
                                                on:click=move |_| {
                                                    set_sort_mode.set(m);
                                                    set_sort_open.set(false);
                                                }
                                            >{label}</button>
                                        }
                                    }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
            </div>

            {move || {
                if let Some(item) = detail_item.get() {
                    return detail_view(item);
                }
                let list = sorted.get();
                if list.is_empty() {
                    return view! {
                        <div class="marketplace-empty">
                            <p class="mp-hint">"No listings match — list an asset, group or portfolio via the 🛒 button in Portfolios, or the Red Marketplace card on an asset."</p>
                        </div>
                    }.into_any();
                }
                view! {
                    <div class="marketplace-grid">
                        {list.into_iter().map(|item| {
                            let app = app_store.get();
                            let orgs = organization_store.get();
                            let owner = owner_name(&app, &orgs, item.owner_id);
                            let click_item = item.clone();
                            let kind = item.kind;
                            let title = item.listing.title.clone();
                            let price = item.listing.price.clone();
                            let condition = item.listing.condition.clone();
                            let status = status_label(&item.listing.status).to_string();
                            let portfolio_name = item.portfolio_name.clone();
                            let saved = item.listing.saved_at.clone();
                            let img = item.image_url.clone();
                            let tags = item.listing.tags.clone();
                            view! {
                                <button class="marketplace-card" on:click=move |_| {
                                    set_edit_field.set(None);
                                    set_selected.set(Some(click_item.clone()));
                                }>
                                    <div class="marketplace-card-media">
                                        {if let Some(u) = img {
                                            view! { <img class="marketplace-card-img" src={u} alt="" /> }.into_any()
                                        } else {
                                            view! { <span class="marketplace-card-placeholder">{kind.icon()}</span> }.into_any()
                                        }}
                                        <span class="marketplace-kind-badge">{format!("{} {}", kind.icon(), kind.label())}</span>
                                    </div>
                                    <div class="marketplace-card-body">
                                        <div class="marketplace-card-title">{title}</div>
                                        <div class="marketplace-card-price">
                                            {if price.is_empty() { "Price on request".to_string() } else { format!("${}", price) }}
                                        </div>
                                        <div class="marketplace-card-meta">
                                            {if !condition.is_empty() { format!("{} · ", condition) } else { String::new() }}
                                            {status}
                                        </div>
                                        <div class="marketplace-card-sub">
                                            {format!("{} · {}", owner, portfolio_name)}
                                        </div>
                                        {if !tags.is_empty() {
                                            view! {
                                                <div class="marketplace-card-tags">
                                                    {tags.iter().take(4).cloned().collect::<Vec<_>>().join(" · ")}
                                                </div>
                                            }.into_any()
                                        } else { ().into_any() }}
                                        {if !saved.is_empty() {
                                            view! {
                                                <div class="marketplace-card-date">{saved.get(..10).unwrap_or("").to_string()}</div>
                                            }.into_any()
                                        } else { ().into_any() }}
                                    </div>
                                </button>
                            }.into_any()
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

/// Comma-separated list parsing shared by the CSV-edited listing fields.
fn csv_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Apply an inline-edited field value onto a listing clone. Unknown
/// fields are ignored; `saved_at` is bumped on every successful edit.
fn apply_field_edit(l: &mut MarketplaceListing, field: &str, value: String) {
    match field {
        "title" => l.title = value,
        "description" => l.description = value,
        "price" => l.price = value,
        "category" => l.category = value,
        "condition" => l.condition = value,
        "asset_condition" => l.asset_condition = value,
        "location" => l.location = value,
        "sku" => l.sku = value,
        "tags" => l.tags = csv_list(&value),
        "docs" => l.docs = csv_list(&value),
        "photos" => l.photos = csv_list(&value),
        _ => return,
    }
    l.saved_at = chrono::Utc::now().to_rfc3339();
}
