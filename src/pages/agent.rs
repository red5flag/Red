use leptos::prelude::*;
use leptos::task::spawn_local;
use web_sys::HtmlInputElement;

#[derive(Clone, Debug, PartialEq)]
enum MessageRole {
    User,
    Agent,
}

#[derive(Clone, Debug)]
enum AttachmentKind {
    Image(String),
    Video(String),
    File(String),
}

#[derive(Clone, Debug)]
struct ChatMessage {
    id: u32,
    role: MessageRole,
    text: String,
    attachments: Vec<AttachmentKind>,
}

#[derive(Clone, Debug, PartialEq)]
enum AgentTab {
    Chat,
    Portfolios,
    Analytics,
    Documents,
    Calendar,
    Tasks,
}

impl AgentTab {
    fn label(&self) -> &'static str {
        match self {
            AgentTab::Chat       => "💬 Chat",
            AgentTab::Portfolios => "🏢 Portfolios",
            AgentTab::Analytics  => "📊 Analytics",
            AgentTab::Documents  => "📄 Documents",
            AgentTab::Calendar   => "📅 Calendar",
            AgentTab::Tasks      => "✅ Tasks",
        }
    }

    fn placeholder(&self) -> &'static str {
        match self {
            AgentTab::Chat       => "Ask anything about your business...",
            AgentTab::Portfolios => "Ask about your portfolios and assets...",
            AgentTab::Analytics  => "Ask for reports, trends, P&L summaries...",
            AgentTab::Documents  => "Ask to generate or find documents...",
            AgentTab::Calendar   => "Ask about bookings and calendar events...",
            AgentTab::Tasks      => "Ask to create tasks or manage your to-do list...",
        }
    }

    fn greeting(&self) -> &'static str {
        match self {
            AgentTab::Chat       => "Hello! I'm your Farley AI assistant. How can I help you today?",
            AgentTab::Portfolios => "I can help you analyse portfolios, assets, and investment performance. What would you like to know?",
            AgentTab::Analytics  => "I can generate analytics, P&L reports, and trend summaries. What data are you looking for?",
            AgentTab::Documents  => "I can help find, summarise, or generate documents. What do you need?",
            AgentTab::Calendar   => "I can help with bookings, events, and scheduling. What can I assist with?",
            AgentTab::Tasks      => "I can help manage your tasks and to-do list. What would you like to track?",
        }
    }
}

fn make_greeting(tab: &AgentTab, id: u32) -> ChatMessage {
    ChatMessage {
        id,
        role: MessageRole::Agent,
        text: tab.greeting().to_string(),
        attachments: vec![],
    }
}

