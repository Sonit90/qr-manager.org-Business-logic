use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use uuid::Uuid;

use qr_domain::{QrManager, ScanLogger, QrImageGenerator};
use qr_api::extractors::AuthenticatedUser;
use qr_db::models::*;

#[post("/")]
pub async fn create_qr(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    req: web::Json<CreateQrRequest>,
) -> Result<HttpResponse> {
    match QrManager::create_qr(&pool, user.0.id, req.target_url.clone()).await {
        Ok((qr, target_url)) => {
            let response = QrCodeResponse {
                id: qr.id,
                code: qr.code,
                target_url: target_url.url,
                enabled: qr.enabled,
                scan_count: 0,
                created_at: qr.created_at,
                updated_at: qr.updated_at,
            };
            Ok(HttpResponse::Created().json(response))
        }
        Err(e) => {
            log::error!("Failed to create QR: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({"error": e})))
        }
    }
}

#[get("/")]
pub async fn list_user_qrs(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    match QrManager::get_user_qrs(&pool, user.0.id).await {
        Ok(qrs) => Ok(HttpResponse::Ok().json(qrs)),
        Err(e) => {
            log::error!("Failed to list QRs: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({"error": e})))
        }
    }
}

#[get("/{id}")]
pub async fn get_qr(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let qr_id = path.into_inner();

    match QrManager::get_qr_with_target(&pool, qr_id).await {
        Ok(Some((qr, target_url))) => {
            // Verify ownership
            if qr.user_id != user.0.id {
                return Ok(HttpResponse::Forbidden().finish());
            }

            let scan_count = match qr_db::QrRepository::get_scan_count(&pool, qr_id).await {
                Ok(count) => count,
                Err(e) => {
                    log::error!("Failed to get scan count: {}", e);
                    return Ok(HttpResponse::InternalServerError().finish());
                }
            };

            let response = QrCodeResponse {
                id: qr.id,
                code: qr.code,
                target_url,
                enabled: qr.enabled,
                scan_count,
                created_at: qr.created_at,
                updated_at: qr.updated_at,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            log::error!("Failed to get QR: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({"error": e})))
        }
    }
}

#[put("/{id}/target")]
pub async fn update_target_url(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateTargetUrlRequest>,
) -> Result<HttpResponse> {
    let qr_id = path.into_inner();

    // Verify ownership
    match qr_db::QrRepository::get_qr_by_id(&pool, qr_id).await {
        Ok(Some(qr)) if qr.user_id == user.0.id => {}
        Ok(Some(_)) => return Ok(HttpResponse::Forbidden().finish()),
        Ok(None) => return Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            log::error!("Failed to verify QR ownership: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    }

    match QrManager::update_target_url(&pool, qr_id, req.url.clone()).await {
        Ok(target_url) => {
            let response = serde_json::json!({
                "id": target_url.id,
                "url": target_url.url,
                "created_at": target_url.created_at
            });
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Failed to update target URL: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({"error": e})))
        }
    }
}

#[patch("/{id}/enabled")]
pub async fn toggle_qr_enabled(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: web::Json<serde_json::json::Value>,
) -> Result<HttpResponse> {
    let qr_id = path.into_inner();

    // Verify ownership
    match qr_db::QrRepository::get_qr_by_id(&pool, qr_id).await {
        Ok(Some(qr)) if qr.user_id == user.0.id => {}
        Ok(Some(_)) => return Ok(HttpResponse::Forbidden().finish()),
        Ok(None) => return Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            log::error!("Failed to verify QR ownership: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    }

    let enabled = req.get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    match QrManager::toggle_qr_enabled(&pool, qr_id, enabled).await {
        Ok(qr) => {
            let response = serde_json::json!({
                "id": qr.id,
                "enabled": qr.enabled,
                "updated_at": qr.updated_at
            });
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Failed to toggle QR: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({"error": e})))
        }
    }
}

#[get("/{id}/image")]
pub async fn get_qr_image(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let qr_id = path.into_inner();

    // Verify ownership
    match qr_db::QrRepository::get_qr_by_id(&pool, qr_id).await {
        Ok(Some(qr)) if qr.user_id == user.0.id => {}
        Ok(Some(_)) => return Ok(HttpResponse::Forbidden().finish()),
        Ok(None) => return Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            log::error!("Failed to verify QR ownership: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    }

    // Get target URL
    match qr_db::QrRepository::get_current_target_url(&pool, qr_id).await {
        Ok(Some(_)) => {
            // Generate PNG
            let redirect_url = std::env::var("REDIRECT_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3001/r".to_string());

            match QrImageGenerator::generate_png(&qr_id.to_string(), &redirect_url) {
                Ok(png_data) => Ok(HttpResponse::Ok()
                    .content_type("image/png")
                    .body(png_data)),
                Err(e) => {
                    log::error!("Failed to generate QR image: {}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
            }
        }
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            log::error!("Failed to fetch target URL: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

#[get("/{id}/scans")]
pub async fn get_scan_logs(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let qr_id = path.into_inner();

    // Verify ownership
    match qr_db::QrRepository::get_qr_by_id(&pool, qr_id).await {
        Ok(Some(qr)) if qr.user_id == user.0.id => {}
        Ok(Some(_)) => return Ok(HttpResponse::Forbidden().finish()),
        Ok(None) => return Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            log::error!("Failed to verify QR ownership: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    }

    match ScanLogger::get_scan_stats(&pool, qr_id).await {
        Ok(stats) => {
            let logs = qr_db::QrRepository::get_scan_logs(&pool, qr_id, 100)
                .await
                .unwrap_or_default();

            let response = serde_json::json!({
                "total_scans": stats.total_scans,
                "device_breakdown": stats.device_breakdown,
                "browser_breakdown": stats.browser_breakdown,
                "recent_logs": logs.iter().map(|log| ScanLogResponse {
                    ip: log.ip.clone(),
                    browser: log.browser.clone(),
                    device_type: log.device_type.clone(),
                    timestamp: log.timestamp,
                }).collect::<Vec<_>>()
            });
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Failed to get scan stats: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({"error": e})))
        }
    }
}

#[delete("/{id}")]
pub async fn delete_qr(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let qr_id = path.into_inner();

    // Verify ownership
    match qr_db::QrRepository::get_qr_by_id(&pool, qr_id).await {
        Ok(Some(qr)) if qr.user_id == user.0.id => {}
        Ok(Some(_)) => return Ok(HttpResponse::Forbidden().finish()),
        Ok(None) => return Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            log::error!("Failed to verify QR ownership: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    }

    match QrManager::delete_qr(&pool, qr_id).await {
        Ok(true) => Ok(HttpResponse::NoContent().finish()),
        Ok(false) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => {
            log::error!("Failed to delete QR: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({"error": e})))
        }
    }
}
