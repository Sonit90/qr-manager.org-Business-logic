pub mod health;
pub mod qr;
pub mod redirect;

use axum::{
    middleware as axum_mw,
    routing::{delete, get, patch, post, put},
    Router,
};

use crate::{middleware::require_auth, state::AppState};

pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/qr", post(qr::create_qr))
        .route("/qr", get(qr::list_user_qrs))
        .route("/qr/{id}", get(qr::get_qr))
        .route("/qr/{id}", delete(qr::delete_qr))
        .route("/qr/{id}/target", put(qr::update_target_url))
        .route("/qr/{id}/name", patch(qr::update_name))
        .route("/qr/{id}/enabled", patch(qr::toggle_qr_enabled))
        .route("/qr/{id}/scans", get(qr::get_scan_logs))
        .route_layer(axum_mw::from_fn_with_state(state.clone(), require_auth));

    let public = Router::new()
        .route("/api/health", get(health::health))
        .route("/r/{qr_code}", get(redirect::redirect_qr));

    Router::new()
        .nest("/api/v1", protected)
        .merge(public)
        .with_state(state)
}
