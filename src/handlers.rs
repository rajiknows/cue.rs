use poem::{Response, handler};
use serde::Serialize;
use serde_json::json;
use tracing::info;

use crate::{PROMETHEUS_HANDLE, job::Job};

#[derive(Serialize)]
struct HealthCheck {
    status: String,
    message: String,
    timestamp: String,
}

#[handler]
pub async fn health_check() -> Response {
    info!("Health check requested");

    let health = HealthCheck {
        status: "healthy".to_string(),
        message: "Service is running".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Response::builder()
        .content_type("application/json")
        .body(json!(health).to_string())
}

#[handler]
pub async fn metrics() -> String {
    PROMETHEUS_HANDLE.get().unwrap().render()
}

#[handler]
pub async fn post_job(job: impl Job) -> Response {}
