use crate::pages::networking::Channel;
use leptos::prelude::*;

pub(crate) fn render_channels(channels: &[Channel]) -> impl IntoView {
    view! {
        <div class="net-tab-content">
            <div class="net-channels-grid">
                {channels.iter().map(|ch| {
                    let status_cls = ch.status.css_class();
                    let icon = match ch.channel_type.as_str() {
                        "Communication" => "📧",
                        "Platform" => "🌐",
                        "Social" => "👤",
                        "Financial" => "🏦",
                        "Network" => "🔗",
                        _ => "📡",
                    };
                    view! {
                        <div class="net-channel-card">
                            <div class="net-channel-icon">{icon}</div>
                            <div class="net-channel-name">{ch.name.clone()}</div>
                            <div class="net-channel-type">{ch.channel_type.clone()}</div>
                            {ch.address.as_ref().map(|a| view! {
                                <div class="net-channel-addr">{a.clone()}</div>
                            }.into_any()).unwrap_or_else(|| ().into_any())}
                            <span class={format!("net-rel-status {}", status_cls)}>{ch.status.label()}</span>
                            {ch.linked_contact.as_ref().map(|c| view! {
                                <div class="net-channel-contact">{format!("Linked: {}", c)}</div>
                            }.into_any()).unwrap_or_else(|| ().into_any())}
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
