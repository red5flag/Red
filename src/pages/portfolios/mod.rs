use crate::models::User;
use crate::stores::use_ui_store;
use leptos::prelude::*;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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

/// Clamp context-menu coordinates so the menu stays inside the viewport.
pub(crate) fn clamp_context_menu(x: i32, y: i32) -> (i32, i32) {
    const MENU_MAX_W: i32 = 240;
    const MENU_MAX_H: i32 = 320;
    let Some(window) = web_sys::window() else {
        return (x, y);
    };
    let Some(vw) = window.inner_width().ok().and_then(|v| v.as_f64()) else {
        return (x, y);
    };
    let Some(vh) = window.inner_height().ok().and_then(|v| v.as_f64()) else {
        return (x, y);
    };
    let vw = vw as i32;
    let vh = vh as i32;
    let x = x.min((vw - MENU_MAX_W).max(0)).max(0);
    let y = y.min((vh - MENU_MAX_H).max(0)).max(0);
    (x, y)
}

/// Trigger a browser download for the given document.
pub(crate) fn download_document(doc: &crate::models::Document) {
    let filename = if doc
        .name
        .rsplit('.')
        .next()
        .map_or(false, |ext| !ext.is_empty())
    {
        doc.name.clone()
    } else {
        format!("{}.{}", doc.name, doc.file_type)
    };

    if !doc.url.is_empty() && doc.url != "#" {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Ok(anchor) = document
            .create_element("a")
            .map(|e| e.dyn_into::<web_sys::HtmlAnchorElement>())
        else {
            return;
        };
        let Ok(anchor) = anchor else {
            return;
        };
        anchor.set_href(&doc.url);
        anchor.set_download(&filename);
        let _ = anchor.click();
        return;
    }

    let body = doc.content.clone().unwrap_or_default().into_bytes();
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let arr = js_sys::Array::new();
    let uint8 = js_sys::Uint8Array::from(body.as_slice());
    arr.push(&uint8);
    let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence(&arr) else {
        return;
    };
    let Ok(anchor) = document
        .create_element("a")
        .map(|e| e.dyn_into::<web_sys::HtmlAnchorElement>())
    else {
        return;
    };
    let Ok(anchor) = anchor else {
        return;
    };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
        return;
    };
    anchor.set_href(&url);
    anchor.set_download(&filename);
    let _ = anchor.click();
    let _ = web_sys::Url::revoke_object_url(&url);
}

/// Read the first image file from a file input change event and return its data URL.
pub(crate) fn read_image_as_data_url(
    ev: &leptos::ev::Event,
    on_data_url: impl FnOnce(String) + 'static,
) {
    read_uploaded_file(ev, move |_, data_url| on_data_url(data_url));
}

