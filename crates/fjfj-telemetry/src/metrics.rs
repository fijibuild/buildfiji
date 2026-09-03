//! Build metrics: the counters/histograms/gauges described in
//! `docs/design/telemetry.md` ("action counts by strategy and cache status,
//! critical path, CAS bytes up/down, worker pool utilisation"), recorded
//! through `opentelemetry::metrics` rather than a bespoke stats struct —
//! same "OpenTelemetry is the primary observability model" principle as
//! `tracing` spans, so a metrics backend gets these for free from whatever
//! already collects fjfj's OTLP export.
//!
//! [`BuildMetrics`] only wraps instrument creation and naming; it has no
//! opinion on when to call it. Nothing calls it yet — there is no execution
//! engine producing actions, cache lookups or CAS transfers to measure — so
//! this is where that caller's bookkeeping will attach once one exists.

use opentelemetry::KeyValue;
use opentelemetry::metrics::{Counter, Gauge, Histogram, Meter};

/// How an action was run, for the `strategy` attribute on the action
/// counter. Matches Bazel's own strategy names where one applies (`local`,
/// `worker`, `remote`) so dashboards built against Bazel's metrics still
/// make sense pointed at fjfj.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    Local,
    Worker,
    Remote,
}

impl Strategy {
    fn as_str(self) -> &'static str {
        match self {
            Strategy::Local => "local",
            Strategy::Worker => "worker",
            Strategy::Remote => "remote",
        }
    }
}

/// Whether an action's result came from a cache, for the `cache_status`
/// attribute on the action counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStatus {
    Miss,
    DiskCacheHit,
    RemoteCacheHit,
}

impl CacheStatus {
    fn as_str(self) -> &'static str {
        match self {
            CacheStatus::Miss => "miss",
            CacheStatus::DiskCacheHit => "disk_cache_hit",
            CacheStatus::RemoteCacheHit => "remote_cache_hit",
        }
    }
}

/// Direction of a CAS blob transfer, for [`BuildMetrics::record_cas_bytes`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CasDirection {
    Upload,
    Download,
}

/// The build-wide metric instruments, created once per [`Meter`] (typically
/// [`crate::meter`]'s global one) and cheap to clone — every instrument type
/// here is itself a cheap `Arc`-backed handle.
#[derive(Clone)]
pub struct BuildMetrics {
    action_count: Counter<u64>,
    critical_path_seconds: Histogram<f64>,
    cas_bytes: Counter<u64>,
    worker_utilization: Gauge<f64>,
}

impl BuildMetrics {
    pub fn new(meter: &Meter) -> Self {
        BuildMetrics {
            action_count: meter
                .u64_counter("fjfj.action.count")
                .with_description("Number of actions executed, by strategy and cache status.")
                .with_unit("{action}")
                .build(),
            critical_path_seconds: meter
                .f64_histogram("fjfj.build.critical_path")
                .with_description("Wall-clock length of the build's critical path.")
                .with_unit("s")
                .build(),
            cas_bytes: meter
                .u64_counter("fjfj.remote.cas.bytes")
                .with_description("Bytes transferred to/from the CAS, by direction.")
                .with_unit("By")
                .build(),
            worker_utilization: meter
                .f64_gauge("fjfj.worker.utilization")
                .with_description("Fraction of the local worker pool currently busy.")
                .with_unit("1")
                .build(),
        }
    }

    /// One action finished, run with `strategy` and resolved as `cache_status`.
    pub fn record_action(&self, strategy: Strategy, cache_status: CacheStatus) {
        self.action_count.add(
            1,
            &[
                KeyValue::new("strategy", strategy.as_str()),
                KeyValue::new("cache_status", cache_status.as_str()),
            ],
        );
    }

    /// The build's critical path length, in seconds. Recorded once, at the
    /// end of the build, once the whole action graph's timing is known.
    pub fn record_critical_path(&self, seconds: f64) {
        self.critical_path_seconds.record(seconds, &[]);
    }

    /// `bytes` transferred to (`Upload`) or from (`Download`) the CAS.
    pub fn record_cas_bytes(&self, direction: CasDirection, bytes: u64) {
        let label = match direction {
            CasDirection::Upload => "upload",
            CasDirection::Download => "download",
        };
        self.cas_bytes
            .add(bytes, &[KeyValue::new("direction", label)]);
    }

