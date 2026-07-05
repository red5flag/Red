use crate::pages::networking::Integration;
use leptos::prelude::*;

pub(crate) fn render_integrations(integrations: &[Integration]) -> impl IntoView {
    view! {
        <div class="net-tab-content">
            <div class="net-cards-list">
                {integrations.iter().map(|i| {
                    let status_cls = i.status.css_class();
                    let icon = match i.integration_type.as_str() {
                        "Accounting" => "📊",
                        "Document" => "📄",
                        "Payment" => "💳",
                        "Calendar" => "📅",
                        "Communication" => "💬",
                        _ => "⚙️",
                    };
                    view! {
                        <div class="net-relationship-card">
                            <div class="net-rel-avatar net-rel-avatar-int">{icon}</div>
                            <div class="net-rel-body">
                                <div class="net-rel-name">{i.name.clone()}</div>
                                <div class="net-rel-type">{i.integration_type.clone()}</div>
                                <div class="net-rel-meta">
                                    <span class={format!("net-rel-status {}", status_cls)}>{i.status.label()}</span>
                                </div>
                                <div class="net-rel-linked">{i.description.clone()}</div>
                                {i.last_sync.as_ref().map(|s| view! {
                                    <div class="net-rel-activity">{format!("Last sync: {}", s)}</div>
                                }.into_any()).unwrap_or_else(|| ().into_any())}
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
