use axum::extract::FromRef;
use serde::Deserialize;

use crate::state::AppState;

mod diff;
mod handlers;
mod history;
mod issues;

pub use handlers::{
    run_workspace_publish_handler, workspace_publish_checks_handler,
    workspace_publish_diff_handler, workspace_publish_history_handler,
};

#[cfg(test)]
use crate::dto::{PublishCheckCategoryDto, PublishCheckScopeDto};

#[derive(Clone)]
pub struct PublishState {
    pub app_service: qryvanta_application::AppService,
    pub metadata_service: qryvanta_application::MetadataService,
    pub security_admin_service: qryvanta_application::SecurityAdminService,
}

impl FromRef<AppState> for PublishState {
    fn from_ref(input: &AppState) -> Self {
        Self {
            app_service: input.app_service.clone(),
            metadata_service: input.metadata_service.clone(),
            security_admin_service: input.security_admin_service.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PublishHistoryQuery {
    pub limit: Option<usize>,
}

#[cfg(test)]
mod tests;
