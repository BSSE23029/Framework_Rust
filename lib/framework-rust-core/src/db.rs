use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection,
    DatabaseTransaction, DbErr, Statement, TransactionTrait,
};
use std::time::Duration;
use tracing::info;
use uuid::Uuid;

use crate::entity::user_role_designation_departments;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

/// Opens a transaction, switches to the non-superuser `framework_rust_rls` role, and sets
/// `app.tenant_id` so PostgreSQL RLS policies activate.
/// The superuser session bypasses RLS; switching role drops that privilege for the
/// duration of this transaction so FORCE ROW LEVEL SECURITY actually fires.
pub async fn begin_tenant_txn(
    db: &DatabaseConnection,
    tenant_id: Uuid,
) -> Result<DatabaseTransaction, DbErr> {
    let txn = db.begin().await?;
    txn.execute_unprepared("SET LOCAL ROLE framework_rust_rls")
        .await?;
    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT set_config('app.tenant_id', $1, true)",
        [tenant_id.to_string().into()],
    ))
    .await?;
    Ok(txn)
}

/// Adds a verified contextual identity to an already tenant-scoped transaction.
/// Audit triggers read this transaction-local value; request payloads never control
/// `created_by`, `updated_by`, or `deleted_by` directly.
pub async fn set_actor_context(
    txn: &DatabaseTransaction,
    actor_urdd_id: Uuid,
) -> Result<(), DbErr> {
    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT set_config('app.actor_urdd_id', $1, true)",
        [actor_urdd_id.to_string().into()],
    ))
    .await?;
    Ok(())
}

/// Opens a tenant transaction and derives the audit actor from the authenticated
/// user. The caller supplies a user identity, never a URDD identity.
pub async fn begin_user_tenant_txn(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    user_id: Option<Uuid>,
) -> Result<(DatabaseTransaction, Option<Uuid>), DbErr> {
    begin_actor_tenant_txn(db, tenant_id, user_id, None).await
}

pub async fn begin_actor_tenant_txn(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    requested_urdd_id: Option<Uuid>,
) -> Result<(DatabaseTransaction, Option<Uuid>), DbErr> {
    begin_cross_tenant_actor_txn(db, tenant_id, tenant_id, user_id, requested_urdd_id).await
}

/// Resolves the performer inside their authenticated tenant, then opens the
/// target tenant transaction. This keeps master-admin actions attributable to
/// their real URDD during legitimate cross-tenant administration.
pub async fn begin_cross_tenant_actor_txn(
    db: &DatabaseConnection,
    target_tenant_id: Uuid,
    actor_tenant_id: Uuid,
    user_id: Option<Uuid>,
    requested_urdd_id: Option<Uuid>,
) -> Result<(DatabaseTransaction, Option<Uuid>), DbErr> {
    let actor_txn = begin_tenant_txn(db, actor_tenant_id).await?;
    let actor = match user_id {
        Some(user_id) => {
            let mut query = user_role_designation_departments::Entity::find()
                .filter(user_role_designation_departments::Column::TenantId.eq(actor_tenant_id))
                .filter(user_role_designation_departments::Column::UserId.eq(user_id))
                .filter(user_role_designation_departments::Column::Status.eq("active"))
                .filter(user_role_designation_departments::Column::DeletedAt.is_null());
            if let Some(requested) = requested_urdd_id {
                query = query.filter(user_role_designation_departments::Column::Id.eq(requested));
            }
            query
                .order_by_asc(user_role_designation_departments::Column::CreatedAt)
                .one(&actor_txn)
                .await?
                .map(|row| row.id)
        }
        None => None,
    };
    actor_txn.rollback().await?;
    let txn = begin_tenant_txn(db, target_tenant_id).await?;
    if let Some(actor_id) = actor {
        set_actor_context(&txn, actor_id).await?;
    }
    Ok((txn, actor))
}

pub async fn connect(
    database_url: &str,
    max_connections: u32,
    min_connections: u32,
) -> Result<DatabaseConnection, DbErr> {
    let mut opts = ConnectOptions::new(database_url.to_owned());
    opts.max_connections(max_connections)
        .min_connections(min_connections)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .sqlx_logging(false);

    let db = Database::connect(opts).await?;
    info!("Connected to PostgreSQL ✓");
    Ok(db)
}
