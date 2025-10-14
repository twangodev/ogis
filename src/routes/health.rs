pub async fn handler() -> &'static str {
    tracing::debug!("health check endpoint called");
    "ok"
}
