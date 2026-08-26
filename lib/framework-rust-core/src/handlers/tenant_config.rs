use crate::{
    entity::tenant_config_definitions,
    error::AppError,
    extractors::TenantDb,
    services::tenant_config_service::{self, ConfigValueType},
};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DefineConfigRequest {
    pub config_key: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub scope_type: String,
    pub value_type: ConfigValueType,
    pub is_required: bool,
    pub is_multi_value: bool,
    pub default_value: Option<Value>,
    pub allowed_values: Option<Value>,
    pub constraints: Value,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ConfigDefinitionResponse {
    pub id: Uuid,
    pub config_key: String,
    pub display_name: String,
    pub category: String,
    pub scope_type: String,
    pub value_type: String,
    pub is_required: bool,
    pub is_multi_value: bool,
    pub default_value: Option<Value>,
    pub allowed_values: Option<Value>,
    pub constraints: Value,
}

impl From<tenant_config_definitions::Model> for ConfigDefinitionResponse {
    fn from(row: tenant_config_definitions::Model) -> Self {
        Self {
            id: row.id,
            config_key: row.config_key,
            display_name: row.display_name,
            category: row.category,
            scope_type: row.scope_type,
            value_type: row.value_type,
            is_required: row.is_required,
            is_multi_value: row.is_multi_value,
            default_value: row.default_value,
            allowed_values: row.allowed_values,
            constraints: row.constraints,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetConfigValueRequest {
    pub scope_id: Option<Uuid>,
    pub value: Value,
}

#[derive(Debug, Deserialize)]
pub struct ResolveQuery {
    pub scope_id: Option<Uuid>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ConfigValueResponse {
    pub id: Uuid,
    pub definition_id: Uuid,
    pub scope_type: String,
    pub scope_id: Option<Uuid>,
    pub value: Value,
}

#[utoipa::path(get, path = "/settings/config/definitions", tag = "settings",
    responses((status = 200, body = [ConfigDefinitionResponse])), security(("bearer_auth" = [])))]
pub async fn list_definitions(db: TenantDb) -> Result<impl IntoResponse, AppError> {
    let rows = tenant_config_definitions::Entity::find()
        .filter(tenant_config_definitions::Column::TenantId.eq(db.tenant_id))
        .filter(tenant_config_definitions::Column::Status.eq("active"))
        .filter(tenant_config_definitions::Column::DeletedAt.is_null())
        .all(&db.txn)
        .await?;
    db.txn.commit().await?;
    Ok(Json(
        rows.into_iter()
            .map(ConfigDefinitionResponse::from)
            .collect::<Vec<_>>(),
    ))
}

#[utoipa::path(post, path = "/settings/config/definitions", tag = "settings",
    request_body = DefineConfigRequest, responses((status = 201, body = ConfigDefinitionResponse)),
    security(("bearer_auth" = [])))]
pub async fn define(
    db: TenantDb,
    Json(input): Json<DefineConfigRequest>,
) -> Result<impl IntoResponse, AppError> {
    let row = tenant_config_service::define(
        &db.txn,
        tenant_config_service::DefineInput {
            tenant_id: db.tenant_id,
            config_key: input.config_key,
            display_name: input.display_name,
            description: input.description,
            category: input.category,
            scope_type: input.scope_type,
            value_type: input.value_type,
            is_required: input.is_required,
            is_multi_value: input.is_multi_value,
            default_value: input.default_value,
            allowed_values: input.allowed_values,
            constraints: input.constraints,
        },
    )
    .await?;
    db.txn.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(ConfigDefinitionResponse::from(row)),
    ))
}

#[utoipa::path(put, path = "/settings/config/{definition_id}", tag = "settings",
    params(("definition_id" = Uuid, Path, description = "Configuration definition UUID")),
    request_body = SetConfigValueRequest, responses((status = 200, body = ConfigValueResponse)),
    security(("bearer_auth" = [])))]
pub async fn set_value(
    db: TenantDb,
    Path(definition_id): Path<Uuid>,
    Json(input): Json<SetConfigValueRequest>,
) -> Result<impl IntoResponse, AppError> {
    let row = tenant_config_service::set_value(
        &db.txn,
        db.tenant_id,
        definition_id,
        input.scope_id,
        input.value,
    )
    .await?;
    db.txn.commit().await?;
    Ok(Json(ConfigValueResponse {
        id: row.id,
        definition_id: row.definition_id,
        scope_type: row.scope_type,
        scope_id: row.scope_id,
        value: row.config_value,
    }))
}

pub async fn resolve(
    db: TenantDb,
    Path(definition_id): Path<Uuid>,
    Query(query): Query<ResolveQuery>,
) -> Result<impl IntoResponse, AppError> {
    let value =
        tenant_config_service::resolve(&db.txn, db.tenant_id, definition_id, query.scope_id)
            .await?;
    db.txn.commit().await?;
    Ok(Json(serde_json::json!({"value": value})))
}

pub async fn deactivate(
    db: TenantDb,
    Path(value_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    tenant_config_service::deactivate_value(&db.txn, db.tenant_id, value_id).await?;
    db.txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
