use super::*;

impl AuthorizationService {
    /// Returns effective field-level runtime access for a subject and entity.
    pub async fn runtime_field_access(
        &self,
        tenant_id: TenantId,
        subject: &str,
        entity_logical_name: &str,
    ) -> AppResult<Option<RuntimeFieldAccess>> {
        let grants = self
            .repository
            .list_runtime_field_grants_for_subject(tenant_id, subject, entity_logical_name)
            .await?;

        if grants.is_empty() {
            return Ok(None);
        }

        let mut readable_fields = std::collections::BTreeSet::new();
        let mut writable_fields = std::collections::BTreeSet::new();

        for grant in grants {
            if grant.can_read {
                readable_fields.insert(grant.field_logical_name.clone());
            }
            if grant.can_write {
                writable_fields.insert(grant.field_logical_name);
            }
        }

        Ok(Some(RuntimeFieldAccess {
            readable_fields,
            writable_fields,
        }))
    }
}
