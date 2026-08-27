use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::AppError, extractors::TenantDb, middleware::auth::AuthUser,
    services::trusted_device_service, state::AppState,
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TrustDeviceRequest {
    pub name: String,
    pub fingerprint: Option<String>,
    pub ttl_days: Option<i64>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TrustedDeviceResponse {
    pub id: Uuid,
    pub name: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub trusted_at: String,
    pub last_used_at: Option<String>,
    pub expires_at: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TrustDeviceResponse {
    #[serde(flatten)]
    pub device: TrustedDeviceResponse,
    pub token: String,
}

impl From<crate::entity::trusted_devices::Model> for TrustedDeviceResponse {
    fn from(row: crate::entity::trusted_devices::Model) -> Self {
        Self {
            id: row.id,
            name: row.name,
            user_agent: row.user_agent,
            ip_address: row.ip_address,
            trusted_at: row.trusted_at.to_rfc3339(),
            last_used_at: row.last_used_at.map(|value| value.to_rfc3339()),
            expires_at: row.expires_at.to_rfc3339(),
        }
    }
}

#[utoipa::path(post, path = "/users/me/trusted-devices", tag = "user",
    request_body = TrustDeviceRequest,
    responses((status = 201, body = TrustDeviceResponse)),
    security(("bearer_auth" = [])))]
pub async fn trust_device(
    State(_state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    headers: HeaderMap,
    db: TenantDb,
    Json(payload): Json<TrustDeviceRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let issued = trusted_device_service::trust(
        &db.txn,
        trusted_device_service::TrustDeviceInput {
            tenant_id: db.tenant_id,
            user_id: auth.user_id,
            name: payload.name,
            fingerprint: payload.fingerprint,
            user_agent,
            ip_address: None,
            ttl_days: payload.ttl_days.unwrap_or(30),
        },
    )
    .await?;
    db.txn.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(TrustDeviceResponse {
            device: issued.device.into(),
            token: issued.token,
        }),
    ))
}

#[utoipa::path(get, path = "/users/me/trusted-devices", tag = "user",
    responses((status = 200, body = [TrustedDeviceResponse])),
    security(("bearer_auth" = [])))]
pub async fn list_devices(
    Extension(auth): Extension<AuthUser>,
    db: TenantDb,
) -> Result<impl IntoResponse, AppError> {
    let rows = trusted_device_service::list(&db.txn, db.tenant_id, auth.user_id).await?;
    db.txn.commit().await?;
    Ok(Json(
        rows.into_iter()
            .map(TrustedDeviceResponse::from)
            .collect::<Vec<_>>(),
    ))
}

#[utoipa::path(delete, path = "/users/me/trusted-devices/{device_id}", tag = "user",
    params(("device_id" = Uuid, Path, description = "Trusted device UUID")),
    responses((status = 204)),
    security(("bearer_auth" = [])))]
pub async fn revoke_device(
    Extension(auth): Extension<AuthUser>,
    Path(device_id): Path<Uuid>,
    db: TenantDb,
) -> Result<impl IntoResponse, AppError> {
    trusted_device_service::revoke(&db.txn, db.tenant_id, auth.user_id, device_id).await?;
    db.txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
