use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{ContactMessage, QrCode, ScanLog, TargetUrl};

pub struct QrRepository;

impl QrRepository {
    pub async fn create_qr(
        pool: &PgPool,
        user_id: Uuid,
        code: String,
        name: Option<String>,
        url: String,
    ) -> Result<(QrCode, TargetUrl), sqlx::Error> {
        let mut tx = pool.begin().await?;

        let qr = sqlx::query_as::<_, QrCode>(
            "INSERT INTO qr.qr_codes (user_id, code, name) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(user_id)
        .bind(&code)
        .bind(name)
        .fetch_one(&mut *tx)
        .await?;

        let target_url = sqlx::query_as::<_, TargetUrl>(
            "INSERT INTO qr.target_urls (qr_code_id, url) VALUES ($1, $2) RETURNING *",
        )
        .bind(qr.id)
        .bind(url)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok((qr, target_url))
    }

    pub async fn set_image_url(
        pool: &PgPool,
        qr_id: Uuid,
        image_url: String,
    ) -> Result<QrCode, sqlx::Error> {
        sqlx::query_as::<_, QrCode>(
            "UPDATE qr.qr_codes SET image_url = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        )
        .bind(image_url)
        .bind(qr_id)
        .fetch_one(pool)
        .await
    }

    pub async fn update_name(
        pool: &PgPool,
        qr_id: Uuid,
        name: String,
    ) -> Result<QrCode, sqlx::Error> {
        sqlx::query_as::<_, QrCode>(
            "UPDATE qr.qr_codes SET name = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        )
        .bind(name)
        .bind(qr_id)
        .fetch_one(pool)
        .await
    }

    pub async fn get_qr_by_code(pool: &PgPool, code: &str) -> Result<Option<QrCode>, sqlx::Error> {
        sqlx::query_as::<_, QrCode>("SELECT * FROM qr.qr_codes WHERE code = $1")
            .bind(code)
            .fetch_optional(pool)
            .await
    }

    pub async fn get_qr_by_id(pool: &PgPool, id: Uuid) -> Result<Option<QrCode>, sqlx::Error> {
        sqlx::query_as::<_, QrCode>("SELECT * FROM qr.qr_codes WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn get_current_target_url(
        pool: &PgPool,
        qr_code_id: Uuid,
    ) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar::<_, String>(
            "SELECT url FROM qr.target_urls WHERE qr_code_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(qr_code_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn get_user_qrs_with_urls(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<QrWithTargetRow>, sqlx::Error> {
        sqlx::query_as::<_, QrWithTargetRow>(
            r#"
            SELECT
                q.id as id,
                q.user_id as user_id,
                q.code as code,
                q.name as name,
                q.image_url as image_url,
                q.enabled as enabled,
                q.created_at as created_at,
                q.updated_at as updated_at,
                (
                    SELECT url FROM qr.target_urls
                    WHERE qr_code_id = q.id
                    ORDER BY created_at DESC
                    LIMIT 1
                ) as target_url,
                (
                    SELECT COUNT(*) FROM qr.scan_logs WHERE qr_code_id = q.id
                ) as scan_count
            FROM qr.qr_codes q
            WHERE q.user_id = $1
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
        let mut tx = pool.begin().await?;

        let target_url = sqlx::query_as::<_, TargetUrl>(
            "INSERT INTO qr.target_urls (qr_code_id, url) VALUES ($1, $2) RETURNING *",
        )
        .bind(qr_code_id)
        .bind(url)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query("UPDATE qr.qr_codes SET updated_at = NOW() WHERE id = $1")
            .bind(qr_code_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(target_url)
    }

    pub async fn toggle_qr_enabled(
        pool: &PgPool,
        qr_id: Uuid,
        enabled: bool,
    ) -> Result<QrCode, sqlx::Error> {
        sqlx::query_as::<_, QrCode>(
            "UPDATE qr.qr_codes SET enabled = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        )
        .bind(enabled)
        .bind(qr_id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete_qr(pool: &PgPool, qr_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM qr.qr_codes WHERE id = $1")
            .bind(qr_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_scan_count(pool: &PgPool, qr_code_id: Uuid) -> Result<i64, sqlx::Error> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM qr.scan_logs WHERE qr_code_id = $1")
                .bind(qr_code_id)
                .fetch_one(pool)
                .await?;

        Ok(count.0)
    }

    pub async fn get_scan_logs(
        pool: &PgPool,
        qr_code_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ScanLog>, sqlx::Error> {
        sqlx::query_as::<_, ScanLog>(
            "SELECT * FROM qr.scan_logs WHERE qr_code_id = $1 ORDER BY timestamp DESC LIMIT $2",
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
            INSERT INTO qr.scan_logs (qr_code_id, ip, user_agent, browser, device_type)
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

pub struct ContactRepository;

impl ContactRepository {
    pub async fn create(
        pool: &PgPool,
        name: String,
        email: String,
        message: String,
        ip: Option<String>,
        user_agent: Option<String>,
        captcha_token: Option<String>,
    ) -> Result<ContactMessage, sqlx::Error> {
        sqlx::query_as::<_, ContactMessage>(
            r#"
            INSERT INTO qr.contact_messages
                (name, email, message, ip, user_agent, captcha_token)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(email)
        .bind(message)
        .bind(ip)
        .bind(user_agent)
        .bind(captcha_token)
        .fetch_one(pool)
        .await
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct QrWithTargetRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub name: Option<String>,
    pub image_url: Option<String>,
    pub enabled: bool,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    pub target_url: Option<String>,
    pub scan_count: Option<i64>,
}
