use opentelemetry_sdk::trace::{self, RandomIdGenerator, Sampler};
use opentelemetry_sdk::Resource;
use opentelemetry::KeyValue;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, Registry};

/// Initialize tracing and OpenTelemetry for the given service.
///
/// Sets up `tracing_subscriber` with EnvFilter (default: `info`)
/// and configures a Jaeger exporter via `opentelemetry_jaeger`.
/// Must be called once at service startup.
pub fn init_telemetry(service_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_tracing(service_name)?;
    init_opentelemetry(service_name)?;
    Ok(())
}

fn init_tracing(service_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .with_filter(filter);

    Registry::default()
        .with(fmt_layer)
        .init();

    tracing::info!("Logging initialized for service: {}", service_name);
    Ok(())
}

fn init_opentelemetry(service_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(service_name)
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name.to_string()),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Replace the global subscriber with one that includes OpenTelemetry
    tracing::subscriber::set_global_default(
        Registry::default()
            .with(telemetry_layer)
            .with(tracing_subscriber::fmt::layer().with_target(false)),
    )?;

    tracing::info!("OpenTelemetry initialized for service: {}", service_name);
    Ok(())
}

/// Gracefully shut down the tracer provider.
///
/// Call this during service shutdown to flush remaining spans.
pub fn shutdown() {
    opentelemetry::global::shutdown_tracer_provider();
}

/// Log an HTTP request start with method and path.
#[macro_export]
macro_rules! trace_request {
    ($method:expr, $path:expr) => {
        tracing::info!(
            method = $method,
            path = $path,
            "HTTP request started"
        );
    };
}

/// Log an HTTP response with status code and duration.
#[macro_export]
macro_rules! trace_response {
    ($status:expr, $duration:expr) => {
        tracing::info!(
            status = $status.as_u16(),
            duration_ms = $duration.as_millis(),
            "HTTP request completed"
        );
    };
}

/// Log a database query execution at debug level.
#[macro_export]
macro_rules! trace_database_query {
    ($query:expr) => {
        tracing::debug!(
            query = $query,
            "Database query executed"
        );
    };
}

/// Log medical event processing with type and ID.
#[macro_export]
macro_rules! trace_event_processing {
    ($event_type:expr, $event_id:expr) => {
        tracing::info!(
            event_type = ?$event_type,
            event_id = %$event_id,
            "Processing medical event"
        );
    };
}
