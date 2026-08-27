use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entity::tenant_domains, error::AppError, handlers::admin_auth,
    services::tenant_hierarchy_service, state::AppState,
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateDomainRequest {
    pub domain_name: String,
    pub domain_type: String,
    #[serde(default)]
    pub is_primary: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DomainResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub domain_name: String,
    pub domain_type: String,
    pub is_primary: bool,
    pub is_verified: bool,
    pub verified_at: Option<String>,
    pub ssl_enabled: bool,
    pub status: String,
}

impl From<tenant_domains::Model> for DomainResponse {
    fn from(row: tenant_domains::Model) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            domain_name: row.domain_name,
            domain_type: row.domain_type,
            is_primary: row.is_primary,
            is_verified: row.is_verified,
            verified_at: row.verified_at.map(|value| value.to_rfc3339()),
            ssl_enabled: row.ssl_enabled,
            status: row.status,
        }
    }
}

fn authorize(headers: &HeaderMap, state: &AppState, tenant_id: Uuid) -> Result<(), AppError> {
    let scope = admin_auth::require_admin(headers, &state.config, state.master_tenant_id)?;
    if scope.is_some_and(|allowed| allowed != tenant_id) {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

async fn transaction(
    state: &AppState,
    headers: &HeaderMap,
    tenant_id: Uuid,
) -> Result<sea_orm::DatabaseTransaction, AppError> {
    authorize(headers, state, tenant_id)?;
    admin_auth::begin_tenant_txn(state, headers, tenant_id).await
}

#[utoipa::path(
    get,
    path = "/admin/tenants/{tenant_id}/domains",
    tag = "tenant domains",
    params(("tenant_id" = String, Path, description = "Tenant UUID")),
    responses((status = 200, body = Vec<DomainResponse>)),
    security(("admin_key" = []), ("bearer_auth" = []))
)]
pub async fn list_domains(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let rows = tenant_hierarchy_service::list_domains(&txn, tenant_id).await?;
    txn.commit().await?;
    Ok(Json(
        rows.into_iter()
            .map(DomainResponse::from)
            .collect::<Vec<_>>(),
    ))
}

#[utoipa::path(
    post,
    path = "/admin/tenants/{tenant_id}/domains",
    tag = "tenant domains",
    params(("tenant_id" = String, Path, description = "Tenant UUID")),
    request_body = CreateDomainRequest,
    responses((status = 201, body = DomainResponse)),
    security(("admin_key" = []), ("bearer_auth" = []))
)]
pub async fn create_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(payload): Json<CreateDomainRequest>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let domain = tenant_hierarchy_service::create_domain(
        &txn,
        tenant_id,
        &payload.domain_name,
        &payload.domain_type,
        payload.is_primary,
    )
    .await?;
    txn.commit().await?;
    Ok((StatusCode::CREATED, Json(DomainResponse::from(domain))))
}

async fn apply_action(
    state: AppState,
    headers: HeaderMap,
    tenant_id: Uuid,
    domain_id: Uuid,
    action: &str,
) -> Result<StatusCode, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    match action {
        "primary" => tenant_hierarchy_service::set_primary(&txn, tenant_id, domain_id).await?,
        "verify" => tenant_hierarchy_service::verify_domain(&txn, tenant_id, domain_id).await?,
        "deactivate" => {
            tenant_hierarchy_service::set_domain_active(&txn, tenant_id, domain_id, false).await?
        }
        "restore" => {
            tenant_hierarchy_service::set_domain_active(&txn, tenant_id, domain_id, true).await?
        }
        _ => return Err(AppError::NotFound),
    }
    txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn domain_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, domain_id, action)): Path<(Uuid, Uuid, String)>,
) -> Result<StatusCode, AppError> {
    apply_action(state, headers, tenant_id, domain_id, &action).await
}
