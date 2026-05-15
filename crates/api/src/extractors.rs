use actix_web::{FromRequest, HttpRequest};
use futures::future::{ok, Ready};
use generic_auth_auth::Claims;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub role: String,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        AuthUser {
            id: claims.sub,
            role: claims.role,
        }
    }
}

pub struct AuthenticatedUser(pub AuthUser);

impl FromRequest for AuthenticatedUser {
    type Error = actix_web::error::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest) -> Self::Future {
        if let Some(claims) = req.extensions().get::<Claims>() {
            let user = AuthUser::from(claims.clone());
            ok(AuthenticatedUser(user))
        } else {
            Err(actix_web::error::ErrorUnauthorized("Missing or invalid token"))
                .into()
        }
    }
}
