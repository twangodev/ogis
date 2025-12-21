use opentelemetry_otlp::{MetricExporter, SpanExporter, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::{
    Resource,
    metrics::{Aggregation, Instrument, InstrumentKind, PeriodicReader, SdkMeterProvider, Stream},
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
};
use std::time::Duration;

use super::config::OtelSettings;

/// Create a blocking reqwest HTTP client for OTLP export.
/// The batch processor runs in a separate thread without Tokio runtime,
/// so we must use the blocking client.
fn create_http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failed to build OTLP HTTP client")
}

/// Create the TracerProvider for distributed tracing
pub fn create_tracer_provider(
    settings: &OtelSettings,
) -> Result<SdkTracerProvider, Box<dyn std::error::Error>> {
    let endpoint = settings
        .endpoint
        .as_ref()
        .ok_or("OTEL endpoint not configured")?;
    let headers = settings.headers();
    let client = create_http_client();

    let mut builder = SpanExporter::builder()
        .with_http()
        .with_http_client(client)
        .with_endpoint(format!("{}/v1/traces", endpoint));

    if !headers.is_empty() {
        builder = builder.with_headers(headers);
    }

    let exporter = builder.build()?;

    let sampler = if settings.sample_ratio >= 1.0 {
        Sampler::AlwaysOn
    } else if settings.sample_ratio <= 0.0 {
        Sampler::AlwaysOff
    } else {
        Sampler::TraceIdRatioBased(settings.sample_ratio)
    };

    let resource = Resource::builder()
        .with_service_name(settings.service_name.clone())
        .build();

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(sampler)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource)
        .build();

    Ok(provider)
}

/// Create the MeterProvider for metrics
pub fn create_meter_provider(
    settings: &OtelSettings,
) -> Result<SdkMeterProvider, Box<dyn std::error::Error>> {
    let endpoint = settings
        .endpoint
        .as_ref()
        .ok_or("OTEL endpoint not configured")?;
    let headers = settings.headers();
    let client = create_http_client();

    let mut builder = MetricExporter::builder()
        .with_http()
        .with_http_client(client)
        .with_endpoint(format!("{}/v1/metrics", endpoint));

    if !headers.is_empty() {
        builder = builder.with_headers(headers);
    }

    let exporter = builder.build()?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(Duration::from_secs(settings.metrics_interval_secs))
        .build();

    let resource = Resource::builder()
        .with_service_name(settings.service_name.clone())
        .build();

    let exponential_histogram_view = |inst: &Instrument| -> Option<Stream> {
        if inst.kind() == InstrumentKind::Histogram {
            Some(
                Stream::builder()
                    .with_aggregation(Aggregation::Base2ExponentialHistogram {
                        max_size: 160,
                        max_scale: 20,
                        record_min_max: true,
                    })
                    .build()
                    .ok()?,
            )
        } else {
            None
        }
    };

    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .with_view(exponential_histogram_view)
        .build();

    Ok(provider)
}
