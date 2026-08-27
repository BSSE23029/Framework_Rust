use super::*;

pub(super) fn previous_tab(tab: &Tab) -> Tab {
    match tab {
        Tab::Clients => Tab::AuditLog,
        Tab::Users => Tab::Clients,
        Tab::Roles => Tab::Users,
        Tab::Permissions => Tab::Roles,
        Tab::Access => Tab::Permissions,
        Tab::Sessions => Tab::Access,
        Tab::Settings => Tab::Sessions,
        Tab::IdentityProviders => Tab::Settings,
        Tab::Domains => Tab::IdentityProviders,
        Tab::Plans => Tab::Domains,
        Tab::Config => Tab::Plans,
        Tab::Security => Tab::Config,
        Tab::AuditLog => Tab::Security,
    }
}

pub(super) fn next_tab(tab: &Tab) -> Tab {
    match tab {
        Tab::Clients => Tab::Users,
        Tab::Users => Tab::Roles,
        Tab::Roles => Tab::Permissions,
        Tab::Permissions => Tab::Access,
        Tab::Access => Tab::Sessions,
        Tab::Sessions => Tab::Settings,
        Tab::Settings => Tab::IdentityProviders,
        Tab::IdentityProviders => Tab::Domains,
        Tab::Domains => Tab::Plans,
        Tab::Plans => Tab::Config,
        Tab::Config => Tab::Security,
        Tab::Security => Tab::AuditLog,
        Tab::AuditLog => Tab::Clients,
    }
}

pub(super) async fn move_up(app: &mut App) {
    match app.tab {
        Tab::Clients => app.client_selected = app.client_selected.saturating_sub(1),
        Tab::Users => app.user_selected = app.user_selected.saturating_sub(1),
        Tab::Roles => app.role_selected = app.role_selected.saturating_sub(1),
        Tab::Permissions => app.permission_selected = app.permission_selected.saturating_sub(1),
        Tab::Access => app.access_selected = app.access_selected.saturating_sub(1),
        Tab::Sessions => app.session_selected = app.session_selected.saturating_sub(1),
        Tab::IdentityProviders => app.idp_selected = app.idp_selected.saturating_sub(1),
        Tab::Domains => app.domain_selected = app.domain_selected.saturating_sub(1),
        Tab::Plans => {
            app.plan_selected = app.plan_selected.saturating_sub(1);
            load_selected_plan_quotas(app).await;
        }
        Tab::Config => app.config_selected = app.config_selected.saturating_sub(1),
        Tab::Security => {
            app.trusted_device_selected = app.trusted_device_selected.saturating_sub(1)
        }
        Tab::AuditLog => app.audit_log_selected = app.audit_log_selected.saturating_sub(1),
        Tab::Settings => {}
    }
}

pub(super) async fn move_down(app: &mut App) {
    let access_length = access::section_len(app);
    let (selected, length) = match app.tab {
        Tab::Clients => (&mut app.client_selected, app.clients.len()),
        Tab::Users => (&mut app.user_selected, app.users.len()),
        Tab::Roles => (&mut app.role_selected, app.roles.len()),
        Tab::Permissions => (&mut app.permission_selected, app.permissions.len()),
        Tab::Access => (&mut app.access_selected, access_length),
        Tab::Sessions => (&mut app.session_selected, app.sessions.len()),
        Tab::IdentityProviders => (&mut app.idp_selected, app.identity_providers.len()),
        Tab::Domains => (&mut app.domain_selected, app.domains.len()),
        Tab::Config => (&mut app.config_selected, app.config_definitions.len()),
        Tab::Security => (&mut app.trusted_device_selected, app.trusted_devices.len()),
        Tab::AuditLog => (&mut app.audit_log_selected, app.audit_log.len()),
        Tab::Plans => {
            if app.plan_selected + 1 < app.plans.len() {
                app.plan_selected += 1;
                load_selected_plan_quotas(app).await;
            }
            return;
        }
        Tab::Settings => return,
    };
    if *selected + 1 < length {
        *selected += 1;
    }
}
