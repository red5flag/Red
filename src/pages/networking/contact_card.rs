use crate::pages::networking::ExternalContact;
use crate::stores::use_messenger_store;
use leptos::prelude::*;

pub(crate) fn render_contact_card(
    c: ExternalContact,
    set_selected: WriteSignal<Option<ExternalContact>>,
) -> impl IntoView {
    let c_for_click = c.clone();
    let c_for_msg = c.clone();
    let status_cls = c.status.css_class();
    let risk_cls = c.risk_level.css_class();
    view! {
        <div class="net-relationship-card" on:click=move |_| set_selected.set(Some(c_for_click.clone()))>
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
                {c.last_message.as_ref().map(|m| view! {
                    <div class="net-rel-activity">{format!("Last message: {}", m)}</div>
                }.into_any()).unwrap_or_else(|| ().into_any())}
            </div>
            <div class="net-rel-actions" on:click=|ev| ev.stop_propagation()>
                <button class="net-rel-btn" on:click=move |_| {
                    use_messenger_store().update(|s| s.set_message_drawer(true));
                }>"Message"</button>
                <button class="net-rel-btn" on:click=move |_| set_selected.set(Some(c_for_msg.clone()))>"View"</button>
            </div>
        </div>
    }
}
