use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use qr_db::{models::*, QrRepository};
use qr_domain::{ExportFormat, QrImageGenerator, QrManager, ScanLogger};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    extractors::AuthUser,
    state::AppState,
    storage,
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
    if is_own_redirect_url(&req.target_url, &state.config.redirect_base_url) {
        return Err(AppError::Validation(
            "target URL cannot point to a scan URL of this service".to_string(),
        ));
    }

    let (qr, target_url) =
        QrManager::create_qr(&state.pool, user.id, req.name, req.target_url).await?;

    let (image_url, fmt) = upload_qr_image(&state, &qr).await?;
    let qr = QrManager::set_image_url(&state.pool, qr.id, image_url, fmt).await?;

    let response = QrCodeResponse {
        id: qr.id,
        code: qr.code,
        name: qr.name,
        image_url: qr.image_url,
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
    let (mut qr, target_url) = QrManager::get_qr_with_target(&state.pool, qr_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if qr.user_id != user.id {
        return Err(AppError::Forbidden);
    }

    if qr.image_url.is_none() {
        let (image_url, fmt) = upload_qr_image(&state, &qr).await?;
        qr = QrManager::set_image_url(&state.pool, qr.id, image_url, fmt).await?;
    }

    let scan_count = QrRepository::get_scan_count(&state.pool, qr_id).await?;

    Ok(Json(QrCodeResponse {
        id: qr.id,
        code: qr.code,
        name: qr.name,
        image_url: qr.image_url,
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

    if is_own_redirect_url(&req.url, &state.config.redirect_base_url) {
        return Err(AppError::Validation(
            "target URL cannot point to a scan URL of this service".to_string(),
        ));
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
    path = "/api/v1/qr/{id}/name",
    tag = "QR",
    params(("id" = Uuid, Path, description = "QR code id")),
    request_body = UpdateNameRequest,
    responses(
        (status = 200, description = "Name updated", body = UpdateNameResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    security(("BearerAuth" = []))
)]
pub async fn update_name(
    user: AuthUser,
    State(state): State<AppState>,
    Path(qr_id): Path<Uuid>,
    Json(req): Json<UpdateNameRequest>,
) -> AppResult<Json<UpdateNameResponse>> {
    let qr = QrRepository::get_qr_by_id(&state.pool, qr_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if qr.user_id != user.id {
        return Err(AppError::Forbidden);
    }

    let updated = QrManager::update_name(&state.pool, qr_id, req.name).await?;

    Ok(Json(UpdateNameResponse {
        id: updated.id,
        name: updated.name.unwrap_or_default(),
        updated_at: updated.updated_at,
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

    // Detailed scan logs (IP, UA, etc.) only exposed to privileged roles.
    let recent_logs = if matches!(user.role.as_str(), "admin" | "moderator") {
        QrRepository::get_scan_logs(&state.pool, qr_id, 100)
            .await?
            .into_iter()
            .map(|l| ScanLogResponse {
                ip: l.ip,
                browser: l.browser,
                device_type: l.device_type,
                timestamp: l.timestamp,
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(Json(ScanStatsResponse {
        total_scans: stats.total_scans,
        device_breakdown: stats.device_breakdown,
        browser_breakdown: stats.browser_breakdown,
        recent_logs,
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

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ExportQuery {
    /// Output format: `svg` (default), `png`, `jpeg`, `webp`
    pub format: Option<String>,
    /// Width in pixels, 150–5000 (raster formats only)
    pub width: Option<u32>,
    /// Height in pixels, 150–5000 (raster formats only)
    pub height: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/api/v1/qr/{id}/export",
    tag = "QR",
    params(
        ("id" = Uuid, Path, description = "QR code ID"),
        ExportQuery,
    ),
    responses(
        (status = 200, description = "QR code exported as image file"),
        (status = 400, description = "Invalid parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    security(("BearerAuth" = []))
)]
pub async fn export_qr(
    user: AuthUser,
    State(state): State<AppState>,
    Path(qr_id): Path<Uuid>,
    Query(params): Query<ExportQuery>,
) -> AppResult<Response> {
    let (qr, _) = QrManager::get_qr_with_target(&state.pool, qr_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if qr.user_id != user.id {
        return Err(AppError::Forbidden);
    }

    const MIN_DIM: u32 = 150;
    const MAX_DIM: u32 = 5000;

    if let Some(w) = params.width {
        if w < MIN_DIM || w > MAX_DIM {
            return Err(AppError::Validation(format!(
                "width must be {MIN_DIM}–{MAX_DIM}"
            )));
        }
    }
    if let Some(h) = params.height {
        if h < MIN_DIM || h > MAX_DIM {
            return Err(AppError::Validation(format!(
                "height must be {MIN_DIM}–{MAX_DIM}"
            )));
        }
    }

    let scan_url = format!(
        "{}/{}",
        state.config.redirect_base_url.trim_end_matches('/'),
        qr.code
    );

    let format_str = params.format.as_deref().unwrap_or("svg");

    if format_str == "svg" {
        let svg = QrImageGenerator::generate_svg(&scan_url)?;
        let filename = format!("qr-{}.svg", qr.code);
        return Ok(download_response(svg.into_bytes(), "image/svg+xml", &filename));
    }

    let export_format = ExportFormat::from_str(format_str).ok_or_else(|| {
        AppError::Validation(format!(
            "unsupported format '{format_str}'; use svg, png, jpeg, webp"
        ))
    })?;

    let svg = QrImageGenerator::generate_svg(&scan_url)?;
    let bytes = QrImageGenerator::rasterize(&svg, export_format, params.width, params.height)?;

    let filename = format!("qr-{}.{}", qr.code, export_format.extension());
    Ok(download_response(bytes, export_format.content_type(), &filename))
}

fn download_response(bytes: Vec<u8>, content_type: &str, filename: &str) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type).unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")).unwrap(),
    );
    (headers, Body::from(bytes)).into_response()
}

fn is_own_redirect_url(url: &str, redirect_base_url: &str) -> bool {
    let base = redirect_base_url.trim_end_matches('/');
    url.starts_with(&format!("{}/", base)) || url == base
}

async fn upload_qr_image(state: &AppState, qr: &QrCode) -> AppResult<(String, &'static str)> {
    let scan_url = format!(
        "{}/{}",
        state.config.redirect_base_url.trim_end_matches('/'),
        qr.code
    );
    let svg = QrImageGenerator::generate_svg(&scan_url)?;
    let key = storage::object_key(qr.id, "svg");

    storage::upload_svg(&state.s3, &state.config.storage.bucket, &key, svg)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, qr_id = %qr.id, "Failed to upload QR SVG to storage");
            AppError::Database(sqlx::Error::Configuration(
                format!("storage upload failed: {}", e).into(),
            ))
        })?;

    Ok((
        storage::public_url(&state.config.storage.public_base_url, &key),
        "svg",
    ))
}
