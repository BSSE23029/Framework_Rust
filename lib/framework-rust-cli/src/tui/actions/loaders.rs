use super::*;

pub(super) async fn load_tenants(app: &mut App) {
    app.tenants_loading = true;
    match app.client.list_tenants().await {
        Ok(list) => {
            app.tenants = list;
            app.tenant_selected = app.tenant_selected.min(app.tenants.len().saturating_sub(1));
            app.clear_status();
        }
        Err(e) => app.set_status(format!("Error: {e}")),
    }
    app.tenants_loading = false;
}

pub(super) async fn load_access_contexts(app: &mut App, tenant_id: &str) {
    app.client.set_actor_context(None);
    match app.client.list_access_contexts(tenant_id).await {
        Ok(contexts) => {
            app.access_context_selected = app
                .access_context_selected
                .min(contexts.len().saturating_sub(1));
            app.access_contexts = contexts;
            let selected = app
                .active_access_context()
                .map(|context| context.urdd_id.clone());
            app.client.set_actor_context(selected);
        }
        Err(error) => {
            app.access_contexts.clear();
            app.client.set_actor_context(None);
            on_load_err(app, error, "Access contexts");
        }
    }
}

pub(super) async fn load_clients(app: &mut App, tenant_id: String) {
    app.clients = vec![];
    app.client_selected = 0;
    app.clients_loading = true;
    match app.client.list_clients(&tenant_id).await {
        Ok(list) => {
            app.clients = list;
            app.clear_status();
        }
        Err(e) => on_load_err(app, e, "Clients"),
    }
    app.clients_loading = false;
}

pub(super) async fn load_users(app: &mut App, tenant_id: String) {
    app.users = vec![];
    app.user_selected = 0;
    app.users_loading = true;
    match app.client.list_users(&tenant_id).await {
        Ok(list) => {
            app.users = list;
            app.clear_status();
        }
        Err(e) => on_load_err(app, e, "Users"),
    }
    app.users_loading = false;
}

pub(super) async fn load_roles(app: &mut App, tenant_id: String) {
    app.roles = vec![];
    app.role_selected = 0;
    app.roles_loading = true;
    match app.client.list_roles(&tenant_id).await {
        Ok(list) => {
            app.roles = list;
            app.clear_status();
        }
        Err(e) => on_load_err(app, e, "Roles"),
    }
    app.roles_loading = false;
}

pub(super) async fn load_sessions(app: &mut App, tenant_id: String) {
    app.sessions = vec![];
    app.session_selected = 0;
    app.sessions_loading = true;
    match app.client.list_sessions(&tenant_id).await {
        Ok(list) => {
            app.sessions = list;
            app.clear_status();
        }
        Err(e) => on_load_err(app, e, "Sessions"),
    }
    app.sessions_loading = false;
}

pub(super) async fn check_health(app: &mut App) {
    match app.client.health().await {
        Ok(v) => {
            app.health_status = Some(v["status"].as_str().unwrap_or("ok").to_owned());
            app.health_error = None;
        }
        Err(e) => {
            app.health_status = None;
            app.health_error = Some(e.to_string());
        }
    }
}
