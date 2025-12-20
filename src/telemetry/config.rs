use clap::Args;
use std::collections::HashMap;

/// Parse and validate sample ratio (must be between 0.0 and 1.0)
fn parse_sample_ratio(s: &str) -> Result<f64, String> {
    let value: f64 = s.parse().map_err(|_| format!("invalid float: {}", s))?;
    if (0.0..=1.0).contains(&value) {
        Ok(value)
    } else {
        Err(format!(
            "sample ratio must be between 0.0 and 1.0, got {}",
            value
        ))
    }
}

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
    #[arg(
        long,
        default_value = "1.0",
        env = "OGIS_OTEL_SAMPLE_RATIO",
        value_parser = parse_sample_ratio
    )]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample_ratio_valid_values() {
        assert_eq!(parse_sample_ratio("0.0").unwrap(), 0.0);
        assert_eq!(parse_sample_ratio("0.5").unwrap(), 0.5);
        assert_eq!(parse_sample_ratio("1.0").unwrap(), 1.0);
        assert_eq!(parse_sample_ratio("0.001").unwrap(), 0.001);
    }

    #[test]
    fn parse_sample_ratio_rejects_out_of_range() {
        assert!(parse_sample_ratio("-0.1").is_err());
        assert!(parse_sample_ratio("1.1").is_err());
        assert!(parse_sample_ratio("2.0").is_err());
    }

    #[test]
    fn parse_sample_ratio_rejects_invalid_input() {
        assert!(parse_sample_ratio("abc").is_err());
        assert!(parse_sample_ratio("").is_err());
    }
}
