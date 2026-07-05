use crate::pages::agent::{AgentTab, AttachmentKind};
use leptos::prelude::*;

#[component]
pub(crate) fn AgentControls(
    active_tab: Signal<AgentTab>,
    input_text: Signal<String>,
    set_input_text: WriteSignal<String>,
    pending_attachments: Signal<Vec<AttachmentKind>>,
    set_pending_attachments: WriteSignal<Vec<AttachmentKind>>,
    is_thinking: Signal<bool>,
    on_send: Callback<leptos::ev::MouseEvent>,
    on_keydown: Callback<leptos::ev::KeyboardEvent>,
    on_file_pick: Callback<leptos::ev::Event>,
    on_video_pick: Callback<leptos::ev::Event>,
    file_input_ref: NodeRef<leptos::html::Input>,
    video_input_ref: NodeRef<leptos::html::Input>,
    on_tab_switch: Callback<AgentTab>,
) -> impl IntoView {
    view! {
        // Pending attachments preview
        {move || {
            let atts = pending_attachments.get();
            if atts.is_empty() { return ().into_any(); }
            view! {
                <div class="agent-attach-preview">
                    {atts.into_iter().enumerate().map(|(i, att)| {
                        let label = match &att {
                            AttachmentKind::Image(n) => format!("🖼 {}", n),
                            AttachmentKind::Video(n) => format!("🎬 {}", n),
                            AttachmentKind::File(n)  => format!("📎 {}", n),
                        };
                        view! {
                            <div class="agent-attach-chip">
                                {label}
                                <button class="agent-attach-remove"
                                    on:click=move |_| set_pending_attachments.update(|a| { a.remove(i); })>
                                    "✕"
                                </button>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            }.into_any()
        }}

        // Input bar
        <div class="agent-input-bar">
            <input
                type="file"
                accept="image/*"
                multiple
                style="display:none"
                node_ref=file_input_ref
                on:change=move |ev| on_file_pick.run(ev)
            />
            <input
                type="file"
                accept="video/*"
                multiple
                style="display:none"
                node_ref=video_input_ref
                on:change=move |ev| on_video_pick.run(ev)
            />
            <button class="agent-attach-btn" title="Upload image"
                on:click=move |_| {
                    if let Some(el) = file_input_ref.get() { let _ = el.click(); }
                }>"🖼"</button>
            <button class="agent-attach-btn" title="Upload video"
                on:click=move |_| {
                    if let Some(el) = video_input_ref.get() { let _ = el.click(); }
                }>"🎬"</button>
            <textarea
                class="agent-textarea"
                rows="1"
                placeholder=move || active_tab.get().placeholder()
                prop:value=move || input_text.get()
                on:input=move |ev| set_input_text.set(event_target_value(&ev))
                on:keydown=move |ev| on_keydown.run(ev)
            ></textarea>
            <button class="agent-send-btn" on:click=move |ev| on_send.run(ev)
                disabled=move || is_thinking.get()>
                "⏎"
            </button>
        </div>

        // Context tabs
        <div class="agent-tabs-bar">
            {[AgentTab::Chat, AgentTab::Portfolios, AgentTab::Analytics,
              AgentTab::Documents, AgentTab::Calendar, AgentTab::Tasks]
                .into_iter().map(|tab| {
                    let label = tab.label();
                    let tab_clone = tab.clone();
                    let on_tab_switch = on_tab_switch.clone();
                    view! {
                        <button
                            class="agent-tab-btn"
                            class:agent-tab-active=move || active_tab.get() == tab_clone
                            on:click=move |_| on_tab_switch.run(tab.clone())
                        >
                            {label}
                        </button>
                    }
                }).collect::<Vec<_>>()}
        </div>
    }
}
