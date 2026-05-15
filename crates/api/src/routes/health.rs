#[utoipa::path(
    get,
    path = "/api/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = String)
    )
)]
pub async fn health() -> &'static str {
    "ok"
}
