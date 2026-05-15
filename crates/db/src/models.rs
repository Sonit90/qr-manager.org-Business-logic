use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QrCode {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TargetUrl {
    pub id: Uuid,
    pub qr_code_id: Uuid,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScanLog {
    pub id: Uuid,
    pub qr_code_id: Uuid,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub browser: Option<String>,
    pub device_type: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// DTOs for API requests/responses
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateQrRequest {
    pub target_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTargetUrlRequest {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QrCodeResponse {
    pub id: Uuid,
    pub code: String,
    pub target_url: String,
    pub enabled: bool,
    pub scan_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanLogResponse {
    pub ip: Option<String>,
    pub browser: Option<String>,
    pub device_type: Option<String>,
    pub timestamp: DateTime<Utc>,
}
