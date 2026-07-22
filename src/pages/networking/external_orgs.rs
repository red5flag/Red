use crate::pages::networking::{ExternalOrganization, NetTab};
use leptos::prelude::*;

pub(crate) fn render_external_orgs(
    orgs: Vec<ExternalOrganization>,
    set_selected: WriteSignal<Option<ExternalOrganization>>,
    visible_count: Signal<usize>,
    on_expand: Callback<NetTab>,
) -> impl IntoView {
    let items_for = orgs.clone();
    let items_memo = Memo::new(move |_| items_for.clone());
    let visible_items = Memo::new(move |_| {
        let total = items_memo.get().len();
        let visible = visible_count.get().min(total);
        items_memo
            .get()
            .into_iter()
            .take(visible)
            .collect::<Vec<_>>()
    });
    view! {
        <div class="net-tab-content">
            {move || {
                let total = items_memo.get().len();
                let visible = visible_count.get().min(total);
                let remaining = total.saturating_sub(visible);
                if total == 0 {
                    view! {
                        <div class="data-card">
                            <div class="empty-state"><div class="empty-text">"No external organizations found"</div></div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div>
                            <For
                                each=move || visible_items.get()
                                key=|o: &ExternalOrganization| o.id
                                children=move |o| {
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
                        }
                            />
                            {if remaining > 0 {
                                view! {
                                    <button
                                        class="pf-show-more-btn"
                                        on:click=move |_| on_expand.run(NetTab::ExternalOrgs)
                                    >
                                        {format!("Expand View + ({}/{}) ", visible, total)}
                                    </button>
                                }.into_any()
                            } else { ().into_any() }}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
