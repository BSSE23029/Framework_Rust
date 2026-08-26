use framework_rust_core::{
    config::Config,
    db,
    entity::tenants,
    services::{
        lockout_service, mfa_service, oauth_service, one_time_token_service,
        password_history_service, password_policy_service, role_service, token_service,
        user_service,
    },
};
use sea_orm::{sea_query::OnConflict, ConnectionTrait, EntityTrait, Set};
use std::collections::HashMap;
use uuid::Uuid;

#[path = "auth_test/lifecycle_security.rs"]
mod lifecycle_security;
#[path = "auth_test/policy_and_rbac.rs"]
mod policy_and_rbac;
#[path = "auth_test/token_flows.rs"]
mod token_flows;

async fn setup() -> (sea_orm::DatabaseConnection, Config, Uuid, String) {
    dotenvy::dotenv().ok();
    let cfg = Config::from_env().expect("config");
    let db = db::connect(&cfg.database_url, 5, 1).await.expect("db");

    let tenant_key_plain = "dev-test-tenant-key-32chars-long!";
    let encrypted_key = hefesto::encrypt(
        tenant_key_plain,
        &cfg.tenant_wrap_key,
        &cfg.master_encryption_key,
    )
    .expect("encrypt");

    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    tenants::Entity::insert(tenants::ActiveModel {
        id: Set(tenant_id),
        name: Set("Dev Tenant".into()),
        slug: Set("dev".into()),
        encryption_key: Set(encrypted_key),
        ..Default::default()
    })
    .on_conflict(
        OnConflict::column(tenants::Column::Id)
            .update_column(tenants::Column::EncryptionKey)
            .to_owned(),
    )
    .exec(&db)
    .await
    .expect("upsert tenant");

    (db, cfg, tenant_id, tenant_key_plain.to_string())
}

async fn create_test_user(
    db: &sea_orm::DatabaseConnection,
    cfg: &Config,
    tenant_id: Uuid,
    tenant_key: &str,
    email: &str,
    password: &str,
) -> framework_rust_core::entity::users::Model {
    let _ = db
        .execute_unprepared(&format!("SET app.tenant_id = '{tenant_id}'"))
        .await;
    let lookup = hefesto::hash_for_lookup(email, tenant_key).expect("hash lookup");
    let _ = db
        .execute_unprepared(&format!(
            "DELETE FROM users WHERE tenant_id = '{tenant_id}' AND email_lookup = '{lookup}'"
        ))
        .await;

    let txn = db::begin_tenant_txn(db, tenant_id).await.unwrap();
    let user = user_service::create(
        &txn,
        user_service::CreateUserInput {
            tenant_id,
            email_encrypted: hefesto::encrypt(email, tenant_key, &cfg.master_encryption_key)
                .unwrap(),
            email_lookup: lookup,
            password_hash: hefesto::hash_password(password).unwrap(),
        },
    )
    .await
    .unwrap();
    txn.commit().await.unwrap();
    user
}
