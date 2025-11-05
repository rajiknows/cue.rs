use crate::handlers::{health_check, metrics};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use poem::{
    EndpointExt, Route, Server, get,
    listener::TcpListener,
    middleware::{TokioMetrics, Tracing},
    post,
};
use tracing_subscriber::{fmt, prelude::*};

mod handlers;
mod job;

/// prometheus handle
static PROMETHEUS_HANDLE: std::sync::OnceLock<PrometheusHandle> = std::sync::OnceLock::new();

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize Prometheus exporter
    let builder = PrometheusBuilder::new();
    let handle = builder
        .install_recorder()
        .expect("Failed to install Prometheus recorder");
    PROMETHEUS_HANDLE
        .set(handle)
        .expect("Prometheus handle already set");

    // Initialize tracing
    fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_file(true)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app = Route::new()
        .at("/health", get(health_check))
        .at("/metrics", get(metrics))
        .at("/job/enqueue", post(post_job))
        .with(Tracing::default())
        .with(TokioMetrics::new());

    println!("Server listening on http://0.0.0.0:3000");
    println!("  Health  → http://localhost:3000/health");
    println!("  Metrics → http://localhost:3000/metrics (Prometheus)");
    println!("  Set RUST_LOG=debug for verbose tracing");

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
