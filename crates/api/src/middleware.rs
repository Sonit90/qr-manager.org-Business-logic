use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use generic_auth_auth::Claims;

use crate::{error::AppError, state::AppState};

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req.headers().get(header::AUTHORIZATION);

    let raw = match auth_header.and_then(|h| h.to_str().ok()) {
        Some(v) => v,
        None => {
            tracing::warn!("auth: missing or non-ASCII Authorization header");
            return Err(AppError::Unauthorized);
        }
    };

    let token = match raw.strip_prefix("Bearer ") {
        Some(t) => t,
        None => {
            tracing::warn!(prefix = %&raw[..raw.len().min(16)], "auth: Authorization header missing 'Bearer ' prefix");
            return Err(AppError::Unauthorized);
        }
    };

    let claims: Claims = match state.jwt_codec.verify(token) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = ?e, "auth: JWT verify failed (check JWT_SECRET / JWT_ISSUER match auth service)");
            return Err(AppError::Unauthorized);
        }
    };

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
