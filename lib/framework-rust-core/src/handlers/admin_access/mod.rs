pub(crate) mod organizations;
pub(crate) mod permissions;

pub use organizations::*;
pub use permissions::*;

use axum::http::HeaderMap;
use sea_orm::DatabaseTransaction;
use uuid::Uuid;

use crate::{error::AppError, handlers::admin_auth, state::AppState};

async fn transaction(
    state: &AppState,
    headers: &HeaderMap,
    tenant_id: Uuid,
) -> Result<DatabaseTransaction, AppError> {
    let scope = admin_auth::require_admin(headers, &state.config, state.master_tenant_id)?;
    if scope.is_some_and(|allowed| allowed != tenant_id) {
        return Err(AppError::Forbidden);
    }
    admin_auth::begin_tenant_txn(state, headers, tenant_id).await
}
