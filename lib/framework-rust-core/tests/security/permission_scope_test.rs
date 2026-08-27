use framework_rust_core::{
    config::Config,
    db,
    entity::{
        permission_scopes, tenants, user_role_designation_departments,
        user_role_designation_permission_scopes,
    },
    error::AppError,
    services::{permission_scope_service, permission_service, role_service, user_service},
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn resource_permission_scope_complete_cycle() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let database = db::connect(&config.database_url, 5, 1).await.expect("db");
    Migrator::up(&database, None).await.expect("migrations");
    let suffix = Uuid::new_v4();
    let tenant = tenants::ActiveModel {
        name: Set(format!("Scope {suffix}")),
        slug: Set(format!("scope-{suffix}")),
        encryption_key: Set("permission-scope-test-key-32-chars".into()),
        ..Default::default()
    }
    .insert(&database)
    .await
    .expect("tenant");
    let txn = db::begin_tenant_txn(&database, tenant.id)
        .await
        .expect("txn");
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
    .expect("user");
    let role = role_service::create(
        &txn,
        role_service::CreateRoleInput {
            tenant_id: tenant.id,
            name: format!("reader-{suffix}"),
            description: "Scoped reader".into(),
        },
    )
    .await
    .expect("role");
    let permission = permission_service::create(
        &txn,
        permission_service::CreatePermissionInput {
            tenant_id: tenant.id,
            name: format!("document-{suffix}:read"),
            description: "Read documents".into(),
        },
    )
    .await
    .expect("permission");
    permission_service::assign_to_role(&txn, role.id, permission.id, tenant.id)
        .await
        .expect("permission grant");
    role_service::assign(&txn, user.id, role.id, tenant.id)
        .await
        .expect("role assignment");
    let urdd_id = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant.id))
        .filter(user_role_designation_departments::Column::UserId.eq(user.id))
        .one(&txn)
        .await
        .expect("urdd query")
        .expect("urdd")
        .id;

    assert!(permission_scope_service::is_allowed(
        &txn,
        tenant.id,
        user.id,
        &permission.name,
        Some("document"),
        Some(Uuid::new_v4())
    )
    .await
    .expect("tenant access"));
    let tenant_scope = permission_scopes::Entity::find()
        .one(&txn)
        .await
        .expect("scope query")
        .expect("default tenant scope");
    db::set_actor_context(&txn, urdd_id).await.expect("actor");
    permission_scope_service::deactivate(&txn, tenant.id, tenant_scope.id)
        .await
        .expect("deactivate tenant scope");
    assert!(!permission_scope_service::is_allowed(
        &txn,
        tenant.id,
        user.id,
        &permission.name,
        Some("document"),
        Some(Uuid::new_v4())
    )
    .await
    .expect("denied without scope"));
    let deactivated = permission_scopes::Entity::find_by_id(tenant_scope.id)
        .one(&txn)
        .await
        .expect("deactivated query")
        .expect("deactivated scope");
    assert_eq!(deactivated.deleted_by, Some(urdd_id));
    assert!(deactivated.deleted_at.is_some());

    let type_scope = permission_scope_service::create(
        &txn,
        permission_scope_service::CreateScopeInput {
            tenant_id: tenant.id,
            permission_id: permission.id,
            name: "All documents".into(),
            resource_type: "document".into(),
            resource_id: None,
            conditions: json!({}),
        },
    )
    .await
    .expect("type scope");
    permission_scope_service::assign(&txn, tenant.id, urdd_id, permission.id, type_scope.id)
        .await
        .expect("type scope assignment");
    assert!(permission_scope_service::is_allowed(
        &txn,
        tenant.id,
        user.id,
        &permission.name,
        Some("document"),
        Some(Uuid::new_v4())
    )
    .await
    .expect("type access"));
    assert!(!permission_scope_service::is_allowed(
        &txn,
        tenant.id,
        user.id,
        &permission.name,
        Some("invoice"),
        Some(Uuid::new_v4())
    )
    .await
    .expect("wrong type denied"));

    permission_scope_service::deactivate(&txn, tenant.id, type_scope.id)
        .await
        .expect("deactivate type scope");
    let document_id = Uuid::new_v4();
    let exact_scope = permission_scope_service::create(
        &txn,
        permission_scope_service::CreateScopeInput {
            tenant_id: tenant.id,
            permission_id: permission.id,
            name: "One document".into(),
            resource_type: "document".into(),
            resource_id: Some(document_id),
            conditions: json!({"owner": true}),
        },
    )
    .await
    .expect("exact scope");
    permission_scope_service::assign(&txn, tenant.id, urdd_id, permission.id, exact_scope.id)
        .await
        .expect("exact scope assignment");
    assert!(permission_scope_service::is_allowed(
        &txn,
        tenant.id,
        user.id,
        &permission.name,
        Some("document"),
        Some(document_id)
    )
    .await
    .expect("exact access"));
    assert!(!permission_scope_service::is_allowed(
        &txn,
        tenant.id,
        user.id,
        &permission.name,
        Some("document"),
        Some(Uuid::new_v4())
    )
    .await
    .expect("other resource denied"));

    permission_scope_service::restore(&txn, tenant.id, tenant_scope.id)
        .await
        .expect("restore tenant scope");
    let link = user_role_designation_permission_scopes::Entity::find()
        .one(&txn)
        .await
        .expect("link query")
        .expect("scope link");
    assert_eq!(link.tenant_id, tenant.id);
    txn.commit().await.expect("commit cycle");

    let other = tenants::ActiveModel {
        name: Set(format!("Other {suffix}")),
        slug: Set(format!("other-scope-{suffix}")),
        encryption_key: Set("other-scope-test-key-32-characters".into()),
        ..Default::default()
    }
    .insert(&database)
    .await
    .expect("other tenant");
    let other_txn = db::begin_tenant_txn(&database, other.id)
        .await
        .expect("other txn");
    let error = permission_scope_service::assign(
        &other_txn,
        other.id,
        urdd_id,
        permission.id,
        exact_scope.id,
    )
    .await
    .expect_err("cross-tenant scope must be hidden");
    assert!(matches!(error, AppError::NotFound));
    other_txn.rollback().await.expect("rollback other txn");
}
