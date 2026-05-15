use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct QrCode {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String,
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
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateTargetUrlRequest {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ToggleEnabledRequest {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QrCodeResponse {
    pub id: Uuid,
    pub code: String,
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