    /// `busy` out of `total` local workers currently running an action.
    /// `total == 0` records `0.0` rather than dividing by zero (no worker
    /// pool configured is not the same claim as "the pool is idle", but
    /// there is no meaningful utilisation fraction to report either way).
    pub fn record_worker_utilization(&self, busy: usize, total: usize) {
        let fraction = if total == 0 {
            0.0
        } else {
            busy as f64 / total as f64
        };
        self.worker_utilization.record(fraction, &[]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::metrics::MeterProvider as _;
    use opentelemetry_sdk::error::OTelSdkResult;
    use opentelemetry_sdk::metrics::SdkMeterProvider;
    use opentelemetry_sdk::metrics::data::ResourceMetrics;
    use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData};
    use opentelemetry_sdk::metrics::reader::MetricReader;
    use opentelemetry_sdk::metrics::{InstrumentKind, ManualReader, Pipeline, Temporality};
    use std::sync::{Arc, Weak};
    use std::time::Duration;

    /// `MetricReader` isn't implemented for `Arc<ManualReader>`, but a test
    /// needs to both hand a reader to `SdkMeterProvider` (which takes it by
    /// value) and keep a handle to call `collect` on afterwards — this just
    /// delegates every method to a shared `ManualReader`.
    #[derive(Debug, Clone)]
    struct SharedReader(Arc<ManualReader>);

    impl MetricReader for SharedReader {
        fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
            self.0.register_pipeline(pipeline)
        }
        fn collect(&self, rm: &mut ResourceMetrics) -> OTelSdkResult {
            self.0.collect(rm)
        }
        fn force_flush(&self) -> OTelSdkResult {
            self.0.force_flush()
        }
        fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
            self.0.shutdown_with_timeout(timeout)
        }
        fn temporality(&self, kind: InstrumentKind) -> Temporality {
            self.0.temporality(kind)
        }
    }

    fn u64_metric<'a>(rm: &'a ResourceMetrics, name: &str) -> &'a MetricData<u64> {
        let AggregatedMetrics::U64(data) = rm
            .scope_metrics()
            .flat_map(|sm| sm.metrics())
            .find(|m| m.name() == name)
            .unwrap_or_else(|| panic!("no metric named {name}"))
            .data()
        else {
            panic!("{name} is not a u64 metric");
        };
        data
    }

    fn f64_metric<'a>(rm: &'a ResourceMetrics, name: &str) -> &'a MetricData<f64> {
        let AggregatedMetrics::F64(data) = rm
            .scope_metrics()
            .flat_map(|sm| sm.metrics())
            .find(|m| m.name() == name)
            .unwrap_or_else(|| panic!("no metric named {name}"))
            .data()
        else {
            panic!("{name} is not an f64 metric");
        };
        data
    }

    /// A `BuildMetrics` wired to an in-process reader instead of an OTLP
    /// exporter, so a test can inspect exactly what was recorded without a
    /// collector to talk to. The provider must be kept alive for as long as
    /// the reader is used: dropping it tears down the pipeline the reader's
    /// `collect` depends on.
    fn collectable() -> (BuildMetrics, SharedReader, SdkMeterProvider) {
        let reader = SharedReader(Arc::new(ManualReader::builder().build()));
        let provider = SdkMeterProvider::builder()
            .with_reader(reader.clone())
            .build();
        let meter = provider.meter("fjfj_test");
        (BuildMetrics::new(&meter), reader, provider)
    }

    fn collect(reader: &SharedReader) -> ResourceMetrics {
        let mut rm = ResourceMetrics::default();
        reader.collect(&mut rm).unwrap();
        rm
    }

    #[test]
    fn action_count_carries_strategy_and_cache_status() {
        let (metrics, reader, _provider) = collectable();
        metrics.record_action(Strategy::Remote, CacheStatus::RemoteCacheHit);
        metrics.record_action(Strategy::Local, CacheStatus::Miss);
        let rm = collect(&reader);
        let MetricData::Sum(sum) = u64_metric(&rm, "fjfj.action.count") else {
            panic!("expected a Sum");
        };
        assert_eq!(sum.data_points().count(), 2);
        let remote_hit = sum
            .data_points()
            .find(|dp| {
                dp.attributes()
                    .any(|kv| kv.value.as_str().as_ref() == "remote_cache_hit")
            })
            .unwrap();
        assert_eq!(remote_hit.value(), 1);
    }

    #[test]
    fn critical_path_is_recorded_as_a_histogram() {
        let (metrics, reader, _provider) = collectable();
        metrics.record_critical_path(12.5);
        let rm = collect(&reader);
        let MetricData::Histogram(hist) = f64_metric(&rm, "fjfj.build.critical_path") else {
            panic!("expected a Histogram");
        };
        let dp = hist.data_points().next().unwrap();
        assert_eq!(dp.count(), 1);
        assert_eq!(dp.sum(), 12.5);
    }

    #[test]
    fn cas_bytes_split_by_direction() {
        let (metrics, reader, _provider) = collectable();
        metrics.record_cas_bytes(CasDirection::Upload, 100);
        metrics.record_cas_bytes(CasDirection::Download, 250);
        let rm = collect(&reader);
        let MetricData::Sum(sum) = u64_metric(&rm, "fjfj.remote.cas.bytes") else {
            panic!("expected a Sum");
        };
        let total: u64 = sum.data_points().map(|dp| dp.value()).sum();
        assert_eq!(total, 350);
    }

    #[test]
    fn worker_utilization_is_a_fraction() {
        let (metrics, reader, _provider) = collectable();
        metrics.record_worker_utilization(3, 4);
        let rm = collect(&reader);
        let MetricData::Gauge(gauge) = f64_metric(&rm, "fjfj.worker.utilization") else {
            panic!("expected a Gauge");
        };
        assert_eq!(gauge.data_points().next().unwrap().value(), 0.75);
    }

    #[test]
    fn zero_total_workers_records_zero_not_nan() {
        let (metrics, reader, _provider) = collectable();
        metrics.record_worker_utilization(0, 0);
        let rm = collect(&reader);
        let MetricData::Gauge(gauge) = f64_metric(&rm, "fjfj.worker.utilization") else {
            panic!("expected a Gauge");
        };
        assert_eq!(gauge.data_points().next().unwrap().value(), 0.0);
    }
}
