use crate::stores::use_app_store;
use leptos::prelude::*;
#[component]
pub fn OverviewPage() -> impl IntoView {
    let app_store = use_app_store();

    let user_name = move || app_store.get().current_user.name.clone();
    let portfolio_count = move || app_store.get().portfolios.len();
    let asset_count = move || app_store.get().portfolios.iter().map(|p| p.get_all_assets().len()).sum::<usize>();
    let org_count = move || app_store.get().organization_users.len();
    let document_count = move || app_store.get().portfolios.iter().map(|p| p.documents.len()).sum::<usize>();

    let first_asset_image = move || {
        app_store.get().portfolios.iter()
            .flat_map(|p| p.get_all_assets().into_iter())
            .find(|a| !a.images.is_empty())
            .map(|a| a.images.first().cloned().unwrap_or_default())
    };

    let recent_property_change = move || {
        app_store.get().portfolios.iter()
            .max_by_key(|p| p.updated_at)
            .map(|p| format!("{} updated", p.name))
            .unwrap_or_else(|| "No recent changes".to_string())
    };

    let relevant_property_image = move || {
        app_store.get().portfolios.iter()
            .flat_map(|p| p.get_all_assets().into_iter())
            .find(|a| !a.images.is_empty())
            .and_then(|a| a.images.first().cloned())
    };

    let unread_message_count = move || app_store.get().unread_message_count();

    let recent_messages = move || {
        let mut messages = app_store.get().messages.clone();
        messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        messages.into_iter().take(5).collect::<Vec<_>>()
    };

    let sender_name_for_message = move |msg: &crate::models::Message| {
        let store = app_store.get();
        store.messenger_contacts.iter().find(|c| c.id == msg.sender_id).map(|c| c.name.clone())
            .or_else(|| store.organization_users.iter().find(|u| u.id == msg.sender_id).map(|u| u.name.clone()))
            .unwrap_or_else(|| "Unknown".to_string())
    };

    let on_open_messages = move |_| {
        app_store.update(|s| s.set_message_drawer(true));
    };

    view! {
        <div class="overview-content">
            <div class="overview-greeting">
                {move || format!("Welcome, {}", user_name())}
            </div>

            <div class="overview-square-grid">
                // Recent messages square
                <div class="overview-square overview-square-messages" on:click=on_open_messages>
                    <div class="overview-square-header">
                        <span class="overview-square-icon">"💬"</span>
                        <span class="overview-square-count">{move || format!("{}", unread_message_count())}</span>
                    </div>
                    <div class="overview-square-label">"Recent Messages"</div>
                    <div class="overview-square-messages">
                        {move || {
                            let messages = recent_messages();
                            if messages.is_empty() {
                                view! { <div class="overview-square-empty">"No messages yet"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overview-message-list-compact">
                                        {messages.into_iter().map(|m| {
                                            let sender = sender_name_for_message(&m);
                                            let preview = if m.content.len() > 36 { format!("{}...", &m.content[..36]) } else { m.content.clone() };
                                            let time = m.timestamp.format("%H:%M").to_string();
                                            let unread = !m.read;
                                            view! {
                                                <div class="overview-message-row-compact" class:unread=unread>
                                                    <div class="overview-message-row-top">
                                                        <span class="overview-message-sender">{sender}</span>
                                                        <span class="overview-message-time">{time}</span>
                                                    </div>
                                                    <div class="overview-message-preview">{preview}</div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // Recent property change
                <div class="overview-square overview-square-change">
                    <div class="overview-square-icon">"🏠"</div>
                    <div class="overview-square-label">"Recent Property Change"</div>
                    <div class="overview-square-value">{recent_property_change}</div>
                </div>

                // Recent booking
                <div class="overview-square overview-square-booking">
                    <div class="overview-square-icon">"📅"</div>
                    <div class="overview-square-label">"Recent Booking"</div>
                    <div class="overview-square-value">{move || format!("{} assets", asset_count())}</div>
                </div>

                // Recent report
                <div class="overview-square overview-square-report">
                    <div class="overview-square-icon">"📊"</div>
                    <div class="overview-square-label">"Recent Report"</div>
                    <div class="overview-square-value">{move || format!("{} docs", document_count())}</div>
                </div>

                // Recent contact
                <div class="overview-square overview-square-contact">
                    <div class="overview-square-icon">"👤"</div>
                    <div class="overview-square-label">"Recent Contact"</div>
                    <div class="overview-square-value">{move || format!("{} people", org_count())}</div>
                </div>

                // Relevant property image
                <div class="overview-square overview-square-image">
                    {move || match relevant_property_image() {
                        Some(url) => view! {
                            <img class="overview-square-img" src={url} alt="Property" />
                        }.into_any(),
                        None => view! {
                            <div class="overview-square-img-placeholder">
                                <div class="overview-square-icon">"🏞"</div>
                                <div class="overview-square-label">"Relevant Property Image"</div>
                            </div>
                        }.into_any()
                    }}
                </div>

                // First asset image
                <div class="overview-square overview-square-image">
                    {move || match first_asset_image() {
                        Some(url) => view! {
                            <img class="overview-square-img" src={url} alt="Property" />
                        }.into_any(),
                        None => view! {
                            <div class="overview-square-img-placeholder">
                                <div class="overview-square-icon">"🏞"</div>
                                <div class="overview-square-label">"Image"</div>
                            </div>
                        }.into_any()
                    }}
                </div>

                // Portfolios
                <div class="overview-square overview-square-portfolios">
                    <div class="overview-square-icon">"📁"</div>
                    <div class="overview-square-label">"Portfolios"</div>
                    <div class="overview-square-value">{portfolio_count}</div>
                </div>

                // Property overview (full width)
                <div class="overview-square overview-square-wide">
                    <div class="overview-square-label">"Property Overview"</div>
                    {move || {
                        let store = app_store.get();
                        let portfolio = store.portfolios.iter().max_by_key(|p| p.updated_at);
                        if let Some(p) = portfolio {
                            let img = p.get_all_assets().into_iter().find(|a| !a.images.is_empty()).and_then(|a| a.images.first().cloned());
                            view! {
                                <div class="overview-property-overview">
                                    <div class="overview-property-text">
                                        <div class="overview-property-title">{p.name.clone()}</div>
                                        <div class="overview-property-address">{p.description.clone().unwrap_or_default()}</div>
                                        <div class="overview-property-summary">{format!("{} assets · ${:.2}M", p.get_all_assets().len(), p.total_value / 1_000_000.0)}</div>
                                    </div>
                                    {match img {
                                        Some(url) => view! { <img class="overview-property-img" src={url} alt="Property" /> }.into_any(),
                                        None => view! { <div class="overview-property-img-placeholder">"🏞"</div> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            view! { <div class="overview-square-empty">"No property overview available"</div> }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
