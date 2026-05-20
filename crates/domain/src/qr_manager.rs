use qr_db::{models::*, QrRepository};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{DomainError, DomainResult};

pub struct QrManager;

impl QrManager {
    pub async fn create_qr(
        pool: &PgPool,
        user_id: Uuid,
        name: Option<String>,
        target_url: String,
    ) -> DomainResult<(QrCode, TargetUrl)> {
        let code = Uuid::new_v4().to_string();
        let result = QrRepository::create_qr(pool, user_id, code, name, target_url).await?;
        Ok(result)
    }

    pub async fn set_image_url(
        pool: &PgPool,
        qr_id: Uuid,
        image_url: String,
        image_format: &str,
    ) -> DomainResult<QrCode> {
        let qr = QrRepository::set_image_url(pool, qr_id, image_url, image_format).await?;
        Ok(qr)
    }

    pub async fn update_name(
        pool: &PgPool,
        qr_id: Uuid,
        name: String,
    ) -> DomainResult<QrCode> {
        let qr = QrRepository::update_name(pool, qr_id, name).await?;
        Ok(qr)
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
                name: row.name,
                image_url: row.image_url,
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
}
