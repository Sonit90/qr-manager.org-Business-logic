use qr_db::models::*;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::routes::{health, qr, redirect};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "QR Link API",
        description = "Dynamic QR code management",
        version = "0.1.0"
    ),
    paths(
        health::health,
        qr::create_qr,
        qr::list_user_qrs,
        qr::get_qr,
        qr::update_target_url,
        qr::toggle_qr_enabled,
        qr::get_qr_image,
        qr::get_scan_logs,
        qr::delete_qr,
        redirect::redirect_qr,
    ),
    components(schemas(
        CreateQrRequest,
        UpdateTargetUrlRequest,
        ToggleEnabledRequest,
        QrCodeResponse,
        ScanLogResponse,
        ScanStatsResponse,
        TargetUrlResponse,
        ToggleEnabledResponse,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "QR", description = "QR code management"),
        (name = "Analytics", description = "Scan statistics"),
        (name = "Public", description = "Public scan endpoint"),
        (name = "Health", description = "Service health check")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "BearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}
