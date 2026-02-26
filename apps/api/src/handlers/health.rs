use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::dto::{HealthDependencyStatus, HealthResponse};
use crate::state::AppState;

mod checks;
mod handlers;

pub use handlers::health_handler;
