use crate::pages::networking::ExternalContact;
use crate::stores::{use_app_store, use_messenger_store};
use crate::types::TabType;
use leptos::prelude::*;

pub(crate) fn render_contact_card(c: ExternalContact) -> impl IntoView {
    let c_for_view = c.clone();
    let status_cls = c.status.css_class();
    let risk_cls = c.risk_level.css_class();
    let app_store = use_app_store();
    let messenger_store = use_messenger_store();
    let (details_open, set_details_open) = signal(false);

    view! {
        <div class="net-relationship-wrap">
            <div
                class="net-relationship-card net-rel-card-header"
                on:click=move |_| set_details_open.update(|v| *v = !*v)
            >
                <img class="net-rel-avatar" src={c.avatar_url.clone().unwrap_or_default()} alt={c.name.clone()} />
                <div class="net-rel-body">
                    <div class="net-rel-name">{c.name.clone()}</div>
                    <div class="net-rel-type">{format!("{} • {}", c.title, c.company)}</div>
                    <div class="net-rel-meta">
                        <span class={format!("net-rel-status {}", status_cls)}>{c.status.label()}</span>
                        <span class={format!("net-rel-risk {}", risk_cls)}>{format!("Risk: {}", c.risk_level.label())}</span>
                    </div>
                    <div class="net-rel-linked">
                        {format!("Portfolios: {} • Transactions: {} • Reports: {}",
                            c.linked_portfolios.len(), c.linked_transactions.len(), c.linked_reports.len())}
                    </div>
                </div>
                <div class="net-rel-actions" on:click=|ev| ev.stop_propagation()>
                    <button
                        class="net-rel-btn net-rel-icon-btn"
                        title="Message"
                        on:click=move |_| messenger_store.update(|s| s.set_message_drawer(true))
                    >"💬"</button>
                    <button
                        class="net-rel-btn net-rel-icon-btn net-rel-view-btn"
                        title="View details"
                        on:click=move |_| set_details_open.update(|v| *v = !*v)
                    >
                        <span class="net-rel-dropdown-arrow" class:open={move || details_open.get()}>"▼"</span>
                    </button>
                </div>
            </div>

            {move || if details_open.get() { view! {
                <div class="net-rel-detail" on:click=|ev| ev.stop_propagation()>
                    <div class="net-detail-section">
                        <div class="net-detail-row"><span class="net-detail-label">"Type"</span><span class="net-detail-value">{c_for_view.relationship_type.clone()}</span></div>
                        <div class="net-detail-row"><span class="net-detail-label">"Status"</span><span class={format!("net-detail-value {}", c_for_view.status.css_class())}>{c_for_view.status.label()}</span></div>
                        <div class="net-detail-row"><span class="net-detail-label">"Risk"</span><span class={format!("net-detail-value {}", c_for_view.risk_level.css_class())}>{c_for_view.risk_level.label()}</span></div>
                        {c_for_view.email.as_ref().map(|e| view! { <div class="net-detail-row"><span class="net-detail-label">"Email"</span><span class="net-detail-value">{e.clone()}</span></div> }.into_any()).unwrap_or_else(|| ().into_any())}
                        {c_for_view.phone.as_ref().map(|p| view! { <div class="net-detail-row"><span class="net-detail-label">"Phone"</span><span class="net-detail-value">{p.clone()}</span></div> }.into_any()).unwrap_or_else(|| ().into_any())}
                    </div>
                    <div class="net-detail-section">
                        <div class="net-detail-section-title">"Linked Items"</div>
                        <div class="net-detail-row"><span class="net-detail-label">"Portfolios"</span><span class="net-detail-value">{c_for_view.linked_portfolios.join(", ")}</span></div>
                        <div class="net-detail-row"><span class="net-detail-label">"Transactions"</span><span class="net-detail-value">{c_for_view.linked_transactions.join(", ")}</span></div>
                        <div class="net-detail-row"><span class="net-detail-label">"Reports"</span><span class="net-detail-value">{if c_for_view.linked_reports.is_empty() { "—".into() } else { c_for_view.linked_reports.join(", ") }}</span></div>
                    </div>
                    <div class="net-detail-section">
                        <div class="net-detail-section-title">"Channels"</div>
                        <div class="net-detail-channels">
                            {c_for_view.channels.iter().map(|ch| view! { <span class="net-channel-tag">{ch.clone()}</span> }).collect::<Vec<_>>()}
                        </div>
                    </div>
                    <div class="net-detail-section">
                        <div class="net-detail-section-title">"Recent Activity"</div>
                        <div class="net-detail-row"><span class="net-detail-label">"Last transaction"</span><span class="net-detail-value">{c_for_view.last_transaction.clone().unwrap_or_else(|| "—".into())}</span></div>
                    </div>
                    <div class="net-detail-actions">
                        <button class="net-detail-btn net-detail-btn-primary" on:click=move |_| messenger_store.update(|s| s.set_message_drawer(true))>"Message"</button>
                        <button class="net-detail-btn" on:click=move |_| app_store.update(|s| s.expand_tab(TabType::Transactions))>"View Transactions"</button>
                        <button class="net-detail-btn" on:click=move |_| app_store.update(|s| s.expand_tab(TabType::Reporting))>"View Reports"</button>
                    </div>
                </div>
            }.into_any() } else { ().into_any() }}
        </div>
    }
}
