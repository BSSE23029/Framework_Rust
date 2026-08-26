use super::*;

// ── /oauth/introspect ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct IntrospectRequest {
    pub token: String,
}

#[utoipa::path(
    post,
    path = "/oauth/introspect",
    tag = "oauth",
    request_body(content = IntrospectRequest, content_type = "application/x-www-form-urlencoded"),
    responses(
        (status = 200, description = "Token introspection result"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("admin_key" = [])
    )
)]
pub async fn introspect(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(req): Form<IntrospectRequest>,
) -> Result<impl IntoResponse, AppError> {
    let Some(expected_key) = &state.config.admin_key else {
        return Err(AppError::NotFound);
    };
    let provided = headers
        .get("x-framework-rust-admin-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if provided != expected_key {
        return Err(AppError::Unauthorized);
    }

    match token_service::validate_access_token(
        &req.token,
        &state.config.jwt_secret,
        state.config.jwt_secret_previous.as_deref(),
    ) {
        Ok(claims) => Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "active": true,
                "sub": claims.sub,
                "tid": claims.tid,
                "email": claims.email,
                "jti": claims.jti,
                "exp": claims.exp,
                "iat": claims.iat,
            })),
        )),
        Err(_) => Ok((StatusCode::OK, Json(serde_json::json!({ "active": false })))),
    }
}
