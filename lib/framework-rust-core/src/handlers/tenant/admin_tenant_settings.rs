use axum::{response::IntoResponse, Json};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};

use crate::{
    entity::password_policies,
    error::AppError,
    extractors::TenantDb,
    services::{password_policy_service, tenant_settings_service},
};

// ── Password policy ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PolicyResponse {
    pub min_length: i32,
    pub require_uppercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
    pub history_size: i32,
}

#[utoipa::path(
    get,
    path = "/settings/password-policy",
    tag = "settings",
    responses(
        (status = 200, description = "Current password policy", body = PolicyResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant UUID"),
    )
)]
pub async fn get_policy(db: TenantDb) -> Result<impl IntoResponse, AppError> {
    let response = match password_policies::Entity::find_by_id(db.tenant_id)
        .one(&db.txn)
        .await?
    {
        Some(p) => Json(PolicyResponse {
            min_length: p.min_length,
            require_uppercase: p.require_uppercase,
            require_digit: p.require_digit,
            require_special: p.require_special,
            history_size: p.history_size,
        }),
        None => Json(PolicyResponse {
            min_length: 8,
            require_uppercase: false,
            require_digit: false,
            require_special: false,
            history_size: 0,
        }),
    };
    db.txn.commit().await?;
    Ok(response)
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpsertPolicyRequest {
    pub min_length: i32,
    pub require_uppercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
    pub history_size: i32,
}

#[utoipa::path(
    put,
    path = "/settings/password-policy",
    tag = "settings",
    request_body = UpsertPolicyRequest,
    responses(
        (status = 200, description = "Password policy saved"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant UUID"),
    )
)]
pub async fn put_policy(
    db: TenantDb,
    Json(req): Json<UpsertPolicyRequest>,
) -> Result<impl IntoResponse, AppError> {
    if req.min_length < 1 || req.min_length > 128 {
        return Err(AppError::InvalidInput(
            "min_length must be between 1 and 128".into(),
        ));
    }
    password_policy_service::upsert(
        &db.txn,
        db.tenant_id,
        req.min_length,
        req.require_uppercase,
        req.require_digit,
        req.require_special,
        req.history_size,
    )
    .await?;
    db.txn.commit().await?;
    Ok(Json(
        serde_json::json!({ "message": "password policy saved" }),
    ))
}

// ── Lockout policy ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LockoutResponse {
    pub max_attempts: i32,
    pub window_minutes: i32,
    pub duration_minutes: i32,
}

#[utoipa::path(
    get,
    path = "/settings/lockout",
    tag = "settings",
    responses(
        (status = 200, description = "Current lockout policy", body = LockoutResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant UUID"),
    )
)]
pub async fn get_lockout(db: TenantDb) -> Result<impl IntoResponse, AppError> {
    let s = tenant_settings_service::get(&db.txn, db.tenant_id).await?;
    let response = Json(LockoutResponse {
        max_attempts: s.lockout_max_attempts as i32,
        window_minutes: s.lockout_window_minutes as i32,
        duration_minutes: s.lockout_duration_minutes as i32,
    });
    db.txn.commit().await?;
    Ok(response)
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpsertLockoutRequest {
    pub max_attempts: i32,
    pub window_minutes: i32,
    pub duration_minutes: i32,
}

#[utoipa::path(
    put,
    path = "/settings/lockout",
    tag = "settings",
    request_body = UpsertLockoutRequest,
    responses(
        (status = 200, description = "Lockout policy saved"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant UUID"),
    )
)]
pub async fn put_lockout(
    db: TenantDb,
    Json(req): Json<UpsertLockoutRequest>,
) -> Result<impl IntoResponse, AppError> {
    if req.max_attempts < 1 || req.window_minutes < 1 || req.duration_minutes < 1 {
        return Err(AppError::InvalidInput(
            "all lockout values must be >= 1".into(),
        ));
    }
    let s = tenant_settings_service::get(&db.txn, db.tenant_id).await?;
    tenant_settings_service::upsert(
        &db.txn,
        db.tenant_id,
        req.max_attempts,
        req.window_minutes,
        req.duration_minutes,
        s.access_token_ttl_minutes as i32,
        s.refresh_token_ttl_days as i32,
        s.allow_public_registration,
        s.require_email_verified,
    )
    .await?;
    db.txn.commit().await?;
    Ok(Json(
        serde_json::json!({ "message": "lockout policy saved" }),
    ))
}

