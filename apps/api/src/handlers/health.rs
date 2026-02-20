use axum::Json;

use crate::dto::HealthResponse;

pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
