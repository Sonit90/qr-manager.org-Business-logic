use qr_db::{ScanRepository, QrRepository};
use sqlx::PgPool;
use uuid::Uuid;
use user_agent::UserAgent;

pub struct ScanLogger;

impl ScanLogger {
    pub async fn log_scan(
        pool: &PgPool,
        qr_code_id: Uuid,
        ip: Option<String>,
        user_agent_header: Option<String>,
    ) -> Result<qr_db::ScanLog, String> {
        let (browser, device_type) = user_agent_header
            .as_ref()
            .map(|ua| Self::parse_user_agent(ua))
            .unwrap_or_default();

        ScanRepository::log_scan(
            pool,
            qr_code_id,
            ip,
            user_agent_header,
            browser,
            device_type,
        )
        .await
        .map_err(|e| format!("Failed to log scan: {}", e))
    }

    pub async fn get_scan_stats(
        pool: &PgPool,
        qr_id: Uuid,
    ) -> Result<ScanStats, String> {
        let total_scans = QrRepository::get_scan_count(pool, qr_id)
            .await
            .map_err(|e| format!("Failed to fetch scan count: {}", e))?;

        let logs = QrRepository::get_scan_logs(pool, qr_id, 1000)
            .await
            .map_err(|e| format!("Failed to fetch scan logs: {}", e))?;

        let mut device_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        let mut browser_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();

        for log in logs {
            if let Some(device) = log.device_type {
                *device_counts.entry(device).or_insert(0) += 1;
            }
            if let Some(browser) = log.browser {
                *browser_counts.entry(browser).or_insert(0) += 1;
            }
        }

        Ok(ScanStats {
            total_scans,
            device_breakdown: device_counts,
            browser_breakdown: browser_counts,
        })
    }

    fn parse_user_agent(ua_string: &str) -> (Option<String>, Option<String>) {
        let ua = UserAgent::parse(ua_string);

        let browser = Self::extract_browser(&ua, ua_string);
        let device = Self::extract_device(&ua, ua_string);

        (browser, device)
    }

    fn extract_browser(ua: &UserAgent, ua_string: &str) -> Option<String> {
        if ua_string.contains("Chrome") {
            Some("Chrome".to_string())
        } else if ua_string.contains("Firefox") {
            Some("Firefox".to_string())
        } else if ua_string.contains("Safari") && !ua_string.contains("Chrome") {
            Some("Safari".to_string())
        } else if ua_string.contains("Edge") || ua_string.contains("Edg") {
            Some("Edge".to_string())
        } else if ua_string.contains("Opera") {
            Some("Opera".to_string())
        } else {
            None
        }
    }

    fn extract_device(ua: &UserAgent, ua_string: &str) -> Option<String> {
        if ua_string.contains("Mobile") || ua_string.contains("Android") || ua_string.contains("iPhone") || ua_string.contains("iPad") {
            if ua_string.contains("iPad") {
                Some("Tablet".to_string())
            } else {
                Some("Mobile".to_string())
            }
        } else if ua_string.contains("Windows") || ua_string.contains("Macintosh") || ua_string.contains("Linux") {
            Some("Desktop".to_string())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanStats {
    pub total_scans: i64,
    pub device_breakdown: std::collections::HashMap<String, i32>,
    pub browser_breakdown: std::collections::HashMap<String, i32>,
}
