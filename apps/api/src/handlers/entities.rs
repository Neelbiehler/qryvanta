use std::str::FromStr;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::{
    FieldType, FormTab, FormType, ViewColumn, ViewFilterGroup, ViewSort, ViewType,
};

use crate::dto::{
    CreateEntityRequest, CreateFieldRequest, CreateFormRequest, CreateOptionSetRequest,
    CreateViewRequest, EntityResponse, FieldResponse, FormResponse, OptionSetResponse,
    PublishedSchemaResponse, UpdateFieldRequest, ViewResponse,
};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_entities_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<Vec<EntityResponse>>> {
    let entities = state
        .metadata_service
        .list_entities(&user)
        .await?
        .into_iter()
        .map(EntityResponse::from)
        .collect();

    Ok(Json(entities))
}

pub async fn create_entity_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<CreateEntityRequest>,
) -> ApiResult<(StatusCode, Json<EntityResponse>)> {
    let entity = state
        .metadata_service
        .register_entity_with_details(
            &user,
            payload.logical_name,
            payload.display_name,
            payload.description,
            payload.plural_display_name,
            payload.icon,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(EntityResponse::from(entity))))
}

pub async fn list_fields_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<Vec<FieldResponse>>> {
    let fields = state
        .metadata_service
        .list_fields(&user, entity_logical_name.as_str())
        .await?
        .into_iter()
        .map(FieldResponse::from)
        .collect();

    Ok(Json(fields))
}

pub async fn save_field_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<CreateFieldRequest>,
) -> ApiResult<(StatusCode, Json<FieldResponse>)> {
    let field_type = FieldType::from_str(payload.field_type.as_str())?;
    let field = state
        .metadata_service
        .save_field(
            &user,
            qryvanta_application::SaveFieldInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                field_type,
                is_required: payload.is_required,
                is_unique: payload.is_unique,
                default_value: payload.default_value,
                relation_target_entity: payload.relation_target_entity,
                option_set_logical_name: payload.option_set_logical_name,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(FieldResponse::from(field))))
}

pub async fn update_field_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, field_logical_name)): Path<(String, String)>,
    Json(payload): Json<UpdateFieldRequest>,
) -> ApiResult<Json<FieldResponse>> {
    let field = state
        .metadata_service
        .update_field(
            &user,
            qryvanta_application::UpdateFieldInput {
                entity_logical_name,
                logical_name: field_logical_name,
                display_name: payload.display_name,
                description: payload.description,
                default_value: payload.default_value,
                max_length: payload.max_length,
                min_value: payload.min_value,
                max_value: payload.max_value,
            },
        )
        .await?;

    Ok(Json(FieldResponse::from(field)))
}

pub async fn delete_field_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, field_logical_name)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    state
        .metadata_service
        .delete_field(
            &user,
            entity_logical_name.as_str(),
            field_logical_name.as_str(),
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn publish_entity_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<PublishedSchemaResponse>> {
    let published_schema = state
        .metadata_service
        .publish_entity(&user, entity_logical_name.as_str())
        .await?;

    Ok(Json(PublishedSchemaResponse::from(published_schema)))
}

pub async fn list_option_sets_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<Vec<OptionSetResponse>>> {
    let option_sets = state
        .metadata_service
        .list_option_sets(&user, entity_logical_name.as_str())
        .await?
        .into_iter()
        .map(OptionSetResponse::from)
        .collect();

    Ok(Json(option_sets))
}

pub async fn save_option_set_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<CreateOptionSetRequest>,
) -> ApiResult<(StatusCode, Json<OptionSetResponse>)> {
    let options = payload
        .options
        .into_iter()
        .map(qryvanta_domain::OptionSetItem::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let option_set = state
        .metadata_service
        .save_option_set(
            &user,
            qryvanta_application::SaveOptionSetInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                options,
            },
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(OptionSetResponse::from(option_set)),
    ))
}

pub async fn update_option_set_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, option_set_logical_name)): Path<(String, String)>,
    Json(payload): Json<CreateOptionSetRequest>,
) -> ApiResult<Json<OptionSetResponse>> {
    if payload.logical_name != option_set_logical_name {
        return Err(AppError::Validation(format!(
            "option set logical name in path '{}' must match payload '{}'",
            option_set_logical_name, payload.logical_name
        ))
        .into());
    }

    let options = payload
        .options
        .into_iter()
        .map(qryvanta_domain::OptionSetItem::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let option_set = state
        .metadata_service
        .save_option_set(
            &user,
            qryvanta_application::SaveOptionSetInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                options,
            },
        )
        .await?;

    Ok(Json(OptionSetResponse::from(option_set)))
}

pub async fn get_option_set_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, option_set_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<OptionSetResponse>> {
    let option_set = state
        .metadata_service
        .find_option_set(
            &user,
            entity_logical_name.as_str(),
            option_set_logical_name.as_str(),
        )
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "option set '{}.{}' does not exist",
                entity_logical_name, option_set_logical_name
            ))
        })?;
    Ok(Json(OptionSetResponse::from(option_set)))
}

pub async fn delete_option_set_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, option_set_logical_name)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    state
        .metadata_service
        .delete_option_set(
            &user,
            entity_logical_name.as_str(),
            option_set_logical_name.as_str(),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn latest_published_schema_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<PublishedSchemaResponse>> {
    let published_schema = state
        .metadata_service
        .latest_published_schema(&user, entity_logical_name.as_str())
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "entity '{}' does not have a published schema",
                entity_logical_name
            ))
        })?;

    Ok(Json(PublishedSchemaResponse::from(published_schema)))
}

