use opentelemetry::metrics::{Counter, Histogram, Meter};
use std::sync::OnceLock;

static METRICS: OnceLock<OgisMetrics> = OnceLock::new();

/// Histogram buckets for duration metrics (in seconds)
/// Covers 5ms to 10s with good granularity for typical render times
const DURATION_BUCKETS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0,
];

/// OGIS-specific metrics following OTEL semantic conventions
pub struct OgisMetrics {
    // Standard HTTP metrics
    /// Request duration in seconds
    pub request_duration: Histogram<f64>,
    /// Response body size in bytes
    pub response_size: Histogram<u64>,

    // Custom business metrics
    /// Image fetch counter (logo/image, domain, cached)
    pub image_fetch: Counter<u64>,
    /// Image size in bytes
    pub image_size: Histogram<u64>,
    /// Render duration in seconds
    pub render_duration: Histogram<f64>,
    /// Render queue wait time in seconds
    pub render_queue_wait: Histogram<f64>,
}

impl OgisMetrics {
    /// Create metrics from a Meter
    pub fn new(meter: &Meter) -> Self {
        Self {
            request_duration: meter
                .f64_histogram("http.server.request.duration")
                .with_description("Duration of HTTP server requests")
                .with_unit("s")
                .with_boundaries(DURATION_BUCKETS.to_vec())
                .build(),
            response_size: meter
                .u64_histogram("http.server.response.body.size")
                .with_description("Size of HTTP response bodies")
                .with_unit("By")
                .build(),
            image_fetch: meter
                .u64_counter("ogis.image.fetch")
                .with_description("Number of image fetch operations")
                .build(),
            image_size: meter
                .u64_histogram("ogis.image.size_bytes")
                .with_description("Size of fetched images")
                .with_unit("By")
                .build(),
            render_duration: meter
                .f64_histogram("ogis.render.duration")
                .with_description("Duration of template generation and PNG rendering")
                .with_unit("s")
                .with_boundaries(DURATION_BUCKETS.to_vec())
                .build(),
            render_queue_wait: meter
                .f64_histogram("ogis.render.queue_wait")
                .with_description("Time waiting for render slot")
                .with_unit("s")
                .with_boundaries(DURATION_BUCKETS.to_vec())
                .build(),
        }
    }
}

/// Initialize global metrics (called once during setup)
pub fn init_metrics(meter: &Meter) {
    let _ = METRICS.set(OgisMetrics::new(meter));
}

/// Get the global metrics instance (returns None if OTEL is disabled)
pub fn get_metrics() -> Option<&'static OgisMetrics> {
    METRICS.get()
}
