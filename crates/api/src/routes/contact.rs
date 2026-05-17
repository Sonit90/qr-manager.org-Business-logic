use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, StatusCode},
    Json,
};
use qr_db::{models::*, ContactRepository};
use std::net::SocketAddr;
use validator::Validate;

use crate::{
    error::{AppError, AppResult},
    state::AppState,
};

// Placeholder. Future: verify token against captcha provider (hCaptcha / Turnstile / reCAPTCHA).
fn verify_captcha(_token: Option<&str>) -> Result<(), AppError> {
    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/v1/contact",
    tag = "Contact",
    request_body = CreateContactMessageRequest,
    responses(
        (status = 201, description = "Message accepted", body = ContactMessageResponse),
        (status = 400, description = "Validation error")
    )
)]
pub async fn submit_contact(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<CreateContactMessageRequest>,
) -> AppResult<(StatusCode, Json<ContactMessageResponse>)> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    verify_captcha(req.captcha_token.as_deref())?;

    let ip = Some(addr.ip().to_string());
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let row = ContactRepository::create(
        &state.pool,
        req.name,
        req.email,
        req.message,
        ip,
        user_agent,
        req.captcha_token,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(ContactMessageResponse {
            id: row.id,
            created_at: row.created_at,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(name: &str, email: &str, message: &str) -> CreateContactMessageRequest {
        CreateContactMessageRequest {
            name: name.into(),
            email: email.into(),
            message: message.into(),
            captcha_token: None,
        }
    }

    #[test]
    fn valid_payload_passes() {
        let req = payload(
            "John Doe",
            "john@example.com",
            "Hello, I would like to know more about your product.",
        );
        assert!(req.validate().is_ok());
    }

    #[test]
    fn name_too_short_fails() {
        let req = payload("J", "john@example.com", "Hello there friend.");
        assert!(req.validate().is_err());
    }

    #[test]
    fn name_too_long_fails() {
        let req = payload(
            &"a".repeat(101),
            "john@example.com",
            "Hello there friend.",
        );
        assert!(req.validate().is_err());
    }

    #[test]
    fn invalid_email_fails() {
        let req = payload("John", "not-an-email", "Hello there friend.");
        assert!(req.validate().is_err());
    }

    #[test]
    fn message_too_short_fails() {
        let req = payload("John", "john@example.com", "short");
        assert!(req.validate().is_err());
    }

    #[test]
    fn message_too_long_fails() {
        let req = payload("John", "john@example.com", &"x".repeat(5001));
        assert!(req.validate().is_err());
    }

    #[test]
    fn message_at_boundaries_passes() {
        let req_min = payload("John", "john@example.com", &"x".repeat(10));
        assert!(req_min.validate().is_ok());
        let req_max = payload("John", "john@example.com", &"x".repeat(5000));
        assert!(req_max.validate().is_ok());
    }

    #[test]
    fn captcha_placeholder_accepts_none() {
        assert!(verify_captcha(None).is_ok());
    }

    #[test]
    fn captcha_placeholder_accepts_some() {
        assert!(verify_captcha(Some("any-token")).is_ok());
    }
}
