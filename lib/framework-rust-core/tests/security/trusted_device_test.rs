use framework_rust_core::{
    config::Config,
    db,
    entity::{tenants, trusted_devices},
    error::AppError,
    services::{role_service, trusted_device_service, user_service},
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;

#[tokio::test]
async fn trusted_device_complete_cycle() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let database = db::connect(&config.database_url, 5, 1).await.expect("db");
    Migrator::up(&database, None).await.expect("migrations");
    let suffix = Uuid::new_v4();
    let tenant = tenants::ActiveModel {
        name: Set(format!("Devices {suffix}")),
        slug: Set(format!("devices-{suffix}")),
        encryption_key: Set("trusted-device-test-key-32-chars".into()),
        ..Default::default()
    }
    .insert(&database)
    .await
    .expect("tenant");
    let setup = db::begin_tenant_txn(&database, tenant.id)
        .await
        .expect("setup txn");
    let user = user_service::create(
        &setup,
        user_service::CreateUserInput {
            tenant_id: tenant.id,
            email_encrypted: format!("encrypted-{suffix}"),
            email_lookup: format!("lookup-{suffix}"),
            password_hash: "test-password-hash".into(),
        },
    )
    .await
    .expect("user");
    let role = role_service::create(
        &setup,
        role_service::CreateRoleInput {
            tenant_id: tenant.id,
            name: format!("device-owner-{suffix}"),
            description: "Device owner".into(),
        },
    )
    .await
    .expect("role");
    role_service::assign(&setup, user.id, role.id, tenant.id)
        .await
        .expect("role assignment");
    setup.commit().await.expect("setup commit");

    let (txn, actor) = db::begin_user_tenant_txn(&database, tenant.id, Some(user.id))
        .await
        .expect("actor txn");
    let actor = actor.expect("actor urdd");
    let issued = trusted_device_service::trust(
        &txn,
        trusted_device_service::TrustDeviceInput {
            tenant_id: tenant.id,
            user_id: user.id,
            name: "Personal laptop".into(),
            fingerprint: Some("browser-fingerprint".into()),
            user_agent: Some("Test Browser".into()),
            ip_address: Some("127.0.0.1".into()),
            ttl_days: 30,
        },
    )
    .await
    .expect("trust device");
    assert_ne!(issued.device.token_hash, issued.token);
    assert_eq!(issued.device.created_by, Some(actor));
    assert_eq!(
        trusted_device_service::list(&txn, tenant.id, user.id)
            .await
            .expect("list")
            .len(),
        1
    );
    assert!(trusted_device_service::verify(
        &txn,
        tenant.id,
        user.id,
        &issued.token,
        Some("wrong-fingerprint")
    )
    .await
    .expect("wrong fingerprint")
    .is_none());
    let verified = trusted_device_service::verify(
        &txn,
        tenant.id,
        user.id,
        &issued.token,
        Some("browser-fingerprint"),
    )
    .await
    .expect("verify")
    .expect("trusted");
    assert!(verified.last_used_at.is_some());

    trusted_device_service::revoke(&txn, tenant.id, user.id, issued.device.id)
        .await
        .expect("revoke");
    let revoked = trusted_devices::Entity::find_by_id(issued.device.id)
        .one(&txn)
        .await
        .expect("revoked query")
        .expect("revoked device");
    assert_eq!(revoked.status, "inactive");
    assert_eq!(revoked.deleted_by, Some(actor));
    assert!(revoked.deleted_at.is_some());
    assert!(trusted_device_service::verify(
        &txn,
        tenant.id,
        user.id,
        &issued.token,
        Some("browser-fingerprint")
    )
    .await
    .expect("revoked verify")
    .is_none());
    let restored = trusted_device_service::restore(&txn, tenant.id, user.id, issued.device.id)
        .await
        .expect("restore");
    assert_eq!(restored.status, "active");
    assert!(restored.deleted_at.is_none());
    txn.commit().await.expect("cycle commit");

    let other = tenants::ActiveModel {
        name: Set(format!("Other Devices {suffix}")),
        slug: Set(format!("other-devices-{suffix}")),
        encryption_key: Set("other-device-test-key-32-characters".into()),
        ..Default::default()
    }
    .insert(&database)
    .await
    .expect("other tenant");
    let other_txn = db::begin_tenant_txn(&database, other.id)
        .await
        .expect("other txn");
    assert!(trusted_devices::Entity::find_by_id(issued.device.id)
        .one(&other_txn)
        .await
        .expect("isolated query")
        .is_none());
    let error = trusted_device_service::revoke(&other_txn, other.id, user.id, issued.device.id)
        .await
        .expect_err("cross-tenant revoke denied");
    assert!(matches!(error, AppError::NotFound));
    other_txn.rollback().await.expect("other rollback");
}
