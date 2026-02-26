use super::*;

impl PostgresSecurityAdminRepository {
    pub(super) async fn create_temporary_access_grant_impl(
        &self,
        tenant_id: TenantId,
        created_by_subject: &str,
        input: CreateTemporaryAccessGrantInput,
    ) -> AppResult<TemporaryAccessGrant> {
        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                AppError::Internal(format!("failed to begin transaction: {error}"))
            })?;

        let grant_row = sqlx::query_as::<_, TemporaryAccessGrantRow>(
            r#"
            INSERT INTO security_temporary_access_grants (
                tenant_id,
                subject,
                reason,
                created_by_subject,
                expires_at
            )
            VALUES ($1, $2, $3, $4, now() + make_interval(mins => $5::INTEGER))
            RETURNING
                id AS grant_id,
                subject,
                reason,
                created_by_subject,
                to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS expires_at,
                NULL::TEXT AS revoked_at,
                NULL::TEXT AS permission
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(input.subject.as_str())
        .bind(input.reason.as_str())
        .bind(created_by_subject)
        .bind(i32::try_from(input.duration_minutes).map_err(|_| {
            AppError::Validation(
                "temporary access duration_minutes exceeds supported range".to_owned(),
            )
        })?)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to create temporary access grant: {error}"))
        })?;

        for permission in &input.permissions {
            sqlx::query(
                r#"
                INSERT INTO security_temporary_access_grant_permissions (grant_id, permission)
                VALUES ($1, $2)
                ON CONFLICT (grant_id, permission) DO NOTHING
                "#,
            )
            .bind(grant_row.grant_id)
            .bind(permission.as_str())
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                AppError::Internal(format!(
                    "failed to persist temporary access grant permissions: {error}"
                ))
            })?;
        }

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!("failed to commit transaction: {error}"))
        })?;

        Ok(TemporaryAccessGrant {
            grant_id: grant_row.grant_id.to_string(),
            subject: input.subject,
            permissions: input.permissions,
            reason: input.reason,
            created_by_subject: grant_row.created_by_subject,
            expires_at: grant_row.expires_at,
            revoked_at: None,
        })
    }

    pub(super) async fn revoke_temporary_access_grant_impl(
        &self,
        tenant_id: TenantId,
        revoked_by_subject: &str,
        grant_id: &str,
        revoke_reason: Option<&str>,
    ) -> AppResult<()> {
        let parsed_grant_id = uuid::Uuid::parse_str(grant_id)
            .map_err(|_| AppError::Validation(format!("invalid grant_id '{}'", grant_id)))?;

        let rows_affected = sqlx::query(
            r#"
            UPDATE security_temporary_access_grants
            SET revoked_at = now(),
                revoked_by_subject = $3,
                revoke_reason = $4
            WHERE tenant_id = $1
              AND id = $2
              AND revoked_at IS NULL
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(parsed_grant_id)
        .bind(revoked_by_subject)
        .bind(revoke_reason)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to revoke temporary access grant: {error}"))
        })?
        .rows_affected();

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "temporary access grant '{}' was not found or already revoked",
                grant_id
            )));
        }

        Ok(())
    }

    pub(super) async fn list_temporary_access_grants_impl(
        &self,
        tenant_id: TenantId,
        query: TemporaryAccessGrantQuery,
    ) -> AppResult<Vec<TemporaryAccessGrant>> {
        let capped_limit = query.limit.clamp(1, 200) as i64;
        let capped_offset = query.offset.min(5_000) as i64;

        let rows = sqlx::query_as::<_, TemporaryAccessGrantRow>(
            r#"
            SELECT
                grants.id AS grant_id,
                grants.subject,
                grants.reason,
                grants.created_by_subject,
                to_char(grants.expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS expires_at,
                CASE
                    WHEN grants.revoked_at IS NULL THEN NULL
                    ELSE to_char(grants.revoked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
                END AS revoked_at,
                permissions.permission
            FROM security_temporary_access_grants AS grants
            LEFT JOIN security_temporary_access_grant_permissions AS permissions
                ON permissions.grant_id = grants.id
            WHERE grants.tenant_id = $1
              AND ($2::TEXT IS NULL OR grants.subject = $2)
              AND (
                  $3::BOOLEAN = false
                  OR (grants.revoked_at IS NULL AND grants.expires_at > now())
              )
            ORDER BY grants.created_at DESC, permissions.permission
            LIMIT $4
            OFFSET $5
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(query.subject)
        .bind(query.active_only)
        .bind(capped_limit)
        .bind(capped_offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to list temporary access grants: {error}"))
        })?;

        aggregate_temporary_access_grants(rows, tenant_id)
    }
}
