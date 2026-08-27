use framework_rust_core::{
    config::Config,
    db,
    entity::{tenant_config_values, tenants},
    error::AppError,
    services::{role_service, tenant_config_service, user_service},
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn typed_tenant_configuration_complete_cycle() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let database = db::connect(&config.database_url, 5, 1).await.expect("db");
    Migrator::up(&database, None).await.expect("migrations");
    let suffix = Uuid::new_v4();
    let tenant = tenants::ActiveModel {
        name: Set(format!("Config {suffix}")),
        slug: Set(format!("config-{suffix}")),
        encryption_key: Set("typed-config-test-key-32-characters".into()),
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
    let role = role_service::create(
        &setup,
        role_service::CreateRoleInput {
            tenant_id: tenant.id,
            name: format!("config-admin-{suffix}"),
            description: "Config admin".into(),
        },
    )
    .await
    .expect("role");
    role_service::assign(&setup, user.id, role.id, tenant.id)
        .await
        .expect("assign role");
    setup.commit().await.expect("setup commit");

    let (txn, actor) = db::begin_user_tenant_txn(&database, tenant.id, Some(user.id))
        .await
        .expect("actor txn");
    let actor = actor.expect("actor");
    let definition = tenant_config_service::define(
        &txn,
        tenant_config_service::DefineInput {
            tenant_id: tenant.id,
            config_key: "session_timeout".into(),
            display_name: "Session Timeout".into(),
            description: "Idle minutes".into(),
            category: "security".into(),
            scope_type: "tenant".into(),
            value_type: tenant_config_service::ConfigValueType::Integer,
            is_required: true,
            is_multi_value: false,
            default_value: Some(json!(15)),
            allowed_values: None,
            constraints: json!({"min": 5, "max": 120}),
        },
    )
    .await
    .expect("definition");
    assert_eq!(definition.created_by, Some(actor));
    assert_eq!(
        tenant_config_service::resolve(&txn, tenant.id, definition.id, None)
            .await
            .expect("default resolve"),
        Some(json!(15))
    );
    assert!(matches!(
        tenant_config_service::set_value(&txn, tenant.id, definition.id, None, json!("thirty"))
            .await,
        Err(AppError::InvalidInput(_))
    ));
    assert!(matches!(
        tenant_config_service::set_value(&txn, tenant.id, definition.id, None, json!(121)).await,
        Err(AppError::InvalidInput(_))
    ));
    let value = tenant_config_service::set_value(&txn, tenant.id, definition.id, None, json!(30))
        .await
        .expect("typed value");
    assert_eq!(value.created_by, Some(actor));
    let updated = tenant_config_service::set_value(&txn, tenant.id, definition.id, None, json!(45))
        .await
        .expect("update value");
    assert_eq!(updated.id, value.id);
    assert_eq!(
        tenant_config_service::resolve(&txn, tenant.id, definition.id, None)
            .await
            .expect("explicit resolve"),
        Some(json!(45))
    );
    tenant_config_service::deactivate_value(&txn, tenant.id, value.id)
        .await
        .expect("deactivate");
    let inactive = tenant_config_values::Entity::find_by_id(value.id)
        .one(&txn)
        .await
        .expect("inactive query")
        .expect("inactive value");
    assert_eq!(inactive.status, "inactive");
    assert_eq!(inactive.deleted_by, Some(actor));
    assert!(inactive.deleted_at.is_some());
    assert_eq!(
        tenant_config_service::resolve(&txn, tenant.id, definition.id, None)
            .await
            .expect("fallback resolve"),
        Some(json!(15))
    );
    let restored =
        tenant_config_service::set_value(&txn, tenant.id, definition.id, None, json!(60))
            .await
            .expect("restore value");
    assert_eq!(restored.id, value.id);
    assert!(restored.deleted_at.is_none());

    let scope_id = Uuid::new_v4();
    let choices = tenant_config_service::define(
        &txn,
        tenant_config_service::DefineInput {
            tenant_id: tenant.id,
            config_key: "login_methods".into(),
            display_name: "Login Methods".into(),
            description: "Allowed methods".into(),
            category: "authentication".into(),
            scope_type: "application".into(),
            value_type: tenant_config_service::ConfigValueType::String,
            is_required: false,
            is_multi_value: true,
            default_value: None,
            allowed_values: Some(json!(["password", "otp", "passkey"])),
            constraints: json!({"min_length": 3}),
        },
    )
    .await
    .expect("multi definition");
    assert!(matches!(
        tenant_config_service::set_value(
            &txn,
            tenant.id,
            choices.id,
            Some(scope_id),
            json!(["otp", "unknown"])
        )
        .await,
        Err(AppError::InvalidInput(_))
    ));
    tenant_config_service::set_value(
        &txn,
        tenant.id,
        choices.id,
        Some(scope_id),
        json!(["otp", "passkey"]),
    )
    .await
    .expect("scoped multi value");
    txn.commit().await.expect("cycle commit");

    let other = tenants::ActiveModel {
        name: Set(format!("Other Config {suffix}")),
        slug: Set(format!("other-config-{suffix}")),
        encryption_key: Set("other-config-test-key-32-characters".into()),
        ..Default::default()
    }
    .insert(&database)
    .await
    .expect("other tenant");
    let other_txn = db::begin_tenant_txn(&database, other.id)
        .await
        .expect("other txn");
    assert!(tenant_config_values::Entity::find_by_id(value.id)
        .one(&other_txn)
        .await
        .expect("isolated query")
        .is_none());
    assert!(matches!(
        tenant_config_service::resolve(&other_txn, other.id, definition.id, None).await,
        Err(AppError::NotFound)
    ));
    other_txn.rollback().await.expect("other rollback");
}
