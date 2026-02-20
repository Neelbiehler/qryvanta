use qryvanta_core::UserIdentity;
use qryvanta_domain::EntityDefinition;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Health response payload.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/health-response.ts"
)]
pub struct HealthResponse {
    pub status: &'static str,
}

/// Incoming payload for entity creation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-entity-request.ts"
)]
pub struct CreateEntityRequest {
    pub logical_name: String,
    pub display_name: String,
}

/// API representation of an entity.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/entity-response.ts"
)]
pub struct EntityResponse {
    pub logical_name: String,
    pub display_name: String,
}

/// API representation of the authenticated user.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/user-identity-response.ts"
)]
pub struct UserIdentityResponse {
    pub subject: String,
    pub display_name: String,
    pub email: Option<String>,
    pub tenant_id: String,
}

impl From<EntityDefinition> for EntityResponse {
    fn from(entity: EntityDefinition) -> Self {
        Self {
            logical_name: entity.logical_name().as_str().to_owned(),
            display_name: entity.display_name().as_str().to_owned(),
        }
    }
}

impl From<UserIdentity> for UserIdentityResponse {
    fn from(identity: UserIdentity) -> Self {
        Self {
            subject: identity.subject().to_owned(),
            display_name: identity.display_name().to_owned(),
            email: identity.email().map(ToOwned::to_owned),
            tenant_id: identity.tenant_id().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CreateEntityRequest, EntityResponse, HealthResponse, UserIdentityResponse};

    use crate::error::ErrorResponse;
    use ts_rs::Config;
    use ts_rs::TS;

    #[test]
    fn export_ts_bindings() -> Result<(), ts_rs::ExportError> {
        let config = Config::default();

        CreateEntityRequest::export(&config)?;
        EntityResponse::export(&config)?;
        ErrorResponse::export(&config)?;
        HealthResponse::export(&config)?;
        UserIdentityResponse::export(&config)?;

        Ok(())
    }
}
