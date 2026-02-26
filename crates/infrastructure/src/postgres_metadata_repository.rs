use std::str::FromStr;

use async_trait::async_trait;
use qryvanta_application::{
    MetadataRepository, RecordListQuery, RuntimeRecordConditionGroup, RuntimeRecordConditionNode,
    RuntimeRecordFilter, RuntimeRecordJoinType, RuntimeRecordLogicalMode, RuntimeRecordOperator,
    RuntimeRecordQuery, RuntimeRecordSort, RuntimeRecordSortDirection, UniqueFieldValue,
};
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{
    BusinessRuleDefinition, EntityDefinition, EntityFieldDefinition, FieldType, FormDefinition,
    OptionSetDefinition, PublishedEntitySchema, RuntimeRecord, ViewDefinition,
};
use serde_json::Value;
use sqlx::{FromRow, PgPool, Postgres};
use uuid::Uuid;

/// PostgreSQL-backed metadata repository.
#[derive(Clone)]
pub struct PostgresMetadataRepository {
    pool: PgPool,
}

impl PostgresMetadataRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct EntityRow {
    logical_name: String,
    display_name: String,
    description: Option<String>,
    plural_display_name: Option<String>,
    icon: Option<String>,
}

#[derive(Debug, FromRow)]
struct FieldRow {
    entity_logical_name: String,
    logical_name: String,
    display_name: String,
    field_type: String,
    is_required: bool,
    is_unique: bool,
    default_value: Option<Value>,
    relation_target_entity: Option<String>,
    option_set_logical_name: Option<String>,
    description: Option<String>,
    calculation_expression: Option<String>,
    max_length: Option<i32>,
    min_value: Option<f64>,
    max_value: Option<f64>,
}

#[derive(Debug, FromRow)]
struct PublishedSchemaRow {
    version: i32,
    schema_json: Value,
}

#[derive(Debug, FromRow)]
struct OptionSetRow {
    entity_logical_name: String,
    logical_name: String,
    display_name: String,
    items_json: Value,
}

#[derive(Debug, FromRow)]
struct FormRow {
    definition_json: Value,
}

#[derive(Debug, FromRow)]
struct ViewRow {
    definition_json: Value,
}

#[derive(Debug, FromRow)]
struct BusinessRuleRow {
    definition_json: Value,
}

#[derive(Debug, FromRow)]
struct LatestSchemaRow {
    schema_json: Value,
}

#[derive(Debug, FromRow)]
struct RuntimeRecordRow {
    id: Uuid,
    entity_logical_name: String,
    data: Value,
}

mod components;
mod definitions;
mod publish;
mod runtime_records;

#[async_trait]
impl MetadataRepository for PostgresMetadataRepository {
    async fn save_entity(&self, tenant_id: TenantId, entity: EntityDefinition) -> AppResult<()> {
        self.save_entity_impl(tenant_id, entity).await
    }

    async fn list_entities(&self, tenant_id: TenantId) -> AppResult<Vec<EntityDefinition>> {
        self.list_entities_impl(tenant_id).await
    }

