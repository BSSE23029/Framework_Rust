use framework_rust_core::{
    config::Config,
    db,
    entity::{one_time_tokens, tenants},
    error::AppError,
    services::{passwordless_service, role_service, user_service},
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use uuid::Uuid;

#[tokio::test]
async fn passwordless_email_otp_complete_cycle() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let database = db::connect(&config.database_url, 5, 1).await.expect("db");
    Migrator::up(&database, None).await.expect("migrations");
    let suffix = Uuid::new_v4();
    let tenant = tenants::ActiveModel {
        name: Set(format!("Passwordless {suffix}")),
        slug: Set(format!("passwordless-{suffix}")),
        encryption_key: Set("passwordless-test-key-32-characters".into()),
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
            name: format!("passwordless-user-{suffix}"),
            description: "Passwordless user".into(),
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
    let actor = actor.expect("actor");
    let first = passwordless_service::issue(&txn, tenant.id, user.id)
        .await
        .expect("first OTP");
    assert_eq!(first.len(), 6);
    assert!(first.chars().all(|character| character.is_ascii_digit()));
    let second = passwordless_service::issue(&txn, tenant.id, user.id)
        .await
        .expect("replacement OTP");
    let records = one_time_tokens::Entity::find()
        .filter(one_time_tokens::Column::UserId.eq(user.id))
        .filter(one_time_tokens::Column::TokenType.eq(one_time_tokens::TYPE_PASSWORDLESS_LOGIN))
        .order_by_asc(one_time_tokens::Column::CreatedAt)
        .all(&txn)
        .await
        .expect("token records");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].status, "inactive");
    assert_eq!(records[0].deleted_by, Some(actor));
    assert!(matches!(
        passwordless_service::consume(&txn, tenant.id, user.id, &first).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        passwordless_service::consume(&txn, tenant.id, user.id, "000000").await,
        Err(AppError::Unauthorized)
    ));
    let attempted = one_time_tokens::Entity::find_by_id(records[1].id)
        .one(&txn)
        .await
        .expect("attempt query")
        .expect("attempted token");
    assert_eq!(attempted.attempt_count, 2);
    passwordless_service::consume(&txn, tenant.id, user.id, &second)
        .await
        .expect("consume valid OTP");
    let consumed = one_time_tokens::Entity::find_by_id(records[1].id)
        .one(&txn)
        .await
        .expect("consumed query")
        .expect("consumed token");
    assert!(consumed.used_at.is_some());
    assert_eq!(consumed.status, "inactive");
    assert_eq!(consumed.deleted_by, Some(actor));
    assert!(matches!(
        passwordless_service::consume(&txn, tenant.id, user.id, &second).await,
        Err(AppError::Unauthorized)
    ));
    txn.commit().await.expect("cycle commit");

    let other = tenants::ActiveModel {
        name: Set(format!("Other Passwordless {suffix}")),
        slug: Set(format!("other-passwordless-{suffix}")),
        encryption_key: Set("other-passwordless-key-32-characters".into()),
        ..Default::default()
    }
    .insert(&database)
    .await
    .expect("other tenant");
    let other_txn = db::begin_tenant_txn(&database, other.id)
        .await
        .expect("other txn");
    assert!(one_time_tokens::Entity::find_by_id(records[1].id)
        .one(&other_txn)
        .await
        .expect("isolated query")
        .is_none());
    assert!(matches!(
        passwordless_service::consume(&other_txn, other.id, user.id, &second).await,
        Err(AppError::Unauthorized)
    ));
    other_txn.rollback().await.expect("other rollback");
}
