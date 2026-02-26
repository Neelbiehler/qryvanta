use super::*;

#[derive(Debug, serde::Deserialize)]
pub struct RuntimeFieldPermissionQuery {
    pub subject: Option<String>,
    pub entity_logical_name: Option<String>,
}

pub async fn save_runtime_field_permissions_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<SaveRuntimeFieldPermissionsRequest>,
) -> ApiResult<Json<Vec<RuntimeFieldPermissionResponse>>> {
    let entries = state
        .security_admin_service
        .save_runtime_field_permissions(
            &user,
            qryvanta_application::SaveRuntimeFieldPermissionsInput {
                subject: payload.subject,
                entity_logical_name: payload.entity_logical_name,
                fields: payload
                    .fields
                    .into_iter()
                    .map(|field| qryvanta_application::RuntimeFieldPermissionInput {
                        field_logical_name: field.field_logical_name,
                        can_read: field.can_read,
                        can_write: field.can_write,
                    })
                    .collect(),
            },
        )
        .await?
        .into_iter()
        .map(RuntimeFieldPermissionResponse::from)
        .collect();

    Ok(Json(entries))
}

pub async fn list_runtime_field_permissions_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Query(query): Query<RuntimeFieldPermissionQuery>,
) -> ApiResult<Json<Vec<RuntimeFieldPermissionResponse>>> {
    let entries = state
        .security_admin_service
        .list_runtime_field_permissions(
            &user,
            query.subject.as_deref(),
            query.entity_logical_name.as_deref(),
        )
        .await?
        .into_iter()
        .map(RuntimeFieldPermissionResponse::from)
        .collect();

    Ok(Json(entries))
}
