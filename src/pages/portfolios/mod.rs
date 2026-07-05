use crate::models::User;
use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone, PartialEq)]
pub enum NotifTarget {
    Portfolio(Uuid),
    Group(Uuid, Uuid),
}

#[derive(Clone, PartialEq, Default)]
pub enum AssetTarget {
    #[default]
    None,
    PortfolioDirect(Uuid),
    Group(Uuid, Uuid),
}

pub(crate) fn document_icon(file_type: &str) -> &'static str {
    match file_type.to_lowercase().as_str() {
        "pdf" => "📕",
        "doc" | "docx" => "📘",
        "odt" => "📘",
        "xls" | "xlsx" => "📗",
        "ppt" | "pptx" => "📙",
        "txt" | "md" | "rs" | "js" | "ts" | "html" | "css" => "📄",
        "zip" | "rar" | "7z" | "tar" => "🗜️",
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" => "🖼️",
        "mp4" | "mov" | "avi" | "mkv" | "webm" => "🎬",
        "mp3" | "wav" | "flac" => "🎵",
        _ => "📎",
    }
}

pub(crate) fn detect_file_type(name: &str) -> String {
    if let Some(idx) = name.rfind('.') {
        let ext = &name[idx + 1..];
        let ext_lower = ext.to_lowercase();
        match ext_lower.as_str() {
            "pdf" | "docx" | "doc" | "txt" | "odt" | "rtf" | "xlsx" | "xls" | "csv" | "pptx"
            | "ppt" | "md" | "json" | "xml" | "html" | "css" | "js" | "ts" | "rs" | "py" | "go"
            | "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "tiff" | "ico" | "mp4"
            | "mov" | "avi" | "mkv" | "webm" | "flv" | "mp3" | "wav" | "flac" | "aac" | "ogg"
            | "zip" | "rar" | "7z" | "tar" | "gz" => ext_lower,
            _ => "txt".to_string(),
        }
    } else {
        "txt".to_string()
    }
}

pub(crate) fn shorthand_name(name: &str) -> String {
    if name.len() <= 16 {
        name.to_string()
    } else {
        format!("{}...", &name[..13])
    }
}

#[component]
pub(crate) fn UserAssignmentPanel(
    assigned: Vec<Uuid>,
    users: Vec<User>,
    on_toggle: Callback<Uuid>,
) -> impl IntoView {
    view! {
        <div class="assignment-panel">
            <div class="assignment-title">"Assigned users"</div>
            {if users.is_empty() {
                view! { <div class="assignment-empty">"No users available"</div> }.into_any()
            } else {
                users.into_iter().map(move |u| {
                    let checked = assigned.contains(&u.id);
                    let uid = u.id;
                    view! {
                        <label class="assignment-row">
                            <input type="checkbox" checked=checked on:change=move |_| on_toggle.run(uid) />
                            <span>{format!("{} ({:?})", u.name, u.role)}</span>
                        </label>
                    }
                }).collect::<Vec<_>>().into_any()
            }}
        </div>
    }
}

mod asset_group;
mod asset_item;
mod asset_viewer;
mod doc_modal;
mod notifications;
mod page;
mod portfolio_list;

pub(crate) use asset_group::AssetGroupItem;
pub(crate) use asset_item::{asset_placeholder_url, AssetDetailView, AssetItem};
pub(crate) use asset_viewer::AssetViewer;
pub(crate) use doc_modal::{DocModal, DocumentViewer};
pub(crate) use notifications::{NotificationContentView, NotificationQuickSettings};
pub(crate) use portfolio_list::PortfolioListItem;

pub use page::PortfoliosPage;
