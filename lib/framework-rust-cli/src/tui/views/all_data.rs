use super::*;

pub(super) async fn load_all(app: &mut App) {
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    load_access_contexts(app, &tid).await;
    let client = app.client.clone();

    let (
        clients_r,
        users_r,
        roles_r,
        perms_r,
        sessions_r,
        idps_r,
        domains_r,
        plans_r,
        config_r,
        devices_r,
        activity_r,
        audit_r,
    ) = tokio::join!(
        client.list_clients(&tid),
        client.list_users(&tid),
        client.list_roles(&tid),
        client.list_permissions(&tid),
        client.list_sessions(&tid),
        client.list_identity_providers(&tid),
        client.list_domains(&tid),
        client.list_plans(),
        client.list_config_definitions(&tid),
        client.list_trusted_devices(&tid),
        client.list_request_activity(&tid),
        client.list_audit_log(&tid),
    );

    match clients_r {
        Ok(list) => {
            app.client_selected = app.client_selected.min(list.len().saturating_sub(1));
            app.clients = list;
        }
        Err(e) => {
            on_load_err(app, e, "Clients");
            return;
        }
    }
    match users_r {
        Ok(list) => {
            app.user_selected = app.user_selected.min(list.len().saturating_sub(1));
            app.users = list;
        }
        Err(e) => {
            on_load_err(app, e, "Users");
            return;
        }
    }
    match roles_r {
        Ok(list) => {
            app.role_selected = app.role_selected.min(list.len().saturating_sub(1));
            app.roles = list;
        }
        Err(e) => {
            on_load_err(app, e, "Roles");
            return;
        }
    }
    match perms_r {
        Ok(list) => {
            app.permission_selected = app.permission_selected.min(list.len().saturating_sub(1));
            app.permissions = list;
        }
        Err(e) => {
            on_load_err(app, e, "Permissions");
            return;
        }
    }
    match sessions_r {
        Ok(list) => {
            app.session_selected = app.session_selected.min(list.len().saturating_sub(1));
            app.sessions = list;
        }
        Err(e) => {
            on_load_err(app, e, "Sessions");
            return;
        }
    }
    match idps_r {
        Ok(list) => {
            app.idp_selected = app.idp_selected.min(list.len().saturating_sub(1));
            app.identity_providers = list;
        }
        Err(e) => on_load_err(app, e, "IdP"),
    }
    match audit_r {
        Ok(list) => {
            app.audit_log_selected = app.audit_log_selected.min(list.len().saturating_sub(1));
            app.audit_log = list;
        }
        Err(e) => on_load_err(app, e, "AuditLog"),
    }
    match domains_r {
        Ok(list) => {
            app.domain_selected = app.domain_selected.min(list.len().saturating_sub(1));
            app.domains = list;
        }
        Err(e) => on_load_err(app, e, "Domains"),
    }
    match plans_r {
        Ok(list) => {
            app.plan_selected = app.plan_selected.min(list.len().saturating_sub(1));
            app.plans = list;
            load_selected_plan_quotas(app).await;
        }
        Err(e) => on_load_err(app, e, "Plans"),
    }
    match config_r {
        Ok(list) => {
            app.config_selected = app.config_selected.min(list.len().saturating_sub(1));
            app.config_definitions = list;
            load_config_values(app, &tid).await;
        }
        Err(e) => on_load_err(app, e, "Configuration"),
    }
    match devices_r {
        Ok(list) => {
            app.trusted_device_selected = app
                .trusted_device_selected
                .min(list.len().saturating_sub(1));
            app.trusted_devices = list;
        }
        Err(e) => on_load_err(app, e, "Trusted devices"),
    }
    match activity_r {
        Ok(list) => app.request_activity = list,
        Err(e) => on_load_err(app, e, "Request activity"),
    }
}

pub(super) async fn load_current_tab(app: &mut App) {
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    match app.tab {
        Tab::Clients => load_clients(app, tid).await,
        Tab::Users => load_users(app, tid).await,
        Tab::Roles => load_roles(app, tid).await,
        Tab::Permissions => load_permissions(app, tid).await,
        Tab::Access => access::load(app, tid).await,
        Tab::Sessions => load_sessions(app, tid).await,
        Tab::Settings => load_settings(app, tid).await,
        Tab::IdentityProviders => load_idps(app, tid).await,
        Tab::Domains => load_domains(app, tid).await,
        Tab::Plans => load_plans(app).await,
        Tab::Config => load_config(app, tid).await,
        Tab::Security => load_security(app, tid).await,
        Tab::AuditLog => load_audit_log(app, tid).await,
    }
}
