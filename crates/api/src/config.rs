use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub app_env: String,
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub auth: AuthConfig,
    pub cors: CorsConfig,
    pub disabled_qr_frontend_url: String,
    pub redirect_base_url: String,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_issuer: String,
    pub jwt_access_ttl_min: i64,
    pub jwt_refresh_ttl_days: i64,
}

#[derive(Debug, Clone, Default)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .context("invalid SERVER_PORT")?,
            database_url: env::var("DATABASE_URL").context("DATABASE_URL not set")?,
            auth: AuthConfig {
                jwt_secret: env::var("APP__AUTH__JWT_SECRET")
                    .context("APP__AUTH__JWT_SECRET not set")?,
                jwt_issuer: env::var("APP__AUTH__JWT_ISSUER")
                    .unwrap_or_else(|_| "generic-auth".to_string()),
                jwt_access_ttl_min: env::var("APP__AUTH__JWT_ACCESS_TTL_MIN")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .context("invalid APP__AUTH__JWT_ACCESS_TTL_MIN")?,
                jwt_refresh_ttl_days: env::var("APP__AUTH__JWT_REFRESH_TTL_DAYS")
                    .unwrap_or_else(|_| "14".to_string())
                    .parse()
                    .context("invalid APP__AUTH__JWT_REFRESH_TTL_DAYS")?,
            },
            cors: CorsConfig {
                allowed_origins: env::var("APP__CORS__ALLOWED_ORIGINS")
                    .unwrap_or_default()
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
            },
            disabled_qr_frontend_url: env::var("DISABLED_QR_FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000/qr-disabled".to_string()),
            redirect_base_url: env::var("REDIRECT_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3001/r".to_string()),
        })
    }

    pub fn is_development(&self) -> bool {
        self.app_env == "development"
    }
}
