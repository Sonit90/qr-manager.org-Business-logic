use qr_db::{QrRepository, models::*};
use sqlx::PgPool;
use uuid::Uuid;
use rand::Rng;

pub struct QrManager;

impl QrManager {
    pub async fn create_qr(
        pool: &PgPool,
        user_id: Uuid,
        target_url: String,
    ) -> Result<(qr_db::QrCode, qr_db::TargetUrl), String> {
        let code = Self::generate_unique_code(pool).await?;

        QrRepository::create_qr(pool, user_id, code, target_url)
            .await
            .map_err(|e| format!("Failed to create QR: {}", e))
    }

    pub async fn get_qr_with_target(
        pool: &PgPool,
        qr_id: Uuid,
    ) -> Result<Option<(qr_db::QrCode, String)>, String> {
        let qr = QrRepository::get_qr_by_id(pool, qr_id)
            .await
            .map_err(|e| format!("Failed to fetch QR: {}", e))?;

        match qr {
            Some(qr_code) => {
                let target_url = QrRepository::get_current_target_url(pool, qr_id)
                    .await
                    .map_err(|e| format!("Failed to fetch target URL: {}", e))?;

                match target_url {
                    Some(url) => Ok(Some((qr_code, url))),
                    None => Err("No target URL found for QR".to_string()),
                }
            }
            None => Ok(None),
        }
    }

    pub async fn get_user_qrs(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<QrCodeResponse>, String> {
        let qrs = QrRepository::get_user_qrs(pool, user_id)
            .await
            .map_err(|e| format!("Failed to fetch QRs: {}", e))?;

        let mut responses = Vec::new();
        for (qr, url) in qrs {
            let scan_count = QrRepository::get_scan_count(pool, qr.id)
                .await
                .map_err(|e| format!("Failed to fetch scan count: {}", e))?;

            responses.push(QrCodeResponse {
                id: qr.id,
                code: qr.code,
                target_url: url,
                enabled: qr.enabled,
                scan_count,
                created_at: qr.created_at,
                updated_at: qr.updated_at,
            });
        }

        Ok(responses)
    }

    pub async fn update_target_url(
        pool: &PgPool,
        qr_id: Uuid,
        new_url: String,
    ) -> Result<qr_db::TargetUrl, String> {
        // Verify QR exists
        let qr = QrRepository::get_qr_by_id(pool, qr_id)
            .await
            .map_err(|e| format!("Failed to fetch QR: {}", e))?
            .ok_or_else(|| "QR not found".to_string())?;

        // Insert new target URL
        QrRepository::update_target_url(pool, qr.id, new_url)
            .await
            .map_err(|e| format!("Failed to update target URL: {}", e))
    }

    pub async fn toggle_qr_enabled(
        pool: &PgPool,
        qr_id: Uuid,
        enabled: bool,
    ) -> Result<qr_db::QrCode, String> {
        QrRepository::toggle_qr_enabled(pool, qr_id, enabled)
            .await
            .map_err(|e| format!("Failed to toggle QR: {}", e))
    }

    pub async fn delete_qr(
        pool: &PgPool,
        qr_id: Uuid,
    ) -> Result<bool, String> {
        QrRepository::delete_qr(pool, qr_id)
            .await
            .map_err(|e| format!("Failed to delete QR: {}", e))
    }

    pub async fn get_qr_by_code(
        pool: &PgPool,
        code: &str,
    ) -> Result<Option<qr_db::QrCode>, String> {
        QrRepository::get_qr_by_code(pool, code)
            .await
            .map_err(|e| format!("Failed to fetch QR: {}", e))
    }

    async fn generate_unique_code(pool: &PgPool) -> Result<String, String> {
        let mut rng = rand::thread_rng();
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const CODE_LEN: usize = 8;

        for _ in 0..10 {
            let code: String = (0..CODE_LEN)
                .map(|_| {
                    let idx = rng.gen_range(0..CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect();

            // Check if code exists
            match QrRepository::get_qr_by_code(pool, &code).await {
                Ok(None) => return Ok(code),
                Ok(Some(_)) => continue,
                Err(e) => return Err(format!("Failed to check code uniqueness: {}", e)),
            }
        }

        Err("Failed to generate unique QR code after 10 attempts".to_string())
    }
}
