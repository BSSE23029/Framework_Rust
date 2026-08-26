use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait,
    IntoActiveModel, QueryFilter, Set,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    entity::{plan_permission_groups, plan_quotas, plans, tenant_subscriptions, tenants},
    error::AppError,
};

pub struct CreatePlanInput {
    pub code: String,
    pub name: String,
    pub billing_interval: String,
    pub price_cents: i64,
    pub currency: String,
    pub trial_days: i32,
    pub features: Value,
}

pub async fn create<C: ConnectionTrait>(
    db: &C,
    input: CreatePlanInput,
) -> Result<plans::Model, AppError> {
    if !matches!(
        input.billing_interval.as_str(),
        "monthly" | "yearly" | "one_time" | "lifetime"
    ) || input.code.trim().is_empty()
        || input.name.trim().is_empty()
        || input.price_cents < 0
        || input.trial_days < 0
    {
        return Err(AppError::InvalidInput("invalid plan definition".into()));
    }
    Ok(plans::ActiveModel {
        code: Set(input.code.trim().to_ascii_lowercase()),
        name: Set(input.name.trim().into()),
        billing_interval: Set(input.billing_interval),
        price_cents: Set(input.price_cents),
        currency: Set(input.currency.trim().to_ascii_uppercase()),
        trial_days: Set(input.trial_days),
        features: Set(input.features),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

pub async fn set_quota<C: ConnectionTrait>(
    db: &C,
    plan_id: Uuid,
    resource: &str,
    limit: i64,
) -> Result<plan_quotas::Model, AppError> {
    if resource.trim().is_empty() || limit < -1 {
        return Err(AppError::InvalidInput("invalid quota".into()));
    }
    plans::Entity::find_by_id(plan_id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound)?;
    if let Some(row) = plan_quotas::Entity::find()
        .filter(plan_quotas::Column::PlanId.eq(plan_id))
        .filter(plan_quotas::Column::Resource.eq(resource))
        .one(db)
        .await?
    {
        let mut active = row.into_active_model();
        active.limit_value = Set(limit);
        active.status = Set("active".into());
        active.deleted_at = Set(None);
        active.deleted_by = Set(None);
        return Ok(active.update(db).await?);
    }
    Ok(plan_quotas::ActiveModel {
        plan_id: Set(plan_id),
        resource: Set(resource.trim().into()),
        limit_value: Set(limit),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

pub async fn link_permission_group(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    plan_id: Uuid,
    permission_group_id: Uuid,
) -> Result<(), AppError> {
    if plan_permission_groups::Entity::find_by_id((plan_id, permission_group_id))
        .one(txn)
        .await?
        .is_none()
    {
        plan_permission_groups::ActiveModel {
            plan_id: Set(plan_id),
            permission_group_id: Set(permission_group_id),
            tenant_id: Set(tenant_id),
            ..Default::default()
        }
        .insert(txn)
        .await?;
    }
    Ok(())
}

pub async fn subscribe(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    plan_id: Uuid,
    start_trial: bool,
) -> Result<tenant_subscriptions::Model, AppError> {
    let plan = plans::Entity::find_by_id(plan_id)
        .filter(plans::Column::Status.eq("active"))
        .filter(plans::Column::DeletedAt.is_null())
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let now = Utc::now().fixed_offset();
    let trial = start_trial && plan.trial_days > 0;
    let state = if trial { "trial" } else { "active" };
    let trial_ends =
        trial.then(|| (Utc::now() + Duration::days(plan.trial_days.into())).fixed_offset());
    let ends_at = if trial {
        None
    } else {
        subscription_end(&plan.billing_interval)
    };
    let previous = tenant_subscriptions::Entity::find()
        .filter(tenant_subscriptions::Column::TenantId.eq(tenant_id))
        .filter(tenant_subscriptions::Column::Status.eq("active"))
        .all(txn)
        .await?;
    for row in previous {
        let mut active = row.into_active_model();
        active.status = Set("inactive".into());
        active.update(txn).await?;
    }
    let subscription = tenant_subscriptions::ActiveModel {
        tenant_id: Set(tenant_id),
        plan_id: Set(plan.id),
        subscription_state: Set(state.into()),
        starts_at: Set(now),
        trial_ends_at: Set(trial_ends),
        ends_at: Set(ends_at),
        auto_renew: Set(plan.is_auto_renewable),
        ..Default::default()
    }
    .insert(txn)
    .await?;
    let tenant = tenants::Entity::find_by_id(tenant_id)
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut active = tenant.into_active_model();
    active.current_subscription_plan_id = Set(Some(plan.id));
    active.subscription_status = Set(state.into());
    active.trial_ends_at = Set(trial_ends);
    active.subscription_starts_at = Set(Some(now));
    active.subscription_ends_at = Set(ends_at);
    active.is_active = Set(true);
    active.status = Set("active".into());
    active.update(txn).await?;
    Ok(subscription)
}

fn subscription_end(interval: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    match interval {
        "monthly" => Some((Utc::now() + Duration::days(30)).fixed_offset()),
        "yearly" => Some((Utc::now() + Duration::days(365)).fixed_offset()),
        _ => None,
    }
}
