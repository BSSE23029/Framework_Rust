use super::*;

pub(super) async fn handle_sidebar_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Up if app.tenant_selected > 0 => {
            app.tenant_selected -= 1;
        }
        KeyCode::Down if app.tenant_selected + 1 < app.tenants.len() => {
            app.tenant_selected += 1;
        }
        KeyCode::Enter | KeyCode::Right => {
            if let Some(t) = app.selected_tenant() {
                let tid = t.id.clone();
                let switching_tenant = app.active_tenant_id.as_deref() != Some(&tid);
                if switching_tenant {
                    app.client.set_actor_context(None);
                    app.access_contexts.clear();
                    app.access_context_selected = 0;
                    app.clients = vec![];
                    app.users = vec![];
                    app.roles = vec![];
                    app.permissions = vec![];
                    app.sessions = vec![];
                    app.client_selected = 0;
                    app.user_selected = 0;
                    app.role_selected = 0;
                    app.permission_selected = 0;
                    app.session_selected = 0;
                }
                app.active_tenant_id = Some(tid.clone());
                app.focus = Focus::Content;
                load_all(app).await;
            }
        }
        KeyCode::Tab if app.active_tenant_id.is_some() => {
            app.focus = Focus::Content;
        }
        KeyCode::Char('n') => {
            app.modal = Modal::CreateTenant {
                name: String::new(),
                slug: String::new(),
                field: 0,
            };
        }
        KeyCode::Char('h') => {
            if let Some(tenant) = app.selected_tenant() {
                app.modal = Modal::TenantHierarchy {
                    tenant_id: tenant.id.clone(),
                    tenant_type: tenant.tenant_type.clone(),
                    parent_tenant_id: tenant.parent_tenant_id.clone().unwrap_or_default(),
                    field: 0,
                };
            }
        }
        KeyCode::Char('r') => load_tenants(app).await,
        _ => {}
    }
}
