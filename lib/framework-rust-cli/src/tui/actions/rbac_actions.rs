use super::*;

pub(super) async fn perform_create_idp(
    app: &mut App,
    provider: String,
    client_id: String,
    client_secret: String,
    redirect_url: String,
    scopes_str: String,
) {
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    let scopes: Vec<String> = scopes_str
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();
    match app
        .client
        .create_identity_provider(
            &tid,
            &provider,
            &client_id,
            &client_secret,
            &redirect_url,
            scopes,
        )
        .await
    {
        Ok(_) => {
            app.set_status(format!("IdP '{provider}' created"));
            load_all(app).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}

pub(super) async fn perform_edit_idp(
    app: &mut App,
    id: String,
    client_id: String,
    client_secret: Option<String>,
    redirect_url: String,
    scopes_str: String,
    enabled: bool,
) {
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    let scopes: Vec<String> = scopes_str
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();
    match app
        .client
        .update_identity_provider(
            &tid,
            &id,
            &client_id,
            client_secret.as_deref(),
            &redirect_url,
            scopes,
            enabled,
        )
        .await
    {
        Ok(_) => {
            app.set_status("IdP updated");
            load_all(app).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}

pub(super) async fn load_permissions(app: &mut App, tenant_id: String) {
    app.permissions = vec![];
    app.permission_selected = 0;
    app.permissions_loading = true;
    match app.client.list_permissions(&tenant_id).await {
        Ok(list) => {
            app.permissions = list;
            app.clear_status();
        }
        Err(e) => on_load_err(app, e, "Permissions"),
    }
    app.permissions_loading = false;
}

/// Eager-load everything needed to open EditUser modal.
pub(super) async fn open_edit_user(
    app: &mut App,
    user_id: String,
    email: String,
    is_active: bool,
    tid: String,
) {
    let client = app.client.clone();
    // Always fetch fresh from the API so we get current-tenant data regardless of tab cache.
    let (roles_result, assigned_result, passkeys_result) = tokio::join!(
        client.list_roles(&tid),
        client.list_user_roles(&tid, &user_id),
        client.list_user_passkeys(&tid, &user_id),
    );
    let all_roles = match roles_result {
        Ok(r) => {
            app.roles = r.clone();
            r
        }
        Err(e) => {
            app.modal = Modal::Error(format!("Failed to load roles: {e}"));
            return;
        }
    };
    let assigned_ids: std::collections::HashSet<String> = match assigned_result {
        Ok(r) => r.iter().map(|r| r.id.clone()).collect(),
        Err(e) => {
            app.modal = Modal::Error(format!("Failed to load user roles: {e}"));
            return;
        }
    };

    let role_entries: Vec<(String, String, bool)> = all_roles
        .iter()
        .map(|r| (r.id.clone(), r.name.clone(), assigned_ids.contains(&r.id)))
        .collect();

    // Derive permissions from currently assigned roles (eager).
    let permissions = derive_user_permissions(&role_entries, &tid, &client).await;

    let passkeys = passkeys_result.unwrap_or_default();

    app.modal = Modal::EditUser {
        id: user_id,
        email,
        password: String::new(),
        is_active,
        all_roles: role_entries,
        permissions,
        passkeys,
        field: 0,
        role_selected: 0,
        passkey_selected: 0,
    };
}

/// Derive permission names from currently-assigned roles via fresh API calls.
pub(super) async fn derive_user_permissions(
    role_entries: &[(String, String, bool)],
    tid: &str,
    client: &crate::api::Client,
) -> Vec<String> {
    let mut names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (role_id, _, assigned) in role_entries {
        if *assigned {
            if let Ok(perms) = client.list_role_permissions(tid, role_id).await {
                for p in perms {
                    names.insert(p.name);
                }
            }
        }
    }
    let mut v: Vec<String> = names.into_iter().collect();
    v.sort();
    v
}

/// Eager-load everything needed to open EditRole modal (fresh from API, not cached state).
pub(super) async fn open_edit_role(
    app: &mut App,
    role_id: String,
    name: String,
    description: String,
    tid: String,
) {
    let client = app.client.clone();
    let (all_perms_result, assigned_result) = tokio::join!(
        client.list_permissions(&tid),
        client.list_role_permissions(&tid, &role_id),
    );
    let all_perms = match all_perms_result {
        Ok(p) => {
            app.permissions = p.clone();
            p
        }
        Err(e) => {
            app.modal = Modal::Error(format!("Failed to load permissions: {e}"));
            return;
        }
    };
    let assigned_ids: std::collections::HashSet<String> = match assigned_result {
        Ok(p) => p.iter().map(|p| p.id.clone()).collect(),
        Err(e) => {
            app.modal = Modal::Error(format!("Failed to load role permissions: {e}"));
            return;
        }
    };
    let perm_entries: Vec<(String, String, bool)> = all_perms
        .iter()
        .map(|p| (p.id.clone(), p.name.clone(), assigned_ids.contains(&p.id)))
        .collect();
    app.modal = Modal::EditRole {
        id: role_id,
        name,
        description,
        all_permissions: perm_entries,
        field: 0,
        perm_selected: 0,
    };
}

pub(super) async fn perform_edit_user(
    app: &mut App,
    id: String,
    email: String,
    password: Option<String>,
    is_active: bool,
    role_entries: Vec<(String, String, bool)>,
) {
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    let pw = password.as_deref();
    match app
        .client
        .update_user_email(&tid, &id, &email, pw, is_active)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            app.modal = Modal::Error(format!("{e}"));
            return;
        }
    }
    // Save role assignments
    let client = app.client.clone();
    for (role_id, _, assigned) in &role_entries {
        if *assigned {
            let _ = client.assign_user_role(&tid, &id, role_id).await;
        } else {
            let _ = client.revoke_user_role(&tid, &id, role_id).await;
        }
    }
    app.set_status("User updated");
    load_all(app).await;
}

pub(super) async fn perform_edit_role(
    app: &mut App,
    id: String,
    name: String,
    description: String,
    perm_entries: Vec<(String, String, bool)>,
) {
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    match app.client.update_role(&tid, &id, &name, &description).await {
        Ok(_) => {}
        Err(e) => {
            app.modal = Modal::Error(format!("{e}"));
            return;
        }
    }
    let client = app.client.clone();
    for (perm_id, _, assigned) in &perm_entries {
        if *assigned {
            let _ = client.assign_role_permission(&tid, &id, perm_id).await;
        } else {
            let _ = client.revoke_role_permission(&tid, &id, perm_id).await;
        }
    }
    app.set_status("Role updated");
    load_all(app).await;
}

pub(super) async fn perform_create_permission(app: &mut App, name: String, description: String) {
    let Some(_tid) = app.active_tenant_id.clone() else {
        return;
    };
    match app
        .client
        .create_permission(&_tid, &name, &description)
        .await
    {
        Ok(_) => {
            app.set_status(format!("Permission '{name}' created"));
            load_all(app).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}

pub(super) async fn perform_edit_permission(
    app: &mut App,
    id: String,
    name: String,
    description: String,
) {
    let Some(_tid) = app.active_tenant_id.clone() else {
        return;
    };
    match app
        .client
        .update_permission(&_tid, &id, &name, &description)
        .await
    {
        Ok(_) => {
            app.set_status("Permission updated");
            load_all(app).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}
