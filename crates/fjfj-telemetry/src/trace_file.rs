//! A Chrome trace file of the build's spans (buildfiji-k62.6), for looking at
//! a run without a collector: <chrome://tracing>, Perfetto, or
//! `fjfj analyze-profile`.
//!
//! Each closed span is one complete event on its own line. The events of an
//! action (the `action` span and the `step` spans under it) share a `tid`, so
//! a viewer draws each action as a row. A `step` span is named by its `step`
//! field. The file is a JSON array; the closing bracket is written when the
//! guard drops, and viewers accept a file without it after a crash.

use serde_json::{Map, Value, json};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::Subscriber;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

/// The open file; shared by the layer and the guard that ends it.
pub struct TraceFile {
    out: Mutex<Option<std::io::BufWriter<std::fs::File>>>,
    epoch: Instant,
}

impl TraceFile {
    pub fn create(path: &std::path::Path) -> std::io::Result<Arc<TraceFile>> {
        let mut out = std::io::BufWriter::new(std::fs::File::create(path)?);
        out.write_all(b"[\n")?;
        Ok(Arc::new(TraceFile {
            out: Mutex::new(Some(out)),
            epoch: Instant::now(),
        }))
    }

    /// Ends the array and closes the file.
    pub fn finish(&self) {
        if let Some(mut out) = self.out.lock().unwrap().take() {
            let _ = out.write_all(b"{}\n]\n");
            let _ = out.flush();
        }
    }

    /// Writes what is buffered, so a reader sees the events so far.
    pub fn flush(&self) {
        if let Some(out) = self.out.lock().unwrap().as_mut() {
            let _ = out.flush();
        }
    }

    fn write(&self, event: &Value) {
        if let Some(out) = self.out.lock().unwrap().as_mut() {
            let _ = writeln!(out, "{event},");
        }
    }
}

/// What a span knows until it closes.
struct Open {
    start: Instant,
    args: Map<String, Value>,
    row: u64,
}

pub struct TraceLayer(pub Arc<TraceFile>);

#[derive(Default)]
struct Fields(Map<String, Value>);

impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_owned(), json!(format!("{value:?}")));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_owned(), json!(value));
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_owned(), json!(value));
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_owned(), json!(value));
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().to_owned(), json!(value));
    }
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for TraceLayer {
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else { return };
        let mut fields = Fields::default();
        attrs.record(&mut fields);
        // The outermost span is the row.
        let row = span
            .scope()
            .last()
            .map_or(id.into_u64(), |root| root.id().into_u64());
        span.extensions_mut().insert(Open {
            start: Instant::now(),
            args: fields.0,
            row,
        });
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id)
            && let Some(open) = span.extensions_mut().get_mut::<Open>()
        {
            let mut fields = Fields(std::mem::take(&mut open.args));
            values.record(&mut fields);
            open.args = fields.0;
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(&id) else { return };
        let Some(open) = span.extensions_mut().remove::<Open>() else {
            return;
        };
        let name = match open.args.get("step").and_then(Value::as_str) {
            Some(step) => step.to_owned(),
            None => span.name().to_owned(),
        };
        let micros = |at: Instant| at.saturating_duration_since(self.0.epoch).as_micros() as u64;
        self.0.write(&json!({
            "name": name,
            "cat": span.name(),
            "ph": "X",
            "ts": micros(open.start),
            "dur": open.start.elapsed().as_micros() as u64,
            "pid": 1,
            "tid": open.row,
            "args": open.args,
        }));
    }
}
