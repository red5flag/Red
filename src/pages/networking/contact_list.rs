use super::contact_card::render_contact_card;
use crate::pages::networking::ExternalContact;
use leptos::prelude::*;

pub(crate) fn render_contacts(
    contacts: Vec<ExternalContact>,
    set_selected: WriteSignal<Option<ExternalContact>>,
) -> impl IntoView {
    view! {
        <div class="net-tab-content">
            {if contacts.is_empty() {
                view! {
                    <div class="data-card">
                        <div class="empty-state"><div class="empty-text">"No contacts found"</div></div>
                    </div>
                }.into_any()
            } else {
                contacts.into_iter().map(|c| render_contact_card(c, set_selected)).collect::<Vec<_>>().into_any()
            }}
        </div>
    }
}