    async fn find_entity(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<EntityDefinition>> {
        self.find_entity_impl(tenant_id, logical_name).await
    }

    async fn update_entity(&self, tenant_id: TenantId, entity: EntityDefinition) -> AppResult<()> {
        self.update_entity_impl(tenant_id, entity).await
    }

    async fn save_field(&self, tenant_id: TenantId, field: EntityFieldDefinition) -> AppResult<()> {
        self.save_field_impl(tenant_id, field).await
    }

    async fn list_fields(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<EntityFieldDefinition>> {
        self.list_fields_impl(tenant_id, entity_logical_name).await
    }

    async fn find_field(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<Option<EntityFieldDefinition>> {
        self.find_field_impl(tenant_id, entity_logical_name, field_logical_name)
            .await
    }

    async fn delete_field(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<()> {
        self.delete_field_impl(tenant_id, entity_logical_name, field_logical_name)
            .await
    }

    async fn field_exists_in_published_schema(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<bool> {
        self.field_exists_in_published_schema_impl(
            tenant_id,
            entity_logical_name,
            field_logical_name,
        )
        .await
    }

    async fn save_option_set(
        &self,
        tenant_id: TenantId,
        option_set: OptionSetDefinition,
    ) -> AppResult<()> {
        self.save_option_set_impl(tenant_id, option_set).await
    }

    async fn list_option_sets(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<OptionSetDefinition>> {
        self.list_option_sets_impl(tenant_id, entity_logical_name)
            .await
    }

    async fn find_option_set(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        option_set_logical_name: &str,
    ) -> AppResult<Option<OptionSetDefinition>> {
        self.find_option_set_impl(tenant_id, entity_logical_name, option_set_logical_name)
            .await
    }

    async fn delete_option_set(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        option_set_logical_name: &str,
    ) -> AppResult<()> {
        self.delete_option_set_impl(tenant_id, entity_logical_name, option_set_logical_name)
            .await
    }

    async fn save_form(&self, tenant_id: TenantId, form: FormDefinition) -> AppResult<()> {
        self.save_form_impl(tenant_id, form).await
    }

    async fn list_forms(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>> {
        self.list_forms_impl(tenant_id, entity_logical_name).await
    }

    async fn find_form(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<Option<FormDefinition>> {
        self.find_form_impl(tenant_id, entity_logical_name, form_logical_name)
            .await
    }

    async fn delete_form(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<()> {
        self.delete_form_impl(tenant_id, entity_logical_name, form_logical_name)
            .await
    }

    async fn save_view(&self, tenant_id: TenantId, view: ViewDefinition) -> AppResult<()> {
        self.save_view_impl(tenant_id, view).await
    }

    async fn list_views(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>> {
        self.list_views_impl(tenant_id, entity_logical_name).await
    }

    async fn find_view(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<Option<ViewDefinition>> {
        self.find_view_impl(tenant_id, entity_logical_name, view_logical_name)
            .await
    }

    async fn delete_view(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<()> {
        self.delete_view_impl(tenant_id, entity_logical_name, view_logical_name)
            .await
    }

    async fn save_business_rule(
        &self,
        tenant_id: TenantId,
        business_rule: BusinessRuleDefinition,
    ) -> AppResult<()> {
        self.save_business_rule_impl(tenant_id, business_rule).await
    }

    async fn list_business_rules(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<BusinessRuleDefinition>> {
        self.list_business_rules_impl(tenant_id, entity_logical_name)
            .await
    }

    async fn find_business_rule(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        business_rule_logical_name: &str,
    ) -> AppResult<Option<BusinessRuleDefinition>> {
        self.find_business_rule_impl(tenant_id, entity_logical_name, business_rule_logical_name)
            .await
    }

    async fn delete_business_rule(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        business_rule_logical_name: &str,
    ) -> AppResult<()> {
        self.delete_business_rule_impl(tenant_id, entity_logical_name, business_rule_logical_name)
            .await
    }

    async fn publish_entity_schema(
        &self,
        tenant_id: TenantId,
        entity: EntityDefinition,
        fields: Vec<EntityFieldDefinition>,
        option_sets: Vec<OptionSetDefinition>,
        published_by: &str,
    ) -> AppResult<PublishedEntitySchema> {
        self.publish_entity_schema_impl(tenant_id, entity, fields, option_sets, published_by)
            .await
    }

    async fn latest_published_schema(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>> {
        self.latest_published_schema_impl(tenant_id, entity_logical_name)
            .await
    }

    async fn save_published_form_snapshots(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        published_schema_version: i32,
        forms: &[FormDefinition],
    ) -> AppResult<()> {
        self.save_published_form_snapshots_impl(
            tenant_id,
            entity_logical_name,
            published_schema_version,
            forms,
        )
        .await
    }

    async fn save_published_view_snapshots(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        published_schema_version: i32,
        views: &[ViewDefinition],
    ) -> AppResult<()> {
        self.save_published_view_snapshots_impl(
            tenant_id,
            entity_logical_name,
            published_schema_version,
            views,
        )
        .await
    }

    async fn list_latest_published_form_snapshots(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>> {
        self.list_latest_published_form_snapshots_impl(tenant_id, entity_logical_name)
            .await
    }

    async fn list_latest_published_view_snapshots(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>> {
        self.list_latest_published_view_snapshots_impl(tenant_id, entity_logical_name)
            .await
    }

    async fn create_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
        created_by_subject: &str,
    ) -> AppResult<RuntimeRecord> {
        self.create_runtime_record_impl(
            tenant_id,
            entity_logical_name,
            data,
            unique_values,
            created_by_subject,
        )
        .await
    }

    async fn update_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
    ) -> AppResult<RuntimeRecord> {
        self.update_runtime_record_impl(
            tenant_id,
            entity_logical_name,
            record_id,
            data,
            unique_values,
        )
        .await
    }

    async fn list_runtime_records(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        self.list_runtime_records_impl(tenant_id, entity_logical_name, query)
            .await
    }

    async fn query_runtime_records(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        self.query_runtime_records_impl(tenant_id, entity_logical_name, query)
            .await
    }

    async fn find_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<Option<RuntimeRecord>> {
        self.find_runtime_record_impl(tenant_id, entity_logical_name, record_id)
            .await
    }

    async fn delete_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        self.delete_runtime_record_impl(tenant_id, entity_logical_name, record_id)
            .await
    }

    async fn runtime_record_exists(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<bool> {
        self.runtime_record_exists_impl(tenant_id, entity_logical_name, record_id)
            .await
    }

    async fn runtime_record_owned_by_subject(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        subject: &str,
    ) -> AppResult<bool> {
        self.runtime_record_owned_by_subject_impl(
            tenant_id,
            entity_logical_name,
            record_id,
            subject,
        )
        .await
    }

    async fn has_relation_reference(
        &self,
        tenant_id: TenantId,
        target_entity_logical_name: &str,
        target_record_id: &str,
    ) -> AppResult<bool> {
        self.has_relation_reference_impl(tenant_id, target_entity_logical_name, target_record_id)
            .await
    }
}

#[cfg(test)]
mod tests;
