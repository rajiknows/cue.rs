-- migrations/001_init.sql
CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    payload JSONB NOT NULL,
    priority SMALLINT DEFAULT 5,
    status TEXT NOT NULL,
    run_at TIMESTAMPTZ NOT NULL,
    attempts SMALLINT DEFAULT 0,
    max_attempts SMALLINT DEFAULT 5,
    idempotency_key TEXT UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_jobs_poll 
    ON jobs(status, run_at) 
    WHERE status IN ('pending', 'scheduled');
