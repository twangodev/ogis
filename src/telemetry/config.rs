use clap::Args;
use std::collections::HashMap;

/// OpenTelemetry configuration settings
#[derive(Clone, Debug, Args)]
pub struct OtelSettings {
    /// OTLP HTTP endpoint (e.g., http://localhost:4318). Enables OTEL when set.
    #[arg(long, env = "OGIS_OTEL_ENDPOINT")]
    pub endpoint: Option<String>,

    /// Service name for telemetry
    #[arg(long, default_value = "ogis", env = "OGIS_OTEL_SERVICE_NAME")]
    pub service_name: String,

    /// Trace sampling ratio (0.0 to 1.0)
    #[arg(long, default_value = "1.0", env = "OGIS_OTEL_SAMPLE_RATIO")]
    pub sample_ratio: f64,

    /// Metrics export interval in seconds
    #[arg(long, default_value = "30", env = "OGIS_OTEL_METRICS_INTERVAL")]
    pub metrics_interval_secs: u64,

    /// Authorization header value for OTLP endpoint (e.g., "Basic <base64>")
    /// For Grafana Cloud: base64 encode "instance_id:api_token"
    #[arg(long, env = "OGIS_OTEL_AUTH")]
    pub auth: Option<String>,
}

impl OtelSettings {
    /// Returns true if OTEL is enabled (endpoint is set and non-empty)
    pub fn is_enabled(&self) -> bool {
        self.endpoint.as_ref().is_some_and(|e| !e.is_empty())
    }

    /// Build headers map for OTLP exporter
    pub fn headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        if let Some(auth) = &self.auth {
            headers.insert("Authorization".to_string(), auth.clone());
        }
        headers
    }
}
