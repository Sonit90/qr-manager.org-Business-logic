pub mod qr;
pub mod redirect;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/qr")
            .route("", web::post().to(qr::create_qr))
            .route("", web::get().to(qr::list_user_qrs))
            .route("/{id}", web::get().to(qr::get_qr))
            .route("/{id}/target", web::put().to(qr::update_target_url))
            .route("/{id}/enabled", web::patch().to(qr::toggle_qr_enabled))
            .route("/{id}/image", web::get().to(qr::get_qr_image))
            .route("/{id}/scans", web::get().to(qr::get_scan_logs))
            .route("/{id}", web::delete().to(qr::delete_qr))
    );
}

pub fn configure_public(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/r")
            .route("/{qr_code}", web::get().to(redirect::redirect_qr))
    );
}
