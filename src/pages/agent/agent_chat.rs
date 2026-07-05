use crate::pages::agent::{agent_message, ChatMessage};
use leptos::prelude::*;

#[component]
pub(crate) fn AgentChat(
    messages: Signal<Vec<ChatMessage>>,
    is_thinking: Signal<bool>,
    chat_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    view! {
        <div class="agent-chat-area" node_ref=chat_ref>
            {move || {
                let msgs = messages.get();
                msgs.into_iter().map(|msg| {
                    view! {
                        <agent_message::AgentMessage msg={msg} />
                    }
                }).collect::<Vec<_>>()
            }}
            {move || if is_thinking.get() {
                view! {
                    <div class="agent-msg agent-msg-agent">
                        <div class="agent-msg-avatar">"🤖"</div>
                        <div class="agent-msg-bubble agent-thinking">
                            <span class="agent-dot"></span>
                            <span class="agent-dot"></span>
                            <span class="agent-dot"></span>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </div>
    }
}
