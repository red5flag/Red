//! Central marketplace channel control: the single place to connect a
//! marketplace account, choose the eBay sites listings publish to, and set
//! per-channel options. Asset listing cards read this state instead of
//! carrying their own connect controls.

use crate::models::{ebay_site, Channel, ChannelType, ConnectionStatus, SyncDirection, EBAY_SITES};
use crate::stores::use_app_store;
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

fn ebay_account_channels(channels: &[Channel]) -> Vec<Channel> {
    channels
        .iter()
        .filter(|c| c.is_account_level() && c.channel_type == ChannelType::EBay)
        .cloned()
        .collect()
}

fn sync_label(d: &SyncDirection) -> &'static str {
    match d {
        SyncDirection::ImportOnly => "Import only",
        SyncDirection::ExportOnly => "Export only",
        SyncDirection::TwoWay => "Two-way",
    }
}

#[component]
pub fn MarketplaceChannelsPanel() -> impl IntoView {
    let app_store = use_app_store();

    let ebay_channels = Memo::new(move |_| app_store.with(|s| ebay_account_channels(&s.channels)));
    let connected = Memo::new(move |_| {
        ebay_channels
            .get()
            .iter()
            .any(|c| c.enabled && c.connection_status == ConnectionStatus::Connected)
    });
    let account = Memo::new(move |_| {
        ebay_channels.get().iter().find_map(|c| c.account_label.clone()).unwrap_or_default()
    });
    let (account_input, set_account_input) = signal(account.get_untracked());
    let (notice, set_notice) = signal(Option::<String>::None);

    // Listings published per eBay site (asset-level channels).
    let published_on = move |site: &str| {
        app_store.with(|s| {
            s.channels
                .iter()
                .filter(|c| {
                    !c.is_account_level()
                        && c.channel_type == ChannelType::EBay
                        && c.marketplace_site.as_deref() == Some(site)
                })
                .count()
        })
    };

    let set_all_status = move |status: ConnectionStatus, label: Option<String>, msg: &str| {
        let now = Utc::now();
        let msg = msg.to_string();
        app_store.update(|s| {
            for c in s.channels.iter_mut().filter(|c| c.is_account_level() && c.channel_type == ChannelType::EBay) {
                c.connection_status = status.clone();
                if label.is_some() {
                    c.account_label = label.clone();
                }
                c.last_sync_status = Some(msg.clone());
                c.updated_at = now;
            }
        });
    };

    let connect = move |_| {
        let label = account_input.get_untracked().trim().to_string();
        if label.is_empty() {
            set_notice.set(Some("Enter the eBay account (user ID or email) first.".to_string()));
            return;
        }
        if !ebay_channels.get_untracked().iter().any(|c| c.enabled) {
            set_notice.set(Some("Select at least one eBay site below first.".to_string()));
            return;
        }
        set_all_status(ConnectionStatus::Connected, Some(label), "Connected (local test — eBay API not wired yet)");
        set_notice.set(Some("eBay account linked for local testing. Real eBay sign-in isn't wired up yet.".to_string()));
    };
    let disconnect = move |_| {
        set_all_status(ConnectionStatus::Disconnected, None, "Disconnected");
        set_notice.set(None);
    };

    let toggle_site = move |site_id: &'static str| {
        let Some(site) = ebay_site(site_id) else { return };
        let status = if connected.get_untracked() { ConnectionStatus::Connected } else { ConnectionStatus::Disconnected };
        let label = account.get_untracked();
        app_store.update(|s| {
            if let Some(c) = s.channels.iter_mut().find(|c| {
                c.is_account_level()
                    && c.channel_type == ChannelType::EBay
                    && c.marketplace_site.as_deref() == Some(site_id)
            }) {
                c.enabled = !c.enabled;
                if c.enabled {
                    c.connection_status = status;
                }
                c.updated_at = Utc::now();
            } else {
                let mut c = Channel::new_ebay_site_channel(site, (!label.is_empty()).then_some(label));
                c.connection_status = status;
                s.add_channel(c);
            }
        });
    };

    let update_channel = move |id: Uuid, f: Box<dyn FnOnce(&mut Channel)>| {
        app_store.update(|s| {
            if let Some(c) = s.channels.iter_mut().find(|c| c.id == id) {
                f(c);
                c.updated_at = Utc::now();
            }
        });
    };

    view! {
        <div class="mkt-channels">
            <section class="mkt-ch-section" aria-labelledby="mkt-ch-ebay-h">
                <h2 id="mkt-ch-ebay-h" class="mkt-ch-title">"eBay account"</h2>
                <p class="mkt-ch-hint">
                    "Connect once here. Every asset's eBay listing uses this account and the sites selected below."
                </p>
                <div class="mkt-ch-row">
                    <span class="mkt-ch-status" class:on=move || connected.get()>
                        {move || if connected.get() {
                            format!("● Connected — {}", account.get())
                        } else {
                            "○ Not connected".to_string()
                        }}
                    </span>
                </div>
                <div class="mkt-ch-row">
                    <label class="mkt-ch-label" for="mkt-ch-account">"Account"</label>
                    <input id="mkt-ch-account" class="mkt-ch-input" type="text"
                        placeholder="eBay user ID or email"
                        prop:value=move || account_input.get()
                        on:input=move |ev| set_account_input.set(event_target_value(&ev))
                    />
                    {move || if connected.get() {
                        view! { <button type="button" class="pf-small-btn" on:click=disconnect>"Disconnect"</button> }.into_any()
                    } else {
                        view! { <button type="button" class="pf-small-btn" on:click=connect>"Connect"</button> }.into_any()
                    }}
                </div>
                {move || notice.get().map(|n| view! { <p class="mkt-ch-notice" role="status">{n}</p> })}
            </section>

            <section class="mkt-ch-section" aria-labelledby="mkt-ch-sites-h">
                <h2 id="mkt-ch-sites-h" class="mkt-ch-title">"eBay sites"</h2>
                <p class="mkt-ch-hint">"Choose which eBay sites listings can publish to."</p>
                <div class="mkt-ch-sites">
                    {EBAY_SITES.iter().map(|site| {
                        let id = site.id;
                        let on = move || ebay_channels.get().iter().any(|c| c.enabled && c.marketplace_site.as_deref() == Some(id));
                        view! {
                            <label class="mkt-ch-site" class:on=on>
                                <input type="checkbox" prop:checked=on on:change=move |_| toggle_site(id) />
                                <span class="mkt-ch-site-name">{site.label}</span>
                                <span class="mkt-ch-site-meta">{format!("{} · {}", site.domain, site.currency_code)}</span>
                            </label>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            <section class="mkt-ch-section" aria-labelledby="mkt-ch-settings-h">
                <h2 id="mkt-ch-settings-h" class="mkt-ch-title">"Channel settings"</h2>
                {move || {
                    let list: Vec<Channel> = ebay_channels.get().into_iter().filter(|c| c.enabled).collect();
                    if list.is_empty() {
                        return view! { <p class="mkt-ch-hint">"No channels enabled yet. Select an eBay site above."</p> }.into_any();
                    }
                    view! {
                        <table class="mkt-ch-table">
                            <thead><tr>
                                <th>"Channel"</th><th>"Currency"</th><th>"Status"</th>
                                <th>"Sync"</th><th>"Fee %"</th><th>"Listings"</th>
                            </tr></thead>
                            <tbody>
                                {list.into_iter().map(|c| {
                                    let id = c.id;
                                    let site = c.marketplace_site.clone().unwrap_or_default();
                                    let count = published_on(&site);
                                    let sync = sync_label(&c.sync_direction);
                                    let fee = c.commission_percent.map(|v| v.to_string()).unwrap_or_default();
                                    view! {
                                        <tr>
                                            <td>{c.name.clone()}</td>
                                            <td>{c.currency.to_string()}</td>
                                            <td>{if c.connection_status == ConnectionStatus::Connected { "Connected" } else { "Not connected" }}</td>
                                            <td>
                                                <select class="mkt-ch-select" aria-label="Sync direction"
                                                    on:change=move |ev| {
                                                        let d = match event_target_value(&ev).as_str() {
                                                            "Import only" => SyncDirection::ImportOnly,
                                                            "Two-way" => SyncDirection::TwoWay,
                                                            _ => SyncDirection::ExportOnly,
                                                        };
                                                        update_channel(id, Box::new(move |c| c.sync_direction = d));
                                                    }>
                                                    {["Export only", "Import only", "Two-way"].into_iter().map(|o| view! {
                                                        <option value=o selected={o == sync}>{o}</option>
                                                    }).collect::<Vec<_>>()}
                                                </select>
                                            </td>
                                            <td>
                                                <input class="mkt-ch-input mkt-ch-fee" type="number" min="0" step="0.1"
                                                    aria-label="Fee percent" value=fee
                                                    on:change=move |ev| {
                                                        let v = event_target_value(&ev).trim().parse::<f64>().ok();
                                                        update_channel(id, Box::new(move |c| c.commission_percent = v));
                                                    }
                                                />
                                            </td>
                                            <td>{count}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    }.into_any()
                }}
            </section>

            <section class="mkt-ch-section" aria-labelledby="mkt-ch-other-h">
                <h2 id="mkt-ch-other-h" class="mkt-ch-title">"Other channels"</h2>
                <ul class="mkt-ch-list">
                    <li>"Red Marketplace — built in, always on."</li>
                    <li>"Facebook Marketplace and Gumtree — no self-serve listing API; not supported yet."</li>
                </ul>
            </section>
        </div>
    }
}
