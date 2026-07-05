use crate::pages::transactions::{currency_symbol, Card, Wallet};
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub(crate) fn DigitalCard(
    wallet: Wallet,
    card: Card,
    on_click: impl Fn(Uuid, Uuid) + 'static,
) -> impl IntoView {
    let w_name = wallet.name.clone();
    let w_type = wallet.wallet_type.clone();
    let w_currency = wallet.currency.clone();
    let w_balance = wallet.balance;
    let c_label = card.label.clone();
    let c_last4 = card.last4.clone();
    let wid = wallet.id;
    let cid = card.id;
    let is_crypto = w_type == "Crypto";

    view! {
        <div class="dcard" class:dcard-crypto={is_crypto} on:click=move |_| on_click(wid, cid)>
            <div class="dcard-top">
                <span class="dcard-brand">{if is_crypto { "⚡" } else { "💳" }}</span>
                <span class="dcard-type">{w_type.clone()}</span>
            </div>
            <div class="dcard-number">"•••• •••• •••• "{c_last4.clone()}</div>
            <div class="dcard-bottom">
                <div class="dcard-info">
                    <div class="dcard-label">{c_label.clone()}</div>
                    <div class="dcard-wallet">{w_name.clone()}</div>
                </div>
                <div class="dcard-balance">
                    <div class="dcard-bal-num">{format!("{}{:.2}", currency_symbol(&w_currency), w_balance)}</div>
                    <div class="dcard-bal-cur">{w_currency.clone()}</div>
                </div>
            </div>
        </div>
    }
}