#[component]
pub fn AgentPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(AgentTab::Chat);
    let (messages, set_messages) = signal::<Vec<ChatMessage>>(vec![
        make_greeting(&AgentTab::Chat, 0),
    ]);
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
        move |tab: AgentTab| {
            let id = next_id.get();
            let greeting = make_greeting(&tab, id);
            set_next_id.set(id + 1);
            set_messages.set(vec![greeting]);
            set_active_tab.set(tab);
        }
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
            id,
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
                id: reply_id,
                role: MessageRole::Agent,
                text: reply,
                attachments: vec![],
            };
            set_messages.update(|m| m.push(agent_msg));
            set_is_thinking.set(false);
        });
    };

    let on_send = {
        let do_send = do_send.clone();
        move |_: leptos::ev::MouseEvent| do_send()
    };

    let on_keydown = {
        let do_send = do_send.clone();
        move |ev: leptos::ev::KeyboardEvent| {
            if ev.key() == "Enter" && !ev.shift_key() {
                ev.prevent_default();
                do_send();
            }
        }
    };

    let on_file_pick = move |ev: leptos::ev::Event| {
        use wasm_bindgen::JsCast;
        let input = ev.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
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
    };

    let on_video_pick = move |ev: leptos::ev::Event| {
        use wasm_bindgen::JsCast;
        let input = ev.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
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
    };

    view! {
        <div class="home-screen agent-page">

            // ── HEADER ──
            <div class="agent-topbar">
                <div class="agent-topbar-left">
                    <div class="agent-avatar-sm">"🤖"</div>
                    <div>
                        <div class="agent-topbar-title">"Farley Agent"</div>
                        <div class="agent-topbar-status">"● Online"</div>
                    </div>
                </div>
                <div class="agent-topbar-right">
                    <span class="agent-badge">"AI"</span>
                </div>
            </div>

            // ── CHAT HISTORY ──
            <div class="agent-chat-area" node_ref=chat_ref>
                {move || {
                    let msgs = messages.get();
                    msgs.into_iter().map(|msg| {
                        let is_user = msg.role == MessageRole::User;
                        let cls = if is_user { "agent-msg agent-msg-user" } else { "agent-msg agent-msg-agent" };
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

            // ── PENDING ATTACHMENTS PREVIEW ──
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

            // ── INPUT BAR ──
            <div class="agent-input-bar">
                <input
                    type="file"
                    accept="image/*"
                    multiple
                    style="display:none"
                    node_ref=file_input_ref
                    on:change=on_file_pick
                />
                <input
                    type="file"
                    accept="video/*"
                    multiple
                    style="display:none"
                    node_ref=video_input_ref
                    on:change=on_video_pick
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
                    on:keydown=on_keydown
                ></textarea>
                <button class="agent-send-btn" on:click=on_send
                    disabled=move || is_thinking.get()>
                    "⏎"
                </button>
            </div>

            // ── CONTEXT TABS ──
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
                                on:click=move |_| on_tab_switch(tab.clone())
                            >
                                {label}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
            </div>

        </div>
    }
}

fn simulate_agent_reply(tab: &AgentTab, input: &str, attachments: &[AttachmentKind]) -> String {
    let input_lower = input.to_lowercase();
    if !attachments.is_empty() {
        let count = attachments.len();
        let kinds: Vec<&str> = attachments.iter().map(|a| match a {
            AttachmentKind::Image(_) => "image",
            AttachmentKind::Video(_) => "video",
            AttachmentKind::File(_)  => "file",
        }).collect();
        let kind_str = kinds.join(", ");
        return format!(
            "I've received {} attachment{} ({}). I can analyse these once connected to an AI provider. \
            For now, tell me more about what you'd like me to do with them.",
            count, if count == 1 { "" } else { "s" }, kind_str
        );
    }
    match tab {
        AgentTab::Chat => {
            if input_lower.contains("hello") || input_lower.contains("hi") {
                "Hello! How can I assist your business today?".to_string()
            } else if input_lower.contains("help") {
                "I can help with portfolios, analytics, documents, calendar events, and task management. \
                Use the tabs below to switch context, or just ask me anything here.".to_string()
            } else {
                format!("Got it: \"{}\". I'll process that once connected to your AI provider.", input)
            }
        }
        AgentTab::Portfolios => {
            if input_lower.contains("value") || input_lower.contains("worth") {
                "To get portfolio valuations, connect to your Farley data source. \
                I'll summarise total value, P&L, and asset breakdown across all portfolios.".to_string()
            } else {
                format!("Portfolio query received: \"{}\". I'll analyse your assets when connected.", input)
            }
        }
        AgentTab::Analytics => {
            "I can generate P&L reports, trend charts, and forecasts. \
            Connect to your AI provider to get live analytics.".to_string()
        }
        AgentTab::Documents => {
            "I can draft, summarise, or locate documents. \
            Once connected to an AI provider I'll be able to generate PDFs and reports automatically.".to_string()
        }
        AgentTab::Calendar => {
            "I can list upcoming bookings, check for conflicts, and suggest scheduling optimisations. \
            Connect your AI provider to enable full calendar intelligence.".to_string()
        }
        AgentTab::Tasks => {
            format!("Task noted: \"{}\". I'll track this and remind you when due.", input)
        }
    }
}
