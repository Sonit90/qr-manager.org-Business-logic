use axum::{
    extract::{ConnectInfo, Path, State},
    http::{header, HeaderMap},
    response::Redirect,
};
use qr_db::QrRepository;
use qr_domain::{QrManager, ScanLogger};
use std::collections::HashSet;
use std::net::SocketAddr;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

const MAX_REDIRECT_HOPS: usize = 3;

#[utoipa::path(
    get,
    path = "/r/{qr_code}",
    tag = "Public",
    params(("qr_code" = String, Path, description = "QR code identifier")),
    responses(
        (status = 302, description = "Redirect to target URL or disabled/loop page"),
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
        let url = format!("{}?qr_id={}", state.config.disabled_qr_frontend_url, qr.id);
        return Ok(Redirect::to(&url));
    }

    let mut target_url = QrRepository::get_current_target_url(&state.pool, qr.id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Layer 2: follow QR-to-QR chains up to MAX_REDIRECT_HOPS deep.
    // Cycle detection via visited set; loop/exceeded hops redirect to disabled page.
    let mut visited: HashSet<Uuid> = HashSet::new();
    visited.insert(qr.id);
    let mut hops = 0;

    loop {
        let Some(next_code) = extract_qr_code(&target_url, &state.config.redirect_base_url) else {
            break;
        };

        if hops >= MAX_REDIRECT_HOPS {
            tracing::warn!(qr_id = %qr.id, "Redirect hop limit reached");
            let url = format!(
                "{}?qr_id={}&reason=loop",
                state.config.disabled_qr_frontend_url, qr.id
            );
            return Ok(Redirect::to(&url));
        }

        let next_qr = match QrManager::get_qr_by_code(&state.pool, &next_code).await? {
            Some(q) => q,
            None => break, // dead link — let browser follow and 404
        };

        if visited.contains(&next_qr.id) {
            tracing::warn!(qr_id = %qr.id, cycle_at = %next_qr.id, "Redirect cycle detected");
            let url = format!(
                "{}?qr_id={}&reason=loop",
                state.config.disabled_qr_frontend_url, qr.id
            );
            return Ok(Redirect::to(&url));
        }

        if !next_qr.enabled {
            let url = format!(
                "{}?qr_id={}",
                state.config.disabled_qr_frontend_url, next_qr.id
            );
            return Ok(Redirect::to(&url));
        }

        visited.insert(next_qr.id);
        hops += 1;

        target_url = match QrRepository::get_current_target_url(&state.pool, next_qr.id).await? {
            Some(url) => url,
            None => break,
        };
    }

    Ok(Redirect::to(&target_url))
}

/// Extracts the QR code identifier from a URL if it matches our redirect base.
/// Returns `None` for any external URL.
fn extract_qr_code(url: &str, redirect_base_url: &str) -> Option<String> {
    let base = redirect_base_url.trim_end_matches('/');
    let prefix = format!("{}/", base);
    let rest = url.strip_prefix(&prefix)?;
    // Accept only a bare code — no extra path segments or query strings
    let code = rest
        .split('/')
        .next()
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("");
    if code.is_empty() {
        None
    } else {
        Some(code.to_string())
    }
}
