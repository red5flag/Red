use super::contact_card::render_contact_card;
use crate::pages::networking::ExternalContact;
use leptos::prelude::*;

pub(crate) fn render_contacts(
    contacts: Vec<ExternalContact>,
    set_selected: WriteSignal<Option<ExternalContact>>,
) -> impl IntoView {
    let items_for = contacts.clone();
    let items_memo = Memo::new(move |_| items_for.clone());
    view! {
        <div class="net-tab-content">
            {if items_memo.get().is_empty() {
                view! {
                    <div class="data-card">
                        <div class="empty-state"><div class="empty-text">"No contacts found"</div></div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <For
                        each=move || items_memo.get()
                        key=|c| c.id
                        children=move |c| render_contact_card(c, set_selected)
                    />
                }.into_any()
            }}
        </div>
    }
}
