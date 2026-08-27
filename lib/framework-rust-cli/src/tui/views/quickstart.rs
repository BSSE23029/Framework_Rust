use super::*;

pub(super) async fn perform_create_tenant(app: &mut App, name: String, slug: String) {
    match app.client.create_tenant(&name, &slug).await {
        Ok(t) => {
            app.modal = Modal::PostCreateTenant {
                tenant_id: t.id.clone(),
                tenant_name: name.clone(),
            };
            load_tenants(app).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}

pub(super) async fn perform_create_client(
    app: &mut App,
    name: String,
    redirect_uri: String,
    scopes_str: String,
    client_type: u8,
) {
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    let scopes: Vec<String> = scopes_str
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();

    // Derive is_confidential and grant_types from client_type:
    //   0 = Confidential/Web  → is_confidential=true,  grant_types=["authorization_code"]
    //   1 = Public/SPA        → is_confidential=false, grant_types=["authorization_code"]
    //   2 = Machine/M2M       → is_confidential=true,  grant_types=["client_credentials"]
    let (is_confidential, grant_types) = match client_type {
        1 => (false, vec!["authorization_code".to_owned()]),
        2 => (true, vec!["client_credentials".to_owned()]),
        _ => (true, vec!["authorization_code".to_owned()]),
    };

    // Machine clients have no redirect URI.
    let redirect_uris = if client_type == 2 {
        vec![]
    } else {
        vec![redirect_uri]
    };

    match app
        .client
        .create_client(
            &tid,
            &name,
            redirect_uris,
            scopes,
            is_confidential,
            grant_types,
            None,
            None,
        )
        .await
    {
        Ok(c) => {
            if let Some(secret) = c.client_secret {
                app.modal = Modal::ShowSecret {
                    client_id: c.client_id,
                    secret,
                };
            } else {
                app.set_status(format!("Client '{name}' created"));
            }
            load_all(app).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}

pub(super) async fn perform_create_user(app: &mut App, email: String, password: String) {
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    match app.client.create_user(&tid, &email, &password).await {
        Ok(_) => {
            load_all(app).await;
            app.set_status(format!("User '{email}' created"));
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}

pub(super) fn copy_to_clipboard(app: &mut App, text: &str, success_msg: &str) {
    match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(text)) {
        Ok(_) => app.set_status(success_msg.to_string()),
        Err(_) => app.set_status("clipboard unavailable".to_string()),
    }
}

pub(super) async fn handle_quickstart_key(app: &mut App, code: KeyCode) {
    let Modal::QuickStart(mut qs) = app.modal.clone() else {
        return;
    };

    if qs.step == 4 {
        match code {
            KeyCode::Char('c') => {
                qs.show_secret = !qs.show_secret;
                app.modal = Modal::QuickStart(qs);
            }
            KeyCode::Char('i') => {
                if let Some(cid) = &qs.created_client_id.clone() {
                    copy_to_clipboard(app, cid, "client_id copied");
                }
                app.modal = Modal::QuickStart(qs);
            }
            KeyCode::Char('s') => {
                if let Some(secret) = &qs.created_secret.clone() {
                    copy_to_clipboard(app, secret, "secret copied");
                }
                app.modal = Modal::QuickStart(qs);
            }
            KeyCode::Enter | KeyCode::Esc => {
                app.modal = Modal::None;
                load_tenants(app).await;
            }
            _ => {}
        }
        return;
    }

    let max_fields: usize = match qs.step {
        2 => 4, // name, redirect_uri, scopes, type-toggle
        _ => 2,
    };

    match code {
        KeyCode::Esc => {
            app.modal = Modal::None;
        }
        KeyCode::Tab => {
            qs.field = (qs.field + 1) % max_fields;
            qs.error = None;
            app.modal = Modal::QuickStart(qs);
        }
        // Left/Right on the type toggle field (step 2, field 3) cycle the client type.
        KeyCode::Left if qs.step == 2 && qs.field == 3 => {
            qs.client_type = (qs.client_type + 2) % 3;
            app.modal = Modal::QuickStart(qs);
        }
        KeyCode::Right if qs.step == 2 && qs.field == 3 => {
            qs.client_type = (qs.client_type + 1) % 3;
            app.modal = Modal::QuickStart(qs);
        }
        KeyCode::Backspace => {
            pop_quickstart_field(&mut qs);
            app.modal = Modal::QuickStart(qs);
        }
        KeyCode::Char(c) => {
            push_quickstart_field(&mut qs, c);
            app.modal = Modal::QuickStart(qs);
        }
        KeyCode::Enter => {
            qs.error = None;
            let client = app.client.clone();
            match qs.step {
                1 => {
                    if qs.tenant_name.is_empty() || qs.tenant_slug.is_empty() {
                        qs.error = Some("Name and Slug are required".to_string());
                        app.modal = Modal::QuickStart(qs);
                        return;
                    }
                    match client
                        .create_tenant(&qs.tenant_name.clone(), &qs.tenant_slug.clone())
                        .await
                    {
                        Ok(t) => {
                            qs.created_tenant_id = Some(t.id);
                            qs.created_tenant_name = Some(qs.tenant_name.clone());
                            qs.step = 2;
                            qs.field = 0;
                            app.modal = Modal::QuickStart(qs);
                        }
                        Err(e) => {
                            qs.error = Some(format!("{e}"));
                            app.modal = Modal::QuickStart(qs);
                        }
                    }
                }
                2 => {
                    let needs_uri = qs.client_type != 2;
                    if qs.client_name.is_empty() || (needs_uri && qs.redirect_uri.is_empty()) {
                        qs.error = Some(if needs_uri {
                            "Name and Redirect URI are required".to_string()
                        } else {
                            "Name is required".to_string()
                        });
                        app.modal = Modal::QuickStart(qs);
                        return;
                    }
                    let Some(tid) = qs.created_tenant_id.clone() else {
                        return;
                    };
                    let scopes: Vec<String> =
                        qs.scopes.split_whitespace().map(|s| s.to_owned()).collect();
                    let (is_confidential, grant_types) = match qs.client_type {
                        1 => (false, vec!["authorization_code".to_owned()]),
                        2 => (true, vec!["client_credentials".to_owned()]),
                        _ => (true, vec!["authorization_code".to_owned()]),
                    };
                    let redirect_uris = if qs.client_type == 2 {
                        vec![]
                    } else {
                        vec![qs.redirect_uri.clone()]
                    };
                    match client
                        .create_client(
                            &tid,
                            &qs.client_name.clone(),
                            redirect_uris,
                            scopes,
                            is_confidential,
                            grant_types,
                            None,
                            None,
                        )
                        .await
                    {
                        Ok(c) => {
                            qs.created_client_id = Some(c.client_id);
                            qs.created_secret = c.client_secret;
                            qs.step = 3;
                            qs.field = 0;
                            app.modal = Modal::QuickStart(qs);
                        }
                        Err(e) => {
                            qs.error = Some(format!("{e}"));
                            app.modal = Modal::QuickStart(qs);
                        }
                    }
                }
                3 => {
                    if qs.user_email.is_empty() || qs.user_password.is_empty() {
                        qs.error = Some("Email and Password are required".to_string());
                        app.modal = Modal::QuickStart(qs);
                        return;
                    }
                    let Some(tid) = qs.created_tenant_id.clone() else {
                        return;
                    };
                    match client
                        .create_user(&tid, &qs.user_email.clone(), &qs.user_password.clone())
                        .await
                    {
                        Ok(_) => {
                            qs.step = 4;
                            app.modal = Modal::QuickStart(qs);
                        }
                        Err(e) => {
                            qs.error = Some(format!("{e}"));
                            app.modal = Modal::QuickStart(qs);
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

pub(super) fn pop_quickstart_field(qs: &mut QuickStartState) {
    match (qs.step, qs.field) {
        (1, 0) => {
            qs.tenant_name.pop();
        }
        (1, 1) => {
            qs.tenant_slug.pop();
        }
        (2, 0) => {
            qs.client_name.pop();
        }
        (2, 1) => {
            qs.redirect_uri.pop();
        }
        (2, 2) => {
            qs.scopes.pop();
        }
        (3, 0) => {
            qs.user_email.pop();
        }
        (3, 1) => {
            qs.user_password.pop();
        }
        _ => {}
    }
}

pub(super) fn push_quickstart_field(qs: &mut QuickStartState, c: char) {
    match (qs.step, qs.field) {
        (1, 0) => qs.tenant_name.push(c),
        (1, 1) => qs.tenant_slug.push(c),
        (2, 0) => qs.client_name.push(c),
        (2, 1) => qs.redirect_uri.push(c),
        (2, 2) => qs.scopes.push(c),
        (3, 0) => qs.user_email.push(c),
        (3, 1) => qs.user_password.push(c),
        _ => {}
    }
}
