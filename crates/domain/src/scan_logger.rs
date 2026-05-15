use qr_db::{QrRepository, ScanRepository};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::DomainResult;

pub struct ScanLogger;

#[derive(Debug, Clone)]
pub struct ScanStats {
    pub total_scans: i64,
    pub device_breakdown: HashMap<String, i64>,
    pub browser_breakdown: HashMap<String, i64>,
}

impl ScanLogger {
    pub async fn log_scan(
        pool: &PgPool,
        qr_code_id: Uuid,
        ip: Option<String>,
        user_agent_header: Option<String>,
    ) -> DomainResult<qr_db::ScanLog> {
        let (browser, device_type) = user_agent_header
            .as_ref()
            .map(|ua| Self::parse_user_agent(ua))
            .unwrap_or((None, None));

        let result = ScanRepository::log_scan(
            pool,
            qr_code_id,
            ip,
            user_agent_header,
            browser,
            device_type,
        )
        .await?;
        Ok(result)
    }

    pub async fn get_scan_stats(pool: &PgPool, qr_id: Uuid) -> DomainResult<ScanStats> {
        let total_scans = QrRepository::get_scan_count(pool, qr_id).await?;
        let logs = QrRepository::get_scan_logs(pool, qr_id, 1000).await?;

        let mut device_counts: HashMap<String, i64> = HashMap::new();
        let mut browser_counts: HashMap<String, i64> = HashMap::new();

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

    pub fn parse_user_agent(ua_string: &str) -> (Option<String>, Option<String>) {
        let browser = Self::extract_browser(ua_string);
        let device = Self::extract_device(ua_string);
        (browser, device)
    }

    fn extract_browser(ua: &str) -> Option<String> {
        if ua.contains("Edg") {
            Some("Edge".to_string())
        } else if ua.contains("OPR") || ua.contains("Opera") {
            Some("Opera".to_string())
        } else if ua.contains("Chrome") {
            Some("Chrome".to_string())
        } else if ua.contains("Firefox") {
            Some("Firefox".to_string())
        } else if ua.contains("Safari") {
            Some("Safari".to_string())
        } else {
            None
        }
    }

    fn extract_device(ua: &str) -> Option<String> {
        if ua.contains("iPad") || ua.contains("Tablet") {
            Some("Tablet".to_string())
        } else if ua.contains("Mobile") || ua.contains("Android") || ua.contains("iPhone") {
            Some("Mobile".to_string())
        } else if ua.contains("Windows") || ua.contains("Macintosh") || ua.contains("Linux") || ua.contains("X11") {
            Some("Desktop".to_string())
        } else {
            None
        }
    }
}
