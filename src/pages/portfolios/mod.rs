use crate::models::User;
use leptos::prelude::*;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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

/// Truncate text to the first sentence and append an ellipsis if there is more.
pub(crate) fn single_sentence(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut end = 0;
    for (idx, ch) in trimmed.char_indices() {
        if ch == '.' || ch == '?' || ch == '!' {
            end = idx + ch.len_utf8();
            break;
        }
    }

    if end == 0 {
        return trimmed.to_string();
    }

    let first = &trimmed[..end];
    let rest = trimmed[end..].trim();
    if rest.is_empty() {
        first.to_string()
    } else {
        format!("{}...", first)
    }
}

/// Read the first image file from a file input change event and return its data URL.
pub(crate) fn read_image_as_data_url(
    ev: &leptos::ev::Event,
    on_data_url: impl FnOnce(String) + 'static,
) {
    let input = ev
        .target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .expect("file input target");
    if let Some(files) = input.files() {
        if let Some(file) = files.get(0) {
            let reader = match web_sys::FileReader::new() {
                Ok(r) => r,
                Err(_) => return,
            };
            let onload = Closure::once(Box::new(move |e: web_sys::Event| {
                if let Some(target) = e.target() {
                    if let Ok(reader) = target.dyn_into::<web_sys::FileReader>() {
                        if let Ok(result) = reader.result() {
                            if let Some(data_url) = result.as_string() {
                                on_data_url(data_url);
                            }
                        }
                    }
                }
            }) as Box<dyn FnOnce(web_sys::Event)>);
            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            let _ = reader.read_as_data_url(&file);
            onload.forget();
        }
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
                let assigned_for_for = assigned.clone();
                view! {
                    <For
                        each=move || users.clone()
                        key=|u| u.id
                        children=move |u| {
                            let checked = assigned_for_for.contains(&u.id);
                            let uid = u.id;
                            view! {
                                <label class="assignment-row">
                                    <input type="checkbox" checked=checked on:change=move |_| on_toggle.run(uid) />
                                    <span>{format!("{} ({:?})", u.name, u.role)}</span>
                                </label>
                            }
                        }
                    />
                }.into_any()
            }}
        </div>
    }
}

mod asset_channels;
mod asset_group;
mod asset_item;
mod asset_viewer;
mod doc_modal;
mod notifications;
mod page;
mod portfolio_list;

pub(crate) use asset_channels::{AssetChannelManagement, AssetChannelsSection};
pub(crate) use asset_group::AssetGroupItem;
pub(crate) use asset_item::{asset_placeholder_url, AssetDetailView, AssetItem};
pub(crate) use asset_viewer::AssetViewer;
pub(crate) use doc_modal::{DocModal, DocumentViewer};
pub(crate) use notifications::{NotificationContentView, NotificationQuickSettings};
pub(crate) use portfolio_list::PortfolioListItem;

pub use page::PortfoliosPage;
