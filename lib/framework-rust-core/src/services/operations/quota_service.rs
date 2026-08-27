use crate::{
    entity::{plan_quotas, tenant_usage, tenants, users},
    error::AppError,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, QuerySelect, Set,
};
use uuid::Uuid;

pub async fn limit(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    resource: &str,
) -> Result<Option<i64>, AppError> {
    let tenant = tenants::Entity::find_by_id(tenant_id)
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let Some(plan_id) = tenant.current_subscription_plan_id else {
        return Ok(None);
    };
    Ok(plan_quotas::Entity::find()
        .filter(plan_quotas::Column::PlanId.eq(plan_id))
        .filter(plan_quotas::Column::Resource.eq(resource))
        .filter(plan_quotas::Column::Status.eq("active"))
        .filter(plan_quotas::Column::DeletedAt.is_null())
        .one(txn)
        .await?
        .map(|quota| quota.limit_value))
}

pub async fn ensure_user_capacity(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
) -> Result<(), AppError> {
    tenants::Entity::find_by_id(tenant_id)
        .lock_exclusive()
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let Some(limit) = limit(txn, tenant_id, "users").await? else {
        return Ok(());
    };
    if limit == -1 {
        return Ok(());
    }
    let count = users::Entity::find()
        .filter(users::Column::TenantId.eq(tenant_id))
        .filter(users::Column::IsActive.eq(true))
        .count(txn)
        .await? as i64;
    if count >= limit {
        return Err(AppError::QuotaExceeded {
            resource: "users".into(),
            limit,
        });
    }
    Ok(())
}

pub async fn reserve(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    resource: &str,
    amount: i64,
) -> Result<i64, AppError> {
    if amount <= 0 {
        return Err(AppError::InvalidInput(
            "quota reservation must be positive".into(),
        ));
    }
    tenants::Entity::find_by_id(tenant_id)
        .lock_exclusive()
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let quota = limit(txn, tenant_id, resource).await?;
    let row = tenant_usage::Entity::find_by_id((tenant_id, resource.to_owned()))
        .one(txn)
        .await?;
    let used = row.as_ref().map_or(0, |value| value.used_quantity);
    if let Some(limit) = quota.filter(|limit| *limit != -1) {
        if used.saturating_add(amount) > limit {
            return Err(AppError::QuotaExceeded {
                resource: resource.into(),
                limit,
            });
        }
    }
    let updated = match row {
        Some(row) => {
            let mut active = row.into_active_model();
            active.used_quantity = Set(used + amount);
            active.update(txn).await?
        }
        None => {
            tenant_usage::ActiveModel {
                tenant_id: Set(tenant_id),
                resource: Set(resource.into()),
                used_quantity: Set(amount),
                ..Default::default()
            }
            .insert(txn)
            .await?
        }
    };
    Ok(updated.used_quantity)
}
