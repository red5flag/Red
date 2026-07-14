use crate::models::Portfolio;
use crate::stores::AppStore;
use crate::types::Currency;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub(crate) fn PortfolioAccessList(
    #[prop(into)] org_id: Uuid,
    app_store: RwSignal<AppStore>,
    #[prop(into)] can_edit: bool,
) -> impl IntoView {
    let portfolios = Memo::new(move |_| {
        app_store
            .get()
            .portfolios
            .iter()
            .filter(|p| p.organization_id == Some(org_id))
            .cloned()
            .collect::<Vec<_>>()
    });

    let unassigned_portfolios = Memo::new(move |_| {
        app_store
            .get()
            .portfolios
            .iter()
            .filter(|p| p.organization_id != Some(org_id))
            .cloned()
            .collect::<Vec<_>>()
    });

    let (new_portfolio_name, set_new_portfolio_name) = signal(String::new());
    let (show_create, set_show_create) = signal(false);
    let (selected_portfolio_id, set_selected_portfolio_id) = signal(Uuid::nil());
    let (show_assign, set_show_assign) = signal(false);

    let create_portfolio = move |_| {
        let name = new_portfolio_name.get().trim().to_string();
        if name.is_empty() {
            return;
        }
        app_store.update(|s| {
            let mut p = Portfolio::new(name, s.current_user.id, Currency::USD);
            p.organization_id = Some(org_id);
            s.portfolios.push(p);
        });
        set_new_portfolio_name.set(String::new());
        set_show_create.set(false);
    };

    let assign_portfolio = move |_| {
        let pid = selected_portfolio_id.get();
        if pid == Uuid::nil() {
            return;
        }
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                p.organization_id = Some(org_id);
            }
        });
        set_selected_portfolio_id.set(Uuid::nil());
        set_show_assign.set(false);
    };

    view! {
        <div class="org-sub-tab-header">
            <span class="org-sub-tab-title">"Portfolios"</span>
            {if can_edit {
                view! {
                    <div style="display:flex;gap:6px;">
                        <button class="add-btn-small"
                            on:click=move |_| set_show_assign.update(|v| *v = !*v)>
                            "+ Assign Portfolio"
                        </button>
                        <button class="add-btn-small"
                            on:click=move |_| set_show_create.update(|v| *v = !*v)>
                            "+ Create Portfolio"
                        </button>
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </div>

        {move || if show_assign.get() {
            let opts = unassigned_portfolios.get();
            view! {
                <div class="add-form" style="margin:0;border-radius:0;border-left:none;border-right:none;">
                    {if opts.is_empty() {
                        view! {
                            <div class="empty-text">"No unassigned portfolios available."</div>
                        }.into_any()
                    } else {
                        view! {
                            <select class="login-input"
                                prop:value={move || selected_portfolio_id.get().to_string()}
                                on:change=move |ev| {
                                    if let Ok(id) = event_target_value(&ev).parse::<Uuid>() {
                                        set_selected_portfolio_id.set(id);
                                    }
                                }>
                                <option value={Uuid::nil().to_string()}>"Select portfolio to assign"</option>
                                {opts.into_iter().map(|p| {
                                    let id = p.id;
                                    let name = p.name.clone();
                                    view! {
                                        <option value={id.to_string()}>{name}</option>
                                    }
                                }).collect::<Vec<_>>()}
                            </select>
                            <div style="display:flex;gap:6px;">
                                <button class="login-btn" style="flex:1;" on:click=assign_portfolio>"Assign"</button>
                                <button class="view-btn" style="flex:1;" on:click=move |_| set_show_assign.set(false)>"Cancel"</button>
                            </div>
                        }.into_any()
                    }}
                </div>
            }.into_any()
        } else { ().into_any() }}

        {move || if show_create.get() {
            view! {
                <div class="add-form" style="margin:0;border-radius:0;border-left:none;border-right:none;">
                    <input class="login-input" type="text" placeholder="New portfolio name"
                        prop:value={move || new_portfolio_name.get()}
                        on:input=move |ev| set_new_portfolio_name.set(event_target_value(&ev)) />
                    <div style="display:flex;gap:6px;">
                        <button class="login-btn" style="flex:1;" on:click=create_portfolio>"Create"</button>
                        <button class="view-btn" style="flex:1;" on:click=move |_| set_show_create.set(false)>"Cancel"</button>
                    </div>
                </div>
            }.into_any()
        } else { ().into_any() }}

        {move || if portfolios.get().is_empty() {
            view! {
                <div class="empty-state org-section-empty">
                    <div class="empty-text">"No portfolios."</div>
                    {if can_edit {
                        view! {
                            <div class="org-section-empty-actions">
                                <button class="add-btn-small" on:click=move |_| set_show_assign.set(true)>
                                    "+ Assign Portfolio"
                                </button>
                                <button class="add-btn-small" on:click=move |_| set_show_create.set(true)>
                                    "+ Create Portfolio"
                                </button>
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
            }.into_any()
        } else {
            let indexed = portfolios.get().into_iter().enumerate().collect::<Vec<_>>();
            view! {
                <div class="asset-list">
                <For
                    each=move || indexed.clone()
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
