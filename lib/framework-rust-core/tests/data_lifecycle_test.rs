use framework_rust_core::{
    config::Config,
    db,
    entity::{roles, tenants, user_role_designation_departments},
    services::{role_service, user_service},
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait, IntoActiveModel,
    QueryFilter, Set, Statement, TryGetable,
};
use uuid::Uuid;

async fn database() -> sea_orm::DatabaseConnection {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let database = db::connect(&config.database_url, 5, 1).await.expect("db");
    Migrator::up(&database, None).await.expect("migrations");
    database
}

#[tokio::test]
async fn every_application_table_has_lifecycle_schema() {
    let database = database().await;
    let missing_columns = database
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
SELECT table_name
FROM information_schema.tables t
WHERE table_schema = 'public'
  AND table_type = 'BASE TABLE'
  AND table_name <> 'seaql_migrations'
  AND EXISTS (
      SELECT 1
      FROM unnest(ARRAY['status','created_at','created_by','updated_at',
                              'updated_by','deleted_at','deleted_by']) required(column_name)
      WHERE NOT EXISTS (
          SELECT 1 FROM information_schema.columns c
          WHERE c.table_schema = t.table_schema
            AND c.table_name = t.table_name
            AND c.column_name = required.column_name
      )
  )
ORDER BY table_name
"#,
        ))
        .await
        .expect("schema query");
    let missing: Vec<String> = missing_columns
        .iter()
        .map(|row| String::try_get(row, "", "table_name").expect("table name"))
        .collect();
    assert!(
        missing.is_empty(),
        "tables missing lifecycle columns: {missing:?}"
    );

    let invalid_actor_fks = database
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
SELECT c.table_name, c.column_name
FROM information_schema.columns c
WHERE c.table_schema = 'public'
  AND c.column_name IN ('created_by', 'updated_by', 'deleted_by')
  AND NOT EXISTS (
      SELECT 1
      FROM information_schema.table_constraints tc
      JOIN information_schema.key_column_usage kcu
        ON tc.constraint_name = kcu.constraint_name
       AND tc.constraint_schema = kcu.constraint_schema
      JOIN information_schema.constraint_column_usage ccu
        ON tc.constraint_name = ccu.constraint_name
       AND tc.constraint_schema = ccu.constraint_schema
      WHERE tc.constraint_type = 'FOREIGN KEY'
        AND tc.table_schema = c.table_schema
        AND tc.table_name = c.table_name
        AND kcu.column_name = c.column_name
        AND ccu.table_name = 'user_role_designation_departments'
        AND ccu.column_name = 'id'
  )
ORDER BY c.table_name, c.column_name
"#,
        ))
        .await
        .expect("foreign key query");
    assert!(
        invalid_actor_fks.is_empty(),
        "actor columns must reference URDD"
    );
}

#[tokio::test]
async fn performer_urdd_controls_create_update_delete_and_restore() {
    let database = database().await;
    let suffix = Uuid::new_v4();
    let tenant = tenants::ActiveModel {
        name: Set(format!("Lifecycle {suffix}")),
        slug: Set(format!("lifecycle-{suffix}")),
        encryption_key: Set("lifecycle-key-at-least-32-characters".into()),
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
    for name in ["Actor", "Spoof"] {
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
            .expect("assignment");
    }
    let urdds = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::UserId.eq(user.id))
        .all(&setup)
        .await
        .expect("URDDs");
    setup.commit().await.expect("setup commit");
    let actor = urdds[0].id;
    let spoofed = urdds[1].id;

    let (txn, resolved) =
        db::begin_actor_tenant_txn(&database, tenant.id, Some(user.id), Some(actor))
            .await
            .expect("actor transaction");
    assert_eq!(resolved, Some(actor));
    let created = roles::ActiveModel {
        tenant_id: Set(tenant.id),
        name: Set(format!("Audited-{suffix}")),
        description: Set("audited lifecycle".into()),
        created_by: Set(Some(spoofed)),
        updated_by: Set(Some(spoofed)),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .expect("create role");
    assert_eq!(created.created_by, Some(actor));
    assert_eq!(created.updated_by, Some(actor));

    let mut update = created.into_active_model();
    update.description = Set("updated".into());
    update.updated_by = Set(Some(spoofed));
    let updated = update.update(&txn).await.expect("update role");
    assert_eq!(updated.updated_by, Some(actor));

    role_service::delete(&txn, updated.id)
        .await
        .expect("soft delete role");
    let deleted = roles::Entity::find_by_id(updated.id)
        .one(&txn)
        .await
        .expect("deleted lookup")
        .expect("deleted row retained");
    assert_eq!(deleted.status, "inactive");
    assert!(deleted.deleted_at.is_some());
    assert_eq!(deleted.deleted_by, Some(actor));

    let mut restore = deleted.into_active_model();
    restore.status = Set("active".into());
    let restored = restore.update(&txn).await.expect("restore role");
    assert!(restored.deleted_at.is_none());
    assert!(restored.deleted_by.is_none());
    assert_eq!(restored.updated_by, Some(actor));
    txn.rollback().await.expect("rollback test data");

    let target = tenants::ActiveModel {
        name: Set(format!("Lifecycle Target {suffix}")),
        slug: Set(format!("lifecycle-target-{suffix}")),
        encryption_key: Set("target-lifecycle-key-at-least-32-chars".into()),
        ..Default::default()
    }
    .insert(&database)
    .await
    .expect("target tenant");
    let (cross_tenant, resolved) = db::begin_cross_tenant_actor_txn(
        &database,
        target.id,
        tenant.id,
        Some(user.id),
        Some(actor),
    )
    .await
    .expect("cross-tenant actor transaction");
    assert_eq!(resolved, Some(actor));
    let child_role = roles::ActiveModel {
        tenant_id: Set(target.id),
        name: Set(format!("Child Administered {suffix}")),
        description: Set("created by master actor".into()),
        created_by: Set(Some(spoofed)),
        ..Default::default()
    }
    .insert(&cross_tenant)
    .await
    .expect("cross-tenant role");
    assert_eq!(child_role.created_by, Some(actor));
    cross_tenant
        .rollback()
        .await
        .expect("cross-tenant rollback");
}
