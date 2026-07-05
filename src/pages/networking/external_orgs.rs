use crate::pages::networking::ExternalOrganization;
use leptos::prelude::*;

pub(crate) fn render_external_orgs(
    orgs: Vec<ExternalOrganization>,
    set_selected: WriteSignal<Option<ExternalOrganization>>,
) -> impl IntoView {
    view! {
        <div class="net-tab-content">
            {if orgs.is_empty() {
                view! {
                    <div class="data-card">
                        <div class="empty-state"><div class="empty-text">"No external organizations found"</div></div>
                    </div>
                }.into_any()
            } else {
                orgs.into_iter().map(|o| {
                    let o_for_click = o.clone();
                    let o_for_btn = o.clone();
                    let status_cls = o.status.css_class();
                    let risk_cls = o.risk_level.css_class();
                    let initial = o.name.chars().next().unwrap_or('A');
                    view! {
                        <div class="net-relationship-card" on:click=move |_| set_selected.set(Some(o_for_click.clone()))>
                            <div class="net-rel-avatar net-rel-avatar-org">{initial}</div>
                            <div class="net-rel-body">
                                <div class="net-rel-name">{o.name.clone()}</div>
                                <div class="net-rel-type">{o.org_type.clone()}</div>
                                <div class="net-rel-meta">
                                    <span class={format!("net-rel-status {}", status_cls)}>{o.status.label()}</span>
                                    <span class={format!("net-rel-risk {}", risk_cls)}>{format!("Risk: {}", o.risk_level.label())}</span>
                                </div>
                                <div class="net-rel-linked">
                                    {format!("Portfolios: {} • Transactions: {} • Documents: {}",
                                        o.linked_portfolios.len(), o.transaction_count, o.document_count)}
                                </div>
                                {o.primary_contact.as_ref().map(|p| view! {
                                    <div class="net-rel-activity">{format!("Primary contact: {}", p)}</div>
                                }.into_any()).unwrap_or_else(|| ().into_any())}
                            </div>
                            <div class="net-rel-actions" on:click=|ev| ev.stop_propagation()>
                                <button class="net-rel-btn" on:click=move |_| set_selected.set(Some(o_for_btn.clone()))>"View"</button>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>().into_any()
            }}
        </div>
    }
}
