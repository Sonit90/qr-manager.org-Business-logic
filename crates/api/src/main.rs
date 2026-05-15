use anyhow::{Context, Result};
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use qr_api::{config::AppConfig, openapi::ApiDoc, routes::build_router, state::AppState};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;

    init_tracing(&config);

    tracing::info!(env = %config.app_env, "Starting qr-api");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .context("Failed to connect to database")?;

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .context("Failed to run migrations")?;

    let state = AppState::new(pool, config.clone());

    let app: Router = build_router(state.clone())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port)
        .parse()
        .context("Invalid server address")?;

    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!(%addr, "Listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

fn init_tracing(config: &AppConfig) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,qr_api=debug,sqlx=warn"));

    let registry = tracing_subscriber::registry().with(filter);

    if config.is_development() {
        registry.with(fmt::layer().pretty()).init();
    } else {
        registry.with(fmt::layer().json()).init();
    }
}
