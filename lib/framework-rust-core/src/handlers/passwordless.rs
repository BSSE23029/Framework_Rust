use axum::{
    extract::{ConnectInfo, State},
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr};
use validator::Validate;

use crate::{
    error::{validation_to_app_error, AppError},
    extractors::TenantDb,
    handlers::login::TokenResponse,
    middleware::tenant::TenantContext,
    services::{
        audit_service, email_service, mfa_service, passwordless_service, permission_service,
        role_service, session_service, tenant_settings_service, token_service, user_service,
    },
    state::AppState,
};

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct PasswordlessRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct PasswordlessVerifyRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6, max = 6))]
    pub otp: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PasswordlessRequestResponse {
    pub message: String,
}

#[utoipa::path(post, path = "/auth/passwordless/request", tag = "auth",
    request_body = PasswordlessRequest,
    responses((status = 202, body = PasswordlessRequestResponse)))]
pub async fn request(
    State(state): State<AppState>,
    Extension(ctx): Extension<TenantContext>,
    db: TenantDb,
    Json(payload): Json<PasswordlessRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(validation_to_app_error)?;
    let normalized = payload.email.trim().to_lowercase();
    let lookup = hefesto::hash_for_lookup(&normalized, &db.tenant_key)?;
    let user = user_service::find_by_email_lookup(&db.txn, &lookup).await?;
    let otp = if let Some(user) = user.filter(|user| user.is_active) {
        Some(passwordless_service::issue(&db.txn, db.tenant_id, user.id).await?)
    } else {
        None
    };
    db.txn.commit().await?;
    if let Some(otp) = otp {
        email_service::try_send(
            &state.db,
            ctx.tenant_id,
            &ctx.tenant_key,
            &state.config.master_encryption_key,
            &normalized,
            "Your sign-in code",
            &format!("<p>Your sign-in code is <strong>{otp}</strong>.</p>"),
            &format!("Your sign-in code is {otp}."),
        )
        .await;
    }
    Ok((
        axum::http::StatusCode::ACCEPTED,
        Json(PasswordlessRequestResponse {
            message: "If the account exists, a sign-in code has been sent".into(),
        }),
    ))
}

#[utoipa::path(post, path = "/auth/passwordless/verify", tag = "auth",
    request_body = PasswordlessVerifyRequest,
    responses((status = 200, body = TokenResponse), (status = 401)))]
pub async fn verify(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    db: TenantDb,
    Json(payload): Json<PasswordlessVerifyRequest>,
) -> Result<Response, AppError> {
    payload.validate().map_err(validation_to_app_error)?;
    let normalized = payload.email.trim().to_lowercase();
    let lookup = hefesto::hash_for_lookup(&normalized, &db.tenant_key)?;
    let user = user_service::find_by_email_lookup(&db.txn, &lookup)
        .await?
        .filter(|user| user.is_active)
        .ok_or(AppError::Unauthorized)?;
    passwordless_service::consume(&db.txn, db.tenant_id, user.id, &payload.otp).await?;
    if mfa_service::find_enabled(&db.txn, db.tenant_id, user.id)
        .await?
        .is_some()
    {
        db.txn.commit().await?;
        let mfa_token =
            token_service::generate_mfa_token(user.id, db.tenant_id, &state.config.jwt_secret)?;
        return Ok(Json(serde_json::json!({
            "mfa_required": true,
            "mfa_token": mfa_token,
        }))
        .into_response());
    }
    let settings = tenant_settings_service::get(&state.db, db.tenant_id).await?;
    let email = hefesto::decrypt(
        &user.email,
        &db.tenant_key,
        &state.config.master_encryption_key,
    )?;
    let roles = role_service::list_names_for_user(&db.txn, user.id, db.tenant_id).await?;
    let permissions =
        permission_service::list_names_for_user(&db.txn, user.id, db.tenant_id).await?;
    let access_token = token_service::generate_access_token(
        user.id,
        db.tenant_id,
        &email,
        roles,
        permissions,
        HashMap::new(),
        &state.config.jwt_secret,
        settings.access_token_ttl_minutes,
    )?;
    let refresh_token = token_service::generate_refresh_token();
    token_service::store_refresh_token(
        &db.txn,
        db.tenant_id,
        user.id,
        token_service::hash_refresh_token(&refresh_token),
        settings.refresh_token_ttl_days,
    )
    .await?;
    db.txn.commit().await?;
    let _ = session_service::create(
        &state.db,
        db.tenant_id,
        user.id,
        session_service::SessionData {
            email,
            ip: Some(addr.ip().to_string()),
        },
        settings.refresh_token_ttl_days,
    )
    .await;
    audit_service::record_best_effort(
        state.db.clone(),
        audit_service::AuditEvent::new(
            db.tenant_id,
            Some(user.id),
            "login.passwordless.success",
            serde_json::json!({"ip": addr.ip()}),
        ),
    );
    Ok(Json(TokenResponse {
        access_token,
        refresh_token,
        expires_in: settings.access_token_ttl_minutes * 60,
    })
    .into_response())
}
