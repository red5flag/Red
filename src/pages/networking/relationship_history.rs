use crate::pages::networking::{ExternalContact, ExternalOrganization, RelationshipEvent};
use chrono::Utc;
use leptos::prelude::*;

pub(crate) fn render_relationship_map(
    contacts: &[ExternalContact],
    orgs: &[ExternalOrganization],
) -> impl IntoView {
    view! {
        <div class="net-tab-content">
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Relationship Map"</span>
                </div>
                <div class="net-rel-map">
                    <div class="net-rel-map-center">
                        <div class="net-rel-map-core">"RedOrg"</div>
                    </div>
                    {orgs.iter().map(|o| {
                        let status_cls = o.status.css_class();
                        view! {
                            <div class="net-rel-map-node">
                                <div class="net-rel-map-node-name">{o.name.clone()}</div>
                                <div class="net-rel-map-node-type">{o.org_type.clone()}</div>
                                <span class={format!("net-rel-status {}", status_cls)}>{o.status.label()}</span>
                                <div class="net-rel-map-contacts">
                                    {contacts.iter().filter(|c| c.company == o.name).map(|c| {
                                        view! { <div class="net-rel-map-contact">{c.name.clone()}</div> }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}

pub(crate) fn render_relationship_history(events: &[RelationshipEvent]) -> impl IntoView {
    let now = Utc::now();
    let today: Vec<_> = events
        .iter()
        .filter(|e| (now - e.timestamp).num_hours() < 24)
        .cloned()
        .collect();
    let earlier: Vec<_> = events
        .iter()
        .filter(|e| (now - e.timestamp).num_hours() >= 24)
        .cloned()
        .collect();

    let render_event = |e: &RelationshipEvent| {
        let icon = match e.event_type.as_str() {
            "Message" => "💬",
            "Transaction" => "💰",
            "Banking" => "🏦",
            "Portfolio" => "📊",
            "Document" => "📄",
            "Organization" => "🏢",
            "Integration" => "⚙️",
            _ => "📌",
        };
        view! {
            <div class="net-history-item">
                <span class="net-history-icon">{icon}</span>
                <div class="net-history-body">
                    <div class="net-history-desc">{e.event_description.clone()}</div>
                    <div class="net-history-meta">
                        {format!("{} • {}", e.entity_name, e.timestamp.format("%d %b %Y"))}
                    </div>
                </div>
            </div>
        }
    };

    view! {
        <div class="net-tab-content">
            {if !today.is_empty() {
                view! {
                    <div class="net-history-section">
                        <div class="net-history-day">"Today"</div>
                        {today.iter().map(render_event).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            } else { ().into_any() }}
            {if !earlier.is_empty() {
                view! {
                    <div class="net-history-section">
                        <div class="net-history-day">"Earlier"</div>
                        {earlier.iter().map(render_event).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            } else { ().into_any() }}
            {if today.is_empty() && earlier.is_empty() {
                view! {
                    <div class="data-card">
                        <div class="empty-state"><div class="empty-text">"No relationship history"</div></div>
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </div>
    }
}
