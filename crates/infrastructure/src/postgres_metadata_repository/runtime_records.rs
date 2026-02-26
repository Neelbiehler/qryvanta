use super::*;

mod query;
mod read;
mod relations;
mod write;

fn parse_runtime_record_uuid(record_id: &str) -> AppResult<Uuid> {
    Uuid::parse_str(record_id).map_err(|error| {
        AppError::Validation(format!("invalid runtime record id '{record_id}': {error}"))
    })
}

fn runtime_record_from_row(row: RuntimeRecordRow) -> AppResult<RuntimeRecord> {
    RuntimeRecord::new(row.id.to_string(), row.entity_logical_name, row.data)
}

async fn index_unique_values(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    tenant_id: TenantId,
    entity_logical_name: &str,
    record_id: Uuid,
    unique_values: &[UniqueFieldValue],
) -> AppResult<()> {
    for unique_value in unique_values {
        let result = sqlx::query(
            r#"
            INSERT INTO runtime_record_unique_values (
                tenant_id,
                entity_logical_name,
                field_logical_name,
                field_value_hash,
                record_id
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(unique_value.field_logical_name.as_str())
        .bind(unique_value.field_value_hash.as_str())
        .bind(record_id)
        .execute(&mut **transaction)
        .await;

        if let Err(error) = result {
            if let sqlx::Error::Database(database_error) = &error
                && database_error.code().as_deref() == Some("23505")
            {
                return Err(AppError::Conflict(format!(
                    "unique constraint violated for field '{}'",
                    unique_value.field_logical_name
                )));
            }

            return Err(AppError::Internal(format!(
                "failed to index unique value for field '{}' on entity '{}' in tenant '{}': {error}",
                unique_value.field_logical_name, entity_logical_name, tenant_id
            )));
        }
    }

    Ok(())
}
