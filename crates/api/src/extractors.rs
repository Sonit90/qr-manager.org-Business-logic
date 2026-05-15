use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use generic_auth_auth::Claims;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub role: String,
}

impl From<&Claims> for AuthUser {
    fn from(claims: &Claims) -> Self {
        AuthUser {
            id: claims.sub,
            role: claims.role.clone(),
        }
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .map(AuthUser::from)
            .ok_or(AppError::Unauthorized)
    }
}
