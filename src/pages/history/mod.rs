use crate::types::{ActionType, ChangeSeverity};

pub mod history_card;
pub mod history_filters;
pub mod history_list;
pub mod history_summary;
pub mod page;

pub(crate) use history_card::HistoryCard;
pub(crate) use history_filters::HistoryFilters;
pub(crate) use history_list::HistoryList;
pub(crate) use history_summary::HistorySummary;
pub use page::HistoryPage;

pub(crate) fn action_type_badge(action_type: &ActionType) -> (&'static str, &'static str) {
    match action_type {
        ActionType::Create => ("Create", "badge-create"),
        ActionType::Update => ("Update", "badge-update"),
        ActionType::Delete => ("Delete", "badge-delete"),
        ActionType::View => ("View", "badge-view"),
        ActionType::Navigate => ("Navigate", "badge-nav"),
        ActionType::Setting => ("Setting", "badge-setting"),
        ActionType::Payment => ("Payment", "badge-payment"),
        ActionType::Notification => ("Notification", "badge-notif"),
        ActionType::Search => ("Search", "badge-search"),
        ActionType::Undo => ("Undo", "badge-undo"),
        ActionType::Redo => ("Redo", "badge-redo"),
        ActionType::Login => ("Login", "badge-login"),
        ActionType::Logout => ("Logout", "badge-logout"),
    }
}

pub(crate) fn severity_badge(severity: &ChangeSeverity) -> (&'static str, &'static str) {
    match severity {
        ChangeSeverity::Major => ("Major", "severity-major"),
        ChangeSeverity::Minor => ("Minor", "severity-minor"),
        ChangeSeverity::System => ("System", "severity-system"),
    }
}
