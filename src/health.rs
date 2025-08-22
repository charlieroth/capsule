use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use sqlx::{Pool, Postgres};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::app_state::AppState;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    status: String,
    database: String,
}

#[utoipa::path(
    get,
    path = "/healthz",
    tag = "health",
    responses(
        (status = 200, description = "Health check successful", body = HealthResponse),
        (status = 503, description = "Service unavailable")
    )
)]
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    match check_database_health(&state.db_pool).await {
        Ok(_) => {
            info!("Health check passed");
            Ok(Json(HealthResponse {
                status: "OK".to_string(),
                database: "healthy".to_string(),
            }))
        }
        Err(_) => {
            error!("Database health check failed");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

async fn check_database_health(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").fetch_one(pool).await?;
    Ok(())
}
