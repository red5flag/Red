pub mod agent_chat;
pub mod agent_controls;
pub mod agent_message;
pub mod agent_status;
pub mod page;

pub use page::AgentPage;

#[derive(Clone, Debug, PartialEq)]
pub enum MessageRole {
    User,
    Agent,
}

#[derive(Clone, Debug)]
pub enum AttachmentKind {
    Image(String),
    Video(String),
    File(String),
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub _id: u32,
    pub role: MessageRole,
    pub text: String,
    pub attachments: Vec<AttachmentKind>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AgentTab {
    Chat,
    Portfolios,
    Analytics,
    Documents,
    Calendar,
    Tasks,
}

impl AgentTab {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            AgentTab::Chat => "💬 Chat",
            AgentTab::Portfolios => "🏢 Portfolios",
            AgentTab::Analytics => "📊 Analytics",
            AgentTab::Documents => "📄 Documents",
            AgentTab::Calendar => "📅 Calendar",
            AgentTab::Tasks => "✅ Tasks",
        }
    }

    pub(crate) fn placeholder(&self) -> &'static str {
        match self {
            AgentTab::Chat => "Ask anything about your business...",
            AgentTab::Portfolios => "Ask about your portfolios and assets...",
            AgentTab::Analytics => "Ask for reports, trends, P&L summaries...",
            AgentTab::Documents => "Ask to generate or find documents...",
            AgentTab::Calendar => "Ask about bookings and calendar events...",
            AgentTab::Tasks => "Ask to create tasks or manage your to-do list...",
        }
    }

    pub(crate) fn greeting(&self) -> &'static str {
        match self {
            AgentTab::Chat       => "Hello! I'm your Red AI assistant. How can I help you today?",
            AgentTab::Portfolios => "I can help you analyse portfolios, assets, and investment performance. What would you like to know?",
            AgentTab::Analytics  => "I can generate analytics, P&L reports, and trend summaries. What data are you looking for?",
            AgentTab::Documents  => "I can help find, summarise, or generate documents. What do you need?",
            AgentTab::Calendar   => "I can help with bookings, events, and scheduling. What can I assist with?",
            AgentTab::Tasks      => "I can help manage your tasks and to-do list. What would you like to track?",
        }
    }
}

pub(crate) fn make_greeting(tab: &AgentTab, _id: u32) -> ChatMessage {
    ChatMessage {
        _id,
        role: MessageRole::Agent,
        text: tab.greeting().to_string(),
        attachments: vec![],
    }
}

pub(crate) fn simulate_agent_reply(
    tab: &AgentTab,
    input: &str,
    attachments: &[AttachmentKind],
) -> String {
    let input_lower = input.to_lowercase();
    if !attachments.is_empty() {
        let count = attachments.len();
        let kinds: Vec<&str> = attachments
            .iter()
            .map(|a| match a {
                AttachmentKind::Image(_) => "image",
                AttachmentKind::Video(_) => "video",
                AttachmentKind::File(_) => "file",
            })
            .collect();
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
                "To get portfolio valuations, connect to your Red data source. \
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
