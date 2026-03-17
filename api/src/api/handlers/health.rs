use crate::{db::DbClient, services::background_jobs::BackgroundJobManager};
use axum::{Json, extract::State, http::StatusCode};

/// Health check endpoint that includes background job status
pub async fn health_check(State(db): State<DbClient>) -> (StatusCode, Json<serde_json::Value>) {
    let bg_manager = BackgroundJobManager::new(db.clone());
    let bg_health = bg_manager.get_health_status().await;
    let bg_ok = bg_health.status == "Active";

    // Get Redis connection and status
    let (redis_status, redis_ok) = match db.get_async_redis_conn().await {
        Err(e) => (
            serde_json::json!({
                "status": "error",
                "message": e.to_string()
            }),
            false,
        ),
        Ok(_) => (serde_json::json!("connected"), true),
    };

    // Get database connection and status
    let (db_status, db_ok) = match db.get_db_conn().await {
        Ok(_) => (serde_json::json!("connected"), true),
        Err(e) => (
            serde_json::json!({
                "status": "error",
                "message": e.to_string()
            }),
            false,
        ),
    };

    let overall_ok = redis_ok && bg_ok && db_ok;

    let health_status = serde_json::json!({
        "status": if overall_ok { "ok" } else { "degraded" },
        "database": db_status,
        "redis": redis_status,
        "background_jobs": bg_health,
        "timestamp": chrono::Utc::now()
    });

    let status_code = if overall_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(health_status))
}

/// Background job status endpoint
pub async fn background_job_status(
    State(db): State<DbClient>,
) -> (
    StatusCode,
    Json<crate::services::background_jobs::BackgroundJobHealth>,
) {
    let bg_manager = BackgroundJobManager::new(db);
    let health = bg_manager.get_health_status().await;

    let status_code = match health.status.as_str() {
        "healthy" => StatusCode::OK,
        "unknown" => StatusCode::ACCEPTED,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(health))
}
