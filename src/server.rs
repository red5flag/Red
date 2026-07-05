use crate::models::Portfolio;
use leptos::prelude::*;

#[server(SavePortfolio, "/api")]
pub async fn save_portfolio(portfolio: Portfolio) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::storage::portfolio_store;
        let store = portfolio_store();
        store
            .save_portfolio(&portfolio)
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    }
    Ok(())
}

#[server(ConvertImagesToPdf, "/api")]
pub async fn convert_images_to_pdf(images: Vec<Vec<u8>>) -> Result<Vec<u8>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use printpdf::{
            image_types::RawImage, ops::Op, units::Mm, xobject::XObjectTransform, PdfDocument,
            PdfPage, PdfSaveOptions, PdfWarnMsg, XObjectId,
        };

        let mut pdf = PdfDocument::new("CamScan Document");
        let mut page_specs: Vec<(XObjectId, usize, usize)> = Vec::new();
        let mut pages: Vec<PdfPage> = Vec::new();

        for (idx, img_bytes) in images.iter().enumerate() {
            let mut warnings: Vec<PdfWarnMsg> = Vec::new();
            let raw_image = match RawImage::decode_from_bytes_async(img_bytes, &mut warnings).await
            {
                Ok(img) => img,
                Err(e) => {
                    return Err(ServerFnError::new(format!(
                        "Failed to decode image {}: {}",
                        idx, e
                    )));
                }
            };
            let xobj_id = pdf.add_image(&raw_image);
            page_specs.push((xobj_id, raw_image.width, raw_image.height));
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

            let ops = vec![Op::UseXobject {
                id: xobj_id,
                transform,
            }];
            let pdf_page = PdfPage::new(page_w, page_h, ops);
            pages.push(pdf_page);
        }

        pdf.with_pages(pages);

        let save_opts = PdfSaveOptions::default();
        let mut warnings: Vec<PdfWarnMsg> = Vec::new();
        let pdf_bytes = pdf.save(&save_opts, &mut warnings);
        Ok(pdf_bytes)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(
            "PDF conversion is only available on the server",
        ))
    }
}