/// Read the first file from a file input change event and return its name and data URL.
pub(crate) fn read_uploaded_file(
    ev: &leptos::ev::Event,
    on_file: impl FnOnce(String, String) + 'static,
) {
    let input = ev
        .target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .expect("file input target");
    if let Some(files) = input.files() {
        if let Some(file) = files.get(0) {
            let file_name = file.name();
            let reader = match web_sys::FileReader::new() {
                Ok(r) => r,
                Err(_) => return,
            };
            let onload = Closure::once(Box::new(move |e: web_sys::Event| {
                if let Some(target) = e.target() {
                    if let Ok(reader) = target.dyn_into::<web_sys::FileReader>() {
                        if let Ok(result) = reader.result() {
                            if let Some(data_url) = result.as_string() {
                                on_file(file_name, data_url);
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

/// Create click/double-click handlers for a name element.
/// Single click toggles the dropdown after a short delay; a second click
/// within the delay treats the action as a double-click and starts editing instead.
pub(crate) fn name_click_handlers(
    on_toggle: impl Fn() + Clone + Send + Sync + 'static,
    on_edit: impl Fn() + Clone + Send + Sync + 'static,
) -> (
    impl Fn(leptos::ev::MouseEvent) + Clone,
    impl Fn(leptos::ev::MouseEvent) + Clone,
) {
    use std::sync::Arc;

    let (pending, set_pending) = signal(false);
    let on_toggle: Arc<Box<dyn Fn() + Send + Sync>> = Arc::new(Box::new(on_toggle));
    let on_edit: Arc<Box<dyn Fn() + Send + Sync>> = Arc::new(Box::new(on_edit));

    let on_click = {
        let on_toggle = on_toggle.clone();
        let on_edit = on_edit.clone();
        move |ev: leptos::ev::MouseEvent| {
            ev.stop_propagation();
            if pending.get() {
                set_pending.set(false);
                (*on_edit)();
                return;
            }
            set_pending.set(true);
            let on_toggle = on_toggle.clone();
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(200).await;
                if pending.get() {
                    set_pending.set(false);
                    (*on_toggle)();
                }
            });
        }
    };

    let on_dblclick = {
        let on_edit = on_edit.clone();
        move |ev: leptos::ev::MouseEvent| {
            ev.stop_propagation();
            if pending.get() {
                set_pending.set(false);
                (*on_edit)();
            }
        }
    };

    (on_click, on_dblclick)
}

/// Kinds of entities that can be dropped onto the portfolio nav tab bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavTabKind {
    Organization,
    Portfolio,
    AssetGroup,
    Asset,
    Document,
}

impl NavTabKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            NavTabKind::Organization => "organization",
            NavTabKind::Portfolio => "portfolio",
            NavTabKind::AssetGroup => "asset-group",
            NavTabKind::Asset => "asset",
            NavTabKind::Document => "document",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "organization" => Some(NavTabKind::Organization),
            "portfolio" => Some(NavTabKind::Portfolio),
            "asset-group" => Some(NavTabKind::AssetGroup),
            "asset" => Some(NavTabKind::Asset),
            "document" => Some(NavTabKind::Document),
            _ => None,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            NavTabKind::Organization => "🏢",
            NavTabKind::Portfolio => "📁",
            NavTabKind::AssetGroup => "📂",
            NavTabKind::Asset => "📦",
            NavTabKind::Document => "📄",
        }
    }
}

/// A tab pinned to the portfolio navigation bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavTab {
    pub kind: NavTabKind,
    pub id: Uuid,
    pub name: String,
}

/// Build the JSON payload used for drag-and-drop onto the portfolio nav bar.
pub(crate) fn nav_tab_drag_payload(kind: &NavTabKind, id: Uuid, name: &str) -> String {
    serde_json::json!({
        "kind": kind.as_str(),
        "id": id.to_string(),
        "name": name,
    })
    .to_string()
}

/// Attach nav tab drag data to a DragEvent.
pub(crate) fn set_drag_tab_data(ev: &leptos::ev::DragEvent, kind: &NavTabKind, id: Uuid, name: &str) {
    if let Some(dt) = ev.data_transfer() {
        let payload = nav_tab_drag_payload(kind, id, name);
        let _ = dt.set_data("text/plain", &payload);
        let _ = dt.set_effect_allowed("copy");
        #[cfg(target_arch = "wasm32")]
        set_tab_drag_image(ev, kind);
        leptos::logging::log!("set_drag_tab_data: {} {}", kind.as_str(), payload);
    }
}

#[cfg(target_arch = "wasm32")]
fn set_tab_drag_image(ev: &leptos::ev::DragEvent, kind: &NavTabKind) {
    use std::cell::RefCell;
    use web_sys::{Document, HtmlElement};
    use wasm_bindgen::JsCast;

    thread_local! {
        static DRAG_IMAGE: RefCell<Option<HtmlElement>> = RefCell::new(None);
    }

    let dt = match ev.unchecked_ref::<web_sys::DragEvent>().data_transfer() {
        Some(dt) => dt,
        None => return,
    };

    let img = DRAG_IMAGE.with(|cell| -> Option<HtmlElement> {
        let mut cell = cell.borrow_mut();
        if let Some(el) = cell.as_ref() {
            el.set_inner_html(&format!("<span>{}</span>", kind.icon()));
            return Some(el.clone());
        }

        let doc: Document = leptos::prelude::document();
        let el: HtmlElement = doc
            .create_element("div")
            .ok()?
            .dyn_into::<HtmlElement>()
            .ok()?;
        el.set_class_name("pf-nav-tab-drag-image");
        el.style().set_css_text(
            "position:fixed;left:-1000px;top:-1000px;width:48px;height:48px;display:flex;align-items:center;justify-content:center;background:var(--secondary-bg);border:2px solid var(--border-color);border-radius:0;color:var(--text-primary);font-size:24px;pointer-events:none;z-index:-1;"
        );
        el.set_inner_html(&format!("<span>{}</span>", kind.icon()));
        doc.body().and_then(|b| b.append_child(el.as_ref()).ok());
        let cloned = el.clone();
        *cell = Some(el);
        Some(cloned)
    });

    if let Some(img) = img {
        let _ = dt.set_drag_image(img.as_ref(), 24, 24);
    }
}

/// Parse a NavTab from a DragEvent, if one was provided.
pub(crate) fn nav_tab_from_drag_event(ev: &leptos::ev::DragEvent) -> Option<NavTab> {
    let dt = ev.data_transfer()?;
    let data = dt.get_data("text/plain").ok()?;
    let v: serde_json::Value = serde_json::from_str(&data).ok()?;
    let kind = v.get("kind")?.as_str()?;
    let id = v.get("id")?.as_str()?.parse::<Uuid>().ok()?;
    let name = v.get("name")?.as_str()?.to_string();
    Some(NavTab {
        kind: NavTabKind::from_str(kind)?,
        id,
        name,
    })
}

/// Return mobile touch handlers that:
/// - Long-press (tap-hold) opens a context menu at the touch location.
/// - Long-press then drag, released over the portfolio tab bar, opens the item in a new tab.
pub(crate) fn mobile_tab_gestures(
    kind: NavTabKind,
    id: Uuid,
    name: String,
    open_context_menu: impl Fn(i32, i32) + Clone + 'static,
    close_context_menu: impl Fn() + Clone + 'static,
) -> (
    impl Fn(leptos::ev::TouchEvent),
    impl Fn(leptos::ev::TouchEvent),
    impl Fn(leptos::ev::TouchEvent),
) {
    let ui_store = use_ui_store();
    let (touch_anchor, set_touch_anchor) = signal(Option::<(i32, i32)>::None);
    let (long_fired, set_long_fired) = signal(false);
    let (dragging, set_dragging) = signal(false);

    let start = move |ev: leptos::ev::TouchEvent| {
        if let Some(t) = ev.touches().get(0) {
            let x = t.client_x();
            let y = t.client_y();
            set_touch_anchor.set(Some((x, y)));
            set_long_fired.set(false);
            set_dragging.set(false);
            let open_context_menu = open_context_menu.clone();
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(500).await;
                if touch_anchor.get().is_some() && !long_fired.get() && !dragging.get() {
                    let (x, y) = clamp_context_menu(x, y);
                    set_long_fired.set(true);
                    open_context_menu(x, y);
                }
            });
        }
    };

    let move_fn = move |ev: leptos::ev::TouchEvent| {
        if let (Some(t), Some((ax, ay))) = (ev.touches().get(0), touch_anchor.get()) {
            let dx = (t.client_x() - ax).abs();
            let dy = (t.client_y() - ay).abs();
            if !long_fired.get() && (dx > 10 || dy > 10) {
                set_touch_anchor.set(None);
            } else if long_fired.get() && (dx > 10 || dy > 10) && !dragging.get() {
                set_dragging.set(true);
                close_context_menu.clone()();
            }
        }
    };

    let end = move |ev: leptos::ev::TouchEvent| {
        if dragging.get() {
            if let Some(t) = ev.changed_touches().get(0) {
                let x = t.client_x();
                let y = t.client_y();
                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                    if let Some(elem) = doc.element_from_point(x as f32, y as f32) {
                        if let Ok(Some(_)) = elem.closest(".portfolio-controls-bar, .pf-nav-controls, .pf-nav-tabs-scroll") {
                            ui_store.update(|s| s.open_nav_tab(kind.as_str(), id, &name));
                        }
                    }
                }
            }
            ev.prevent_default();
        } else if long_fired.get() {
            ev.prevent_default();
        }
        set_touch_anchor.set(None);
        set_long_fired.set(false);
        set_dragging.set(false);
    };

    (start, move_fn, end)
}

