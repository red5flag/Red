use super::contact_card::render_contact_card;
use crate::pages::networking::{ExternalContact, NetTab};
use leptos::prelude::*;

pub(crate) fn render_contacts(
    contacts: Vec<ExternalContact>,
    set_selected: WriteSignal<Option<ExternalContact>>,
    visible_count: Signal<usize>,
    on_expand: Callback<NetTab>,
) -> impl IntoView {
    let items_for = contacts.clone();
    let items_memo = Memo::new(move |_| items_for.clone());
    view! {
        <div class="net-tab-content">
            {move || {
                let total = items_memo.get().len();
                let visible = visible_count.get().min(total);
                let remaining = total.saturating_sub(visible);
                if total == 0 {
                    view! {
                        <div class="data-card">
                            <div class="empty-state"><div class="empty-text">"No contacts found"</div></div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div>
                            <For
                                each=move || items_memo.get().into_iter().take(visible).collect::<Vec<_>>()
                                key=|c| c.id
                                children=move |c| render_contact_card(c, set_selected)
                            />
                            {if remaining > 0 {
                                view! {
                                    <button
                                        class="pf-show-more-btn"
                                        on:click=move |_| on_expand.run(NetTab::Contacts)
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
