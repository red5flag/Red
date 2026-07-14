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
    let wallets_for = wallets.clone();
    let wallets_memo = Memo::new(move |_| wallets_for.clone());
    let wallets_for_cards = wallets.clone();
    let selected_wallet_cards = Memo::new(move |_| {
        wallets_for_cards
            .iter()
            .find(|w| w.id == selected_wallet.get())
            .map(|w| w.cards.clone())
            .unwrap_or_default()
    });
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
                <For
                    each=move || wallets_memo.get()
                    key=|w| w.id
                    children=move |w| {
                        let id = w.id.to_string();
                        view! { <option value={id.clone()}>{format!("{} ({}{:.2} {})", w.name, currency_symbol(&w.currency), w.balance, w.currency)}</option> }
                    }
                />
            </select>
        </div>
        {move || {
            if selected_wallet_cards.get().is_empty() {
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
                            <For
                                each=move || selected_wallet_cards.get()
                                key=|c| c.id
                                children=move |c| {
                                    let id = c.id.to_string();
                                    view! { <option value={id.clone()}>{format!("{} ending {}", c.label, c.last4)}</option> }
                                }
                            />
                        </select>
                    </div>
                }.into_any()
            }
        }}
    }
}
