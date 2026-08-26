use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{error::AppError, services::organization_service, state::AppState};

use super::transaction;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateDepartmentRequest {
    pub name: String,
    pub code: String,
    #[serde(default)]
    pub description: String,
}

pub async fn list_departments(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let rows = organization_service::list_departments(&txn, tenant_id).await?;
    txn.commit().await?;
    Ok(Json(rows))
}

pub async fn create_department(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(input): Json<CreateDepartmentRequest>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let row = organization_service::create_department(
        &txn,
        tenant_id,
        &input.name,
        &input.code,
        &input.description,
    )
    .await?;
    txn.commit().await?;
    Ok((StatusCode::CREATED, Json(row)))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateDesignationRequest {
    pub name: String,
    pub code: String,
    #[serde(default)]
    pub description: String,
    pub senior_designation_id: Option<Uuid>,
}

pub async fn list_designations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let rows = organization_service::list_designations(&txn, tenant_id).await?;
    txn.commit().await?;
    Ok(Json(rows))
}

pub async fn create_designation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(input): Json<CreateDesignationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let row = organization_service::create_designation(
        &txn,
        tenant_id,
        &input.name,
        &input.code,
        &input.description,
        input.senior_designation_id,
    )
    .await?;
    txn.commit().await?;
    Ok((StatusCode::CREATED, Json(row)))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateRddRequest {
    pub role_id: Uuid,
    pub designation_id: Uuid,
    pub department_id: Uuid,
}

pub async fn list_rdds(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let rows = organization_service::list_rdds(&txn, tenant_id).await?;
    txn.commit().await?;
    Ok(Json(rows))
}

pub async fn create_rdd(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(input): Json<CreateRddRequest>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let row = organization_service::create_rdd(
        &txn,
        tenant_id,
        input.role_id,
        input.designation_id,
        input.department_id,
    )
    .await?;
    txn.commit().await?;
    Ok((StatusCode::CREATED, Json(row)))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AssignUserContextRequest {
    pub user_id: Uuid,
    pub rdd_id: Uuid,
    #[serde(default = "default_attributes")]
    pub attributes: Value,
}

fn default_attributes() -> Value {
    serde_json::json!({})
}

pub async fn list_urdds(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let rows = organization_service::list_urdds(&txn, tenant_id).await?;
    txn.commit().await?;
    Ok(Json(rows))
}

pub async fn assign_user_context(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(input): Json<AssignUserContextRequest>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    let row = organization_service::assign_user_context(
        &txn,
        tenant_id,
        input.user_id,
        input.rdd_id,
        input.attributes,
    )
    .await?;
    txn.commit().await?;
    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn revoke_user_context(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, urdd_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let txn = transaction(&state, &headers, tenant_id).await?;
    organization_service::revoke_user_context(&txn, tenant_id, urdd_id).await?;
    txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
