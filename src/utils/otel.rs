//! Professional Observability (OpenTelemetry)
//! 
//! Provides a centralized telemetry system for tracing and metrics collection.
//! Derived from codex-rs patterns.

use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{propagation::TraceContextPropagator, runtime, trace as sdktrace, Resource};
use opentelemetry::trace::TracerProvider; // Import trait for .tracer()
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use std::error::Error;

pub struct OtelGuard;

impl Drop for OtelGuard {
    fn drop(&mut self) {
        global::shutdown_tracer_provider();
    }
}

pub fn init_telemetry(service_name: &str) -> Result<OtelGuard, Box<dyn Error>> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    // 1. Configure OTLP Span Exporter
    // Changed from SpanExporter::builder() to new_exporter().tonic()
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .build_span_exporter()?;

    // 2. Configure Tracer Provider with Batch Exporter
    // Use config() for resource to be safe across versions
    let trace_config = sdktrace::Config::default().with_resource(Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("environment", "development"),
    ]));

    let provider = sdktrace::TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        //.with_resource(...) // Moved to config
        .with_config(trace_config)
        .build();

    // Set global provider
    global::set_tracer_provider(provider.clone());
    
    // 3. Create Tracing Layer
    // Use provider.tracer() to get sdktrace::Tracer which implements PreSampledTracer
    let tracer = provider.tracer(service_name.to_string());
    
    // global::tracer returns BoxedTracer which might not implement PreSampledTracer in this version
    // let tracer = global::tracer(service_name.to_string()); 

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // 4. Configure Filter
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("rust_agency=info,opentelemetry=debug"));

    // 5. Initialize Global Subscriber
    Registry::default()
        .with(filter)
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    Ok(OtelGuard)
}
