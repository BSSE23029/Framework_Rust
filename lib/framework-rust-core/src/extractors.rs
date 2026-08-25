use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::{
    db,
    entity::user_role_designation_departments,
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
        let txn = db::begin_tenant_txn(&app_state.db, ctx.tenant_id).await?;
        let actor_urdd_id = if let Some(auth) = parts.extensions.get::<AuthUser>() {
            let requested = parts
                .headers
                .get("x-framework-rust-urdd-id")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| Uuid::parse_str(value).ok());

            let mut query = user_role_designation_departments::Entity::find()
                .filter(user_role_designation_departments::Column::TenantId.eq(ctx.tenant_id))
                .filter(user_role_designation_departments::Column::UserId.eq(auth.user_id))
                .filter(user_role_designation_departments::Column::Status.eq("active"))
                .filter(user_role_designation_departments::Column::DeletedAt.is_null());

            if let Some(id) = requested {
                query = query.filter(user_role_designation_departments::Column::Id.eq(id));
            }

            let actor = query
                .order_by_asc(user_role_designation_departments::Column::CreatedAt)
                .one(&txn)
                .await?;

            if requested.is_some() && actor.is_none() {
                return Err(AppError::Unauthorized);
            }

            if let Some(actor) = actor {
                db::set_actor_context(&txn, actor.id).await?;
                Some(actor.id)
            } else {
                None
            }
        } else {
            None
        };
        Ok(Self {
            txn,
            tenant_id: ctx.tenant_id,
            actor_urdd_id,
            tenant_key: ctx.tenant_key,
        })
    }
}
