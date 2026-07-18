use crate::models::Portfolio;
use crate::pages::portfolios::read_image_as_data_url;
use crate::stores::{use_app_store, use_notification_store, use_organization_store, use_ui_store};
use crate::types::Currency;
use leptos::prelude::*;
use uuid::Uuid;

const EMOJIS: &[&str] = &[
    "🏢", "🏠", "🚗", "💼", "💰", "📈", "🏭", "🌐", "🎨", "🔬", "⚡",
];

fn random_emoji() -> &'static str {
    let id = Uuid::new_v4();
    let bytes = id.as_bytes();
    let idx = bytes.iter().map(|&b| b as usize).sum::<usize>() % EMOJIS.len();
    EMOJIS[idx]
}

#[component]
pub(crate) fn AddPortfolioModal() -> impl IntoView {
    let app_store = use_app_store();
    let organization_store = use_organization_store();
    let notification_store = use_notification_store();
    let ui_store = use_ui_store();

    let (name, set_name) = signal(String::new());
    let (desc, set_desc) = signal(String::new());
    let (image_url, set_image_url) = signal(Option::<String>::None);
    let (org_id, set_org_id) = signal(String::new());

    let close = move |_| {
        ui_store.update(|s| s.show_add_portfolio = false);
    };

    let create = move |_| {
        let name_str = name.get();
        if name_str.trim().is_empty() {
            return;
        }
        let owner_id = app_store.get().current_user.id;
        let mut p = Portfolio::new(name_str, owner_id, Currency::USD);
        let desc_str = desc.get().trim().to_string();
        p.description = if desc_str.is_empty() { None } else { Some(desc_str) };
        p.image_url = image_url.get();
        if p.image_url.is_none() {
            p.emoji = Some(random_emoji().to_string());
        }
        if let Ok(oid) = Uuid::parse_str(&org_id.get()) {
            p.organization_id = Some(oid);
        }

        notification_store.update(|ns| {
            app_store.update(|s| s.add_portfolio(p, ns));
        });
        ui_store.update(|s| s.show_add_portfolio = false);
    };

    view! {
        <div class="apf-dropdown" on:click=|ev| ev.stop_propagation()>
            <div class="apf-dropdown-header">
                <span class="apf-dropdown-title">"Create Portfolio"</span>
                <button class="apf-dropdown-close" aria-label="Close" on:click=move |_| close(())>"×"</button>
            </div>
            <div class="apf-dropdown-body">
                <div class="add-form apf-form">
                    <label class="org-edit-label">"Title"</label>
                    <input
                        class="login-input apf-input"
                        type="text"
                        placeholder="Portfolio name"
                        prop:value={move || name.get()}
                        on:input=move |ev| set_name.set(event_target_value(&ev)) />
                    <label class="org-edit-label">"Subtitle / Description"</label>
                    <input
                        class="login-input apf-input"
                        type="text"
                        placeholder="Description (optional)"
                        prop:value={move || desc.get()}
                        on:input=move |ev| set_desc.set(event_target_value(&ev)) />
                    <label class="org-edit-label">"Image (optional)"</label>
                    <input
                        class="login-input apf-input apf-file-input"
                        type="file"
                        accept="image/*"
                        on:change=move |ev| read_image_as_data_url(&ev, move |url| set_image_url.set(Some(url))) />
                    <label class="org-edit-label">"Organization"</label>
                    <select
                        class="login-input apf-input"
                        prop:value={move || org_id.get()}
                        on:change=move |ev| set_org_id.set(event_target_value(&ev))>
                        <option value="">"No organization"</option>
                        <For
                            each={move || organization_store.get().organizations.clone()}
                            key={|o| o.id}
                            children={move |o| view! {
                                <option value={o.id.to_string()}>{o.name.clone()}</option>
                            }}
                        />
                    </select>
                    <div class="apf-actions">
                        <button class="login-btn" type="button" on:click=create>"Create Portfolio"</button>
                        <button class="view-btn" type="button" on:click=move |_| close(())>"Cancel"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}