/// Read every selected image file from a file input change event and call the callback for each.
pub(crate) fn read_images_as_data_urls(
    ev: &leptos::ev::Event,
    on_data_url: impl Fn(String) + Clone + 'static,
) {
    let input = ev
        .target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .expect("file input target");
    if let Some(files) = input.files() {
        let len = files.length();
        for i in 0..len {
            if let Some(file) = files.get(i) {
                let reader = match web_sys::FileReader::new() {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let on_data_url = on_data_url.clone();
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

mod add_portfolio_modal;
mod asset_channels;
mod asset_editor;
mod asset_group;
mod asset_item;
mod asset_viewer;
mod doc_dropdown;
mod doc_modal;
mod doc_slider;
mod page;
mod portfolio_list;

pub(crate) use add_portfolio_modal::AddPortfolioModal;
pub(crate) use asset_channels::{
    AddChannelModal, AssetBookingControls, AssetChannelsSection, AssetLinkingControls,
    ChannelManagementWindow,
};
pub(crate) use asset_editor::AssetDetailView;
pub(crate) use asset_group::AssetGroupItem;
pub(crate) use asset_item::{asset_placeholder_url, AssetItem};
pub(crate) use asset_viewer::AssetViewer;
pub(crate) use doc_dropdown::DocEntry;
pub(crate) use doc_modal::DocModal;
pub(crate) use doc_slider::DocSlider;
pub(crate) use portfolio_list::PortfolioListItem;

pub use page::PortfoliosPage;
