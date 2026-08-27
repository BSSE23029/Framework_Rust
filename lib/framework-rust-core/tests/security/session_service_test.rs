use chrono::{Duration, Utc};
use framework_rust_core::{
    config::Config,
    db,
    entity::{sessions, tenants},
    services::{session_service, user_service},
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};
use uuid::Uuid;

#[tokio::test]
async fn session_touch_uses_a_scoped_seaorm_update() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let database = db::connect(&config.database_url, 5, 1).await.expect("db");
    Migrator::up(&database, None).await.expect("migrations");
    let suffix = Uuid::new_v4();
    let tenant = tenants::ActiveModel {
        name: Set(format!("Session {suffix}")),
        slug: Set(format!("session-{suffix}")),
        encryption_key: Set("session-service-test-key-32-chars".into()),
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
    txn.commit().await.expect("commit user");
    let session_id = session_service::create(
        &database,
        tenant.id,
        user.id,
        session_service::SessionData {
            email: "session@example.test".into(),
            ip: None,
        },
        1,
    )
    .await
    .expect("session");
    let row = sessions::Entity::find_by_id(&session_id)
        .one(&database)
        .await
        .expect("session query")
        .expect("session");
    let old_time = (Utc::now() - Duration::days(1)).fixed_offset();
    let mut old = row.into_active_model();
    old.last_seen_at = Set(old_time);
    old.update(&database).await.expect("age session");

    session_service::touch(&database, "' OR TRUE; --")
        .await
        .expect("hostile id is harmless");
    let untouched = sessions::Entity::find_by_id(&session_id)
        .one(&database)
        .await
        .expect("untouched query")
        .expect("untouched session");
    assert_eq!(untouched.last_seen_at, old_time);

    session_service::touch(&database, &session_id)
        .await
        .expect("touch session");
    let touched = sessions::Entity::find_by_id(&session_id)
        .one(&database)
        .await
        .expect("touched query")
        .expect("touched session");
    assert!(touched.last_seen_at > old_time);
    session_service::delete(&database, &session_id)
        .await
        .expect("cleanup session");
}
