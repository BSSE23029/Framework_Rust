use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::AppError, handlers::admin_auth, services::tenant_hierarchy_service, state::AppState,
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetTenantHierarchyRequest {
    pub tenant_type: String,
    pub parent_tenant_id: Option<Uuid>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TenantHierarchyResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub tenant_type: String,
    pub parent_tenant_id: Option<Uuid>,
    pub status: String,
}

pub async fn set_hierarchy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(input): Json<SetTenantHierarchyRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_tenant_admin(&state, &headers, tenant_id)?;
    let txn = admin_auth::begin_tenant_txn(&state, &headers, tenant_id).await?;
    tenant_hierarchy_service::set_hierarchy(
        &txn,
        tenant_id,
        input.tenant_type.trim(),
        input.parent_tenant_id,
    )
    .await?;
    txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_children(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_tenant_admin(&state, &headers, tenant_id)?;
    let txn = admin_auth::begin_tenant_txn(&state, &headers, tenant_id).await?;
    let children = tenant_hierarchy_service::list_children(&txn, tenant_id).await?;
    txn.commit().await?;
    Ok(Json(
        children
            .into_iter()
            .map(|tenant| TenantHierarchyResponse {
                id: tenant.id,
                name: tenant.name,
                slug: tenant.slug,
                tenant_type: tenant.tenant_type,
                parent_tenant_id: tenant.parent_tenant_id,
                status: tenant.status,
            })
            .collect::<Vec<_>>(),
    ))
}

fn require_tenant_admin(
    state: &AppState,
    headers: &HeaderMap,
    tenant_id: Uuid,
) -> Result<(), AppError> {
    let scope = admin_auth::require_admin(headers, &state.config, state.master_tenant_id)?;
    if scope.is_some_and(|allowed| allowed != tenant_id) {
        return Err(AppError::Unauthorized);
    }
    Ok(())
}
