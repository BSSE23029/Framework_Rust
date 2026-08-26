use super::*;

pub(super) async fn handle(app: &mut App, code: KeyCode) -> bool {
    match app.modal.clone() {
        Modal::CreateIdp {
            mut provider,
            mut client_id,
            mut client_secret,
            mut redirect_url,
            mut scopes,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 5;
                    app.modal = Modal::CreateIdp {
                        provider,
                        client_id,
                        client_secret,
                        redirect_url,
                        scopes,
                        field,
                    };
                }
                KeyCode::Enter
                    if !provider.is_empty()
                        && !client_id.is_empty()
                        && !client_secret.is_empty() =>
                {
                    let prov = provider.clone();
                    let cid = client_id.clone();
                    let cs = client_secret.clone();
                    let ru = redirect_url.clone();
                    let sc = scopes.clone();
                    app.modal = Modal::None;
                    perform_create_idp(app, prov, cid, cs, ru, sc).await;
                }
                KeyCode::Backspace => {
                    match field {
                        0 => {
                            provider.pop();
                        }
                        1 => {
                            client_id.pop();
                        }
                        2 => {
                            client_secret.pop();
                        }
                        3 => {
                            redirect_url.pop();
                        }
                        _ => {
                            scopes.pop();
                        }
                    }
                    app.modal = Modal::CreateIdp {
                        provider,
                        client_id,
                        client_secret,
                        redirect_url,
                        scopes,
                        field,
                    };
                }
                KeyCode::Char(c) => {
                    match field {
                        0 => provider.push(c),
                        1 => client_id.push(c),
                        2 => client_secret.push(c),
                        3 => redirect_url.push(c),
                        _ => scopes.push(c),
                    }
                    app.modal = Modal::CreateIdp {
                        provider,
                        client_id,
                        client_secret,
                        redirect_url,
                        scopes,
                        field,
                    };
                }
                _ => {}
            }
            true
        }
        Modal::EditIdp {
            mut client_id,
            mut client_secret,
            mut redirect_url,
            mut scopes,
            mut enabled,
            mut field,
            id,
            provider,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 5;
                    app.modal = Modal::EditIdp {
                        id,
                        provider,
                        client_id,
                        client_secret,
                        redirect_url,
                        scopes,
                        enabled,
                        field,
                    };
                }
                KeyCode::Enter => {
                    let id2 = id.clone();
                    let cid = client_id.clone();
                    let cs = if client_secret.is_empty() {
                        None
                    } else {
                        Some(client_secret.clone())
                    };
                    let ru = redirect_url.clone();
                    let sc = scopes.clone();
                    let en = enabled;
                    app.modal = Modal::None;
                    perform_edit_idp(app, id2, cid, cs, ru, sc, en).await;
                }
                // Space on enabled field (4) toggles it
                KeyCode::Char(' ') if field == 4 => {
                    enabled = !enabled;
                    app.modal = Modal::EditIdp {
                        id,
                        provider,
                        client_id,
                        client_secret,
                        redirect_url,
                        scopes,
                        enabled,
                        field,
                    };
                }
                KeyCode::Backspace => {
                    match field {
                        0 => {
                            client_id.pop();
                        }
                        1 => {
                            client_secret.pop();
                        }
                        2 => {
                            redirect_url.pop();
                        }
                        3 => {
                            scopes.pop();
                        }
                        _ => {}
                    }
                    app.modal = Modal::EditIdp {
                        id,
                        provider,
                        client_id,
                        client_secret,
                        redirect_url,
                        scopes,
                        enabled,
                        field,
                    };
                }
                KeyCode::Char(c) => {
                    match field {
                        0 => client_id.push(c),
                        1 => client_secret.push(c),
                        2 => redirect_url.push(c),
                        3 => scopes.push(c),
                        _ => {}
                    }
                    app.modal = Modal::EditIdp {
                        id,
                        provider,
                        client_id,
                        client_secret,
                        redirect_url,
                        scopes,
                        enabled,
                        field,
                    };
                }
                _ => {}
            }
            true
        }
        _ => false,
    }
}
