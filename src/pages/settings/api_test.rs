use crate::models::Asset;
use crate::pages::portfolios::{AssetBookingControls, AssetLinkingControls};
use crate::stores::use_app_store;
use leptos::prelude::*;
use uuid::Uuid;

/// API Test page — a developer-only sandbox for adding channels and booking days
/// against any asset in the portfolio tree without leaving Settings.
#[component]
pub(crate) fn ApiTestPage() -> impl IntoView {
    let app_store = use_app_store();

    let all_assets = Memo::new(move |_| {
        let mut list: Vec<(Asset, Option<Uuid>, String, Option<String>)> = Vec::new();
        for p in app_store.get().portfolios.iter() {
            for a in &p.assets {
                list.push((a.clone(), Some(p.id), p.name.clone(), None));
            }
            for g in &p.asset_groups {
                for a in &g.assets {
                    list.push((a.clone(), Some(p.id), p.name.clone(), Some(g.name.clone())));
                }
            }
        }
        list
    });

    let (selected_id, set_selected_id) = signal(Option::<Uuid>::None);

    let selected = Memo::new(move |_| {
        let id = selected_id.get()?;
        let (asset, pid, _, _) = all_assets
            .get()
            .into_iter()
            .find(|(a, _, _, _)| a.id == id)?;
        Some((asset, pid))
    });

    view! {
        <div class="data-card">
            <div class="card-header">
                <span class="card-title">"API Test"</span>
            </div>
            <div class="settings-list">
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Target asset"</div>
                        <div class="list-item-desc">"Select an asset to test adding channels and bookings."</div>
                    </div>
                    <div class="list-item-right">
                        <select class="android-select" on:change=move |ev| {
                            let v = event_target_value(&ev);
                            if v.is_empty() {
                                set_selected_id.set(None);
                            } else {
                                set_selected_id.set(v.parse::<Uuid>().ok());
                            }
                        }>
                            <option value="">"— Select asset —"</option>
                            {move || all_assets.get().into_iter().map(|(a, _, pname, gname)| {
                                let label = if let Some(g) = gname {
                                    format!("{} — {}/{}", a.name, pname, g)
                                } else {
                                    format!("{} — {}", a.name, pname)
                                };
                                view! { <option value={a.id.to_string()}>{label}</option> }
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>
                </div>

                {move || selected.get().map(|(asset, portfolio_id)| view! {
                    <div class="api-test-asset-panel">
                        <div class="card-header">
                            <span class="card-title">{asset.name.clone()}</span>
                        </div>
                        <AssetLinkingControls asset_id={asset.id} asset_name={asset.name.clone()} portfolio_id={portfolio_id} can_link={true} />
                        <AssetBookingControls asset_id={asset.id} asset_name={asset.name.clone()} portfolio_id={portfolio_id} can_book={true} />
                    </div>
                }.into_any()).unwrap_or(().into_any())}
            </div>
        </div>
    }
}
