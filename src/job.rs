use std::collections::HashMap;

#[derive(Clone)]
pub struct JobContext {
    pub execution_id: uuid::Uuid,

    // job
    pub job_id: uuid::Uuid,
    pub job_name: String,
    pub attempt: u16,
    pub max_attempts: u16,

    /// Database connection (pooled)
    pub db: sqlx::Pool<sqlx::Postgres>,

    /// Redis client
    pub redis: redis::Client,

    /// HTTP client for external APIs (e.g., send email)
    pub http_client: reqwest::Client,

    /// Tracing span for this execution
    pub span: tracing::Span,

    /// Cancellation token for graceful shutdown
    pub shutdown: tokio::sync::watch::Receiver<bool>,

    /// Optional: Feature flags, secrets, config
    pub config: std::sync::Arc<HashMap<String, String>>,
}

pub trait Job: Send + Sync + 'static {
    const NAME: &'static str;
    type Output;
    type Error: std::error::Error + Send + Sync;

    async fn execute(&self, ctx: JobContext) -> Result<Self::Output, Self::Error>;
}
