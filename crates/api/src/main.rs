use actix_web::{web, App, HttpServer, middleware::Logger};
use dotenv::dotenv;
use std::env;
use qr_api::middleware::AuthMiddleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "3001".to_string());
    let bind_addr = format!("{}:{}", host, port);

    log::info!("Starting server on {}", bind_addr);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            // Public routes (no auth)
            // TODO: Add GET /r/{qr_code} redirect endpoint

            // Protected routes (require auth)
            .service(
                web::scope("/api/v1")
                    .wrap(AuthMiddleware)
                    // TODO: Add QR management endpoints
            )
    })
        .bind(&bind_addr)?
        .run()
        .await
}
