use actix_web::{web, App, HttpServer, middleware::Logger};
use dotenv::dotenv;
use std::env;
use sqlx::postgres::PgPoolOptions;
use qr_api::middleware::AuthMiddleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "3001".to_string());
    let bind_addr = format!("{}:{}", host, port);

    log::info!("Starting server on {}", bind_addr);

    let pool_data = web::Data::new(pool);

    HttpServer::new(move || {
        let mut cfg = web::ServiceConfig::default();
        qr_api::routes::configure_public(&mut cfg);

        App::new()
            .wrap(Logger::default())
            .app_data(pool_data.clone())
            // Public routes (no auth)
            .configure(qr_api::routes::configure_public)
            // Protected routes (require auth)
            .service(
                web::scope("/api/v1")
                    .wrap(AuthMiddleware)
                    .configure(qr_api::routes::configure)
            )
    })
        .bind(&bind_addr)?
        .run()
        .await
}
