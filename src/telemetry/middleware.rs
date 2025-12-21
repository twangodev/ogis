use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use http_body_util::BodyExt;
use opentelemetry::KeyValue;
use std::time::Instant;

use super::get_metrics;

/// Middleware that records HTTP request metrics for all responses
pub async fn metrics_middleware(request: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    let response = next.run(request).await;
    let status = response.status().as_u16();

    // Collect body to measure size
    let (parts, body) = response.into_parts();
    let bytes = body
        .collect()
        .await
        .map(|collected| collected.to_bytes())
        .unwrap_or_default();
    let response_size = bytes.len();

    // Record metrics
    if let Some(m) = get_metrics() {
        let duration = start.elapsed().as_secs_f64();
        let attrs = [
            KeyValue::new("http.request.method", method.to_string()),
            KeyValue::new("http.response.status_code", status as i64),
            KeyValue::new("http.route", path),
        ];
        m.request_duration.record(duration, &attrs);
        m.response_size.record(response_size as u64, &attrs);
    }

    Response::from_parts(parts, Body::from(bytes))
}
