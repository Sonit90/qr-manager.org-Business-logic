use image::{ImageEncoder, Luma};
use qrcode::QrCode;

use crate::error::{DomainError, DomainResult};

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
}
