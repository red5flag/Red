use crate::pages::networking::{ExternalContact, ExternalOrganization};
use leptos::prelude::*;

pub(crate) fn render_by_type(
    contacts: &[ExternalContact],
    orgs: &[ExternalOrganization],
    rel_type: &str,
) -> impl IntoView {
    let filtered_contacts: Vec<_> = contacts
        .iter()
        .filter(|c| c.relationship_type == rel_type)
        .cloned()
        .collect();
    let filtered_orgs: Vec<_> = orgs
        .iter()
        .filter(|o| {
            let ot = o.org_type.to_lowercase();
            match rel_type {
                "Supplier" => ot == "vendor" || ot == "supplier",
                "Partner" => ot == "partner",
                "Client" => ot == "client",
                _ => false,
            }
        })
        .cloned()
        .collect();
    let rel_type = rel_type.to_string();

    view! {
        <div class="net-tab-content">
            {if filtered_contacts.is_empty() && filtered_orgs.is_empty() {
                view! {
                    <div class="data-card">
                        <div class="empty-state"><div class="empty-text">{format!("No {} found", rel_type)}</div></div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div>
                        {if !filtered_orgs.is_empty() {
                            view! {
                                <div class="net-section-title">"Organizations"</div>
                                <div class="net-cards-list">
                                    {filtered_orgs.iter().map(|o| {
                                        let status_cls = o.status.css_class();
                                        let initial = o.name.chars().next().unwrap_or('A');
                                        view! {
                                            <div class="net-relationship-card">
                                                <div class="net-rel-avatar net-rel-avatar-org">{initial}</div>
                                                <div class="net-rel-body">
                                                    <div class="net-rel-name">{o.name.clone()}</div>
                                                    <div class="net-rel-type">{o.org_type.clone()}</div>
                                                    <div class="net-rel-meta">
                                                        <span class={format!("net-rel-status {}", status_cls)}>{o.status.label()}</span>
                                                    </div>
                                                    <div class="net-rel-linked">
                                                        {format!("Portfolios: {} • Transactions: {}",
                                                            o.linked_portfolios.len(), o.transaction_count)}
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        } else { ().into_any() }}
                        {if !filtered_contacts.is_empty() {
                            view! {
                                <div class="net-section-title">"Contacts"</div>
                                <div class="net-cards-list">
                                    {filtered_contacts.iter().map(|c| {
                                        let status_cls = c.status.css_class();
                                        view! {
                                            <div class="net-relationship-card">
                                                <img class="net-rel-avatar" src={c.avatar_url.clone().unwrap_or_default()} alt={c.name.clone()} />
                                                <div class="net-rel-body">
                                                    <div class="net-rel-name">{c.name.clone()}</div>
                                                    <div class="net-rel-type">{format!("{} • {}", c.title, c.company)}</div>
                                                    <div class="net-rel-meta">
                                                        <span class={format!("net-rel-status {}", status_cls)}>{c.status.label()}</span>
                                                    </div>
                                                    <div class="net-rel-linked">
                                                        {format!("Portfolios: {} • Transactions: {}",
                                                            c.linked_portfolios.len(), c.linked_transactions.len())}
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
