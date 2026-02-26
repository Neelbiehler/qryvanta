use super::*;

impl AppService {
    pub(super) async fn require_admin(&self, actor: &UserIdentity) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await
    }

    pub(super) async fn require_app_exists(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<()> {
        let app = self
            .repository
            .find_app(tenant_id, app_logical_name)
            .await?;
        if app.is_none() {
            return Err(AppError::NotFound(format!(
                "app '{}' does not exist for tenant '{}'",
                app_logical_name, tenant_id
            )));
        }
        Ok(())
    }

    pub(super) async fn ensure_subject_can_access_app(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
    ) -> AppResult<()> {
        let can_access = self
            .repository
            .subject_can_access_app(actor.tenant_id(), actor.subject(), app_logical_name)
            .await?;

        if !can_access {
            return Err(AppError::Forbidden(format!(
                "subject '{}' is not assigned to app '{}'",
                actor.subject(),
                app_logical_name
            )));
        }

        Ok(())
    }

    pub(super) async fn require_entity_action(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        action: AppEntityAction,
    ) -> AppResult<()> {
        self.ensure_subject_can_access_app(actor, app_logical_name)
            .await?;
        let permission = self
            .repository
            .subject_entity_permission(
                actor.tenant_id(),
                actor.subject(),
                app_logical_name,
                entity_logical_name,
            )
            .await?;

        if permission
            .map(|value| value.allows(action))
            .unwrap_or(false)
        {
            return Ok(());
        }

        Err(AppError::Forbidden(format!(
            "subject '{}' is missing '{}' access for entity '{}' in app '{}'",
            actor.subject(),
            action.as_str(),
            entity_logical_name,
            app_logical_name
        )))
    }
}
