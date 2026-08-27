use super::*;

pub(super) async fn handle(app: &mut App, code: KeyCode) -> bool {
    match app.modal.clone() {
        Modal::CreateClient {
            mut name,
            mut redirect_uri,
            mut scopes,
            mut client_type,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 4;
                    app.modal = Modal::CreateClient {
                        name,
                        redirect_uri,
                        scopes,
                        client_type,
                        field,
                    };
                }
                // On the type field (3), Space/Left/Right cycles the client_type.
                KeyCode::Char(' ') | KeyCode::Left | KeyCode::Right if field == 3 => {
                    client_type = (client_type + 1) % 3;
                    app.modal = Modal::CreateClient {
                        name,
                        redirect_uri,
                        scopes,
                        client_type,
                        field,
                    };
                }
                KeyCode::Enter => {
                    // Machine clients (type 2) don't require redirect_uri.
                    let requires_uri = client_type != 2;
                    if !name.is_empty() && (!requires_uri || !redirect_uri.is_empty()) {
                        let n = name.clone();
                        let u = redirect_uri.clone();
                        let sc = scopes.clone();
                        let ct = client_type;
                        app.modal = Modal::None;
                        perform_create_client(app, n, u, sc, ct).await;
                    }
                }
                KeyCode::Backspace => {
                    match field {
                        0 => {
                            name.pop();
                        }
                        1 => {
                            redirect_uri.pop();
                        }
                        2 => {
                            scopes.pop();
                        }
                        _ => {} // field 3 is the type selector, not editable
                    }
                    app.modal = Modal::CreateClient {
                        name,
                        redirect_uri,
                        scopes,
                        client_type,
                        field,
                    };
                }
                KeyCode::Char(c) => {
                    match field {
                        0 => name.push(c),
                        1 => redirect_uri.push(c),
                        2 => scopes.push(c),
                        _ => {} // field 3 is the type selector, not editable
                    }
                    app.modal = Modal::CreateClient {
                        name,
                        redirect_uri,
                        scopes,
                        client_type,
                        field,
                    };
                }
                _ => {}
            }
            true
        }
        Modal::CreateUser {
            mut email,
            mut password,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 2;
                    app.modal = Modal::CreateUser {
                        email,
                        password,
                        field,
                    };
                }
                KeyCode::Enter if !email.is_empty() && !password.is_empty() => {
                    let e = email.clone();
                    let p = password.clone();
                    app.modal = Modal::None;
                    perform_create_user(app, e, p).await;
                }
                KeyCode::Backspace => {
                    if field == 0 {
                        email.pop();
                    } else {
                        password.pop();
                    }
                    app.modal = Modal::CreateUser {
                        email,
                        password,
                        field,
                    };
                }
                KeyCode::Char(c) => {
                    if field == 0 {
                        email.push(c);
                    } else {
                        password.push(c);
                    }
                    app.modal = Modal::CreateUser {
                        email,
                        password,
                        field,
                    };
                }
                _ => {}
            }
            true
        }
        Modal::CreateRole {
            mut name,
            mut description,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 2;
                    app.modal = Modal::CreateRole {
                        name,
                        description,
                        field,
                    };
                }
                KeyCode::Enter if !name.is_empty() => {
                    let n = name.clone();
                    let d = description.clone();
                    app.modal = Modal::None;
                    perform_create_role(app, n, d).await;
                }
                KeyCode::Backspace => {
                    if field == 0 {
                        name.pop();
                    } else {
                        description.pop();
                    }
                    app.modal = Modal::CreateRole {
                        name,
                        description,
                        field,
                    };
                }
                KeyCode::Char(c) => {
                    if field == 0 {
                        name.push(c);
                    } else {
                        description.push(c);
                    }
                    app.modal = Modal::CreateRole {
                        name,
                        description,
                        field,
                    };
                }
                _ => {}
            }
            true
        }
        Modal::ClientRoles {
            client_id,
            client_name,
            mut all_roles,
            mut selected,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Up => {
                    selected = selected.saturating_sub(1);
                    app.modal = Modal::ClientRoles {
                        client_id,
                        client_name,
                        all_roles,
                        selected,
                    };
                }
                KeyCode::Down => {
                    if selected + 1 < all_roles.len() {
                        selected += 1;
                    }
                    app.modal = Modal::ClientRoles {
                        client_id,
                        client_name,
                        all_roles,
                        selected,
                    };
                }
                KeyCode::Char(' ') => {
                    if let Some(entry) = all_roles.get_mut(selected) {
                        entry.2 = !entry.2;
                    }
                    app.modal = Modal::ClientRoles {
                        client_id,
                        client_name,
                        all_roles,
                        selected,
                    };
                }
                KeyCode::Enter => {
                    let cid = client_id.clone();
                    let entries = all_roles.clone();
                    app.modal = Modal::None;
                    perform_save_client_roles(app, cid, entries).await;
                }
                _ => {}
            }
            true
        }
        Modal::EditClient {
            mut name,
            mut redirect_uris,
            mut scopes,
            mut access_token_ttl,
            mut refresh_token_ttl,
            mut client_type,
            mut field,
            id,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 6;
                    app.modal = Modal::EditClient {
                        id,
                        name,
                        redirect_uris,
                        scopes,
                        access_token_ttl,
                        refresh_token_ttl,
                        client_type,
                        field,
                    };
                }
                KeyCode::Char(' ') | KeyCode::Left | KeyCode::Right if field == 5 => {
                    client_type = (client_type + 1) % 3;
                    app.modal = Modal::EditClient {
                        id,
                        name,
                        redirect_uris,
                        scopes,
                        access_token_ttl,
                        refresh_token_ttl,
                        client_type,
                        field,
                    };
                }
                KeyCode::Enter if !name.is_empty() => {
                    let id2 = id.clone();
                    let n = name.clone();
                    let ru = redirect_uris.clone();
                    let sc = scopes.clone();
                    let att = access_token_ttl.clone();
                    let rtt = refresh_token_ttl.clone();
                    let ct = client_type;
                    app.modal = Modal::None;
                    perform_edit_client(app, id2, n, ru, sc, att, rtt, ct).await;
                }
                KeyCode::Backspace => {
                    match field {
                        0 => {
                            name.pop();
                        }
                        1 => {
                            redirect_uris.pop();
                        }
                        2 => {
                            scopes.pop();
                        }
                        3 => {
                            access_token_ttl.pop();
                        }
                        4 => {
                            refresh_token_ttl.pop();
                        }
                        _ => {}
                    }
                    app.modal = Modal::EditClient {
                        id,
                        name,
                        redirect_uris,
                        scopes,
                        access_token_ttl,
                        refresh_token_ttl,
                        client_type,
                        field,
                    };
                }
                KeyCode::Char(c) => {
                    match field {
                        0 => name.push(c),
                        1 => redirect_uris.push(c),
                        2 => scopes.push(c),
                        3 => access_token_ttl.push(c),
                        4 => refresh_token_ttl.push(c),
                        _ => {}
                    }
                    app.modal = Modal::EditClient {
                        id,
                        name,
                        redirect_uris,
                        scopes,
                        access_token_ttl,
                        refresh_token_ttl,
                        client_type,
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
