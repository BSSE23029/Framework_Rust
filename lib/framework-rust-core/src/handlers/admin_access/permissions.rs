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

#[utoipa::path(get, path = "/admin/tenants/{tenant_id}/access/permission-groups", tag = "admin-access", responses((status = 200, description = "Permission groups")), security(("admin_key" = [])))]
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

#[utoipa::path(post, path = "/admin/tenants/{tenant_id}/access/permission-groups", tag = "admin-access", request_body = CreatePermissionGroupRequest, responses((status = 201, description = "Permission group created")), security(("admin_key" = [])))]
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

#[utoipa::path(post, path = "/admin/tenants/{tenant_id}/access/permission-groups/{group_id}/permissions/{permission_id}", tag = "admin-access", responses((status = 204, description = "Permission added to group")), security(("admin_key" = [])))]
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

#[utoipa::path(post, path = "/admin/tenants/{tenant_id}/access/rdds/{rdd_id}/permission-groups/{group_id}", tag = "admin-access", responses((status = 204, description = "Group linked to RDD")), security(("admin_key" = [])))]
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

#[utoipa::path(get, path = "/admin/tenants/{tenant_id}/access/permission-scopes", tag = "admin-access", responses((status = 200, description = "Permission scopes")), security(("admin_key" = [])))]
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

#[utoipa::path(post, path = "/admin/tenants/{tenant_id}/access/permission-scopes", tag = "admin-access", request_body = CreatePermissionScopeRequest, responses((status = 201, description = "Permission scope created")), security(("admin_key" = [])))]
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

#[utoipa::path(post, path = "/admin/tenants/{tenant_id}/access/urdds/{urdd_id}/permissions/{permission_id}/scopes/{scope_id}", tag = "admin-access", responses((status = 204, description = "Scope assigned to contextual permission")), security(("admin_key" = [])))]
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
