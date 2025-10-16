/// Health check endpoint
///
/// Returns a simple "ok" response to indicate the service is running
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = str, content_type = "text/plain")
    ),
    tag = "monitoring"
)]
pub async fn health_check() -> &'static str {
    tracing::debug!("healthy");
    "ok"
}
