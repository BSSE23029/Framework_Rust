use super::*;

// ── Phase 4: user lifecycle ───────────────────────────────────────────────────

#[tokio::test]
async fn test_lockout_and_clear() {
    let (db, _cfg, tenant_id, tenant_key) = setup().await;
    let email = "lockout_test@framework-rust.dev";
    let lookup = hefesto::hash_for_lookup(email, &tenant_key).expect("hash");

    lockout_service::clear_attempts(&db, tenant_id, &lookup)
        .await
        .unwrap();

    // Not locked initially
    assert!(!lockout_service::is_locked(&db, tenant_id, &lookup, 3, 15)
        .await
        .unwrap());

    // Record 3 attempts
    for _ in 0..3 {
        lockout_service::record_attempt(&db, tenant_id, &lookup)
            .await
            .unwrap();
    }

    // Now locked
    assert!(lockout_service::is_locked(&db, tenant_id, &lookup, 3, 15)
        .await
        .unwrap());

    // Clear resets it
    lockout_service::clear_attempts(&db, tenant_id, &lookup)
        .await
        .unwrap();
    assert!(!lockout_service::is_locked(&db, tenant_id, &lookup, 3, 15)
        .await
        .unwrap());
}

#[tokio::test]
async fn test_one_time_token_consume() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;
    let user = create_test_user(
        &db,
        &cfg,
        tenant_id,
        &tenant_key,
        "ott@framework-rust.dev",
        "Pass1234!",
    )
    .await;

    let token = one_time_token_service::generate();
    let hash = one_time_token_service::hash(&token);

    one_time_token_service::store(
        &db,
        tenant_id,
        user.id,
        hash,
        framework_rust_core::entity::one_time_tokens::TYPE_PASSWORD_RESET,
        60,
    )
    .await
    .unwrap();

    // First consume succeeds
    let record = one_time_token_service::consume(
        &db,
        &token,
        framework_rust_core::entity::one_time_tokens::TYPE_PASSWORD_RESET,
    )
    .await
    .expect("first consume");
    assert_eq!(record.user_id, user.id);

    // Second consume fails (already used)
    let second = one_time_token_service::consume(
        &db,
        &token,
        framework_rust_core::entity::one_time_tokens::TYPE_PASSWORD_RESET,
    )
    .await;
    assert!(second.is_err(), "token must not be reusable");
}

#[tokio::test]
async fn test_otp_email_verification() {
    let (db, cfg, tenant_id, tenant_key) = setup().await;
    let user = create_test_user(
        &db,
        &cfg,
        tenant_id,
        &tenant_key,
        "otp_verify@framework-rust.dev",
        "Pass1234!",
    )
    .await;

    let otp = one_time_token_service::generate_otp();
    assert_eq!(otp.len(), 6);
    assert!(otp.chars().all(|c| c.is_ascii_digit()));

    one_time_token_service::store_otp(&db, tenant_id, user.id, &otp, 1)
        .await
        .unwrap();

    let record = one_time_token_service::consume_otp(&db, user.id, &otp)
        .await
        .expect("otp consumed");
    assert_eq!(record.user_id, user.id);

    // Second use fails
    let second = one_time_token_service::consume_otp(&db, user.id, &otp).await;
    assert!(second.is_err());
}

// ── Phase 6: MFA / TOTP ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_totp_verify_correct_code() {
    let secret = mfa_service::generate_secret();
    assert!(!secret.is_empty());

    // Generate the current TOTP code and verify it
    let secret_bytes =
        base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret).expect("decode");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let code = totp_lite::totp_custom::<sha1::Sha1>(totp_lite::DEFAULT_STEP, 6, &secret_bytes, now);

    assert!(
        mfa_service::verify_code(&secret, &code),
        "current code must verify"
    );
}

#[tokio::test]
async fn test_totp_rejects_wrong_code() {
    let secret = mfa_service::generate_secret();
    assert!(!mfa_service::verify_code(&secret, "000000"));
    assert!(!mfa_service::verify_code(&secret, "999999"));
    assert!(!mfa_service::verify_code(&secret, "abc123"));
}

