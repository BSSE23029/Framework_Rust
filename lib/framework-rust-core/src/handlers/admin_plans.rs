use crate::{
    entity::{plan_quotas, plans, tenant_subscriptions},
    error::AppError,
    handlers::admin_auth,
    services::plan_service,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreatePlanRequest {
    pub code: String,
    pub name: String,
    pub billing_interval: String,
    pub price_cents: i64,
    pub currency: String,
    pub trial_days: i32,
    pub features: Value,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PlanResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub billing_interval: String,
    pub price_cents: i64,
    pub currency: String,
    pub trial_days: i32,
    pub features: Value,
}

impl From<plans::Model> for PlanResponse {
    fn from(row: plans::Model) -> Self {
        Self {
            id: row.id,
            code: row.code,
            name: row.name,
            billing_interval: row.billing_interval,
            price_cents: row.price_cents,
            currency: row.currency,
            trial_days: row.trial_days,
            features: row.features,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetQuotaRequest {
    pub limit: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct QuotaResponse {
    pub id: Uuid,
    pub resource: String,
    pub limit: i64,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SubscribeRequest {
    pub plan_id: Uuid,
    pub start_trial: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SubscriptionResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub plan_id: Uuid,
    pub state: String,
    pub starts_at: String,
    pub trial_ends_at: Option<String>,
    pub ends_at: Option<String>,
}

impl From<tenant_subscriptions::Model> for SubscriptionResponse {
    fn from(row: tenant_subscriptions::Model) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            plan_id: row.plan_id,
            state: row.subscription_state,
            starts_at: row.starts_at.to_rfc3339(),
            trial_ends_at: row.trial_ends_at.map(|v| v.to_rfc3339()),
            ends_at: row.ends_at.map(|v| v.to_rfc3339()),
        }
    }
}

fn authorize(headers: &HeaderMap, state: &AppState, target: Option<Uuid>) -> Result<(), AppError> {
    let scope = admin_auth::require_admin(headers, &state.config, state.master_tenant_id)?;
    if scope.is_some() && scope != target {
        return Err(AppError::Unauthorized);
    }
    Ok(())
}

#[utoipa::path(get, path = "/plans", tag = "plans", responses((status = 200, body = [PlanResponse])), security(("admin_key" = [])))]
pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    admin_auth::require_admin(&headers, &state.config, state.master_tenant_id)?;
    let txn = admin_auth::begin_global_txn(&state, &headers).await?;
    let rows = plans::Entity::find()
        .filter(plans::Column::Status.eq("active"))
        .filter(plans::Column::DeletedAt.is_null())
        .all(&txn)
        .await?;
    txn.commit().await?;
    Ok(Json(
        rows.into_iter().map(PlanResponse::from).collect::<Vec<_>>(),
    ))
}

#[utoipa::path(post, path = "/plans", tag = "plans", request_body = CreatePlanRequest,
    responses((status = 201, body = PlanResponse)), security(("admin_key" = [])))]
pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreatePlanRequest>,
) -> Result<impl IntoResponse, AppError> {
    authorize(&headers, &state, None)?;
    let txn = admin_auth::begin_global_txn(&state, &headers).await?;
    let row = plan_service::create(
        &txn,
        plan_service::CreatePlanInput {
            code: input.code,
            name: input.name,
            billing_interval: input.billing_interval,
            price_cents: input.price_cents,
            currency: input.currency,
            trial_days: input.trial_days,
            features: input.features,
        },
    )
    .await?;
    txn.commit().await?;
    Ok((StatusCode::CREATED, Json(PlanResponse::from(row))))
}

#[utoipa::path(put, path = "/plans/{plan_id}/quotas/{resource}", tag = "plans",
    request_body = SetQuotaRequest, params(("plan_id" = Uuid, Path, description = "Plan UUID"),
    ("resource" = String, Path, description = "Quota resource")), responses((status = 200, body = QuotaResponse)), security(("admin_key" = [])))]
pub async fn set_quota(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((plan_id, resource)): Path<(Uuid, String)>,
    Json(input): Json<SetQuotaRequest>,
) -> Result<impl IntoResponse, AppError> {
    authorize(&headers, &state, None)?;
    let txn = admin_auth::begin_global_txn(&state, &headers).await?;
    let row = plan_service::set_quota(&txn, plan_id, &resource, input.limit).await?;
    txn.commit().await?;
    Ok(Json(QuotaResponse {
        id: row.id,
        resource: row.resource,
        limit: row.limit_value,
    }))
}

#[utoipa::path(post, path = "/tenants/{tenant_id}/subscription", tag = "plans",
    request_body = SubscribeRequest, params(("tenant_id" = Uuid, Path, description = "Tenant UUID")),
    responses((status = 201, body = SubscriptionResponse)), security(("admin_key" = [])))]
pub async fn subscribe(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(input): Json<SubscribeRequest>,
) -> Result<impl IntoResponse, AppError> {
    authorize(&headers, &state, Some(tenant_id))?;
    let txn = admin_auth::begin_tenant_txn(&state, &headers, tenant_id).await?;
    let row = plan_service::subscribe(&txn, tenant_id, input.plan_id, input.start_trial).await?;
    txn.commit().await?;
    Ok((StatusCode::CREATED, Json(SubscriptionResponse::from(row))))
}

pub async fn list_quotas(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(plan_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    admin_auth::require_admin(&headers, &state.config, state.master_tenant_id)?;
    let txn = admin_auth::begin_global_txn(&state, &headers).await?;
    let rows = plan_quotas::Entity::find()
        .filter(plan_quotas::Column::PlanId.eq(plan_id))
        .filter(plan_quotas::Column::Status.eq("active"))
        .filter(plan_quotas::Column::DeletedAt.is_null())
        .all(&txn)
        .await?;
    txn.commit().await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| QuotaResponse {
                id: row.id,
                resource: row.resource,
                limit: row.limit_value,
            })
            .collect::<Vec<_>>(),
    ))
}
