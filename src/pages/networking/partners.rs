use crate::pages::networking::{ExternalContact, ExternalOrganization, NetTab};
use leptos::prelude::*;

pub(crate) fn render_by_type(
    contacts: &[ExternalContact],
    orgs: &[ExternalOrganization],
    rel_type: &str,
    tab: NetTab,
    visible_count: Signal<usize>,
    on_expand: Callback<NetTab>,
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
    let orgs_for = filtered_orgs.clone();
    let orgs_memo = Memo::new(move |_| orgs_for.clone());
    let contacts_for = filtered_contacts.clone();
    let contacts_memo = Memo::new(move |_| contacts_for.clone());
    let rel_type = rel_type.to_string();
    let org_total = filtered_orgs.len();
    let contact_total = filtered_contacts.len();
    let visible_orgs = Memo::new(move |_| {
        let visible = visible_count.get().min(org_total);
        orgs_memo
            .get()
            .into_iter()
            .take(visible)
            .collect::<Vec<_>>()
    });
    let visible_contacts = Memo::new(move |_| {
        let visible = visible_count.get().min(contact_total);
        contacts_memo
            .get()
            .into_iter()
            .take(visible)
            .collect::<Vec<_>>()
    });

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
                                    <For
                                        each=move || visible_orgs.get()
                                        key=|o: &ExternalOrganization| o.id
                                        children=move |o| {
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
                                        }
                                    />
                                </div>
                                {if visible_count.get().min(org_total) < org_total {
                                    view! {
                                        <button
                                            class="pf-show-more-btn"
                                            on:click=move |_| on_expand.run(tab)
                                        >
                                            {format!("Expand View + ({}/{}) ", visible_count.get().min(org_total), org_total)}
                                        </button>
                                    }.into_any()
                                } else { ().into_any() }}
                            }.into_any()
                        } else { ().into_any() }}
                        {if !filtered_contacts.is_empty() {
                            view! {
                                <div class="net-section-title">"Contacts"</div>
                                <div class="net-cards-list">
                                    <For
                                        each=move || visible_contacts.get()
                                        key=|c: &ExternalContact| c.id
                                        children=move |c| {
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
                                        }
                                    />
                                </div>
                                {if visible_count.get().min(contact_total) < contact_total {
                                    view! {
                                        <button
                                            class="pf-show-more-btn"
                                            on:click=move |_| on_expand.run(tab)
                                        >
                                            {format!("Expand View + ({}/{}) ", visible_count.get().min(contact_total), contact_total)}
                                        </button>
                                    }.into_any()
                                } else { ().into_any() }}
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
