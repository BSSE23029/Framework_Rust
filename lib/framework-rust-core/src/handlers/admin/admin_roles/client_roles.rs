use super::*;

// ── Client roles ──────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/clients/{id}/roles",
    tag = "admin-roles",
    responses(
        (status = 200, description = "List of roles for client", body = Vec<RoleResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("admin_key" = [])
    ),
    params(
        ("id" = String, Path, description = "Client UUID"),
        ("X-Framework-Rust-Tenant-Id" = String, Header, description = "Tenant UUID"),
    )
)]
pub async fn list_client_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(client_uuid): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, &headers)?;
    let tenant_id = extract_tenant_id(&headers)?;

    let txn = admin_auth::begin_tenant_txn(&state, &headers, tenant_id).await?;
    let roles = role_service::list_for_client(&txn, client_uuid).await?;
    txn.commit().await?;

    let resp: Vec<RoleResponse> = roles
        .into_iter()
        .map(|r| RoleResponse {
            id: r.id.to_string(),
            name: r.name,
            description: r.description,
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(resp))
}

#[utoipa::path(
    post,
    path = "/clients/{id}/roles",
    tag = "admin-roles",
    request_body = AssignRoleRequest,
    responses(
        (status = 204, description = "Role assigned to client"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("admin_key" = [])
    ),
    params(
        ("id" = String, Path, description = "Client UUID"),
        ("X-Framework-Rust-Tenant-Id" = String, Header, description = "Tenant UUID"),
    )
)]
pub async fn assign_client_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(client_uuid): Path<Uuid>,
    Json(payload): Json<AssignRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let actor = admin_auth::extract_actor(&headers, &state.config);
    require_admin(&state, &headers)?;
    let tenant_id = extract_tenant_id(&headers)?;

    let role_id = Uuid::parse_str(&payload.role_id)
        .map_err(|_| AppError::InvalidInput("invalid role_id".into()))?;

    let txn = admin_auth::begin_tenant_txn(&state, &headers, tenant_id).await?;
    role_service::assign_client_role(&txn, client_uuid, role_id, tenant_id).await?;
    txn.commit().await?;

    audit_service::record_best_effort(
        state.db.clone(),
        audit_service::AuditEvent::new(
            tenant_id,
            actor,
            "client.role.assigned",
            serde_json::json!({"client_id": client_uuid, "role_id": role_id}),
        ),
    );

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/clients/{client_id}/roles/{role_id}",
    tag = "admin-roles",
    responses(
        (status = 204, description = "Role revoked from client"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("admin_key" = [])
    ),
    params(
        ("client_id" = String, Path, description = "Client UUID"),
        ("role_id" = String, Path, description = "Role UUID"),
        ("X-Framework-Rust-Tenant-Id" = String, Header, description = "Tenant UUID"),
    )
)]
pub async fn revoke_client_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((client_uuid, role_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let actor = admin_auth::extract_actor(&headers, &state.config);
    require_admin(&state, &headers)?;
    let tenant_id = extract_tenant_id(&headers)?;

    let txn = admin_auth::begin_tenant_txn(&state, &headers, tenant_id).await?;
    role_service::revoke_client_role(&txn, client_uuid, role_id).await?;
    txn.commit().await?;

    audit_service::record_best_effort(
        state.db.clone(),
        audit_service::AuditEvent::new(
            tenant_id,
            actor,
            "client.role.revoked",
            serde_json::json!({"client_id": client_uuid, "role_id": role_id}),
        ),
    );

    Ok(StatusCode::NO_CONTENT)
}
