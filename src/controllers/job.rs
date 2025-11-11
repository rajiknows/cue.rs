use chrono::Utc;
use poem::web::Json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::handlers::EnqueueRequest;

#[derive(thiserror::Error, Debug)]
pub enum EnqueueError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
}

pub async fn enqueue_job(pool: &PgPool, req: &EnqueueRequest) -> Result<Uuid, EnqueueError> {
    let job_id = Uuid::new_v4();
    let now = Utc::now();
    let run_at = req.run_at.unwrap_or(now);
    let priority = req.priority.unwrap_or(5);
    let status = if run_at > now { "scheduled" } else { "pending" };

    let row = sqlx::query!(
        r#"
        INSERT INTO jobs (
            id, name, payload, priority, status,
            run_at, attempts, max_attempts, idempotency_key
        ) VALUES ($1, $2, $3, $4, $5, $6, 0, 5, $7)
        ON CONFLICT (idempotency_key) DO UPDATE
            SET status = EXCLUDED.status,
                run_at = EXCLUDED.run_at
        RETURNING id
        "#,
        job_id,
        req.name,
        &req.payload,
        priority as i16,
        status,
        run_at,
        req.idempotency_key
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}
