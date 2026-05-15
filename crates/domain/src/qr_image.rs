use qrcode::QrCode;
use image::Luma;

pub struct QrImageGenerator;

impl QrImageGenerator {
    pub fn generate_png(qr_code: &str, redirect_url: &str) -> Result<Vec<u8>, String> {
        // Create full redirect URL
        let full_url = format!("{}?qr={}", redirect_url, qr_code);

        // Generate QR code
        let code = QrCode::new(&full_url)
            .map_err(|e| format!("Failed to create QR code: {}", e))?;

        // Render to image
        let image = code.render::<Luma<u8>>().build();

        // Encode to PNG
        let mut png_data = Vec::new();
        let mut encoder = image::codecs::png::PngEncoder::new(&mut png_data);

        encoder
            .encode(&image.as_raw(), image.width(), image.height(), image::ColorType::L8)
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;

        Ok(png_data)
    }
}
