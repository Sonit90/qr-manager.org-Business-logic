use actix_web::{web, HttpRequest, HttpResponse, Result};
use sqlx::PgPool;

use qr_domain::{QrManager, ScanLogger};

#[get("/{qr_code}")]
pub async fn redirect_qr(
    qr_code: web::Path<String>,
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let code = qr_code.into_inner();

    // Fetch QR and target URL
    match QrManager::get_qr_by_code(&pool, &code).await {
        Ok(Some(qr)) => {
            // Log scan
            let ip = req.connection_info().peer_addr().map(|s| s.to_string());
            let user_agent = req
                .headers()
                .get("User-Agent")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());

            if let Err(e) = ScanLogger::log_scan(&pool, qr.id, ip, user_agent).await {
                log::warn!("Failed to log scan: {}", e);
            }

            // Check if enabled
            if !qr.enabled {
                let disabled_url = std::env::var("DISABLED_QR_FRONTEND_URL")
                    .unwrap_or_else(|_| "http://localhost:3000/qr-disabled".to_string());
                let redirect_with_param = format!("{}?qr_id={}", disabled_url, qr.id);
                return Ok(HttpResponse::Found()
                    .insert_header(("Location", redirect_with_param))
                    .finish());
            }

            // Get target URL
            match qr_db::QrRepository::get_current_target_url(&pool, qr.id).await {
                Ok(Some(target_url)) => {
                    Ok(HttpResponse::Found()
                        .insert_header(("Location", target_url))
                        .finish())
                }
                Ok(None) => {
                    log::warn!("No target URL found for QR: {}", qr.id);
                    Ok(HttpResponse::NotFound().finish())
                }
                Err(e) => {
                    log::error!("Failed to fetch target URL: {}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
            }
        }
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            log::error!("Failed to fetch QR: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
