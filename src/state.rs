#[derive(Clone)]
pub struct AppState {
    pub redis: redis::Client,
    pub db: sqlx::PgPool,
}
