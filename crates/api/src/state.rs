use aws_sdk_s3::Client as S3Client;
use generic_auth_auth::{JwtCodec, JwtConfig};
use sqlx::PgPool;
use std::sync::Arc;

use crate::{config::AppConfig, storage};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_codec: Arc<JwtCodec>,
    pub s3: Arc<S3Client>,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub fn new(pool: PgPool, config: AppConfig) -> Self {
        let jwt_config = JwtConfig {
            secret: config.auth.jwt_secret.clone(),
            access_ttl_min: config.auth.jwt_access_ttl_min,
            refresh_ttl_days: config.auth.jwt_refresh_ttl_days,
            issuer: config.auth.jwt_issuer.clone(),
        };
        let jwt_codec = Arc::new(JwtCodec::new(jwt_config));

        let s3 = Arc::new(storage::build_client(&config.storage));

        Self {
            pool,
            jwt_codec,
            s3,
            config: Arc::new(config),
        }
    }
}
