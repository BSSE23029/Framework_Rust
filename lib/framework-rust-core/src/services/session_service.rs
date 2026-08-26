use chrono::Utc;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

use crate::{entity::sessions, error::AppError};

pub struct SessionData {
    pub email: String,
    pub ip: Option<String>,
}

pub async fn create(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    user_id: Uuid,
    data: SessionData,
    ttl_days: i64,
) -> Result<String, AppError> {
    let session_id = Uuid::new_v4().to_string();
    let now = Utc::now().fixed_offset();
    let expires_at = (Utc::now() + chrono::Duration::days(ttl_days)).fixed_offset();

    sessions::ActiveModel {
        id: Set(session_id.clone()),
        tenant_id: Set(tenant_id),
        user_id: Set(user_id),
        data: Set(serde_json::json!({
            "email": data.email,
            "ip": data.ip,
        })),
        expires_at: Set(expires_at),
        created_at: Set(now),
        last_seen_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(session_id)
}

pub async fn find_valid(
    db: &DatabaseConnection,
    id: &str,
) -> Result<Option<sessions::Model>, AppError> {
    let now = Utc::now().fixed_offset();
    Ok(sessions::Entity::find_by_id(id)
        .filter(sessions::Column::ExpiresAt.gt(now))
        .filter(sessions::Column::Status.eq("active"))
        .filter(sessions::Column::DeletedAt.is_null())
        .one(db)
        .await?)
}

pub async fn touch(db: &DatabaseConnection, id: &str) -> Result<(), AppError> {
    sessions::Entity::update_many()
        .col_expr(
            sessions::Column::LastSeenAt,
            Expr::value(Utc::now().fixed_offset()),
        )
        .filter(sessions::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn delete<C: ConnectionTrait>(db: &C, id: &str) -> Result<(), AppError> {
    let row = sessions::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut active: sessions::ActiveModel = row.into();
    active.status = Set("inactive".into());
    active.update(db).await?;
    Ok(())
}

pub async fn list_by_tenant(
    db: &DatabaseConnection,
    tenant_id: Uuid,
) -> Result<Vec<sessions::Model>, AppError> {
    let now = Utc::now().fixed_offset();
    Ok(sessions::Entity::find()
        .filter(sessions::Column::TenantId.eq(tenant_id))
        .filter(sessions::Column::ExpiresAt.gt(now))
        .filter(sessions::Column::Status.eq("active"))
        .filter(sessions::Column::DeletedAt.is_null())
        .all(db)
        .await?)
}

pub async fn cleanup_expired(db: &DatabaseConnection) -> Result<u64, AppError> {
    let now = Utc::now().fixed_offset();
    let result = sessions::Entity::delete_many()
        .filter(sessions::Column::ExpiresAt.lt(now))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}
