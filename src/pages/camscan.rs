use base64::Engine;
use leptos::prelude::*;
use uuid::Uuid;

use crate::stores::use_organization_store;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScanSource {
    Files,
    Camera,
    GoogleDrive,
    ProtonDrive,
}

impl ScanSource {
    fn label(&self) -> &'static str {
        match self {
            ScanSource::Files => "Files",
            ScanSource::Camera => "Camera",
            ScanSource::GoogleDrive => "Google Drive",
            ScanSource::ProtonDrive => "Proton Drive",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            ScanSource::Files => "📁",
            ScanSource::Camera => "📷",
            ScanSource::GoogleDrive => "🔗",
            ScanSource::ProtonDrive => "🔗",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CamScanTarget {
    Portfolio,
    AssetGroup,
    Asset,
    Organization,
    User,
    Role,
}

impl CamScanTarget {
    fn label(&self) -> &'static str {
        match self {
            CamScanTarget::Portfolio => "Portfolio",
            CamScanTarget::AssetGroup => "Asset Group",
            CamScanTarget::Asset => "Direct Asset",
            CamScanTarget::Organization => "Organization",
            CamScanTarget::User => "Individual",
            CamScanTarget::Role => "Role",
        }
    }
}

/// Camera preview placeholder with source selection buttons overlaid.
/// Replaces the old inline CamScanView on the Reporting page.
#[component]
pub fn CamScanView(app_store: leptos::prelude::RwSignal<crate::stores::AppStore>) -> impl IntoView {
    let (modal_open, set_modal_open) = signal(false);
    let (active_source, set_active_source) = signal::<Option<ScanSource>>(None);
    let (camera_active, set_camera_active) = signal(false);
    let (camera_error, set_camera_error) = signal(String::new());

    let sources = [
        ScanSource::Files,
        ScanSource::Camera,
        ScanSource::GoogleDrive,
        ScanSource::ProtonDrive,
    ];

    let open_source = move |src: ScanSource| {
        set_active_source.set(Some(src));
        if src == ScanSource::Camera {
            set_camera_error.set(String::new());
            set_camera_active.set(true);
            leptos::task::spawn_local(async move {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "hydrate")] {
                        use wasm_bindgen::JsCast;
                        use wasm_bindgen_futures::JsFuture;
                        let window = match web_sys::window() {
                            Some(w) => w,
                            None => {
                                set_camera_error.set("No window available".to_string());
                                set_camera_active.set(false);
                                return;
                            }
                        };
                        let navigator = window.navigator();
                        let media_devices = match navigator.media_devices() {
                            Ok(md) => md,
                            Err(_) => {
                                set_camera_error.set("Camera not available on this device".to_string());
                                set_camera_active.set(false);
                                return;
                            }
                        };
                        let constraints = web_sys::MediaStreamConstraints::new();
                        constraints.set_video(&wasm_bindgen::JsValue::TRUE);
                        constraints.set_audio(&wasm_bindgen::JsValue::FALSE);
                        let stream_promise = match media_devices.get_user_media_with_constraints(&constraints) {
                            Ok(p) => p,
                            Err(e) => {
                                set_camera_error.set(format!("Camera access denied: {:?}", e));
                                set_camera_active.set(false);
                                return;
                            }
                        };
                        match JsFuture::from(stream_promise).await {
                            Ok(stream_val) => {
                                let stream: web_sys::MediaStream = stream_val.unchecked_into();
                                if let Some(document) = window.document() {
                                    if let Some(video_el) = document.get_element_by_id("camscan-camera-video") {
                                        if let Ok(video) = video_el.dyn_into::<web_sys::HtmlVideoElement>() {
                                            let media: &web_sys::HtmlMediaElement = video.as_ref();
                                            let _ = media.set_src_object(Some(&stream));
                                            let _ = video.play();
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                set_camera_error.set(format!("Camera access denied: {:?}", e));
                                set_camera_active.set(false);
                            }
                        }
                    } else {
                        let _ = ();
                    }
                }
            });
        } else {
            set_camera_active.set(false);
        }
        set_modal_open.set(true);
    };

    view! {
        <div class="camscan-preview-section">
            <div class="camscan-preview-title">"CamScan"</div>
            <div class="camscan-preview-subtitle">"Scan documents with your camera or upload files"</div>

            <div class="camscan-camera-frame">
                // Camera video element (hidden until camera is activated)
                <video
                    id="camscan-camera-video"
                    class="camscan-camera-video"
                    class:camscan-camera-video-active={move || camera_active.get()}
                    autoplay
                    playsinline
                    muted
                ></video>

                // Camera outline placeholder (shown when camera not active)
                <div class="camscan-camera-placeholder" class:camscan-camera-placeholder-hidden={move || camera_active.get()}>
                    <div class="camscan-camera-outline">
                        <div class="camscan-camera-outline-icon">"📷"</div>
                        <div class="camscan-camera-outline-text">"Camera Preview"</div>
                        <div class="camscan-camera-outline-hint">"Select Camera to request access"</div>
                    </div>
                </div>

                // Source buttons overlaid on top of the preview
                <div class="camscan-source-overlay">
                    {sources.iter().map(|src| {
                        let s = *src;
                        view! {
                            <button
                                class="camscan-source-btn"
                                on:click=move |_| open_source(s)
                            >
                                <span class="camscan-source-icon">{s.icon()}</span>
                                <span class="camscan-source-label">{s.label()}</span>
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            {move || {
                let err = camera_error.get();
                if err.is_empty() { ().into_any() }
                else { view! { <div class="camscan-camera-error">{err}</div> }.into_any() }
            }}
        </div>

        // Modal
        {move || if modal_open.get() {
            let src = active_source.get();
            view! {
                <CamScanModal
                    app_store={app_store}
                    source={src}
                    camera_active={camera_active.get()}
                    on_close=Callback::new(move |_| {
                        // Stop camera if active
                        if camera_active.get() {
                            cfg_if::cfg_if! {
                                if #[cfg(feature = "hydrate")] {
                                    use wasm_bindgen::JsCast;
                                    if let Some(window) = web_sys::window() {
                                        if let Some(document) = window.document() {
                                            if let Some(video_el) = document.get_element_by_id("camscan-camera-video") {
                                                if let Ok(video) = video_el.dyn_into::<web_sys::HtmlVideoElement>() {
                                                    let media: &web_sys::HtmlMediaElement = video.as_ref();
                                                    media.set_src_object(None);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            set_camera_active.set(false);
                        }
                        set_modal_open.set(false);
                    })
                />
            }.into_any()
        } else { ().into_any() }}
    }
}

#[component]
fn CamScanModal(
    app_store: leptos::prelude::RwSignal<crate::stores::AppStore>,
    source: Option<ScanSource>,
    camera_active: bool,
    on_close: Callback<()>,
) -> impl IntoView {
    let store = app_store.get();
    let organization_store = use_organization_store();
    let (selected_target, set_selected_target) = signal(CamScanTarget::Portfolio);
    let (selected_id, set_selected_id) = signal(String::new());
    let (doc_name, set_doc_name) = signal(String::new());
    let (status_msg, set_status_msg) = signal(String::new());
    let (is_converting, set_is_converting) = signal(false);
    let (captured_images, set_captured_images) = signal::<Vec<Vec<u8>>>(Vec::new());

    let portfolios: Vec<(Uuid, String)> = store
        .portfolios
        .iter()
        .map(|p| (p.id, p.name.clone()))
        .collect();
    let asset_groups: Vec<(Uuid, String)> = store
        .portfolios
        .iter()
        .flat_map(|p| {
            p.asset_groups
                .iter()
                .map(|g| (g.id, format!("{} / {}", p.name, g.name)))
        })
        .collect();
    let assets: Vec<(Uuid, String)> = store
        .portfolios
        .iter()
        .flat_map(|p| {
            p.get_all_assets()
                .into_iter()
                .map(|a| (a.id, format!("{} / {}", p.name, a.name)))
        })
        .collect();
    let org = organization_store.get();
    let organizations: Vec<(Uuid, String)> = org
        .organizations
        .iter()
        .map(|o| (o.id, o.name.clone()))
        .collect();
    let users: Vec<(Uuid, String)> = org
        .organization_users
        .iter()
        .map(|u| (u.id, u.name.clone()))
        .collect();
    let roles: Vec<(Uuid, String)> = org
        .organizations
        .iter()
        .flat_map(|o| {
            o.roles
                .iter()
                .map(|r| (r.id, format!("{} / {}", o.name, r.name)))
        })
        .collect();

    let target_options = [
        CamScanTarget::Portfolio,
        CamScanTarget::AssetGroup,
        CamScanTarget::Asset,
        CamScanTarget::Organization,
        CamScanTarget::User,
        CamScanTarget::Role,
    ];

    let source_label = source.map(|s| s.label()).unwrap_or("Files");
    let _source_is_camera = source == Some(ScanSource::Camera);
    let source_is_cloud =
        source == Some(ScanSource::GoogleDrive) || source == Some(ScanSource::ProtonDrive);

    // Capture a frame from the live camera video element
    let on_capture_frame = move |_| {
        leptos::task::spawn_local(async move {
            cfg_if::cfg_if! {
                if #[cfg(feature = "hydrate")] {
                    use wasm_bindgen::JsCast;
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            if let Some(video_el) = document.get_element_by_id("camscan-camera-video") {
                                if let Ok(video) = video_el.dyn_into::<web_sys::HtmlVideoElement>() {
                                    let canvas = document.create_element("canvas").unwrap();
                                    let canvas: web_sys::HtmlCanvasElement = canvas.unchecked_into();
                                    let w = video.video_width();
                                    let h = video.video_height();
                                    canvas.set_width(w);
                                    canvas.set_height(h);
                                    let ctx = canvas.get_context("2d").unwrap().unwrap().unchecked_into::<web_sys::CanvasRenderingContext2d>();
                                    let _ = ctx.draw_image_with_html_video_element(&video, 0.0, 0.0);
                                    let data_url = canvas.to_data_url_with_type("image/png").unwrap_or_default();
                                    if let Some(encoded) = data_url.strip_prefix("data:image/png;base64,") {
                                        let bytes = base64::engine::general_purpose::STANDARD.decode(encoded).unwrap_or_default();
                                        set_captured_images.update(|imgs| imgs.push(bytes));
                                        set_status_msg.set(format!("Captured {} image(s)", captured_images.get().len() + 1));
                                    }
                                }
                            }
                        }
                    }
                } else {
                    let _ = ();
                }
            }
        });
    };

    let convert_handler = move |_| {
        let app_store = app_store;
        let target = selected_target.get();
        let id_str = selected_id.get();
        let name = doc_name.get();
        let set_status = set_status_msg;
        let set_converting = set_is_converting;
        let captured = captured_images.get();

        if id_str.is_empty() {
            set_status.set("Please select a target.".to_string());
            return;
        }
        if name.is_empty() {
            set_status.set("Please enter a document name.".to_string());
            return;
        }

        let target_id = match Uuid::parse_str(&id_str) {
            Ok(id) => id,
            Err(_) => {
                set_status.set("Invalid target ID.".to_string());
                return;
            }
        };

        set_converting.set(true);
        set_status.set("Processing images...".to_string());

        leptos::task::spawn_local(async move {
            // Gather images: captured camera frames or file input
            let mut all_image_bytes: Vec<Vec<u8>> = captured.clone();

            if all_image_bytes.is_empty() {
                use wasm_bindgen::JsCast;
                use web_sys::{FileList, HtmlInputElement};

                let window = match web_sys::window() {
                    Some(w) => w,
                    None => {
                        set_status.set("No window available.".to_string());
                        set_converting.set(false);
                        return;
                    }
                };
                let document = match window.document() {
                    Some(d) => d,
                    None => {
                        set_status.set("No document available.".to_string());
                        set_converting.set(false);
                        return;
                    }
                };

                let input_el = match document.get_element_by_id("camscan-file-input") {
                    Some(el) => el,
                    None => {
                        set_status.set(
                            "No images selected. Use the file input or capture from camera."
                                .to_string(),
                        );
                        set_converting.set(false);
                        return;
                    }
                };
                let input: HtmlInputElement = match input_el.dyn_into() {
                    Ok(i) => i,
                    Err(_) => {
                        set_status.set("File input cast failed.".to_string());
                        set_converting.set(false);
                        return;
                    }
                };

                let files: FileList = match input.files() {
                    Some(f) => f,
                    None => {
                        set_status.set("No files selected.".to_string());
                        set_converting.set(false);
                        return;
                    }
                };

                if files.length() == 0 {
                    set_status.set("No files selected.".to_string());
                    set_converting.set(false);
                    return;
                }

                set_status.set(format!("Reading {} image(s)...", files.length()));

                for i in 0..files.length() {
                    let file = files.get(i).unwrap();
                    let array_buffer_promise = file.array_buffer();
                    let array_buffer =
                        match wasm_bindgen_futures::JsFuture::from(array_buffer_promise).await {
                            Ok(ab) => ab,
                            Err(e) => {
                                set_status.set(format!("Failed to read file {}: {:?}", i, e));
                                set_converting.set(false);
                                return;
                            }
                        };
                    let uint8 = js_sys::Uint8Array::new(&array_buffer);
                    let mut bytes = vec![0u8; uint8.length() as usize];
                    uint8.copy_to(&mut bytes);
                    all_image_bytes.push(bytes);
                }
            }

            set_status.set(format!("Decoding {} image(s)...", all_image_bytes.len()));

            let pdf_bytes = match crate::server::convert_images_to_pdf(all_image_bytes).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    set_status.set(format!("PDF conversion failed: {}", e));
                    set_converting.set(false);
                    return;
                }
            };

            set_status.set("Saving document...".to_string());

            let uploaded_by = app_store.get().current_user.id;
            let doc = crate::models::Document {
                id: Uuid::new_v4(),
                name: name.clone(),
                file_type: "pdf".to_string(),
                content: Some(base64::engine::general_purpose::STANDARD.encode(&pdf_bytes)),
                url: format!(
                    "data:application/pdf;base64,{}",
                    base64::engine::general_purpose::STANDARD.encode(&pdf_bytes)
                ),
                uploaded_at: chrono::Utc::now(),
                uploaded_by,
            };

            let mut found = false;
            organization_store.update(|s| match target {
                CamScanTarget::Organization => {
                    if let Some(o) = s.get_organization_mut(target_id) {
                        o.documents.push(doc.clone());
                        found = true;
                    }
                }
                CamScanTarget::User => {
                    if let Some(u) = s.organization_users.iter_mut().find(|u| u.id == target_id) {
                        u.documents.push(doc.clone());
                        found = true;
                    }
                }
                CamScanTarget::Role => {
                    for o in &mut s.organizations {
                        if let Some(r) = o.roles.iter_mut().find(|r| r.id == target_id) {
                            r.documents.push(doc.clone());
                            found = true;
                            break;
                        }
                    }
                }
                _ => {}
            });
            if found {
                return;
            }
            app_store.update(|s| match target {
                CamScanTarget::Portfolio => {
                    if let Some(p) = s.get_portfolio_mut(target_id) {
                        p.documents.push(doc.clone());
                        found = true;
                    }
                }
                CamScanTarget::AssetGroup => {
                    for p in &mut s.portfolios {
                        if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == target_id) {
                            g.documents.push(doc.clone());
                            found = true;
                            break;
                        }
                    }
                }
                CamScanTarget::Asset => {
                    for p in &mut s.portfolios {
                        if let Some(a) = p.assets.iter_mut().find(|a| a.id == target_id) {
                            a.documents.push(doc.clone());
                            found = true;
                            break;
                        }
                        for g in &mut p.asset_groups {
                            if let Some(a) = g.assets.iter_mut().find(|a| a.id == target_id) {
                                a.documents.push(doc.clone());
                                found = true;
                                break;
                            }
                        }
                        if found {
                            break;
                        }
                    }
                }
                _ => {}
            });

            if found {
                set_status.set(format!("Document '{}' created successfully!", name));
                set_converting.set(false);
                set_captured_images.set(Vec::new());
            } else {
                set_status.set("Failed to find target entity.".to_string());
                set_converting.set(false);
            }
        });
    };

    view! {
        <div class="camscan-modal-overlay" on:click=move |_| on_close.run(())>
            <div class="camscan-modal" on:click=|ev| ev.stop_propagation()>
                <div class="camscan-modal-header">
                    <span class="camscan-modal-title">{"CamScan — "}{source_label}</span>
                    <button class="camscan-modal-close" on:click=move |_| on_close.run(())>"✕"</button>
                </div>

                <div class="camscan-modal-body">
                    // Camera capture section
                    {move || if camera_active {
                        view! {
                            <div class="camscan-capture-section">
                                <video
                                    id="camscan-camera-video-modal"
                                    class="camscan-camera-video-modal"
                                    autoplay
                                    playsinline
                                    muted
                                ></video>
                                <div class="camscan-capture-actions">
                                    <button class="camscan-capture-btn" on:click=on_capture_frame>"📷 Capture"
                                    </button>
                                    <span class="camscan-capture-count">
                                        {move || format!("{} captured", captured_images.get().len())}
                                    </span>
                                </div>
                            </div>
                        }.into_any()
                    } else if source_is_cloud {
                        view! {
                            <div class="camscan-cloud-section">
                                <div class="camscan-cloud-icon">"🔗"</div>
                                <div class="camscan-cloud-message">
                                    {format!("Connect to {} to import documents.", source_label)}
                                </div>
                                <button class="camscan-cloud-connect-btn">{"Connect to "}{source_label}</button>
                                <div class="camscan-cloud-hint">"Cloud integration coming soon — use Files for now."</div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="camscan-field">
                                <label>"Select Photos"</label>
                                <input
                                    id="camscan-file-input"
                                    type="file"
                                    class="camscan-file-input"
                                    accept="image/png,image/jpeg,image/jpg"
                                    multiple
                                />
                            </div>
                        }.into_any()
                    }}

                    // Captured images count
                    {move || {
                        let count = captured_images.get().len();
                        if count > 0 {
                            view! {
                                <div class="camscan-captured-info">
                                    {format!("{} image(s) ready to convert", count)}
                                </div>
                            }.into_any()
                        } else { ().into_any() }
                    }}

                    // Target form
                    <div class="camscan-field">
                        <label>"Document Name"</label>
                        <input
                            type="text"
                            class="camscan-input"
                            placeholder="e.g. Inspection Report"
                            on:input=move |ev| set_doc_name.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="camscan-field">
                        <label>"Attach To"</label>
                        <select
                            class="camscan-select"
                            on:change=move |ev| {
                                let v = event_target_value(&ev);
                                let t = match v.as_str() {
                                    "asset_group" => CamScanTarget::AssetGroup,
                                    "asset" => CamScanTarget::Asset,
                                    "organization" => CamScanTarget::Organization,
                                    "user" => CamScanTarget::User,
                                    "role" => CamScanTarget::Role,
                                    _ => CamScanTarget::Portfolio,
                                };
                                set_selected_target.set(t);
                                set_selected_id.set(String::new());
                            }
                        >
                            {target_options.iter().map(|t| {
                                let val = match t {
                                    CamScanTarget::Portfolio => "portfolio",
                                    CamScanTarget::AssetGroup => "asset_group",
                                    CamScanTarget::Asset => "asset",
                                    CamScanTarget::Organization => "organization",
                                    CamScanTarget::User => "user",
                                    CamScanTarget::Role => "role",
                                };
                                view! {
                                    <option value={val}>{t.label()}</option>
                                }
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>

                    <div class="camscan-field">
                        <label>"Target"</label>
                        <select
                            class="camscan-select"
                            on:change=move |ev| set_selected_id.set(event_target_value(&ev))
                        >
                            <option value="" disabled selected>"Select..."</option>
                            {move || {
                                let items = match selected_target.get() {
                                    CamScanTarget::Portfolio => portfolios.clone(),
                                    CamScanTarget::AssetGroup => asset_groups.clone(),
                                    CamScanTarget::Asset => assets.clone(),
                                    CamScanTarget::Organization => organizations.clone(),
                                    CamScanTarget::User => users.clone(),
                                    CamScanTarget::Role => roles.clone(),
                                };
                                items.into_iter().map(|(id, name)| {
                                    view! {
                                        <option value={id.to_string()}>{name}</option>
                                    }
                                }).collect::<Vec<_>>()
                            }}
                        </select>
                    </div>

                    <button
                        class="camscan-convert-btn"
                        disabled={move || is_converting.get()}
                        on:click=convert_handler
                    >
                        {move || if is_converting.get() { "Converting..." } else { "Convert to PDF" }}
                    </button>

                    {move || {
                        let msg = status_msg.get();
                        if msg.is_empty() {
                            view! { <div class="camscan-status"></div> }.into_any()
                        } else {
                            view! { <div class="camscan-status">{msg}</div> }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
