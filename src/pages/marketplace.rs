use crate::pages::portfolios::{collect_red_listings, RedBrowseItem, RedEntityKind};
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

    let items = Memo::new(move |_| collect_red_listings(&app_store.get()));

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

    let open_portfolios = move |_| {
        app_store.update(|s| s.expand_tab(TabType::Portfolios));
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
                                {["All", "Listed", "Draft", "Sold"].iter().map(|s| {
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
                                <button class="marketplace-card" on:click=open_portfolios>
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
