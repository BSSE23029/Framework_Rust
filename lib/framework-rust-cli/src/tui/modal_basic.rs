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
        _ => false,
    }
}
