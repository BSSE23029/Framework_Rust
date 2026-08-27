use axum::{extract::State, response::IntoResponse, Extension, Json};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    db, error::AppError, middleware::auth::AuthUser, services::access_context_service,
    state::AppState,
};

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AccessContextResponse {
    pub urdd_id: Uuid,
    pub rdd_id: Uuid,
    pub role_id: Uuid,
    pub role_name: String,
    pub designation_id: Uuid,
    pub designation_name: String,
    pub department_id: Uuid,
    pub department_name: String,
}

#[utoipa::path(
    get,
    path = "/users/me/access-contexts",
    tag = "users",
    responses((status = 200, body = [AccessContextResponse])),
    security(("bearer_auth" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    let txn = db::begin_tenant_txn(&state.db, auth.tenant_id).await?;
    let contexts = access_context_service::list_for_user(&txn, auth.tenant_id, auth.user_id)
        .await?
        .into_iter()
        .map(|context| AccessContextResponse {
            urdd_id: context.urdd_id,
            rdd_id: context.rdd_id,
            role_id: context.role_id,
            role_name: context.role_name,
            designation_id: context.designation_id,
            designation_name: context.designation_name,
            department_id: context.department_id,
            department_name: context.department_name,
        })
        .collect::<Vec<_>>();
    txn.commit().await?;
    Ok(Json(contexts))
}
