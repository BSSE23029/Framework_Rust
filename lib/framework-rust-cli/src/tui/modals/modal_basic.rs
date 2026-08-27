use super::*;

pub(super) async fn handle(app: &mut App, code: KeyCode) -> bool {
    match app.modal.clone() {
        Modal::ConfirmDelete { id, label: _ } => {
            match code {
                KeyCode::Char('y') | KeyCode::Char('Y') => perform_delete(app, id).await,
                _ => app.modal = Modal::None,
            }
            true
        }
        Modal::ShowSecret { .. } | Modal::Error(_) => {
            app.modal = Modal::None;
            true
        }
        Modal::AccessForm {
            action,
            mut fields,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab | KeyCode::Down => {
                    field = (field + 1) % fields.len();
                    app.modal = Modal::AccessForm {
                        action,
                        fields,
                        field,
                    };
                }
                KeyCode::Up => {
                    field = (field + fields.len() - 1) % fields.len();
                    app.modal = Modal::AccessForm {
                        action,
                        fields,
                        field,
                    };
                }
                KeyCode::Backspace => {
                    fields[field].pop();
                    app.modal = Modal::AccessForm {
                        action,
                        fields,
                        field,
                    };
                }
                KeyCode::Char(value) => {
                    fields[field].push(value);
                    app.modal = Modal::AccessForm {
                        action,
                        fields,
                        field,
                    };
                }
                KeyCode::Enter => access::perform(app, action, fields).await,
                _ => {}
            }
            true
        }
        Modal::CreateTenant {
            mut name,
            mut slug,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 2;
                    app.modal = Modal::CreateTenant { name, slug, field };
                }
                KeyCode::Enter if !name.is_empty() && !slug.is_empty() => {
                    let n = name.clone();
                    let s = slug.clone();
                    app.modal = Modal::None;
                    perform_create_tenant(app, n, s).await;
                }
                KeyCode::Backspace => {
                    if field == 0 {
                        name.pop();
                    } else {
                        slug.pop();
                    }
                    app.modal = Modal::CreateTenant { name, slug, field };
                }
                KeyCode::Char(c) => {
                    if field == 0 {
                        name.push(c);
                    } else {
                        slug.push(c);
                    }
                    app.modal = Modal::CreateTenant { name, slug, field };
                }
                _ => {}
            }
            true
        }
        Modal::TenantHierarchy {
            tenant_id,
            mut tenant_type,
            mut parent_tenant_id,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab | KeyCode::Down => field = (field + 1) % 2,
                KeyCode::Up => field = (field + 1) % 2,
                KeyCode::Backspace => {
                    if field == 0 {
                        tenant_type.pop();
                    } else {
                        parent_tenant_id.pop();
                    }
                }
                KeyCode::Char(value) => {
                    if field == 0 {
                        tenant_type.push(value);
                    } else {
                        parent_tenant_id.push(value);
                    }
                }
                KeyCode::Enter if !tenant_type.trim().is_empty() => {
                    let parent =
                        (!parent_tenant_id.trim().is_empty()).then_some(parent_tenant_id.trim());
                    match app
                        .client
                        .set_tenant_hierarchy(&tenant_id, tenant_type.trim(), parent)
                        .await
                    {
                        Ok(()) => {
                            app.modal = Modal::None;
                            app.set_status("Tenant hierarchy updated");
                            load_tenants(app).await;
                        }
                        Err(error) => app.modal = Modal::Error(error.to_string()),
                    }
                    return true;
                }
                _ => {}
            }
            app.modal = Modal::TenantHierarchy {
                tenant_id,
                tenant_type,
                parent_tenant_id,
                field,
            };
            true
        }
        _ => false,
    }
}
