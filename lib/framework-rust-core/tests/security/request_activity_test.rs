use framework_rust_core::{
    config::Config,
    db,
    entity::{request_activity, tenants, user_role_designation_departments},
    error::AppError,
    services::{request_activity_service, role_service, user_service},
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

#[tokio::test]
async fn request_activity_complete_cycle() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let database = db::connect(&config.database_url, 5, 1).await.expect("db");
    Migrator::up(&database, None).await.expect("migrations");
    let suffix = Uuid::new_v4();
    let tenant = tenants::ActiveModel {
        name: Set(format!("Activity {suffix}")),
        slug: Set(format!("activity-{suffix}")),
        encryption_key: Set("request-activity-key-32-characters".into()),
        ..Default::default()
    }
    .insert(&database)
    .await
    .expect("tenant");
    let setup = db::begin_tenant_txn(&database, tenant.id)
        .await
        .expect("setup");
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
    for name in ["Operator", "Auditor"] {
        let role = role_service::create(
            &setup,
            role_service::CreateRoleInput {
                tenant_id: tenant.id,
                name: format!("{name}-{suffix}"),
                description: name.into(),
            },
        )
        .await
        .expect("role");
        role_service::assign(&setup, user.id, role.id, tenant.id)
            .await
            .expect("assign");
    }
    let urdds = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant.id))
        .filter(user_role_designation_departments::Column::UserId.eq(user.id))
        .all(&setup)
        .await
        .expect("urdds");
    assert_eq!(urdds.len(), 2);
    let selected = urdds[1].id;
    setup.commit().await.expect("setup commit");

    let request_id = Uuid::new_v4();
    let recorded = request_activity_service::record(
        &database,
        request_activity_service::RecordInput {
            tenant_id: tenant.id,
            user_id: user.id,
            requested_urdd_id: Some(selected),
            request_id,
            http_method: "POST".into(),
            request_path: "/settings/config/definition-id".into(),
            response_status: 204,
            response_time_ms: 17,
            ip_address: Some("127.0.0.1".into()),
            user_agent: Some("Test Browser secret-free".into()),
            platform: Some("tui".into()),
            platform_version: Some("0.5.0".into()),
        },
    )
    .await
    .expect("record")
    .expect("activity row");
    assert_eq!(recorded.actor_urdd_id, selected);
    assert_eq!(recorded.created_by, Some(selected));
    assert_eq!(recorded.request_id, request_id);
    assert_eq!(recorded.request_path, "/settings/config/definition-id");
    let txn = db::begin_tenant_txn(&database, tenant.id)
        .await
        .expect("list txn");
    let listed = request_activity_service::list_for_user(&txn, tenant.id, user.id, 1000)
        .await
        .expect("list activity");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].response_status, 204);
    txn.rollback().await.expect("list rollback");

    assert!(matches!(
        request_activity_service::record(
            &database,
            request_activity_service::RecordInput {
                tenant_id: tenant.id,
                user_id: user.id,
                requested_urdd_id: Some(Uuid::new_v4()),
                request_id: Uuid::new_v4(),
                http_method: "GET".into(),
                request_path: "/users/me".into(),
                response_status: 200,
                response_time_ms: 1,
                ip_address: None,
                user_agent: None,
                platform: None,
                platform_version: None,
            }
        )
        .await,
        Err(AppError::Unauthorized)
    ));
    let other = tenants::ActiveModel {
        name: Set(format!("Other Activity {suffix}")),
        slug: Set(format!("other-activity-{suffix}")),
        encryption_key: Set("other-activity-key-32-characters".into()),
        ..Default::default()
    }
    .insert(&database)
    .await
    .expect("other tenant");
    let other_txn = db::begin_tenant_txn(&database, other.id)
        .await
        .expect("other txn");
    assert!(request_activity::Entity::find_by_id(recorded.id)
        .one(&other_txn)
        .await
        .expect("isolated query")
        .is_none());
    other_txn.rollback().await.expect("other rollback");
}
