//! Telemetry: `tracing` for in-process spans/logs, exported over OTLP.
//!
//! Design principle: prefer OpenTelemetry over bespoke profiling/BEP-style
//! streams. Bazel's Build Event Protocol (BEP) is treated as a *compatibility
//! export* derived from the trace, not the primary data model.

pub mod metrics;
pub mod trace_file;

use anyhow::Result;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// The global OTel meter fjfj records all build metrics through (see
/// [`metrics::BuildMetrics`]). Usable whether or not [`init`] set up an
/// OTLP meter provider: with none registered, `opentelemetry::global`
/// falls back to a no-op meter, so recording is always safe, just not
/// exported anywhere until `OTEL_EXPORTER_OTLP_ENDPOINT` is set.
pub fn meter() -> opentelemetry::metrics::Meter {
    opentelemetry::global::meter("fjfj")
}

/// Initialise the global tracing subscriber and, when OTLP export is on,
/// the global OTel meter provider. OTLP export is enabled when
/// `OTEL_EXPORTER_OTLP_ENDPOINT` is set (standard OTel env var); otherwise
/// only the fmt layer is installed and [`meter`] returns OTel's no-op meter.
pub fn init() -> Result<TelemetryGuard> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // FJFJ_SPAN_TIMES=1 prints how long each span took when it closes, for
    // finding where a run spends its time.
    let span_events = if std::env::var_os("FJFJ_SPAN_TIMES").is_some() {
        tracing_subscriber::fmt::format::FmtSpan::CLOSE
    } else {
        tracing_subscriber::fmt::format::FmtSpan::NONE
    };
    let fmt = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_span_events(span_events)
        .with_writer(std::io::stderr);
    // FJFJ_TRACE_FILE=path writes the spans as a Chrome trace (see
    // [`trace_file`]); `fjfj analyze-profile` reads it.
    let trace_file = match std::env::var_os("FJFJ_TRACE_FILE") {
        Some(path) => Some(trace_file::TraceFile::create(std::path::Path::new(&path))?),
        None => None,
    };
    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(fmt)
        .with(trace_file.clone().map(trace_file::TraceLayer));

    if std::env::var_os("OTEL_EXPORTER_OTLP_ENDPOINT").is_some() {
        use opentelemetry::trace::TracerProvider as _;
        let span_exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()?;
        let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(span_exporter)
            .build();
        let tracer = tracer_provider.tracer("fjfj");
        registry
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .init();

        let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .build()?;
        let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .with_periodic_exporter(metric_exporter)
            .build();
        opentelemetry::global::set_meter_provider(meter_provider.clone());

        Ok(TelemetryGuard {
            tracer_provider: Some(tracer_provider),
            meter_provider: Some(meter_provider),
            trace_file,
        })
    } else {
        registry.init();
        Ok(TelemetryGuard {
            tracer_provider: None,
            meter_provider: None,
            trace_file,
        })
    }
}

/// Flushes pending spans and metrics on drop.
pub struct TelemetryGuard {
    tracer_provider: Option<opentelemetry_sdk::trace::SdkTracerProvider>,
    meter_provider: Option<opentelemetry_sdk::metrics::SdkMeterProvider>,
    trace_file: Option<std::sync::Arc<trace_file::TraceFile>>,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(p) = self.tracer_provider.take() {
            let _ = p.shutdown();
        }
        if let Some(p) = self.meter_provider.take() {
            let _ = p.shutdown();
        }
        if let Some(file) = self.trace_file.take() {
            file.finish();
        }
    }
}
