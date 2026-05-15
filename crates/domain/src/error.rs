use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("QR not found")]
    QrNotFound,

    #[error("Target URL not found")]
    TargetUrlNotFound,

    #[error("Failed to generate unique QR code")]
    CodeGenerationFailed,

    #[error("Image generation failed: {0}")]
    ImageGeneration(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
