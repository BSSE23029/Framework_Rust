use super::*;

// ── MFA backup codes ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_backup_codes_generate_and_consume() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;
    let user = create_test_user(
        &db,
        &cfg,
        tenant_id,
        &tenant_key,
        "bkcodes@test.dev",
        "Pass1234!",
    )
    .await;

    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let codes = mfa_service::generate_backup_codes(&txn, tenant_id, user.id)
        .await
        .expect("generate backup codes");
    txn.commit().await.unwrap();

    assert_eq!(
        codes.len(),
        mfa_service::BACKUP_CODE_COUNT,
        "must generate exactly 10 codes"
    );
    assert!(
        codes.iter().all(|c| c.len() == 9 && c.contains('-')),
        "codes must be in XXXX-XXXX format"
    );

    // First use of codes[0] succeeds
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let consumed = mfa_service::consume_backup_code(&txn, tenant_id, user.id, &codes[0])
        .await
        .expect("consume");
    txn.commit().await.unwrap();
    assert!(consumed, "first use must succeed");

    // Second use of same code fails
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let reused = mfa_service::consume_backup_code(&txn, tenant_id, user.id, &codes[0])
        .await
        .expect("consume reuse");
    txn.commit().await.unwrap();
    assert!(!reused, "used code must not be reusable");

    // Different code still works
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let second = mfa_service::consume_backup_code(&txn, tenant_id, user.id, &codes[1])
        .await
        .expect("consume second");
    txn.commit().await.unwrap();
    assert!(second, "unused code must still be consumable");
}

#[tokio::test]
async fn test_backup_codes_regenerate_invalidates_previous() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;
    let user = create_test_user(
        &db,
        &cfg,
        tenant_id,
        &tenant_key,
        "bkregen@test.dev",
        "Pass1234!",
    )
    .await;

    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let first_set = mfa_service::generate_backup_codes(&txn, tenant_id, user.id)
        .await
        .unwrap();
    txn.commit().await.unwrap();

    // Regenerate — new set replaces old
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let _second_set = mfa_service::generate_backup_codes(&txn, tenant_id, user.id)
        .await
        .unwrap();
    txn.commit().await.unwrap();

    // Code from first set must no longer work
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let stale = mfa_service::consume_backup_code(&txn, tenant_id, user.id, &first_set[0])
        .await
        .unwrap();
    txn.commit().await.unwrap();
    assert!(!stale, "codes from previous generation must be invalidated");
}

// ── RBAC claims in token ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_rbac_roles_appear_in_token_claims() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;
    let user = create_test_user(
        &db,
        &cfg,
        tenant_id,
        &tenant_key,
        "rbac@test.dev",
        "Pass1234!",
    )
    .await;

    // Create role and assign to user
    let role_name = format!("editor-{}", uuid::Uuid::new_v4());
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let role = role_service::create(
        &txn,
        role_service::CreateRoleInput {
            tenant_id,
            name: role_name.clone(),
            description: "Can edit content".into(),
        },
    )
    .await
    .expect("create role");
    role_service::assign(&txn, user.id, role.id, tenant_id)
        .await
        .expect("assign role");
    txn.commit().await.unwrap();

    // Fetch role names as the token flow would
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let role_names = role_service::list_names_for_user(&txn, user.id, tenant_id)
        .await
        .expect("list role names");
    txn.commit().await.unwrap();

    assert!(
        role_names.contains(&role_name),
        "role must be returned by list_names_for_user"
    );

    // Generate token with those roles
    let token = token_service::generate_access_token(
        user.id,
        tenant_id,
        "rbac@test.dev",
        role_names,
        vec![],
        HashMap::new(),
        &cfg.jwt_secret,
        15,
    )
    .expect("generate token");

    let claims =
        token_service::validate_access_token(&token, &cfg.jwt_secret, None).expect("validate");

    assert!(
        claims.realm_access.roles.contains(&role_name),
        "role must appear in realm_access.roles claim"
    );
}

