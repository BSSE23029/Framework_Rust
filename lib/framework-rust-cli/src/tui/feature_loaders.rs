use super::*;

pub(super) async fn load_idps(app: &mut App, tenant_id: String) {
    app.identity_providers = vec![];
    app.idp_selected = 0;
    app.idps_loading = true;
    match app.client.list_identity_providers(&tenant_id).await {
        Ok(list) => {
            app.identity_providers = list;
            app.clear_status();
        }
        Err(e) => on_load_err(app, e, "IdP"),
    }
    app.idps_loading = false;
}

pub(super) async fn load_domains(app: &mut App, tenant_id: String) {
    app.domains.clear();
    app.domain_selected = 0;
    app.domains_loading = true;
    match app.client.list_domains(&tenant_id).await {
        Ok(list) => {
            app.domains = list;
            app.clear_status();
        }
        Err(e) => on_load_err(app, e, "Domains"),
    }
    app.domains_loading = false;
}

pub(super) async fn load_plans(app: &mut App) {
    app.plans_loading = true;
    match app.client.list_plans().await {
        Ok(list) => {
            app.plan_selected = app.plan_selected.min(list.len().saturating_sub(1));
            app.plans = list;
            load_selected_plan_quotas(app).await;
            app.clear_status();
        }
        Err(error) => on_load_err(app, error, "Plans"),
    }
    app.plans_loading = false;
}

pub(super) async fn load_selected_plan_quotas(app: &mut App) {
    let Some(plan_id) = app.selected_plan().map(|plan| plan.id.clone()) else {
        app.plan_quotas.clear();
        return;
    };
    match app.client.list_plan_quotas(&plan_id).await {
        Ok(quotas) => app.plan_quotas = quotas,
        Err(error) => on_load_err(app, error, "Plan quotas"),
    }
}

pub(super) async fn load_config(app: &mut App, tenant_id: String) {
    app.config_loading = true;
    match app.client.list_config_definitions(&tenant_id).await {
        Ok(definitions) => {
            app.config_selected = app.config_selected.min(definitions.len().saturating_sub(1));
            app.config_definitions = definitions;
            load_config_values(app, &tenant_id).await;
            app.clear_status();
        }
        Err(error) => on_load_err(app, error, "Configuration"),
    }
    app.config_loading = false;
}

pub(super) async fn load_config_values(app: &mut App, tenant_id: &str) {
    app.config_values.clear();
    let definitions = app.config_definitions.clone();
    for definition in definitions {
        match app
            .client
            .resolve_config_value(tenant_id, &definition.id)
            .await
        {
            Ok(Some(value)) => {
                app.config_values.insert(definition.id, value.to_string());
            }
            Ok(None) => {}
            Err(error) => {
                on_load_err(app, error, "Configuration value");
                return;
            }
        }
    }
}

pub(super) async fn load_security(app: &mut App, tenant_id: String) {
    let (devices, activity) = tokio::join!(
        app.client.list_trusted_devices(&tenant_id),
        app.client.list_request_activity(&tenant_id),
    );
    match devices {
        Ok(list) => {
            app.trusted_device_selected = app
                .trusted_device_selected
                .min(list.len().saturating_sub(1));
            app.trusted_devices = list;
        }
        Err(error) => on_load_err(app, error, "Trusted devices"),
    }
    match activity {
        Ok(list) => app.request_activity = list,
        Err(error) => on_load_err(app, error, "Request activity"),
    }
}

pub(super) async fn load_audit_log(app: &mut App, tenant_id: String) {
    app.audit_log = vec![];
    app.audit_log_selected = 0;
    app.audit_log_loading = true;
    match app.client.list_audit_log(&tenant_id).await {
        Ok(list) => {
            app.audit_log = list;
            app.clear_status();
        }
        Err(e) => on_load_err(app, e, "AuditLog"),
    }
    app.audit_log_loading = false;
}

pub(super) async fn export_audit_log_csv(app: &mut App, tenant_id: String) {
    app.set_status("Exporting audit log...".to_string());
    match app.client.export_audit_log(&tenant_id, 10_000).await {
        Ok(entries) => {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let filename = format!("framework-rust-audit-{tenant_id}-{ts}.csv");
            let path = dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(&filename);
            match write_audit_csv(&path, &entries) {
                Ok(()) => app.set_status(format!(
                    "Exported {} rows -> {}",
                    entries.len(),
                    path.display()
                )),
                Err(e) => app.set_status(format!("Export failed: {e}")),
            }
        }
        Err(e) => app.set_status(format!("Export failed: {e}")),
    }
}

pub(super) fn write_audit_csv(
    path: &std::path::Path,
    entries: &[crate::api::AuditLogEntry],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(["id", "created_at", "action", "user_id", "ip", "metadata"])?;
    for e in entries {
        wtr.write_record([
            e.id.as_str(),
            e.created_at.as_str(),
            e.action.as_str(),
            e.user_id.as_deref().unwrap_or(""),
            e.ip.as_deref().unwrap_or(""),
            e.metadata.as_deref().unwrap_or(""),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}
