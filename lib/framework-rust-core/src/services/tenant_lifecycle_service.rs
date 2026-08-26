use chrono::{DateTime, Duration, FixedOffset, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use uuid::Uuid;

use crate::{
    db,
    entity::{tenant_lifecycle_notifications, tenant_subscriptions, tenants},
    error::AppError,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ReconcileResult {
    pub reactivated: u64,
    pub reminders: u64,
    pub warnings: u64,
    pub expired: u64,
}

pub async fn reconcile(
    database: &DatabaseConnection,
    now: DateTime<FixedOffset>,
    excluded_tenant_id: Option<Uuid>,
    trial_grace_days: i64,
    subscription_grace_days: i64,
) -> Result<ReconcileResult, AppError> {
    let mut result = ReconcileResult::default();
    let rows = tenants::Entity::find()
        .filter(tenants::Column::SubscriptionStatus.is_in(["trial", "active"]))
        .all(database)
        .await?;
    for tenant in rows {
        if Some(tenant.id) == excluded_tenant_id {
            continue;
        }
        let end = if tenant.subscription_status == "trial" {
            tenant.trial_ends_at
        } else {
            tenant.subscription_ends_at
        };
        let Some(end) = end else { continue };
        if end > now && (!tenant.is_active || tenant.status != "active") {
            let state = tenant.subscription_status.clone();
            let tenant_id = tenant.id;
            set_tenant_active(database, tenant, true, None).await?;
            sync_subscription_state(database, tenant_id, &state).await?;
            result.reactivated += 1;
            continue;
        }
        if !tenant.is_active || tenant.status == "inactive" {
            continue;
        }
        let days_until = (end.date_naive() - now.date_naive()).num_days();
        if days_until == 1 && record_stage(database, tenant.id, "ending_tomorrow", now).await? {
            result.reminders += 1;
        }
        if days_until == 0 && record_stage(database, tenant.id, "ending_today", now).await? {
            result.warnings += 1;
        }
        let grace = if tenant.subscription_status == "trial" {
            trial_grace_days.max(1)
        } else {
            subscription_grace_days.max(1)
        };
        if now >= end + Duration::days(grace) {
            expire(database, tenant).await?;
            result.expired += 1;
        }
    }
    Ok(result)
}

async fn set_tenant_active(
    database: &DatabaseConnection,
    tenant: tenants::Model,
    is_active: bool,
    subscription_status: Option<&str>,
) -> Result<(), AppError> {
    let mut active = tenant.into_active_model();
    active.is_active = Set(is_active);
    active.status = Set(if is_active { "active" } else { "inactive" }.into());
    if let Some(status) = subscription_status {
        active.subscription_status = Set(status.into());
    }
    active.update(database).await?;
    Ok(())
}

async fn expire(database: &DatabaseConnection, tenant: tenants::Model) -> Result<(), AppError> {
    let tenant_id = tenant.id;
    set_tenant_active(database, tenant, false, Some("expired")).await?;
    let txn = db::begin_tenant_txn(database, tenant_id).await?;
    if let Some(row) = tenant_subscriptions::Entity::find()
        .filter(tenant_subscriptions::Column::TenantId.eq(tenant_id))
        .filter(tenant_subscriptions::Column::Status.eq("active"))
        .one(&txn)
        .await?
    {
        let mut active = row.into_active_model();
        active.subscription_state = Set("expired".into());
        active.update(&txn).await?;
    }
    txn.commit().await?;
    Ok(())
}

async fn sync_subscription_state(
    database: &DatabaseConnection,
    tenant_id: Uuid,
    state: &str,
) -> Result<(), AppError> {
    let txn = db::begin_tenant_txn(database, tenant_id).await?;
    if let Some(row) = tenant_subscriptions::Entity::find()
        .filter(tenant_subscriptions::Column::TenantId.eq(tenant_id))
        .filter(tenant_subscriptions::Column::Status.eq("active"))
        .one(&txn)
        .await?
    {
        let mut active = row.into_active_model();
        active.subscription_state = Set(state.into());
        active.update(&txn).await?;
    }
    txn.commit().await?;
    Ok(())
}

async fn record_stage(
    database: &DatabaseConnection,
    tenant_id: Uuid,
    stage: &str,
    now: DateTime<FixedOffset>,
) -> Result<bool, AppError> {
    let txn = db::begin_tenant_txn(database, tenant_id).await?;
    let key = (tenant_id, stage.to_owned(), now.date_naive());
    if tenant_lifecycle_notifications::Entity::find_by_id(key)
        .one(&txn)
        .await?
        .is_some()
    {
        txn.rollback().await?;
        return Ok(false);
    }
    tenant_lifecycle_notifications::ActiveModel {
        tenant_id: Set(tenant_id),
        stage: Set(stage.into()),
        notification_date: Set(now.date_naive()),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    txn.commit().await?;
    Ok(true)
}

pub fn now() -> DateTime<FixedOffset> {
    Utc::now().fixed_offset()
}
