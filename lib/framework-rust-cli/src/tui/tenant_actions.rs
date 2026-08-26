use super::*;

pub(super) async fn perform_create_domain(
    app: &mut App,
    domain_name: String,
    domain_type: String,
    is_primary: bool,
) {
    let Some(tenant_id) = app.active_tenant_id.clone() else {
        return;
    };
    match app
        .client
        .create_domain(&tenant_id, &domain_name, &domain_type, is_primary)
        .await
    {
        Ok(_) => {
            app.set_status(format!("Domain '{domain_name}' created"));
            load_domains(app, tenant_id).await;
        }
        Err(error) => app.modal = Modal::Error(error.to_string()),
    }
}

pub(super) async fn perform_domain_action(app: &mut App, action: &str) {
    let Some(tenant_id) = app.active_tenant_id.clone() else {
        return;
    };
    let Some(domain) = app.selected_domain().cloned() else {
        return;
    };
    match app
        .client
        .domain_action(&tenant_id, &domain.id, action)
        .await
    {
        Ok(()) => {
            app.set_status(format!("Domain {}: {action}", domain.domain_name));
            load_domains(app, tenant_id).await;
        }
        Err(error) => app.modal = Modal::Error(error.to_string()),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn perform_create_plan(
    app: &mut App,
    code: String,
    name: String,
    interval: String,
    price_cents: i64,
    currency: String,
    trial_days: i32,
) {
    match app
        .client
        .create_plan(&code, &name, &interval, price_cents, &currency, trial_days)
        .await
    {
        Ok(_) => {
            app.set_status(format!("Plan '{name}' created"));
            load_plans(app).await;
        }
        Err(error) => app.modal = Modal::Error(error.to_string()),
    }
}

pub(super) async fn perform_set_plan_quota(app: &mut App, resource: String, limit: i64) {
    let Some(plan_id) = app.selected_plan().map(|plan| plan.id.clone()) else {
        return;
    };
    match app.client.set_plan_quota(&plan_id, &resource, limit).await {
        Ok(_) => {
            app.set_status(format!("Quota '{resource}' updated"));
            load_selected_plan_quotas(app).await;
        }
        Err(error) => app.modal = Modal::Error(error.to_string()),
    }
}

pub(super) async fn perform_subscribe_plan(app: &mut App) {
    let Some(tenant_id) = app.active_tenant_id.clone() else {
        return;
    };
    let Some(plan) = app.selected_plan().cloned() else {
        return;
    };
    match app
        .client
        .subscribe_tenant(&tenant_id, &plan.id, plan.trial_days > 0)
        .await
    {
        Ok(subscription) => app.set_status(format!(
            "Tenant subscribed to {} ({})",
            plan.name, subscription.state
        )),
        Err(error) => app.modal = Modal::Error(error.to_string()),
    }
}

pub(super) async fn perform_define_config(
    app: &mut App,
    config_key: String,
    display_name: String,
    category: String,
    value_type: String,
    default_value: Option<serde_json::Value>,
) {
    let Some(tenant_id) = app.active_tenant_id.clone() else {
        return;
    };
    match app
        .client
        .define_config(
            &tenant_id,
            &config_key,
            &display_name,
            &category,
            &value_type,
            default_value,
        )
        .await
    {
        Ok(_) => {
            app.set_status(format!("Configuration '{display_name}' defined"));
            load_config(app, tenant_id).await;
        }
        Err(error) => app.modal = Modal::Error(error.to_string()),
    }
}

pub(super) async fn perform_set_config_value(app: &mut App, value: serde_json::Value) {
    let Some(tenant_id) = app.active_tenant_id.clone() else {
        return;
    };
    let Some(definition) = app.selected_config().cloned() else {
        return;
    };
    match app
        .client
        .set_config_value(&tenant_id, &definition.id, value)
        .await
    {
        Ok(_) => {
            app.set_status(format!(
                "Configuration '{}' updated",
                definition.display_name
            ));
            load_config(app, tenant_id).await;
        }
        Err(error) => app.modal = Modal::Error(error.to_string()),
    }
}

pub(super) async fn perform_trust_device(app: &mut App, name: String, ttl_days: i64) {
    let Some(tenant_id) = app.active_tenant_id.clone() else {
        return;
    };
    match app.client.trust_device(&tenant_id, &name, ttl_days).await {
        Ok(issued) => {
            load_security(app, tenant_id).await;
            app.modal = Modal::ShowSecret {
                client_id: format!("Trusted device token — {}", issued.device.name),
                secret: issued.token,
            };
        }
        Err(error) => app.modal = Modal::Error(error.to_string()),
    }
}

pub(super) async fn perform_delete(app: &mut App, id: String) {
    app.modal = Modal::None;
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    let result = match app.tab {
        Tab::Clients => app
            .client
            .deactivate_client(&tid, &id)
            .await
            .map(|_| "Client deactivated"),
        Tab::Users => app
            .client
            .deactivate_user(&tid, &id)
            .await
            .map(|_| "User deactivated"),
        Tab::Roles => app
            .client
            .delete_role(&tid, &id)
            .await
            .map(|_| "Role deleted"),
        Tab::Permissions => app
            .client
            .delete_permission(&tid, &id)
            .await
            .map(|_| "Permission deleted"),
        Tab::Access if app.access_section == 3 => app
            .client
            .revoke_user_context(&tid, &id)
            .await
            .map(|_| "User access context revoked"),
        Tab::Access => return,
        Tab::Sessions => app
            .client
            .delete_session(&tid, &id)
            .await
            .map(|_| "Session revoked"),
        Tab::Settings => return,
        Tab::AuditLog => return,
        Tab::Domains => app
            .client
            .domain_action(&tid, &id, "deactivate")
            .await
            .map(|_| "Domain deactivated"),
        Tab::Plans => return,
        Tab::Config => return,
        Tab::Security => app
            .client
            .revoke_trusted_device(&tid, &id)
            .await
            .map(|_| "Trusted device revoked"),
        Tab::IdentityProviders => app
            .client
            .delete_identity_provider(&tid, &id)
            .await
            .map(|_| "Identity provider deleted"),
    };
    match result {
        Ok(msg) => {
            app.set_status(msg);
            load_all(app).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}

// ── EditSettings modal key handler ───────────────────────────────────────────
