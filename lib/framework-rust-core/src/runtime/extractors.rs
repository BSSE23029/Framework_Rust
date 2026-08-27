use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use sea_orm::DatabaseTransaction;
use uuid::Uuid;

use crate::{
    db,
    error::AppError,
    middleware::{auth::AuthUser, tenant::TenantContext},
    state::AppState,
};

/// Axum extractor that opens a tenant-scoped transaction with RLS active.
///
/// Handlers that declare `TenantDb` cannot accidentally use a raw connection
/// that bypasses row-level security. The transaction is automatically scoped to
/// the tenant from the `X-Tenant-ID` / `X-Tenant-Slug` header.
///
/// Usage:
/// ```text
/// pub async fn handler(db: TenantDb) -> Result<impl IntoResponse, AppError> {
///     let TenantDb { txn, tenant_id, tenant_key } = db;
///     // ... use &txn for SeaORM queries ...
///     txn.commit().await?;
/// }
/// ```
pub struct TenantDb {
    pub txn: DatabaseTransaction,
    pub tenant_id: Uuid,
    /// Verified contextual identity used by audit triggers.
    pub actor_urdd_id: Option<Uuid>,
    /// Decrypted per-tenant encryption key.
    pub tenant_key: String,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for TenantDb
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let ctx = parts
            .extensions
            .get::<TenantContext>()
            .cloned()
            .ok_or(AppError::Unauthorized)?;
        let user_id = parts.extensions.get::<AuthUser>().map(|auth| auth.user_id);
        let requested = if user_id.is_some() {
            parts
                .headers
                .get("x-framework-rust-urdd-id")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| Uuid::parse_str(value).ok())
        } else {
            None
        };
        let (txn, actor_urdd_id) =
            db::begin_actor_tenant_txn(&app_state.db, ctx.tenant_id, user_id, requested).await?;
        if requested.is_some() && actor_urdd_id.is_none() {
            return Err(AppError::Unauthorized);
        }
        Ok(Self {
            txn,
            tenant_id: ctx.tenant_id,
            actor_urdd_id,
            tenant_key: ctx.tenant_key,
        })
    }
}
