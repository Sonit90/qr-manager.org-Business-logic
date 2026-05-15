use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{QrCode, ScanLog, TargetUrl};

pub struct QrRepository;

impl QrRepository {
    pub async fn create_qr(
        pool: &PgPool,
        user_id: Uuid,
        code: String,
        url: String,
    ) -> Result<(QrCode, TargetUrl), sqlx::Error> {
        let mut tx = pool.begin().await?;

        let qr = sqlx::query_as::<_, QrCode>(
            "INSERT INTO qr_codes (user_id, code) VALUES ($1, $2) RETURNING *",
        )
        .bind(user_id)
        .bind(&code)
        .fetch_one(&mut *tx)
        .await?;

        let target_url = sqlx::query_as::<_, TargetUrl>(
            "INSERT INTO target_urls (qr_code_id, url) VALUES ($1, $2) RETURNING *",
        )
        .bind(qr.id)
        .bind(url)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok((qr, target_url))
    }

    pub async fn get_qr_by_code(pool: &PgPool, code: &str) -> Result<Option<QrCode>, sqlx::Error> {
        sqlx::query_as::<_, QrCode>("SELECT * FROM qr_codes WHERE code = $1")
            .bind(code)
            .fetch_optional(pool)
            .await
    }

    pub async fn get_qr_by_id(pool: &PgPool, id: Uuid) -> Result<Option<QrCode>, sqlx::Error> {
        sqlx::query_as::<_, QrCode>("SELECT * FROM qr_codes WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn get_current_target_url(pool: &PgPool, qr_code_id: Uuid) -> Result<Option<String>, sqlx::Error> {
        let url = sqlx::query_scalar::<_, String>(
            "SELECT url FROM target_urls WHERE qr_code_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(qr_code_id)
        .fetch_optional(pool)
        .await?;

        Ok(url)
    }

    pub async fn get_user_qrs(pool: &PgPool, user_id: Uuid) -> Result<Vec<(QrCode, String)>, sqlx::Error> {
        sqlx::query_as::<_, (QrCode, String)>(
            r#"
            SELECT q.*, t.url
            FROM qr_codes q
            LEFT JOIN target_urls t ON q.id = t.qr_code_id
            WHERE q.user_id = $1
            AND t.created_at = (
                SELECT MAX(created_at) FROM target_urls WHERE qr_code_id = q.id
            )
            ORDER BY q.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn update_target_url(
        pool: &PgPool,
        qr_code_id: Uuid,
        url: String,
    ) -> Result<TargetUrl, sqlx::Error> {
        sqlx::query_as::<_, TargetUrl>(
            "INSERT INTO target_urls (qr_code_id, url) VALUES ($1, $2) RETURNING *",
        )
        .bind(qr_code_id)
        .bind(url)
        .fetch_one(pool)
        .await
    }

    pub async fn toggle_qr_enabled(pool: &PgPool, qr_id: Uuid, enabled: bool) -> Result<QrCode, sqlx::Error> {
        sqlx::query_as::<_, QrCode>(
            "UPDATE qr_codes SET enabled = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 RETURNING *",
        )
        .bind(enabled)
        .bind(qr_id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete_qr(pool: &PgPool, qr_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM qr_codes WHERE id = $1")
            .bind(qr_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_scan_count(pool: &PgPool, qr_code_id: Uuid) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM scan_logs WHERE qr_code_id = $1")
            .bind(qr_code_id)
            .fetch_one(pool)
            .await?;

        Ok(count.0)
    }

    pub async fn get_scan_logs(pool: &PgPool, qr_code_id: Uuid, limit: i64) -> Result<Vec<ScanLog>, sqlx::Error> {
        sqlx::query_as::<_, ScanLog>(
            "SELECT * FROM scan_logs WHERE qr_code_id = $1 ORDER BY timestamp DESC LIMIT $2",
        )
        .bind(qr_code_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}

pub struct ScanRepository;

impl ScanRepository {
    pub async fn log_scan(
        pool: &PgPool,
        qr_code_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
        browser: Option<String>,
        device_type: Option<String>,
    ) -> Result<ScanLog, sqlx::Error> {
        sqlx::query_as::<_, ScanLog>(
            r#"
            INSERT INTO scan_logs (qr_code_id, ip, user_agent, browser, device_type)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(qr_code_id)
        .bind(ip)
        .bind(user_agent)
        .bind(browser)
        .bind(device_type)
        .fetch_one(pool)
        .await
    }
}