pub async fn list_forms_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<Vec<FormResponse>>> {
    let forms = state
        .metadata_service
        .list_forms(&user, entity_logical_name.as_str())
        .await?
        .into_iter()
        .map(FormResponse::from)
        .collect();
    Ok(Json(forms))
}

pub async fn save_form_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<CreateFormRequest>,
) -> ApiResult<(StatusCode, Json<FormResponse>)> {
    let form_type = FormType::from_str(payload.form_type.as_str())?;
    let tabs = payload
        .tabs
        .into_iter()
        .map(serde_json::from_value::<FormTab>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AppError::Validation(format!("invalid form tab payload: {error}")))?;
    let form = state
        .metadata_service
        .save_form(
            &user,
            qryvanta_application::SaveFormInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                form_type,
                tabs,
                header_fields: payload.header_fields,
            },
        )
        .await?;
    Ok((StatusCode::CREATED, Json(FormResponse::from(form))))
}

pub async fn get_form_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, form_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<FormResponse>> {
    let form = state
        .metadata_service
        .find_form(
            &user,
            entity_logical_name.as_str(),
            form_logical_name.as_str(),
        )
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "form '{}.{}' does not exist",
                entity_logical_name, form_logical_name
            ))
        })?;
    Ok(Json(FormResponse::from(form)))
}

pub async fn update_form_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, form_logical_name)): Path<(String, String)>,
    Json(payload): Json<CreateFormRequest>,
) -> ApiResult<Json<FormResponse>> {
    if payload.logical_name != form_logical_name {
        return Err(AppError::Validation(format!(
            "form logical name in path '{}' must match payload '{}'",
            form_logical_name, payload.logical_name
        ))
        .into());
    }

    let form_type = FormType::from_str(payload.form_type.as_str())?;
    let tabs = payload
        .tabs
        .into_iter()
        .map(serde_json::from_value::<FormTab>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AppError::Validation(format!("invalid form tab payload: {error}")))?;
    let form = state
        .metadata_service
        .save_form(
            &user,
            qryvanta_application::SaveFormInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                form_type,
                tabs,
                header_fields: payload.header_fields,
            },
        )
        .await?;
    Ok(Json(FormResponse::from(form)))
}

pub async fn delete_form_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, form_logical_name)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    state
        .metadata_service
        .delete_form(
            &user,
            entity_logical_name.as_str(),
            form_logical_name.as_str(),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_views_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<Vec<ViewResponse>>> {
    let views = state
        .metadata_service
        .list_views(&user, entity_logical_name.as_str())
        .await?
        .into_iter()
        .map(ViewResponse::from)
        .collect();
    Ok(Json(views))
}

pub async fn save_view_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<CreateViewRequest>,
) -> ApiResult<(StatusCode, Json<ViewResponse>)> {
    let view_type = ViewType::from_str(payload.view_type.as_str())?;
    let columns = payload
        .columns
        .into_iter()
        .map(serde_json::from_value::<ViewColumn>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AppError::Validation(format!("invalid view column payload: {error}")))?;
    let default_sort = payload
        .default_sort
        .map(serde_json::from_value::<ViewSort>)
        .transpose()
        .map_err(|error| {
            AppError::Validation(format!("invalid view default_sort payload: {error}"))
        })?;
    let filter_criteria = payload
        .filter_criteria
        .map(serde_json::from_value::<ViewFilterGroup>)
        .transpose()
        .map_err(|error| {
            AppError::Validation(format!("invalid view filter_criteria payload: {error}"))
        })?;
    let view = state
        .metadata_service
        .save_view(
            &user,
            qryvanta_application::SaveViewInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                view_type,
                columns,
                default_sort,
                filter_criteria,
                is_default: payload.is_default,
            },
        )
        .await?;
    Ok((StatusCode::CREATED, Json(ViewResponse::from(view))))
}

pub async fn get_view_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, view_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<ViewResponse>> {
    let view = state
        .metadata_service
        .find_view(
            &user,
            entity_logical_name.as_str(),
            view_logical_name.as_str(),
        )
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "view '{}.{}' does not exist",
                entity_logical_name, view_logical_name
            ))
        })?;
    Ok(Json(ViewResponse::from(view)))
}

pub async fn update_view_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, view_logical_name)): Path<(String, String)>,
    Json(payload): Json<CreateViewRequest>,
) -> ApiResult<Json<ViewResponse>> {
    if payload.logical_name != view_logical_name {
        return Err(AppError::Validation(format!(
            "view logical name in path '{}' must match payload '{}'",
            view_logical_name, payload.logical_name
        ))
        .into());
    }

    let view_type = ViewType::from_str(payload.view_type.as_str())?;
    let columns = payload
        .columns
        .into_iter()
        .map(serde_json::from_value::<ViewColumn>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AppError::Validation(format!("invalid view column payload: {error}")))?;
    let default_sort = payload
        .default_sort
        .map(serde_json::from_value::<ViewSort>)
        .transpose()
        .map_err(|error| {
            AppError::Validation(format!("invalid view default_sort payload: {error}"))
        })?;
    let filter_criteria = payload
        .filter_criteria
        .map(serde_json::from_value::<ViewFilterGroup>)
        .transpose()
        .map_err(|error| {
            AppError::Validation(format!("invalid view filter_criteria payload: {error}"))
        })?;
    let view = state
        .metadata_service
        .save_view(
            &user,
            qryvanta_application::SaveViewInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                view_type,
                columns,
                default_sort,
                filter_criteria,
                is_default: payload.is_default,
            },
        )
        .await?;
    Ok(Json(ViewResponse::from(view)))
}

pub async fn delete_view_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, view_logical_name)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    state
        .metadata_service
        .delete_view(
            &user,
            entity_logical_name.as_str(),
            view_logical_name.as_str(),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
