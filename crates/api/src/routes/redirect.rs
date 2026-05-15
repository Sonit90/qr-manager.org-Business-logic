use axum::{
    extract::{ConnectInfo, Path, State},
    http::{header, HeaderMap},
    response::Redirect,
};
use qr_db::QrRepository;
use qr_domain::{QrManager, ScanLogger};
use std::net::SocketAddr;

use crate::{error::AppError, state::AppState};

#[utoipa::path(
    get,
    path = "/r/{qr_code}",
    tag = "Public",
    params(("qr_code" = String, Path, description = "QR code identifier")),
    responses(
        (status = 302, description = "Redirect to target URL or disabled page"),
        (status = 404, description = "QR code not found")
    )
)]
pub async fn redirect_qr(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(qr_code): Path<String>,
) -> Result<Redirect, AppError> {
    let qr = QrManager::get_qr_by_code(&state.pool, &qr_code)
        .await?
        .ok_or(AppError::NotFound)?;

    let ip = Some(addr.ip().to_string());
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    if let Err(e) = ScanLogger::log_scan(&state.pool, qr.id, ip, user_agent).await {
        tracing::warn!(error = ?e, qr_id = %qr.id, "Failed to log scan");
    }

    if !qr.enabled {
        let url = format!(
            "{}?qr_id={}",
            state.config.disabled_qr_frontend_url, qr.id
        );
        return Ok(Redirect::to(&url));
    }

    let target_url = QrRepository::get_current_target_url(&state.pool, qr.id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Redirect::to(&target_url))
}
