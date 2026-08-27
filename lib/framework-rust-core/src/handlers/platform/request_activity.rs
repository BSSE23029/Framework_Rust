use crate::{
    error::AppError, extractors::TenantDb, middleware::auth::AuthUser,
    services::request_activity_service,
};
use axum::{extract::Query, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ActivityQuery {
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ActivityResponse {
    pub id: Uuid,
    pub request_id: Uuid,
    pub actor_urdd_id: Uuid,
    pub http_method: String,
    pub request_path: String,
    pub response_status: i32,
    pub response_time_ms: i64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub platform: Option<String>,
    pub platform_version: Option<String>,
    pub created_at: String,
}

#[utoipa::path(get, path = "/users/me/activity", tag = "user",
    responses((status = 200, body = [ActivityResponse])), security(("bearer_auth" = [])))]
pub async fn list(
    Extension(auth): Extension<AuthUser>,
    Query(query): Query<ActivityQuery>,
    db: TenantDb,
) -> Result<impl IntoResponse, AppError> {
    let rows = request_activity_service::list_for_user(
        &db.txn,
        db.tenant_id,
        auth.user_id,
        query.limit.unwrap_or(50),
    )
    .await?;
    db.txn.commit().await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| ActivityResponse {
                id: row.id,
                request_id: row.request_id,
                actor_urdd_id: row.actor_urdd_id,
                http_method: row.http_method,
                request_path: row.request_path,
                response_status: row.response_status,
                response_time_ms: row.response_time_ms,
                ip_address: row.ip_address,
                user_agent: row.user_agent,
                platform: row.platform,
                platform_version: row.platform_version,
                created_at: row.created_at.to_rfc3339(),
            })
            .collect::<Vec<_>>(),
    ))
}
