pub mod error;
pub mod qr_image;
pub mod qr_manager;
pub mod scan_logger;

#[cfg(test)]
mod tests;

pub use error::{DomainError, DomainResult};
pub use qr_image::{ExportFormat, QrImageGenerator};
pub use qr_manager::QrManager;
pub use scan_logger::{ScanLogger, ScanStats};
