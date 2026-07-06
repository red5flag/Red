use crate::models::Portfolio;
use leptos::prelude::*;

#[component]
pub(crate) fn PortfolioAccessList(#[prop(into)] portfolios: Vec<Portfolio>) -> impl IntoView {
    let portfolios_for = portfolios.clone();
    let indexed_portfolios = Memo::new(move |_| portfolios_for.iter().cloned().enumerate().collect::<Vec<_>>());
    view! {
        {if portfolios.is_empty() {
            view! { <div class="empty-state"><div class="empty-text">"No portfolios."</div></div> }.into_any()
        } else {
            view! {
                <div class="asset-list">
                <For
                    each=move || indexed_portfolios.get()
                    key=|(_, p)| p.id
                    children=move |(idx, p)| {
                        let total = p.total_value;
                        let direct_count = p.assets.len();
                        let group_count = p.asset_groups.len();
                        let asset_count = direct_count + p.asset_groups.iter().map(|g| g.assets.len()).sum::<usize>();
                        let doc_count = p.documents.len();
                        let assigned_count = p.assigned_users.len();
                        let tint = format!("background: rgba(255,255,255,{:.1});", (idx as f64 * 0.07).min(0.4));
                        view! {
                            <div class="asset-item org-portfolio-card" style={tint}>
                                <div class="asset-icon">"\u{1F4C1}"</div>
                                <div class="asset-info">
                                    <div class="asset-name">{p.name.clone()}</div>
                                    {p.description.as_ref().map(|d| view! { <div class="asset-desc">{d.clone()}</div> })}
                                    <div class="asset-subtext">
                                        {format!("{} assets \u{00B7} {} direct \u{00B7} {} groups \u{00B7} {} docs \u{00B7} {} members",
                                            asset_count, direct_count, group_count, doc_count, assigned_count)}
                                    </div>
                                </div>
                                <div class="asset-value" style="color:var(--success);">
                                    {format!("${:.0}", total)}
                                </div>
                            </div>
                        }
                    }
                />
                </div>
            }.into_any()
        }}
    }
}
