use crate::pages::networking::ExternalContact;
use crate::stores::{use_app_store, use_messenger_store};
use crate::types::TabType;
use leptos::prelude::*;

pub(crate) fn render_contact_card(
    c: ExternalContact,
    set_selected: WriteSignal<Option<ExternalContact>>,
) -> impl IntoView {
    let c_for_view = c.clone();
    let status_cls = c.status.css_class();
    let risk_cls = c.risk_level.css_class();
    let app_store = use_app_store();
    let (view_open, set_view_open) = signal(false);
    view! {
        <div class="net-relationship-card" on:click=move |_| set_view_open.update(|v| *v = !*v)>
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
                    on:click=move |_| use_messenger_store().update(|s| s.set_message_drawer(true))
                >"💬"</button>
                <div class="net-rel-dropdown-wrap">
                    <button
                        class="net-rel-btn net-rel-icon-btn net-rel-view-btn"
                        title="View options"
                        on:click=move |_| set_view_open.update(|v| *v = !*v)
                    >
                        "👁"
                        <span class="net-rel-dropdown-arrow">"▼"</span>
                    </button>
                    {move || {
                        let c_for_dropdown = c_for_view.clone();
                        if view_open.get() { view! {
                            <div class="net-rel-dropdown" on:click=|ev| ev.stop_propagation()>
                                <button class="net-rel-dropdown-item" on:click=move |_| { set_selected.set(Some(c_for_dropdown.clone())); set_view_open.set(false); }>"View"</button>
                                <button class="net-rel-dropdown-item" on:click=move |_| { app_store.update(|s| s.expand_tab(TabType::Transactions)); set_view_open.set(false); }>"Transactions"</button>
                                <button class="net-rel-dropdown-item" on:click=move |_| { app_store.update(|s| s.expand_tab(TabType::Reporting)); set_view_open.set(false); }>"Reports"</button>
                            </div>
                        }.into_any() } else { ().into_any() }
                    }}
                </div>
            </div>
        </div>
    }
}