#[tokio::test]
async fn test_mfa_token_roundtrip() {
    dotenvy::dotenv().ok();
    let cfg = Config::from_env().expect("config");
    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();

    let token = token_service::generate_mfa_token(user_id, tenant_id, &cfg.jwt_secret)
        .expect("generate mfa token");

    let claims =
        token_service::verify_mfa_token(&token, &cfg.jwt_secret, None).expect("verify mfa token");

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.tid, tenant_id.to_string());
    assert_eq!(claims.purpose, "mfa_challenge");

    // Wrong secret must fail
    let bad = token_service::verify_mfa_token(&token, "wrong_secret_that_is_long_enough_32c", None);
    assert!(bad.is_err());
}

#[tokio::test]
async fn test_mfa_totp_uri_format() {
    let secret = mfa_service::generate_secret();
    let uri = mfa_service::totp_uri(&secret, "user@example.com", "FRAMEWORK RUST");

    assert!(uri.starts_with("otpauth://totp/"));
    assert!(uri.contains(&secret));
    assert!(uri.contains("digits=6"));
    assert!(uri.contains("period=30"));
}

// ── Key rotation grace period ─────────────────────────────────────────────────

#[tokio::test]
async fn test_key_rotation_previous_secret_accepted() {
    dotenvy::dotenv().ok();
    let cfg = Config::from_env().expect("config");

    let old_secret = "old-jwt-secret-at-least-32-characters-long";
    let new_secret = &cfg.jwt_secret;

    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();

    // Token signed with old secret
    let token = token_service::generate_access_token(
        user_id,
        tenant_id,
        "rotation@test.dev",
        vec![],
        vec![],
        HashMap::new(),
        old_secret,
        15,
    )
    .expect("generate token with old secret");

    // Current secret alone must reject it
    let rejected = token_service::validate_access_token(&token, new_secret, None);
    assert!(
        rejected.is_err(),
        "old token must fail without previous secret"
    );

    // With previous secret as fallback it must succeed
    let claims = token_service::validate_access_token(&token, new_secret, Some(old_secret))
        .expect("old token must validate with previous secret fallback");
    assert_eq!(claims.sub, user_id.to_string());
}

#[tokio::test]
async fn test_key_rotation_wrong_previous_still_rejected() {
    dotenvy::dotenv().ok();
    let cfg = Config::from_env().expect("config");

    let token = token_service::generate_access_token(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "rotation2@test.dev",
        vec![],
        vec![],
        HashMap::new(),
        "totally-different-secret-32-chars!!",
        15,
    )
    .expect("generate");

    // Neither current nor a wrong previous should accept it
    let result = token_service::validate_access_token(
        &token,
        &cfg.jwt_secret,
        Some("also-wrong-previous-secret-32-chars"),
    );
    assert!(result.is_err(), "wrong secret must never validate");
}

// ── JTI blocklist ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_jti_revoked_after_blocklist() {
    let (db, cfg, tenant_id, _tenant_key) = setup().await;

    let token = token_service::generate_access_token(
        Uuid::new_v4(),
        tenant_id,
        "jti@test.dev",
        vec![],
        vec![],
        HashMap::new(),
        &cfg.jwt_secret,
        15,
    )
    .expect("generate token");

    let claims =
        token_service::validate_access_token(&token, &cfg.jwt_secret, None).expect("validate");

    // Not revoked initially
    assert!(
        !token_service::is_jti_revoked(&db, &claims.jti)
            .await
            .unwrap(),
        "fresh token must not be revoked"
    );

    let expires_at = chrono::DateTime::from_timestamp(claims.exp, 0)
        .unwrap()
        .fixed_offset();
    token_service::revoke_jti(&db, &claims.jti, expires_at)
        .await
        .expect("revoke jti");

    // Now marked as revoked
    assert!(
        token_service::is_jti_revoked(&db, &claims.jti)
            .await
            .unwrap(),
        "token must be revoked after blocklist insertion"
    );
}

// NOTE: introspect handler (POST /oauth/introspect) calls validate_access_token
// but does NOT check is_jti_revoked — it returns active:true for a JTI-revoked
// token as long as the signature and expiry are valid. This is a known gap:
// introspect is admin-only and short TTLs bound the risk, but a future improvement
// should add the JTI check there too.
