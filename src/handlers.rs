use crate::controllers::{job::enqueue_job, *};
use chrono::{DateTime, Utc};
use poem::{
    Response, Result, handler,
    web::{Data, Json},
};
use redis::{AsyncCommands, Commands};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;
use validator::Validate;

use crate::{PROMETHEUS_HANDLE, job::Job, state::AppState};

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

#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "lowercase")]
pub struct EnqueueRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub payload: serde_json::Value,
    pub run_at: Option<DateTime<Utc>>,
    pub priority: Option<i16>,
    #[validate(length(min = 1, max = 255))]
    pub idempotency_key: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct EnqueueResponse {
    job_id: uuid::Uuid,
    status: JobStatus,
}

#[handler]
pub async fn enqueue_job_handler(
    Data(state): Data<&AppState>,
    Json(req): Json<EnqueueRequest>,
) -> Result<Json<EnqueueResponse>> {
    req.validate()
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

    let job_id = enqueue_job(&state.db, &req)
        .await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
        if let Err(e) = conn
            .publish::<_, _, ()>("job:available", job_id.to_string())
            .await
        {
            tracing::warn!("Redis publish failed: {}", e);
        }
    }

    Ok(Json(EnqueueResponse {
        job_id,
        status: JobStatus::Pending,
    }))
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}
