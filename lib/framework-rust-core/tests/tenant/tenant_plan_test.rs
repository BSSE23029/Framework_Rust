use chrono::{Duration, Utc};
use framework_rust_core::{
    config::Config,
    db,
    entity::{plan_quotas, tenant_subscriptions, tenant_usage, tenants},
    error::AppError,
    services::{plan_service, quota_service, tenant_lifecycle_service, user_service},
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn tenant_plan_quota_and_lifecycle_complete_cycle() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let database = db::connect(&config.database_url, 5, 1).await.expect("db");
    Migrator::up(&database, None).await.expect("migrations");
    let suffix = Uuid::new_v4();
    let plan = plan_service::create(
        &database,
        plan_service::CreatePlanInput {
            code: format!("starter-{suffix}"),
            name: "Starter".into(),
            billing_interval: "monthly".into(),
            price_cents: 1900,
            currency: "usd".into(),
            trial_days: 14,
            features: json!({"passwordless": true}),
        },
    )
    .await
    .expect("plan");
    let users_quota = plan_service::set_quota(&database, plan.id, "users", 1)
        .await
        .expect("users quota");
    plan_service::set_quota(&database, plan.id, "storage_mb", 100)
        .await
        .expect("storage quota");
    let updated_quota = plan_service::set_quota(&database, plan.id, "users", 1)
        .await
        .expect("quota update");
    assert_eq!(users_quota.id, updated_quota.id);

    let tenant = tenants::ActiveModel {
        name: Set(format!("Plan Tenant {suffix}")),
        slug: Set(format!("plan-tenant-{suffix}")),
        encryption_key: Set("tenant-plan-test-key-32-characters".into()),
        ..Default::default()
    }
    .insert(&database)
    .await
    .expect("tenant");
    let txn = db::begin_tenant_txn(&database, tenant.id)
        .await
        .expect("subscription txn");
    let subscription = plan_service::subscribe(&txn, tenant.id, plan.id, true)
        .await
        .expect("trial subscription");
    assert_eq!(subscription.subscription_state, "trial");
    let user = user_service::create(
        &txn,
        user_service::CreateUserInput {
            tenant_id: tenant.id,
            email_encrypted: format!("encrypted-{suffix}"),
            email_lookup: format!("lookup-{suffix}"),
            password_hash: "test-password-hash".into(),
        },
    )
    .await
    .expect("first user within quota");
    let second = user_service::create(
        &txn,
        user_service::CreateUserInput {
            tenant_id: tenant.id,
            email_encrypted: format!("second-encrypted-{suffix}"),
            email_lookup: format!("second-lookup-{suffix}"),
            password_hash: "test-password-hash".into(),
        },
    )
    .await
    .expect_err("second user exceeds quota");
    assert!(
        matches!(second, AppError::QuotaExceeded { resource, limit: 1 } if resource == "users")
    );
    assert_eq!(
        quota_service::reserve(&txn, tenant.id, "storage_mb", 80)
            .await
            .expect("reserve"),
        80
    );
    assert!(matches!(
        quota_service::reserve(&txn, tenant.id, "storage_mb", 21).await,
        Err(AppError::QuotaExceeded { resource, limit: 100 }) if resource == "storage_mb"
    ));
    txn.commit().await.expect("quota cycle commit");
    assert!(
        tenant_usage::Entity::find_by_id((tenant.id, "storage_mb".to_owned()))
            .one(&database)
            .await
            .expect("usage query")
            .is_some()
    );

    let now = Utc::now().fixed_offset();
    set_trial_end(&database, tenant.id, now + Duration::days(1)).await;
    let reminder = tenant_lifecycle_service::reconcile(&database, now, None, 1, 7)
        .await
        .expect("reminder reconcile");
    assert_eq!(reminder.reminders, 1);
    assert_eq!(
        tenant_lifecycle_service::reconcile(&database, now, None, 1, 7)
            .await
            .expect("idempotent reconcile")
            .reminders,
        0
    );

    set_trial_end(&database, tenant.id, now).await;
    assert_eq!(
        tenant_lifecycle_service::reconcile(&database, now, None, 1, 7)
            .await
            .expect("warning reconcile")
            .warnings,
        1
    );
    set_trial_end(&database, tenant.id, now - Duration::days(1)).await;
    assert_eq!(
        tenant_lifecycle_service::reconcile(&database, now, None, 1, 7)
            .await
            .expect("expiry reconcile")
            .expired,
        1
    );
    let expired = tenants::Entity::find_by_id(tenant.id)
        .one(&database)
        .await
        .expect("expired query")
        .expect("expired tenant");
    assert!(!expired.is_active);
    assert_eq!(expired.status, "inactive");
    assert_eq!(expired.subscription_status, "expired");

    let mut renewed = expired.into_active_model();
    renewed.subscription_status = Set("trial".into());
    renewed.trial_ends_at = Set(Some(now + Duration::days(2)));
    renewed.update(&database).await.expect("renew dates");
    assert_eq!(
        tenant_lifecycle_service::reconcile(&database, now, None, 1, 7)
            .await
            .expect("reactivation reconcile")
            .reactivated,
        1
    );
    let active = tenants::Entity::find_by_id(tenant.id)
        .one(&database)
        .await
        .expect("active query")
        .expect("active tenant");
    assert!(active.is_active);
    assert_eq!(active.status, "active");
    let txn = db::begin_tenant_txn(&database, tenant.id)
        .await
        .expect("state txn");
    let state = tenant_subscriptions::Entity::find_by_id(subscription.id)
        .one(&txn)
        .await
        .expect("subscription query")
        .expect("subscription");
    assert_eq!(state.subscription_state, "trial");
    assert_eq!(user.tenant_id, tenant.id);
    txn.rollback().await.expect("state rollback");
    assert_eq!(
        plan_quotas::Entity::find_by_id(users_quota.id)
            .one(&database)
            .await
            .expect("quota read")
            .expect("quota")
            .limit_value,
        1
    );

    let paid_tenant = tenants::ActiveModel {
        name: Set(format!("Paid Tenant {suffix}")),
        slug: Set(format!("paid-tenant-{suffix}")),
        encryption_key: Set("paid-tenant-test-key-32-characters".into()),
        ..Default::default()
    }
    .insert(&database)
    .await
    .expect("paid tenant");
    let paid_txn = db::begin_tenant_txn(&database, paid_tenant.id)
        .await
        .expect("paid txn");
    plan_service::subscribe(&paid_txn, paid_tenant.id, plan.id, false)
        .await
        .expect("paid subscription");
    paid_txn.commit().await.expect("paid commit");
    set_subscription_end(&database, paid_tenant.id, now - Duration::days(6)).await;
    assert_eq!(
        tenant_lifecycle_service::reconcile(&database, now, None, 1, 7)
            .await
            .expect("subscription grace")
            .expired,
        0
    );
    set_subscription_end(&database, paid_tenant.id, now - Duration::days(7)).await;
    assert_eq!(
        tenant_lifecycle_service::reconcile(&database, now, None, 1, 7)
            .await
            .expect("subscription expiry")
            .expired,
        1
    );

    let row = tenants::Entity::find_by_id(paid_tenant.id)
        .one(&database)
        .await
        .expect("paid query")
        .expect("paid tenant");
    let mut suspended = row.into_active_model();
    suspended.subscription_status = Set("suspended".into());
    suspended.subscription_ends_at = Set(Some(now + Duration::days(30)));
    suspended.update(&database).await.expect("suspend tenant");
    assert_eq!(
        tenant_lifecycle_service::reconcile(&database, now, None, 1, 7)
            .await
            .expect("terminal state reconcile")
            .reactivated,
        0
    );
}

async fn set_trial_end(
    database: &sea_orm::DatabaseConnection,
    tenant_id: Uuid,
    end: chrono::DateTime<chrono::FixedOffset>,
) {
    let row = tenants::Entity::find_by_id(tenant_id)
        .one(database)
        .await
        .expect("tenant query")
        .expect("tenant");
    let mut active = row.into_active_model();
    active.trial_ends_at = Set(Some(end));
    active.subscription_status = Set("trial".into());
    active.update(database).await.expect("set trial end");
}

async fn set_subscription_end(
    database: &sea_orm::DatabaseConnection,
    tenant_id: Uuid,
    end: chrono::DateTime<chrono::FixedOffset>,
) {
    let row = tenants::Entity::find_by_id(tenant_id)
        .one(database)
        .await
        .expect("tenant query")
        .expect("tenant");
    let mut active = row.into_active_model();
    active.subscription_ends_at = Set(Some(end));
    active.subscription_status = Set("active".into());
    active.update(database).await.expect("set subscription end");
}
