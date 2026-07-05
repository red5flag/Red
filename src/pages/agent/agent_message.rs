use crate::pages::agent::{AttachmentKind, ChatMessage, MessageRole};
use leptos::prelude::*;

#[component]
pub(crate) fn AgentMessage(msg: ChatMessage) -> impl IntoView {
    let is_user = msg.role == MessageRole::User;
    let cls = if is_user {
        "agent-msg agent-msg-user"
    } else {
        "agent-msg agent-msg-agent"
    };

    view! {
        <div class=cls>
            {if !is_user {
                view! { <div class="agent-msg-avatar">"🤖"</div> }.into_any()
            } else {
                view! { <div class="agent-msg-avatar agent-msg-avatar-user">"👤"</div> }.into_any()
            }}
            <div class="agent-msg-bubble">
                {if !msg.text.is_empty() {
                    view! { <div class="agent-msg-text">{msg.text.clone()}</div> }.into_any()
                } else { ().into_any() }}
                {msg.attachments.into_iter().map(|att| {
                    match att {
                        AttachmentKind::Image(name) => view! {
                            <div class="agent-attach agent-attach-image">"🖼 "{name}</div>
                        }.into_any(),
                        AttachmentKind::Video(name) => view! {
                            <div class="agent-attach agent-attach-video">"🎬 "{name}</div>
                        }.into_any(),
                        AttachmentKind::File(name) => view! {
                            <div class="agent-attach agent-attach-file">"📎 "{name}</div>
                        }.into_any(),
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
