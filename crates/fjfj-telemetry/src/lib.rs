//! Telemetry: `tracing` for in-process spans/logs, exported over OTLP.
//!
//! Design principle: prefer OpenTelemetry over bespoke profiling/BEP-style
//! streams. Bazel's Build Event Protocol (BEP) is treated as a *compatibility
//! export* derived from the trace, not the primary data model.

use anyhow::Result;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialise the global tracing subscriber. OTLP export is enabled when
/// `OTEL_EXPORTER_OTLP_ENDPOINT` is set (standard OTel env var); otherwise
/// only the fmt layer is installed.
pub fn init() -> Result<TelemetryGuard> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_writer(std::io::stderr);
    let registry = tracing_subscriber::registry().with(filter).with(fmt);

    if std::env::var_os("OTEL_EXPORTER_OTLP_ENDPOINT").is_some() {
        use opentelemetry::trace::TracerProvider as _;
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()?;
        let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .build();
        let tracer = provider.tracer("fjfj");
        registry
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .init();
        Ok(TelemetryGuard {
            provider: Some(provider),
        })
    } else {
        registry.init();
        Ok(TelemetryGuard { provider: None })
    }
}

/// Flushes pending spans on drop.
pub struct TelemetryGuard {
    provider: Option<opentelemetry_sdk::trace::SdkTracerProvider>,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(p) = self.provider.take() {
            let _ = p.shutdown();
        }
    }
}
