use super::*;

pub(super) async fn perform_create_role(app: &mut App, name: String, description: String) {
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    match app.client.create_role(&tid, &name, &description).await {
        Ok(role) => {
            app.set_status(format!(
                "Role '{}' created — assign permissions below",
                role.name
            ));
            load_all(app).await;
            open_edit_role(app, role.id, role.name, role.description, tid).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}

pub(super) async fn open_client_roles(
    app: &mut App,
    client_id: String,
    client_name: String,
    tid: String,
) {
    let api = app.client.clone();
    let (all_roles_r, assigned_r) = tokio::join!(
        api.list_roles(&tid),
        api.list_client_roles(&tid, &client_id),
    );
    let all_roles = match all_roles_r {
        Ok(r) => r,
        Err(e) => {
            app.modal = Modal::Error(format!("Failed to load roles: {e}"));
            return;
        }
    };
    let assigned_ids: std::collections::HashSet<String> = match assigned_r {
        Ok(r) => r.iter().map(|r| r.id.clone()).collect(),
        Err(e) => {
            app.modal = Modal::Error(format!("Failed to load client roles: {e}"));
            return;
        }
    };
    let entries: Vec<(String, String, bool)> = all_roles
        .iter()
        .map(|r| (r.id.clone(), r.name.clone(), assigned_ids.contains(&r.id)))
        .collect();
    app.modal = Modal::ClientRoles {
        client_id,
        client_name,
        all_roles: entries,
        selected: 0,
    };
}

pub(super) async fn perform_save_client_roles(
    app: &mut App,
    client_id: String,
    entries: Vec<(String, String, bool)>,
) {
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    let api = app.client.clone();
    for (role_id, _, assigned) in &entries {
        if *assigned {
            let _ = api.assign_client_role(&tid, &client_id, role_id).await;
        } else {
            let _ = api.revoke_client_role(&tid, &client_id, role_id).await;
        }
    }
    app.set_status("Client roles saved");
}

pub(super) fn reset_to_login(app: &mut App, msg: &str) {
    let mut cfg = config_file::load();
    cfg.token = None;
    let _ = config_file::save(&cfg);
    app.client = crate::api::Client::new(app.client.base_url.clone());
    app.mode = AppMode::Login {
        email: String::new(),
        password: String::new(),
        field: 0,
        error: Some(msg.to_string()),
        tenant_choices: None,
        tenant_choice_idx: 0,
    };
    app.active_tenant_id = None;
    app.clients = vec![];
    app.users = vec![];
    app.roles = vec![];
    app.permissions = vec![];
    app.sessions = vec![];
    app.identity_providers = vec![];
    app.modal = Modal::None;
}

pub(super) fn on_load_err(app: &mut App, e: crate::api::ApiError, ctx: &str) {
    if let crate::api::ApiError::Api { status: 401, .. } = &e {
        reset_to_login(app, "Session expired — please log in again");
    } else {
        app.set_status(format!("{ctx} error: {e}"));
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn perform_edit_client(
    app: &mut App,
    id: String,
    name: String,
    redirect_uris_str: String,
    scopes_str: String,
    access_token_ttl_str: String,
    refresh_token_ttl_str: String,
    client_type: u8,
) {
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    let redirect_uris: Vec<String> = redirect_uris_str
        .split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect();
    let scopes: Vec<String> = scopes_str
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();
    let access_ttl: Option<i32> = access_token_ttl_str.parse().ok();
    let refresh_ttl: Option<i32> = refresh_token_ttl_str.parse().ok();
    let (is_confidential, grant_types) = match client_type {
        1 => (false, vec!["authorization_code".to_owned()]),
        2 => (true, vec!["client_credentials".to_owned()]),
        _ => (true, vec!["authorization_code".to_owned()]),
    };
    match app
        .client
        .update_client(
            &tid,
            &id,
            &name,
            redirect_uris,
            scopes,
            access_ttl,
            refresh_ttl,
            is_confidential,
            grant_types,
        )
        .await
    {
        Ok(_) => {
            app.set_status(format!("Client '{name}' updated"));
            load_all(app).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}
