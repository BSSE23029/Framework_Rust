use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    error::AppError,
    services::{permission_group_service, permission_scope_service},
    state::AppState,
};

use super::transaction;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreatePermissionGroupRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

pub async fn list_groups(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let rows = permission_group_service::list_groups(&txn, tenant_id).await?;
    txn.commit().await?;
    Ok(Json(rows))
}

pub async fn create_group(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(input): Json<CreatePermissionGroupRequest>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let row =
        permission_group_service::create_group(&txn, tenant_id, &input.name, &input.description)
            .await?;
    txn.commit().await?;
    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn add_group_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, group_id, permission_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    permission_group_service::add_permission(&txn, tenant_id, group_id, permission_id).await?;
    txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn link_rdd_group(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, rdd_id, group_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    permission_group_service::link_rdd(&txn, tenant_id, rdd_id, group_id).await?;
    txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreatePermissionScopeRequest {
    pub permission_id: Uuid,
    pub name: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    #[serde(default = "default_conditions")]
    pub conditions: Value,
}

fn default_conditions() -> Value {
    serde_json::json!({})
}

pub async fn list_scopes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let rows = permission_group_service::list_scopes(&txn, tenant_id).await?;
    txn.commit().await?;
    Ok(Json(rows))
}

pub async fn create_scope(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(input): Json<CreatePermissionScopeRequest>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let row = permission_scope_service::create(
        &txn,
        permission_scope_service::CreateScopeInput {
            tenant_id,
            permission_id: input.permission_id,
            name: input.name,
            resource_type: input.resource_type,
            resource_id: input.resource_id,
            conditions: input.conditions,
        },
    )
    .await?;
    txn.commit().await?;
    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn assign_scope(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, urdd_id, permission_id, scope_id)): Path<(Uuid, Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    permission_group_service::assign_scope(&txn, tenant_id, urdd_id, permission_id, scope_id)
        .await?;
    txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
