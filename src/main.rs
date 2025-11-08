use crate::{
    handlers::{enqueue_job_handler, health_check, metrics},
    state::AppState,
};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use poem::{
    EndpointExt, Route, Server, get,
    listener::TcpListener,
    middleware::{TokioMetrics, Tracing},
    post,
};
use redis::Client;
use sqlx::PgPool;
use tracing_subscriber::fmt;

mod controllers;
mod handlers;
mod job;
mod state;

/// prometheus handle
static PROMETHEUS_HANDLE: std::sync::OnceLock<PrometheusHandle> = std::sync::OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Prometheus exporter
    let builder = PrometheusBuilder::new();
    let handle = builder
        .install_recorder()
        .expect("Failed to install Prometheus recorder");
    PROMETHEUS_HANDLE
        .set(handle)
        .expect("Prometheus handle already set");

    // init redis and sqlx connection
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL required");

    let db = PgPool::connect(&db_url).await.unwrap();
    let redis = Client::open(redis_url).unwrap();

    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    let app_state = AppState { redis, db };

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
        .at("/job/enqueue", post(enqueue_job_handler))
        .data(app_state)
        .with(Tracing::default())
        .with(TokioMetrics::new());

    println!("Server listening on http://0.0.0.0:3000");
    println!("  Health  → http://localhost:3000/health");
    println!("  Metrics → http://localhost:3000/metrics (Prometheus)");
    println!("  Set RUST_LOG=debug for verbose tracing");

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await;

    Ok(())
}
