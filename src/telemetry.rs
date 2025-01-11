use crate::configuration;
use secrecy::ExposeSecret;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn get_subscriber<T: AsRef<str>, Sink>(
    name: T,
    telemetry_settings: &configuration::TelemetrySettings,
    sink: Sink,
) -> impl Subscriber + Sync + Send
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&telemetry_settings.log_level));
    let formatting_layer = BunyanFormattingLayer::new(name.as_ref().into(), sink);

    let client = reqwest::Client::new();

    let otel_tracer = opentelemetry_application_insights::new_pipeline_from_connection_string(
        telemetry_settings
            .app_insights_connection_string
            .expose_secret(),
    )
    .expect("Invalid connection string")
    .with_client(client)
    .with_live_metrics(true)
    .with_service_name("todo_app")
    .with_sample_rate(0.3)
    .install_batch(opentelemetry_sdk::runtime::Tokio);

    let opentelemetry_layer = OpenTelemetryLayer::new(otel_tracer);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .with(opentelemetry_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