// ── Token TTL ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TokenTtlResponse {
    pub access_token_ttl_minutes: i32,
    pub refresh_token_ttl_days: i32,
}

#[utoipa::path(
    get,
    path = "/settings/tokens",
    tag = "settings",
    responses(
        (status = 200, description = "Current token TTL settings", body = TokenTtlResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant UUID"),
    )
)]
pub async fn get_token_ttl(db: TenantDb) -> Result<impl IntoResponse, AppError> {
    let s = tenant_settings_service::get(&db.txn, db.tenant_id).await?;
    let response = Json(TokenTtlResponse {
        access_token_ttl_minutes: s.access_token_ttl_minutes as i32,
        refresh_token_ttl_days: s.refresh_token_ttl_days as i32,
    });
    db.txn.commit().await?;
    Ok(response)
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpsertTokenTtlRequest {
    pub access_token_ttl_minutes: i32,
    pub refresh_token_ttl_days: i32,
}

#[utoipa::path(
    put,
    path = "/settings/tokens",
    tag = "settings",
    request_body = UpsertTokenTtlRequest,
    responses(
        (status = 200, description = "Token TTL settings saved"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant UUID"),
    )
)]
pub async fn put_token_ttl(
    db: TenantDb,
    Json(req): Json<UpsertTokenTtlRequest>,
) -> Result<impl IntoResponse, AppError> {
    if req.access_token_ttl_minutes < 1 || req.refresh_token_ttl_days < 1 {
        return Err(AppError::InvalidInput("TTL values must be >= 1".into()));
    }
    let s = tenant_settings_service::get(&db.txn, db.tenant_id).await?;
    tenant_settings_service::upsert(
        &db.txn,
        db.tenant_id,
        s.lockout_max_attempts as i32,
        s.lockout_window_minutes as i32,
        s.lockout_duration_minutes as i32,
        req.access_token_ttl_minutes,
        req.refresh_token_ttl_days,
        s.allow_public_registration,
        s.require_email_verified,
    )
    .await?;
    db.txn.commit().await?;
    Ok(Json(serde_json::json!({ "message": "token TTL saved" })))
}

// ── Registration policy ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RegistrationResponse {
    pub allow_public_registration: bool,
    pub require_email_verified: bool,
}

#[utoipa::path(
    get,
    path = "/settings/registration",
    tag = "settings",
    responses(
        (status = 200, description = "Current registration policy", body = RegistrationResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant UUID"),
    )
)]
pub async fn get_registration(db: TenantDb) -> Result<impl IntoResponse, AppError> {
    let s = tenant_settings_service::get(&db.txn, db.tenant_id).await?;
    let response = Json(RegistrationResponse {
        allow_public_registration: s.allow_public_registration,
        require_email_verified: s.require_email_verified,
    });
    db.txn.commit().await?;
    Ok(response)
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpsertRegistrationRequest {
    pub allow_public_registration: bool,
    pub require_email_verified: bool,
}

#[utoipa::path(
    put,
    path = "/settings/registration",
    tag = "settings",
    request_body = UpsertRegistrationRequest,
    responses(
        (status = 200, description = "Registration policy saved"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant UUID"),
    )
)]
pub async fn put_registration(
    db: TenantDb,
    Json(req): Json<UpsertRegistrationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let s = tenant_settings_service::get(&db.txn, db.tenant_id).await?;
    tenant_settings_service::upsert(
        &db.txn,
        db.tenant_id,
        s.lockout_max_attempts as i32,
        s.lockout_window_minutes as i32,
        s.lockout_duration_minutes as i32,
        s.access_token_ttl_minutes as i32,
        s.refresh_token_ttl_days as i32,
        req.allow_public_registration,
        req.require_email_verified,
    )
    .await?;
    db.txn.commit().await?;
    Ok(Json(
        serde_json::json!({ "message": "registration policy saved" }),
    ))
}
