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
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims: Claims = state
        .jwt_codec
        .verify(token)
        .map_err(|_| AppError::Unauthorized)?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
