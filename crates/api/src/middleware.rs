use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::LocalBoxFuture;
use generic_auth_auth::{JwtCodec, JwtConfig, Claims};
use std::env;
use std::sync::OnceLock;

static JWT_CODEC: OnceLock<JwtCodec> = OnceLock::new();

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());

        if let Some(token) = auth_header.and_then(|h| h.strip_prefix("Bearer ")) {
            let token = token.to_string();
            let fut = self.service.call(req);

            Box::pin(async move {
                if let Ok(claims) = validate_token(&token) {
                    let req = match fut.await {
                        Ok(mut res) => {
                            res.request_mut().extensions_mut().insert(claims);
                            res
                        }
                        Err(e) => return Err(e),
                    };
                    Ok(req)
                } else {
                    Err(actix_web::error::ErrorUnauthorized("Invalid token"))
                }
            })
        } else {
            let fut = self.service.call(req);
            Box::pin(async move { fut.await })
        }
    }
}

pub fn validate_token(token: &str) -> Result<Claims, String> {
    let jwt_secret = env::var("JWT_SECRET").map_err(|_| "JWT_SECRET not set".to_string())?;
    let issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "qr-link".to_string());

    let codec = JWT_CODEC.get_or_init(|| {
        JwtCodec::new(JwtConfig {
            secret: jwt_secret.clone(),
            access_ttl_min: 30,
            refresh_ttl_days: 14,
            issuer: issuer.clone(),
        })
    });

    codec.verify(token).map_err(|e| format!("Token validation failed: {}", e))
}
