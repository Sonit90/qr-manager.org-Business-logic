use image::{DynamicImage, ImageEncoder, Luma, RgbaImage};
use qrcode::QrCode;
use resvg::{tiny_skia, usvg};
use std::io::Cursor;

use crate::error::{DomainError, DomainResult};

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Png,
    Jpeg,
    Webp,
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpeg" | "jpg" => Some(Self::Jpeg),
            "webp" => Some(Self::Webp),
            _ => None,
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
        }
    }
}

pub struct QrImageGenerator;

impl QrImageGenerator {
    pub fn generate_png(scan_url: &str) -> DomainResult<Vec<u8>> {
        let code = QrCode::new(scan_url)
            .map_err(|e| DomainError::ImageGeneration(format!("QR encoding failed: {}", e)))?;

        let image = code.render::<Luma<u8>>().min_dimensions(256, 256).build();

        let mut png_data = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_data);

        encoder
            .write_image(
                image.as_raw(),
                image.width(),
                image.height(),
                image::ExtendedColorType::L8,
            )
            .map_err(|e| DomainError::ImageGeneration(format!("PNG encoding failed: {}", e)))?;

        Ok(png_data)
    }

    pub fn generate_svg(scan_url: &str) -> DomainResult<String> {
        use qrcode::render::svg;

        let code = QrCode::new(scan_url)
            .map_err(|e| DomainError::ImageGeneration(format!("QR encoding failed: {e}")))?;

        let svg = code
            .render::<svg::Color>()
            .min_dimensions(256, 256)
            .quiet_zone(true)
            .build();

        Ok(svg)
    }

    pub fn rasterize(
        svg: &str,
        format: ExportFormat,
        width: Option<u32>,
        height: Option<u32>,
    ) -> DomainResult<Vec<u8>> {
        let opt = usvg::Options::default();
        let utree = usvg::Tree::from_str(svg, &opt)
            .map_err(|e| DomainError::ImageGeneration(format!("SVG parse failed: {e}")))?;

        let nw = utree.size().width().ceil() as u32;
        let nh = utree.size().height().ceil() as u32;

        let (tw, th) = compute_target(nw, nh, width, height);

        let transform = tiny_skia::Transform::from_scale(
            tw as f32 / nw as f32,
            th as f32 / nh as f32,
        );

        let mut pixmap = tiny_skia::Pixmap::new(tw, th)
            .ok_or_else(|| DomainError::ImageGeneration("pixmap alloc failed".to_string()))?;

        {
            let mut pm = pixmap.as_mut();
            resvg::render(&utree, transform, &mut pm);
        }

        match format {
            ExportFormat::Png => pixmap
                .encode_png()
                .map_err(|e| DomainError::ImageGeneration(format!("PNG encode failed: {e}"))),

            ExportFormat::Jpeg => {
                let img = RgbaImage::from_raw(tw, th, pixmap.data().to_vec())
                    .ok_or_else(|| DomainError::ImageGeneration("RGBA conversion failed".to_string()))?;
                let mut buf = Cursor::new(Vec::new());
                DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(img).to_rgb8())
                    .write_to(&mut buf, image::ImageFormat::Jpeg)
                    .map_err(|e| DomainError::ImageGeneration(format!("JPEG encode failed: {e}")))?;
                Ok(buf.into_inner())
            }

            ExportFormat::Webp => {
                let img = RgbaImage::from_raw(tw, th, pixmap.data().to_vec())
                    .ok_or_else(|| DomainError::ImageGeneration("RGBA conversion failed".to_string()))?;
                let mut buf = Cursor::new(Vec::new());
                DynamicImage::ImageRgba8(img)
                    .write_to(&mut buf, image::ImageFormat::WebP)
                    .map_err(|e| DomainError::ImageGeneration(format!("WebP encode failed: {e}")))?;
                Ok(buf.into_inner())
            }
        }
    }
}

fn compute_target(nw: u32, nh: u32, width: Option<u32>, height: Option<u32>) -> (u32, u32) {
    match (width, height) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => {
            let h = ((w as f64 * nh as f64) / nw as f64).round() as u32;
            (w, h.max(1))
        }
        (None, Some(h)) => {
            let w = ((h as f64 * nw as f64) / nh as f64).round() as u32;
            (w.max(1), h)
        }
        (None, None) => (nw, nh),
    }
}
