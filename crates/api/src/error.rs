use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use qr_domain::DomainError;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("not found")]
    NotFound,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("domain error: {0}")]
    Domain(#[from] DomainError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Domain(DomainError::QrNotFound)
            | AppError::Domain(DomainError::TargetUrlNotFound) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            _ => {
                tracing::error!(error = ?self, "Internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