#[tokio::test]
async fn test_rbac_revoked_role_absent_from_new_token() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;
    let user = create_test_user(
        &db,
        &cfg,
        tenant_id,
        &tenant_key,
        "rbac2@test.dev",
        "Pass1234!",
    )
    .await;

    let role_name = format!("moderator-{}", uuid::Uuid::new_v4());
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let role = role_service::create(
        &txn,
        role_service::CreateRoleInput {
            tenant_id,
            name: role_name.clone(),
            description: "".into(),
        },
    )
    .await
    .unwrap();
    role_service::assign(&txn, user.id, role.id, tenant_id)
        .await
        .unwrap();
    txn.commit().await.unwrap();

    // Revoke the role
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    role_service::revoke(&txn, tenant_id, user.id, role.id)
        .await
        .unwrap();
    txn.commit().await.unwrap();

    // New token must not carry the revoked role
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let role_names = role_service::list_names_for_user(&txn, user.id, tenant_id)
        .await
        .unwrap();
    txn.commit().await.unwrap();

    assert!(
        !role_names.contains(&role_name),
        "revoked role must not appear in list_names_for_user"
    );

    let token = token_service::generate_access_token(
        user.id,
        tenant_id,
        "rbac2@test.dev",
        role_names,
        vec![],
        HashMap::new(),
        &cfg.jwt_secret,
        15,
    )
    .unwrap();

    let claims = token_service::validate_access_token(&token, &cfg.jwt_secret, None).unwrap();
    assert!(
        !claims.realm_access.roles.contains(&role_name),
        "revoked role must not appear in token claims"
    );
}

// ── Password history enforcement ──────────────────────────────────────────────

#[tokio::test]
async fn test_password_history_reuse_rejected() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;
    let user = create_test_user(
        &db,
        &cfg,
        tenant_id,
        &tenant_key,
        "histcheck@test.dev",
        "OldPass1!",
    )
    .await;

    // Set history_size = 3 for this tenant
    password_policy_service::upsert(&db, tenant_id, 8, false, false, false, 3)
        .await
        .expect("upsert policy");

    let old_hash = hefesto::hash_password("OldPass1!").unwrap();

    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    password_history_service::record(&txn, tenant_id, user.id, &old_hash)
        .await
        .expect("record initial hash");
    txn.commit().await.unwrap();

    // Reusing the same password must be rejected
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let result = password_history_service::check(&txn, user.id, "OldPass1!", 3).await;
    txn.commit().await.unwrap();
    assert!(
        result.is_err(),
        "reusing a recent password must be rejected"
    );
}

#[tokio::test]
async fn test_password_history_new_password_accepted() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;
    let user = create_test_user(
        &db,
        &cfg,
        tenant_id,
        &tenant_key,
        "histok@test.dev",
        "OldPass2!",
    )
    .await;

    let old_hash = hefesto::hash_password("OldPass2!").unwrap();

    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    password_history_service::record(&txn, tenant_id, user.id, &old_hash)
        .await
        .unwrap();
    txn.commit().await.unwrap();

    // Completely different password must pass the check
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let result = password_history_service::check(&txn, user.id, "NewPass99!", 3).await;
    txn.commit().await.unwrap();
    assert!(result.is_ok(), "new password must be accepted");
}

#[tokio::test]
async fn test_password_history_skipped_when_size_zero() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;
    let user = create_test_user(
        &db,
        &cfg,
        tenant_id,
        &tenant_key,
        "histzero@test.dev",
        "AnyPass1!",
    )
    .await;

    let hash = hefesto::hash_password("AnyPass1!").unwrap();
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    password_history_service::record(&txn, tenant_id, user.id, &hash)
        .await
        .unwrap();
    txn.commit().await.unwrap();

    // history_size = 0 means no check — even reuse must pass
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let result = password_history_service::check(&txn, user.id, "AnyPass1!", 0).await;
    txn.commit().await.unwrap();
    assert!(
        result.is_ok(),
        "history_size=0 must skip the check entirely"
    );
}

#[tokio::test]
async fn test_password_history_window_boundary() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;
    let user = create_test_user(
        &db,
        &cfg,
        tenant_id,
        &tenant_key,
        "histwindow@test.dev",
        "Base1234!",
    )
    .await;

    // Fill history with 3 entries: pw_a, pw_b, pw_c (oldest→newest)
    for pw in &["HistA111!", "HistB222!", "HistC333!"] {
        let h = hefesto::hash_password(pw).unwrap();
        let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
        password_history_service::record(&txn, tenant_id, user.id, &h)
            .await
            .unwrap();
        txn.commit().await.unwrap();
    }

    // With history_size=3, all three must be rejected
    for pw in &["HistA111!", "HistB222!", "HistC333!"] {
        let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
        let r = password_history_service::check(&txn, user.id, pw, 3).await;
        txn.commit().await.unwrap();
        assert!(r.is_err(), "{pw} must be within the 3-entry window");
    }

    // A brand-new password must pass
    let txn = db::begin_tenant_txn(&db, tenant_id).await.unwrap();
    let ok = password_history_service::check(&txn, user.id, "FreshXYZ9!", 3).await;
    txn.commit().await.unwrap();
    assert!(ok.is_ok(), "password outside the window must be accepted");
}
