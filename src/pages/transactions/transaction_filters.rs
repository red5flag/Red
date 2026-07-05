use crate::pages::transactions::{currency_symbol, Wallet};
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub(crate) fn WalletSelector(
    wallets: Vec<Wallet>,
    selected_wallet: ReadSignal<Uuid>,
    set_selected_wallet: WriteSignal<Uuid>,
    selected_card: ReadSignal<Option<Uuid>>,
    set_selected_card: WriteSignal<Option<Uuid>>,
) -> impl IntoView {
    view! {
        <div class="tx-form-row">
            <label class="tx-form-label">"From wallet"</label>
            <select
                class="form-select"
                prop:value={move || selected_wallet.get().to_string()}
                on:change=move |ev| {
                    let v = event_target_value(&ev);
                    if let Ok(id) = Uuid::parse_str(&v) {
                        set_selected_wallet.set(id);
                        set_selected_card.set(None);
                    }
                }
            >
                {wallets.clone().into_iter().map(|w| {
                    let id = w.id.to_string();
                    view! { <option value={id.clone()}>{format!("{} ({}{:.2} {})", w.name, currency_symbol(&w.currency), w.balance, w.currency)}</option> }
                }).collect::<Vec<_>>()}
            </select>
        </div>
        {move || {
            let wallet = wallets.iter().find(|w| w.id == selected_wallet.get()).cloned();
            wallet.map(|w| {
                if w.cards.is_empty() {
                    ().into_any()
                } else {
                    view! {
                        <div class="tx-form-row">
                            <label class="tx-form-label">"From card"</label>
                            <select
                                class="form-select"
                                prop:value={move || selected_card.get().map(|id| id.to_string()).unwrap_or_default()}
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    set_selected_card.set(if v.is_empty() { None } else { Uuid::parse_str(&v).ok() });
                                }
                            >
                                <option value="">"Default wallet balance"</option>
                                {w.cards.into_iter().map(|c| {
                                    let id = c.id.to_string();
                                    view! { <option value={id.clone()}>{format!("{} ending {}", c.label, c.last4)}</option> }
                                }).collect::<Vec<_>>()}
                            </select>
                        </div>
                    }.into_any()
                }
            }).unwrap_or(().into_any())
        }}
    }
}
