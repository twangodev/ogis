mod config;
mod metrics;
mod provider;

pub use config::OtelSettings;
pub use metrics::get_metrics;

use opentelemetry::{metrics::MeterProvider, trace::TracerProvider};
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SdkTracerProvider};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Guard that ensures OTEL providers are properly shut down
#[derive(Default)]
pub struct TelemetryGuard {
    tracer_provider: Option<SdkTracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(tp) = self.tracer_provider.take()
            && let Err(e) = tp.shutdown()
        {
            eprintln!("Error shutting down tracer provider: {:?}", e);
        }
        if let Some(mp) = self.meter_provider.take()
            && let Err(e) = mp.shutdown()
        {
            eprintln!("Error shutting down meter provider: {:?}", e);
        }
    }
}

/// Initialize telemetry. Returns a guard that must be held for the program lifetime.
pub fn init(settings: &OtelSettings) -> Result<TelemetryGuard, Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if settings.is_enabled() {
        // Create OTEL providers
        let tracer_provider = provider::create_tracer_provider(settings)?;
        let meter_provider = provider::create_meter_provider(settings)?;

        // Initialize metrics
        let meter = meter_provider.meter("ogis");
        metrics::init_metrics(&meter);

        // Create the tracing-opentelemetry layer
        let tracer = tracer_provider.tracer("ogis");
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        // Set up the subscriber with both fmt and otel layers
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .with(otel_layer)
            .init();

        tracing::info!(
            endpoint = %settings.endpoint.as_ref().expect("endpoint must be set when OTEL is enabled"),
            service = %settings.service_name,
            "OpenTelemetry enabled"
        );

        Ok(TelemetryGuard {
            tracer_provider: Some(tracer_provider),
            meter_provider: Some(meter_provider),
        })
    } else {
        // No OTEL - just console logging (current behavior)
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .init();

        Ok(TelemetryGuard::default())
    }
}
