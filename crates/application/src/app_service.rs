use std::sync::Arc;

use async_trait::async_trait;

use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AppDefinition, AppEntityAction, AppEntityBinding, AppEntityForm, AppEntityRolePermission,
    AppEntityView, AppEntityViewMode, AppSitemap, AuditAction, ChartAggregation, ChartDefinition,
    ChartType, DashboardDefinition, DashboardWidget, FormDefinition, Permission,
    PublishedEntitySchema, RuntimeRecord, SitemapArea, SitemapGroup, SitemapSubArea, SitemapTarget,
    ViewDefinition,
};
use serde_json::Value;

use crate::app_ports::{
    AppRepository, BindAppEntityInput, CreateAppInput, RuntimeRecordService,
    SaveAppRoleEntityPermissionInput, SaveAppSitemapInput, SubjectEntityPermission,
};
use crate::{
    AuditEvent, AuditRepository, AuthorizationService, MetadataService, RecordListQuery,
    RuntimeRecordQuery,
};

mod access;
mod admin;
mod publish;
mod runtime;
mod sitemap;
mod workspace;

#[async_trait]
impl RuntimeRecordService for MetadataService {
    async fn latest_published_schema_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>> {
        self.latest_published_schema_unchecked(actor, entity_logical_name)
            .await
    }

    async fn list_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        self.list_runtime_records_unchecked(actor, entity_logical_name, query)
            .await
    }

    async fn query_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        self.query_runtime_records_unchecked(actor, entity_logical_name, query)
            .await
    }

    async fn get_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord> {
        self.get_runtime_record_unchecked(actor, entity_logical_name, record_id)
            .await
    }

    async fn create_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.create_runtime_record_unchecked(actor, entity_logical_name, data)
            .await
    }

    async fn update_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.update_runtime_record_unchecked(actor, entity_logical_name, record_id, data)
            .await
    }

    async fn delete_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        self.delete_runtime_record_unchecked(actor, entity_logical_name, record_id)
            .await
    }

    async fn list_forms_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>> {
        self.list_forms_unchecked(actor, entity_logical_name).await
    }

    async fn find_form_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<Option<FormDefinition>> {
        self.find_form_unchecked(actor, entity_logical_name, form_logical_name)
            .await
    }

    async fn list_views_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>> {
        self.list_views_unchecked(actor, entity_logical_name).await
    }

    async fn find_view_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<Option<ViewDefinition>> {
        self.find_view_unchecked(actor, entity_logical_name, view_logical_name)
            .await
    }
}

/// Application service for app builder and app-scoped runtime access.
#[derive(Clone)]
pub struct AppService {
    authorization_service: AuthorizationService,
    repository: Arc<dyn AppRepository>,
    runtime_record_service: Arc<dyn RuntimeRecordService>,
    audit_repository: Arc<dyn AuditRepository>,
}

impl AppService {
    /// Creates a new app service.
    #[must_use]
    pub fn new(
        authorization_service: AuthorizationService,
        repository: Arc<dyn AppRepository>,
        runtime_record_service: Arc<dyn RuntimeRecordService>,
        audit_repository: Arc<dyn AuditRepository>,
    ) -> Self {
        Self {
            authorization_service,
            repository,
            runtime_record_service,
            audit_repository,
        }
    }
}

#[cfg(test)]
mod tests;
