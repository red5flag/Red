use crate::pages::agent::{
    agent_chat, agent_controls, agent_status, make_greeting, simulate_agent_reply, AgentTab,
    AttachmentKind, ChatMessage, MessageRole,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use web_sys::HtmlInputElement;

#[component]
pub fn AgentPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(AgentTab::Chat);
    let (messages, set_messages) =
        signal::<Vec<ChatMessage>>(vec![make_greeting(&AgentTab::Chat, 0)]);
    let (input_text, set_input_text) = signal(String::new());
    let (is_thinking, set_is_thinking) = signal(false);
    let (next_id, set_next_id) = signal(1u32);
    let (pending_attachments, set_pending_attachments) = signal::<Vec<AttachmentKind>>(vec![]);

    let file_input_ref = NodeRef::<leptos::html::Input>::new();
    let video_input_ref = NodeRef::<leptos::html::Input>::new();
    let chat_ref = NodeRef::<leptos::html::Div>::new();

    let _scroll_to_bottom = move || {
        if let Some(el) = chat_ref.get() {
            let scroll_height = el.scroll_height();
            el.set_scroll_top(scroll_height);
        }
    };

    let on_tab_switch = {
        let set_messages = set_messages.clone();
        let set_next_id = set_next_id.clone();
        Callback::new(move |tab: AgentTab| {
            let id = next_id.get();
            let greeting = make_greeting(&tab, id);
            set_next_id.set(id + 1);
            set_messages.set(vec![greeting]);
            set_active_tab.set(tab);
        })
    };

    let do_send = move || {
        let text = input_text.get();
        let attachments = pending_attachments.get();
        if text.trim().is_empty() && attachments.is_empty() {
            return;
        }
        let id = next_id.get();
        set_next_id.set(id + 2);

        let user_msg = ChatMessage {
            _id: id,
            role: MessageRole::User,
            text: text.clone(),
            attachments: attachments.clone(),
        };
        set_messages.update(|m| m.push(user_msg));
        set_input_text.set(String::new());
        set_pending_attachments.set(vec![]);
        set_is_thinking.set(true);

        let tab = active_tab.get();
        let reply_id = id + 1;
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(900).await;
            let reply = simulate_agent_reply(&tab, &text, &attachments);
            let agent_msg = ChatMessage {
                _id: reply_id,
                role: MessageRole::Agent,
                text: reply,
                attachments: vec![],
            };
            set_messages.update(|m| m.push(agent_msg));
            set_is_thinking.set(false);
        });
    };

    let on_send = Callback::new(move |_: leptos::ev::MouseEvent| do_send());

    let on_keydown = Callback::new(move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            do_send();
        }
    });

    let on_file_pick = Callback::new(move |ev: leptos::ev::Event| {
        use wasm_bindgen::JsCast;
        let input = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
        if let Some(input) = input {
            if let Some(files) = input.files() {
                for i in 0..files.length() {
                    if let Some(file) = files.item(i) {
                        let name = file.name();
                        set_pending_attachments.update(|a| a.push(AttachmentKind::Image(name)));
                    }
                }
            }
            input.set_value("");
        }
    });

    let on_video_pick = Callback::new(move |ev: leptos::ev::Event| {
        use wasm_bindgen::JsCast;
        let input = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
        if let Some(input) = input {
            if let Some(files) = input.files() {
                for i in 0..files.length() {
                    if let Some(file) = files.item(i) {
                        let name = file.name();
                        set_pending_attachments.update(|a| a.push(AttachmentKind::Video(name)));
                    }
                }
            }
            input.set_value("");
        }
    });

    view! {
        <div class="home-screen agent-page">
            <agent_status::AgentStatus />

            <agent_chat::AgentChat
                messages={Signal::derive(move || messages.get())}
                is_thinking={Signal::derive(move || is_thinking.get())}
                chat_ref={chat_ref}
            />

            <agent_controls::AgentControls
                active_tab={Signal::derive(move || active_tab.get())}
                input_text={input_text.into()}
                set_input_text={set_input_text}
                pending_attachments={pending_attachments.into()}
                set_pending_attachments={set_pending_attachments}
                is_thinking={Signal::derive(move || is_thinking.get())}
                on_send={on_send}
                on_keydown={on_keydown}
                on_file_pick={on_file_pick}
                on_video_pick={on_video_pick}
                file_input_ref={file_input_ref}
                video_input_ref={video_input_ref}
                on_tab_switch={on_tab_switch}
            />
        </div>
    }
}
