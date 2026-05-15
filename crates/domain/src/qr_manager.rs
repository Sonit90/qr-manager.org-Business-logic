use qr_db::{models::*, QrRepository};
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{DomainError, DomainResult};

pub struct QrManager;

impl QrManager {
    pub async fn create_qr(
        pool: &PgPool,
        user_id: Uuid,
        target_url: String,
    ) -> DomainResult<(QrCode, TargetUrl)> {
        let code = Self::generate_unique_code(pool).await?;
        let result = QrRepository::create_qr(pool, user_id, code, target_url).await?;
        Ok(result)
    }

    pub async fn get_qr_with_target(
        pool: &PgPool,
        qr_id: Uuid,
    ) -> DomainResult<Option<(QrCode, String)>> {
        let qr = QrRepository::get_qr_by_id(pool, qr_id).await?;

        match qr {
            Some(qr_code) => {
                let target_url = QrRepository::get_current_target_url(pool, qr_id)
                    .await?
                    .ok_or(DomainError::TargetUrlNotFound)?;

                Ok(Some((qr_code, target_url)))
            }
            None => Ok(None),
        }
    }

    pub async fn get_user_qrs(
        pool: &PgPool,
        user_id: Uuid,
    ) -> DomainResult<Vec<QrCodeResponse>> {
        let rows = QrRepository::get_user_qrs_with_urls(pool, user_id).await?;

        Ok(rows
            .into_iter()
            .map(|row| QrCodeResponse {
                id: row.id,
                code: row.code,
                target_url: row.target_url.unwrap_or_default(),
                enabled: row.enabled,
                scan_count: row.scan_count.unwrap_or(0),
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect())
    }

    pub async fn update_target_url(
        pool: &PgPool,
        qr_id: Uuid,
        new_url: String,
    ) -> DomainResult<TargetUrl> {
        let qr = QrRepository::get_qr_by_id(pool, qr_id)
            .await?
            .ok_or(DomainError::QrNotFound)?;

        let target_url = QrRepository::update_target_url(pool, qr.id, new_url).await?;
        Ok(target_url)
    }

    pub async fn toggle_qr_enabled(
        pool: &PgPool,
        qr_id: Uuid,
        enabled: bool,
    ) -> DomainResult<QrCode> {
        let result = QrRepository::toggle_qr_enabled(pool, qr_id, enabled).await?;
        Ok(result)
    }

    pub async fn delete_qr(pool: &PgPool, qr_id: Uuid) -> DomainResult<bool> {
        let result = QrRepository::delete_qr(pool, qr_id).await?;
        Ok(result)
    }

    pub async fn get_qr_by_code(pool: &PgPool, code: &str) -> DomainResult<Option<QrCode>> {
        let result = QrRepository::get_qr_by_code(pool, code).await?;
        Ok(result)
    }

    async fn generate_unique_code(pool: &PgPool) -> DomainResult<String> {
        const CHARSET: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const CODE_LEN: usize = 8;

        for _ in 0..10 {
            let code: String = {
                let mut rng = rand::thread_rng();
                (0..CODE_LEN)
                    .map(|_| {
                        let idx = rng.gen_range(0..CHARSET.len());
                        CHARSET[idx] as char
                    })
                    .collect()
            };

            if QrRepository::get_qr_by_code(pool, &code).await?.is_none() {
                return Ok(code);
            }
        }

        Err(DomainError::CodeGenerationFailed)
    }
}
