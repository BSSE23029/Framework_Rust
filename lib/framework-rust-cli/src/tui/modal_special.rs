use super::*;

pub(super) async fn handle(app: &mut App, code: KeyCode) -> bool {
    match app.modal.clone() {
        Modal::QuickStart(_) => {
            handle_quickstart_key(app, code).await;
            true
        }
        Modal::EditSettings(section) => {
            handle_edit_settings_key(app, code, section).await;
            true
        }
        Modal::PostCreateTenant {
            tenant_id,
            tenant_name,
        } => {
            match code {
                KeyCode::Enter => {
                    let tid = tenant_id.clone();
                    let tname = tenant_name.clone();
                    app.modal = Modal::None;
                    // Load tenant data then open QuickStart pre-filled at step 2
                    load_tenants(app).await;
                    // Find the newly created tenant and set it active
                    if let Some(pos) = app.tenants.iter().position(|t| t.id == tid) {
                        app.tenant_selected = pos;
                        app.active_tenant_id = Some(tid.clone());
                        app.focus = crate::app::Focus::Content;
                        load_all(app).await;
                    }
                    app.modal = Modal::QuickStart(QuickStartState {
                        created_tenant_id: Some(tid),
                        created_tenant_name: Some(tname),
                        step: 2,
                        field: 0,
                        ..QuickStartState::default()
                    });
                }
                KeyCode::Esc => {
                    app.modal = Modal::None;
                    load_tenants(app).await;
                }
                _ => {}
            }
            true
        }
        _ => false,
    }
}
