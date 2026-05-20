use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct QrCode {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub name: Option<String>,
    pub image_url: Option<String>,
    pub image_format: String,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TargetUrl {
    pub id: Uuid,
    pub qr_code_id: Uuid,
    pub url: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ScanLog {
    pub id: Uuid,
    pub qr_code_id: Uuid,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub browser: Option<String>,
    pub device_type: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateQrRequest {
    pub target_url: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateTargetUrlRequest {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateNameRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ToggleEnabledRequest {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QrCodeResponse {
    pub id: Uuid,
    pub code: String,
    pub name: Option<String>,
    pub image_url: Option<String>,
    pub target_url: String,
    pub enabled: bool,
    pub scan_count: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ScanLogResponse {
    pub ip: Option<String>,
    pub browser: Option<String>,
    pub device_type: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ScanStatsResponse {
    pub total_scans: i64,
    pub device_breakdown: std::collections::HashMap<String, i64>,
    pub browser_breakdown: std::collections::HashMap<String, i64>,
    pub recent_logs: Vec<ScanLogResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TargetUrlResponse {
    pub id: Uuid,
    pub url: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ToggleEnabledResponse {
    pub id: Uuid,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateNameResponse {
    pub id: Uuid,
    pub name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ContactMessage {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub message: String,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub captcha_token: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateContactMessageRequest {
    #[validate(length(min = 2, max = 100, message = "name must be 2-100 characters"))]
    pub name: String,
    #[validate(email(message = "invalid email format"))]
    #[validate(length(max = 320, message = "email too long"))]
    pub email: String,
    #[validate(length(min = 10, max = 5000, message = "message must be 10-5000 characters"))]
    pub message: String,
    pub captcha_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ContactMessageResponse {
    pub id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}
