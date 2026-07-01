use base64::Engine;
use leptos::prelude::*;
use uuid::Uuid;

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

#[component]
pub fn CamScanView(
    app_store: leptos::prelude::RwSignal<crate::stores::AppStore>,
) -> impl IntoView {
    let store = app_store.get();
    let (selected_target, set_selected_target) = signal(CamScanTarget::Portfolio);
    let (selected_id, set_selected_id) = signal(String::new());
    let (doc_name, set_doc_name) = signal(String::new());
    let (status_msg, set_status_msg) = signal(String::new());
    let (is_converting, set_is_converting) = signal(false);

    let portfolios: Vec<(Uuid, String)> =
        store.portfolios.iter().map(|p| (p.id, p.name.clone())).collect();
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
    let organizations: Vec<(Uuid, String)> =
        store.organizations.iter().map(|o| (o.id, o.name.clone())).collect();
    let users: Vec<(Uuid, String)> =
        store.organization_users.iter().map(|u| (u.id, u.name.clone())).collect();
    let roles: Vec<(Uuid, String)> = store
        .organizations
        .iter()
        .flat_map(|o| {
            o.roles.iter().map(|r| (r.id, format!("{} / {}", o.name, r.name)))
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

    let convert_handler = move |_| {
        let app_store = app_store;
        let target = selected_target.get();
        let id_str = selected_id.get();
        let name = doc_name.get();
        let set_status = set_status_msg;
        let set_converting = set_is_converting;

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
            use wasm_bindgen::JsCast;
            use web_sys::{HtmlInputElement, FileList};

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
                    set_status.set("File input not found.".to_string());
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

            let mut all_image_bytes: Vec<Vec<u8>> = Vec::new();
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

            set_status.set(format!(
                "Decoding {} image(s)...",
                all_image_bytes.len()
            ));

            let pdf_bytes = match build_pdf_from_images(&all_image_bytes, &set_status).await {
                Ok(bytes) => bytes,
                Err(_) => {
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
                url: format!("data:application/pdf;base64,{}", base64::engine::general_purpose::STANDARD.encode(&pdf_bytes)),
                uploaded_at: chrono::Utc::now(),
                uploaded_by,
            };

            let mut found = false;
            app_store.update(|s| {
                match target {
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
                            if found { break; }
                        }
                    }
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
                }
            });

            if found {
                set_status.set(format!("Document '{}' created successfully!", name));
                set_converting.set(false);
            } else {
                set_status.set("Failed to find target entity.".to_string());
                set_converting.set(false);
            }
        });
    };

    view! {
        <div class="reporting-section camscan-section">
            <div class="reporting-section-title">"CamScan"</div>
            <div class="reporting-section-meta">"Convert photos to PDF documents"</div>

            <div class="camscan-form">
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
    }
}

async fn build_pdf_from_images(
    images: &[Vec<u8>],
    set_status: &leptos::prelude::WriteSignal<String>,
) -> Result<Vec<u8>, String> {
    use printpdf::{
        image_types::RawImage, ops::Op, units::Mm, PdfDocument, PdfPage,
        PdfSaveOptions, PdfWarnMsg, XObjectId,
        xobject::XObjectTransform,
    };

    let mut pdf = PdfDocument::new("CamScan Document");
    let mut page_specs: Vec<(XObjectId, usize, usize)> = Vec::new();
    let mut pages: Vec<PdfPage> = Vec::new();

    for (idx, img_bytes) in images.iter().enumerate() {
        let mut warnings: Vec<PdfWarnMsg> = Vec::new();
        let raw_image = match RawImage::decode_from_bytes_async(img_bytes, &mut warnings).await {
            Ok(img) => img,
            Err(e) => {
                let msg = format!("Failed to decode image {}: {}", idx, e);
                set_status.set(msg.clone());
                return Err(msg);
            }
        };
        let img_w = raw_image.width;
        let img_h = raw_image.height;
        let xobj_id = pdf.add_image(&raw_image);
        page_specs.push((xobj_id, img_w, img_h));
    }

    let page_w = Mm(210.0);
    let page_h = Mm(297.0);

    for (xobj_id, img_w, img_h) in page_specs {
        let img_w_mm = Mm(img_w as f32 * 25.4 / 300.0);
        let img_h_mm = Mm(img_h as f32 * 25.4 / 300.0);
        let scale = if img_w_mm.0 > page_w.0 || img_h_mm.0 > page_h.0 {
            let sx = page_w.0 / img_w_mm.0;
            let sy = page_h.0 / img_h_mm.0;
            sx.min(sy) * 0.9
        } else {
            1.0
        };
        let final_w = img_w_mm.0 * scale;
        let final_h = img_h_mm.0 * scale;
        let offset_x = (page_w.0 - final_w) / 2.0;
        let offset_y = (page_h.0 - final_h) / 2.0;

        let transform = XObjectTransform {
            translate_x: Some(printpdf::units::Pt(offset_x * 2.83465)),
            translate_y: Some(printpdf::units::Pt(offset_y * 2.83465)),
            scale_x: Some(scale),
            scale_y: Some(scale),
            rotate: None,
            dpi: Some(300.0),
        };

        let ops = vec![Op::UseXobject { id: xobj_id, transform }];
        let pdf_page = PdfPage::new(page_w, page_h, ops);
        pages.push(pdf_page);
    }

    pdf.with_pages(pages);

    let save_opts = PdfSaveOptions::default();
    let mut warnings: Vec<PdfWarnMsg> = Vec::new();
    let pdf_bytes = pdf.save(&save_opts, &mut warnings);
    Ok(pdf_bytes)
}
