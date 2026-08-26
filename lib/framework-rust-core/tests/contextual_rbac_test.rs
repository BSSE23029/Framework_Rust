use framework_rust_core::{
    config::Config,
    db,
    entity::{
        departments, designations, permission_groups, role_designation_departments, tenants,
        user_role_designation_departments, user_role_designation_permissions,
    },
    services::{access_context_service, permission_service, role_service, user_service},
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

#[tokio::test]
async fn contextual_rbac_complete_cycle() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let database = db::connect(&config.database_url, 5, 1).await.expect("db");
    Migrator::up(&database, None).await.expect("migrations");

    let suffix = Uuid::new_v4();
    let tenant = tenants::ActiveModel {
        name: Set(format!("Contextual RBAC {suffix}")),
        slug: Set(format!("contextual-rbac-{suffix}")),
        encryption_key: Set("contextual-rbac-test-key-32-chars".into()),
        ..Default::default()
    }
    .insert(&database)
    .await
    .expect("tenant");

    let txn = db::begin_tenant_txn(&database, tenant.id)
        .await
        .expect("tenant txn");
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
            name: format!("operator-{suffix}"),
            description: "Contextual operator".into(),
        },
    )
    .await
    .expect("role");
    let permission = permission_service::create(
        &txn,
        permission_service::CreatePermissionInput {
            tenant_id: tenant.id,
            name: format!("resource-{suffix}:read"),
            description: "Read a scoped resource".into(),
        },
    )
    .await
    .expect("permission");

    permission_service::assign_to_role(&txn, role.id, permission.id, tenant.id)
        .await
        .expect("grant permission");
    role_service::assign(&txn, user.id, role.id, tenant.id)
        .await
        .expect("assign role");

    let department = departments::Entity::find()
        .filter(departments::Column::TenantId.eq(tenant.id))
        .filter(departments::Column::Code.eq("GENERAL"))
        .one(&txn)
        .await
        .expect("department query")
        .expect("department");
    let designation = designations::Entity::find()
        .filter(designations::Column::TenantId.eq(tenant.id))
        .filter(designations::Column::Code.eq("MEMBER"))
        .one(&txn)
        .await
        .expect("designation query")
        .expect("designation");
    let rdd = role_designation_departments::Entity::find()
        .filter(role_designation_departments::Column::TenantId.eq(tenant.id))
        .filter(role_designation_departments::Column::RoleId.eq(role.id))
        .one(&txn)
        .await
        .expect("rdd query")
        .expect("rdd");
    assert_eq!(rdd.department_id, department.id);
    assert_eq!(rdd.designation_id, designation.id);

    let urdd = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant.id))
        .filter(user_role_designation_departments::Column::UserId.eq(user.id))
        .one(&txn)
        .await
        .expect("urdd query")
        .expect("urdd");
    let group = permission_groups::Entity::find()
        .filter(permission_groups::Column::TenantId.eq(tenant.id))
        .filter(permission_groups::Column::Name.eq(&role.name))
        .one(&txn)
        .await
        .expect("group query")
        .expect("permission group");
    assert_eq!(group.status, "active");

    let urdp = user_role_designation_permissions::Entity::find_by_id((urdd.id, permission.id))
        .one(&txn)
        .await
        .expect("urdp query")
        .expect("urdp");
    assert_eq!(urdp.status, "active");
    let contexts = access_context_service::list_for_user(&txn, tenant.id, user.id)
        .await
        .expect("access contexts");
    assert_eq!(contexts.len(), 1);
    assert_eq!(contexts[0].urdd_id, urdd.id);
    assert_eq!(contexts[0].rdd_id, rdd.id);
    assert_eq!(contexts[0].role_name, role.name);
    assert_eq!(contexts[0].designation_name, designation.name);
    assert_eq!(contexts[0].department_name, department.name);
    assert!(
        permission_service::list_names_for_user(&txn, user.id, tenant.id)
            .await
            .expect("effective permissions")
            .contains(&permission.name)
    );

    db::set_actor_context(&txn, urdd.id)
        .await
        .expect("actor context");
    permission_service::revoke_from_role(&txn, tenant.id, role.id, permission.id)
        .await
        .expect("revoke permission");
    let revoked = user_role_designation_permissions::Entity::find_by_id((urdd.id, permission.id))
        .one(&txn)
        .await
        .expect("revoked query")
        .expect("revoked urdp");
    assert_eq!(revoked.status, "inactive");
    assert_eq!(revoked.deleted_by, Some(urdd.id));
    assert!(revoked.deleted_at.is_some());
    assert!(
        permission_service::list_names_for_user(&txn, user.id, tenant.id)
            .await
            .expect("permissions after revoke")
            .is_empty()
    );

    permission_service::assign_to_role(&txn, role.id, permission.id, tenant.id)
        .await
        .expect("restore permission");
    role_service::revoke(&txn, tenant.id, user.id, role.id)
        .await
        .expect("revoke role");
    assert!(role_service::list_for_user(&txn, user.id, tenant.id)
        .await
        .expect("roles after revoke")
        .is_empty());

    role_service::assign(&txn, user.id, role.id, tenant.id)
        .await
        .expect("restore role");
    assert_eq!(
        permission_service::list_names_for_user(&txn, user.id, tenant.id)
            .await
            .expect("permissions after restore"),
        vec![permission.name]
    );
    txn.commit().await.expect("commit cycle");
}
