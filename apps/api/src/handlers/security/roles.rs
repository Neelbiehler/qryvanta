use super::*;

pub async fn list_roles_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<Vec<RoleResponse>>> {
    let roles = state
        .security_admin_service
        .list_roles(&user)
        .await?
        .into_iter()
        .map(RoleResponse::from)
        .collect();

    Ok(Json(roles))
}

pub async fn create_role_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<CreateRoleRequest>,
) -> ApiResult<(StatusCode, Json<RoleResponse>)> {
    let permissions = payload
        .permissions
        .iter()
        .map(|value| Permission::from_transport(value.as_str()))
        .collect::<Result<Vec<_>, _>>()?;

    let role = state
        .security_admin_service
        .create_role(
            &user,
            qryvanta_application::CreateRoleInput {
                name: payload.name,
                permissions,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(RoleResponse::from(role))))
}

pub async fn assign_role_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<AssignRoleRequest>,
) -> ApiResult<StatusCode> {
    state
        .security_admin_service
        .assign_role(&user, payload.subject.as_str(), payload.role_name.as_str())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn unassign_role_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<RemoveRoleAssignmentRequest>,
) -> ApiResult<StatusCode> {
    state
        .security_admin_service
        .unassign_role(&user, payload.subject.as_str(), payload.role_name.as_str())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_role_assignments_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<Vec<RoleAssignmentResponse>>> {
    let assignments = state
        .security_admin_service
        .list_role_assignments(&user)
        .await?
        .into_iter()
        .map(RoleAssignmentResponse::from)
        .collect();

    Ok(Json(assignments))
}
