use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use qr_db::{models::*, QrRepository};
use qr_domain::{QrImageGenerator, QrManager, ScanLogger};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    extractors::AuthUser,
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/v1/qr",
    tag = "QR",
    request_body = CreateQrRequest,
    responses(
        (status = 201, description = "QR code created", body = QrCodeResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("BearerAuth" = []))
)]
pub async fn create_qr(
    user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateQrRequest>,
) -> AppResult<(StatusCode, Json<QrCodeResponse>)> {
    let (qr, target_url) = QrManager::create_qr(&state.pool, user.id, req.target_url).await?;

    let response = QrCodeResponse {
        id: qr.id,
        code: qr.code,
        target_url: target_url.url,
        enabled: qr.enabled,
        scan_count: 0,
        created_at: qr.created_at,
        updated_at: qr.updated_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/qr",
    tag = "QR",
    responses(
        (status = 200, description = "List user's QR codes", body = [QrCodeResponse]),
        (status = 401, description = "Unauthorized")
    ),
    security(("BearerAuth" = []))
)]
pub async fn list_user_qrs(
    user: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<QrCodeResponse>>> {
    let qrs = QrManager::get_user_qrs(&state.pool, user.id).await?;
    Ok(Json(qrs))
}

#[utoipa::path(
    get,
    path = "/api/v1/qr/{id}",
    tag = "QR",
    params(("id" = Uuid, Path, description = "QR code id")),
    responses(
        (status = 200, description = "QR code details", body = QrCodeResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    security(("BearerAuth" = []))
)]
pub async fn get_qr(
    user: AuthUser,
    State(state): State<AppState>,
    Path(qr_id): Path<Uuid>,
) -> AppResult<Json<QrCodeResponse>> {
    let (qr, target_url) = QrManager::get_qr_with_target(&state.pool, qr_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if qr.user_id != user.id {
        return Err(AppError::Forbidden);
    }

    let scan_count = QrRepository::get_scan_count(&state.pool, qr_id).await?;

    Ok(Json(QrCodeResponse {
        id: qr.id,
        code: qr.code,
        target_url,
        enabled: qr.enabled,
        scan_count,
        created_at: qr.created_at,
        updated_at: qr.updated_at,
    }))
}

#[utoipa::path(
    put,
    path = "/api/v1/qr/{id}/target",
    tag = "QR",
    params(("id" = Uuid, Path, description = "QR code id")),
    request_body = UpdateTargetUrlRequest,
    responses(
        (status = 200, description = "Target URL updated", body = TargetUrlResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    security(("BearerAuth" = []))
)]
pub async fn update_target_url(
    user: AuthUser,
    State(state): State<AppState>,
    Path(qr_id): Path<Uuid>,
    Json(req): Json<UpdateTargetUrlRequest>,
) -> AppResult<Json<TargetUrlResponse>> {
    let qr = QrRepository::get_qr_by_id(&state.pool, qr_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if qr.user_id != user.id {
        return Err(AppError::Forbidden);
    }

    let target_url = QrManager::update_target_url(&state.pool, qr_id, req.url).await?;

    Ok(Json(TargetUrlResponse {
        id: target_url.id,
        url: target_url.url,
        created_at: target_url.created_at,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/qr/{id}/enabled",
    tag = "QR",
    params(("id" = Uuid, Path, description = "QR code id")),
    request_body = ToggleEnabledRequest,
    responses(
        (status = 200, description = "QR enabled state updated", body = ToggleEnabledResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    security(("BearerAuth" = []))
)]
pub async fn toggle_qr_enabled(
    user: AuthUser,
    State(state): State<AppState>,
    Path(qr_id): Path<Uuid>,
    Json(req): Json<ToggleEnabledRequest>,
) -> AppResult<Json<ToggleEnabledResponse>> {
    let qr = QrRepository::get_qr_by_id(&state.pool, qr_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if qr.user_id != user.id {
        return Err(AppError::Forbidden);
    }

    let updated = QrManager::toggle_qr_enabled(&state.pool, qr_id, req.enabled).await?;

    Ok(Json(ToggleEnabledResponse {
        id: updated.id,
        enabled: updated.enabled,
        updated_at: updated.updated_at,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/qr/{id}/image",
    tag = "QR",
    params(("id" = Uuid, Path, description = "QR code id")),
    responses(
        (status = 200, description = "PNG image", content_type = "image/png"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    security(("BearerAuth" = []))
)]
pub async fn get_qr_image(
    user: AuthUser,
    State(state): State<AppState>,
    Path(qr_id): Path<Uuid>,
) -> AppResult<Response> {
    let qr = QrRepository::get_qr_by_id(&state.pool, qr_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if qr.user_id != user.id {
        return Err(AppError::Forbidden);
    }

    let scan_url = format!("{}/{}", state.config.redirect_base_url.trim_end_matches('/'), qr.code);
    let png = QrImageGenerator::generate_png(&scan_url)?;

    Ok(([(header::CONTENT_TYPE, "image/png")], png).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/qr/{id}/scans",
    tag = "Analytics",
    params(("id" = Uuid, Path, description = "QR code id")),
    responses(
        (status = 200, description = "Scan statistics", body = ScanStatsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    security(("BearerAuth" = []))
)]
pub async fn get_scan_logs(
    user: AuthUser,
    State(state): State<AppState>,
    Path(qr_id): Path<Uuid>,
) -> AppResult<Json<ScanStatsResponse>> {
    let qr = QrRepository::get_qr_by_id(&state.pool, qr_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if qr.user_id != user.id {
        return Err(AppError::Forbidden);
    }

    let stats = ScanLogger::get_scan_stats(&state.pool, qr_id).await?;
    let logs = QrRepository::get_scan_logs(&state.pool, qr_id, 100).await?;

    Ok(Json(ScanStatsResponse {
        total_scans: stats.total_scans,
        device_breakdown: stats.device_breakdown,
        browser_breakdown: stats.browser_breakdown,
        recent_logs: logs
            .into_iter()
            .map(|l| ScanLogResponse {
                ip: l.ip,
                browser: l.browser,
                device_type: l.device_type,
                timestamp: l.timestamp,
            })
            .collect(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/qr/{id}",
    tag = "QR",
    params(("id" = Uuid, Path, description = "QR code id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    security(("BearerAuth" = []))
)]
pub async fn delete_qr(
    user: AuthUser,
    State(state): State<AppState>,
    Path(qr_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let qr = QrRepository::get_qr_by_id(&state.pool, qr_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if qr.user_id != user.id {
        return Err(AppError::Forbidden);
    }

    let deleted = QrManager::delete_qr(&state.pool, qr_id).await?;
    if !deleted {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
