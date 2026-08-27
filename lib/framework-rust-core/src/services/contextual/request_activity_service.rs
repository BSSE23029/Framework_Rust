use crate::{db, entity::request_activity, error::AppError};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

pub struct RecordInput {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub requested_urdd_id: Option<Uuid>,
    pub request_id: Uuid,
    pub http_method: String,
    pub request_path: String,
    pub response_status: i32,
    pub response_time_ms: i64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub platform: Option<String>,
    pub platform_version: Option<String>,
}

pub async fn record(
    database: &DatabaseConnection,
    input: RecordInput,
) -> Result<Option<request_activity::Model>, AppError> {
    let (txn, actor) = db::begin_actor_tenant_txn(
        database,
        input.tenant_id,
        Some(input.user_id),
        input.requested_urdd_id,
    )
    .await?;
    let Some(actor) = actor else {
        txn.rollback().await?;
        return if input.requested_urdd_id.is_some() {
            Err(AppError::Unauthorized)
        } else {
            Ok(None)
        };
    };
    let row = request_activity::ActiveModel {
        tenant_id: Set(input.tenant_id),
        user_id: Set(input.user_id),
        actor_urdd_id: Set(actor),
        request_id: Set(input.request_id),
        http_method: Set(truncate(input.http_method, 16)),
        request_path: Set(truncate(input.request_path, 2048)),
        response_status: Set(input.response_status),
        response_time_ms: Set(input.response_time_ms.max(0)),
        ip_address: Set(input.ip_address.map(|v| truncate(v, 64))),
        user_agent: Set(input.user_agent.map(|v| truncate(v, 512))),
        platform: Set(input.platform.map(|v| truncate(v, 64))),
        platform_version: Set(input.platform_version.map(|v| truncate(v, 64))),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    txn.commit().await?;
    Ok(Some(row))
}

pub async fn list_for_user(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
    limit: u64,
) -> Result<Vec<request_activity::Model>, AppError> {
    Ok(request_activity::Entity::find()
        .filter(request_activity::Column::TenantId.eq(tenant_id))
        .filter(request_activity::Column::UserId.eq(user_id))
        .filter(request_activity::Column::Status.eq("active"))
        .order_by_desc(request_activity::Column::CreatedAt)
        .limit(limit.clamp(1, 100))
        .all(txn)
        .await?)
}

fn truncate(mut value: String, max_chars: usize) -> String {
    if value.chars().count() > max_chars {
        value = value.chars().take(max_chars).collect();
    }
    value
}
