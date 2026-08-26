use super::*;

// ── Phase 1-2: token generation & refresh rotation ───────────────────────────

#[tokio::test]
async fn test_register_and_login() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;

    let email = "integration@framework-rust.dev";
    let password = "Test1234!";

    let user = create_test_user(&db, &cfg, tenant_id, &tenant_key, email, password).await;
    assert_eq!(user.tenant_id, tenant_id);

    let email_lookup = hefesto::hash_for_lookup(email, &tenant_key).expect("hash");
    let txn = db::begin_tenant_txn(&db, tenant_id).await.expect("txn");
    let found = user_service::find_by_email_lookup(&txn, &email_lookup)
        .await
        .expect("find")
        .expect("user exists");
    txn.commit().await.unwrap();

    assert!(hefesto::verify_password(password, &found.password_hash));
    assert_eq!(found.id, user.id);

    let token = token_service::generate_access_token(
        user.id,
        tenant_id,
        email,
        vec![],
        vec![],
        HashMap::new(),
        &cfg.jwt_secret,
        cfg.jwt_expiration_minutes,
    )
    .expect("generate token");

    let claims = token_service::validate_access_token(&token, &cfg.jwt_secret, None)
        .expect("validate token");

    assert_eq!(claims.sub, user.id.to_string());
    assert_eq!(claims.tid, tenant_id.to_string());
    assert_eq!(claims.email, email);
    assert!(!claims.jti.is_empty());
}

#[tokio::test]
async fn test_me_endpoint_logic() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;

    let email = "me_test@framework-rust.dev";
    let password = "Secret5678!";

    let user = create_test_user(&db, &cfg, tenant_id, &tenant_key, email, password).await;

    let token = token_service::generate_access_token(
        user.id,
        tenant_id,
        email,
        vec![],
        vec![],
        HashMap::new(),
        &cfg.jwt_secret,
        cfg.jwt_expiration_minutes,
    )
    .unwrap();

    let claims = token_service::validate_access_token(&token, &cfg.jwt_secret, None).unwrap();
    assert_eq!(claims.sub, user.id.to_string());

    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let fetched = framework_rust_core::entity::users::Entity::find_by_id(user.id)
        .one(&txn)
        .await
        .unwrap()
        .expect("user found");
    txn.commit().await.unwrap();

    let decrypted =
        hefesto::decrypt(&fetched.email, &tenant_key, &cfg.master_encryption_key).unwrap();
    assert_eq!(decrypted, email);
}

#[tokio::test]
async fn test_refresh_token_rotation() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;

    let user = create_test_user(
        &db,
        &cfg,
        tenant_id,
        &tenant_key,
        "refresh@framework-rust.dev",
        "Pass1234!",
    )
    .await;

    let rt1 = token_service::generate_refresh_token();
    let hash1 = token_service::hash_refresh_token(&rt1);
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    token_service::store_refresh_token(&txn, tenant_id, user.id, hash1.clone(), 30)
        .await
        .unwrap();
    txn.commit().await.unwrap();

    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let record = token_service::find_valid_refresh_token(&txn, &hash1)
        .await
        .unwrap()
        .expect("token found");
    token_service::revoke_token(&txn, record).await.unwrap();

    let second = token_service::find_valid_refresh_token(&txn, &hash1)
        .await
        .unwrap();
    assert!(second.is_none(), "rotated token must not be reusable");
    txn.commit().await.unwrap();
}

#[tokio::test]
async fn test_revoke_all_tokens() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;

    let user = create_test_user(
        &db,
        &cfg,
        tenant_id,
        &tenant_key,
        "revoke@framework-rust.dev",
        "Pass1234!",
    )
    .await;

    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    for _ in 0..3 {
        let rt = token_service::generate_refresh_token();
        let hash = token_service::hash_refresh_token(&rt);
        token_service::store_refresh_token(&txn, tenant_id, user.id, hash, 30)
            .await
            .unwrap();
    }
    txn.commit().await.unwrap();

    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    token_service::revoke_all_user_tokens(&txn, user.id)
        .await
        .unwrap();
    txn.commit().await.unwrap();

    use framework_rust_core::entity::refresh_tokens;
    use sea_orm::{ColumnTrait, QueryFilter};
    let active: Vec<refresh_tokens::Model> = refresh_tokens::Entity::find()
        .filter(refresh_tokens::Column::UserId.eq(user.id))
        .filter(refresh_tokens::Column::RevokedAt.is_null())
        .all(&db)
        .await
        .unwrap();
    assert!(active.is_empty(), "all tokens must be revoked");
}

#[tokio::test]
async fn test_oauth_state_roundtrip() {
    dotenvy::dotenv().ok();
    let cfg = Config::from_env().expect("config");
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let state = oauth_service::generate_state(tenant_id, &cfg.jwt_secret);
    let recovered = oauth_service::verify_state(&state, &cfg.jwt_secret);
    assert_eq!(recovered, Some(tenant_id));

    let mut bad = state.clone();
    bad.push('x');
    assert_eq!(oauth_service::verify_state(&bad, &cfg.jwt_secret), None);
    assert_eq!(oauth_service::verify_state(&state, "wrong_secret"), None);
}
