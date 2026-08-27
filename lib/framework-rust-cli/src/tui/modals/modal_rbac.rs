use super::*;

pub(super) async fn handle(app: &mut App, code: KeyCode) -> bool {
    match app.modal.clone() {
        Modal::EditUser {
            id,
            mut email,
            mut password,
            mut is_active,
            mut all_roles,
            permissions,
            mut passkeys,
            mut field,
            mut role_selected,
            mut passkey_selected,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    // Cycle: email(0) → password(1) → is_active(2) → roles(3) → passkeys(4) → email
                    field = (field + 1) % 5;
                    app.modal = Modal::EditUser {
                        id,
                        email,
                        password,
                        is_active,
                        all_roles,
                        permissions,
                        passkeys,
                        field,
                        role_selected,
                        passkey_selected,
                    };
                }
                KeyCode::Up if field == 3 => {
                    role_selected = role_selected.saturating_sub(1);
                    app.modal = Modal::EditUser {
                        id,
                        email,
                        password,
                        is_active,
                        all_roles,
                        permissions,
                        passkeys,
                        field,
                        role_selected,
                        passkey_selected,
                    };
                }
                KeyCode::Down if field == 3 => {
                    if role_selected + 1 < all_roles.len() {
                        role_selected += 1;
                    }
                    app.modal = Modal::EditUser {
                        id,
                        email,
                        password,
                        is_active,
                        all_roles,
                        permissions,
                        passkeys,
                        field,
                        role_selected,
                        passkey_selected,
                    };
                }
                KeyCode::Up if field == 4 => {
                    passkey_selected = passkey_selected.saturating_sub(1);
                    app.modal = Modal::EditUser {
                        id,
                        email,
                        password,
                        is_active,
                        all_roles,
                        permissions,
                        passkeys,
                        field,
                        role_selected,
                        passkey_selected,
                    };
                }
                KeyCode::Down if field == 4 => {
                    if passkey_selected + 1 < passkeys.len() {
                        passkey_selected += 1;
                    }
                    app.modal = Modal::EditUser {
                        id,
                        email,
                        password,
                        is_active,
                        all_roles,
                        permissions,
                        passkeys,
                        field,
                        role_selected,
                        passkey_selected,
                    };
                }
                KeyCode::Char('d') if field == 4 => {
                    if let Some(pk) = passkeys.get(passkey_selected) {
                        let cred_id = pk.credential_id.clone();
                        let user_id = id.clone();
                        if let Some(tid) = app.active_tenant_id.clone() {
                            match app
                                .client
                                .delete_user_passkey(&tid, &user_id, &cred_id)
                                .await
                            {
                                Ok(_) => {
                                    passkeys.remove(passkey_selected);
                                    if passkey_selected > 0 && passkey_selected >= passkeys.len() {
                                        passkey_selected -= 1;
                                    }
                                    app.set_status("Passkey deleted");
                                }
                                Err(e) => app.set_status(format!("Delete failed: {e}")),
                            }
                        }
                    }
                    app.modal = Modal::EditUser {
                        id,
                        email,
                        password,
                        is_active,
                        all_roles,
                        permissions,
                        passkeys,
                        field,
                        role_selected,
                        passkey_selected,
                    };
                }
                KeyCode::Char(' ') if field == 2 => {
                    is_active = !is_active;
                    app.modal = Modal::EditUser {
                        id,
                        email,
                        password,
                        is_active,
                        all_roles,
                        permissions,
                        passkeys,
                        field,
                        role_selected,
                        passkey_selected,
                    };
                }
                KeyCode::Char(' ') if field == 3 => {
                    if let Some(entry) = all_roles.get_mut(role_selected) {
                        entry.2 = !entry.2;
                    }
                    app.modal = Modal::EditUser {
                        id,
                        email,
                        password,
                        is_active,
                        all_roles,
                        permissions,
                        passkeys,
                        field,
                        role_selected,
                        passkey_selected,
                    };
                }
                KeyCode::Enter => {
                    let id2 = id.clone();
                    let e = email.clone();
                    let pw = if password.is_empty() {
                        None
                    } else {
                        Some(password.clone())
                    };
                    let roles = all_roles.clone();
                    app.modal = Modal::None;
                    perform_edit_user(app, id2, e, pw, is_active, roles).await;
                }
                KeyCode::Backspace if field == 0 => {
                    email.pop();
                    app.modal = Modal::EditUser {
                        id,
                        email,
                        password,
                        is_active,
                        all_roles,
                        permissions,
                        passkeys,
                        field,
                        role_selected,
                        passkey_selected,
                    };
                }
                KeyCode::Backspace if field == 1 => {
                    password.pop();
                    app.modal = Modal::EditUser {
                        id,
                        email,
                        password,
                        is_active,
                        all_roles,
                        permissions,
                        passkeys,
                        field,
                        role_selected,
                        passkey_selected,
                    };
                }
                KeyCode::Char(c) if field == 0 => {
                    email.push(c);
                    app.modal = Modal::EditUser {
                        id,
                        email,
                        password,
                        is_active,
                        all_roles,
                        permissions,
                        passkeys,
                        field,
                        role_selected,
                        passkey_selected,
                    };
                }
                KeyCode::Char(c) if field == 1 => {
                    password.push(c);
                    app.modal = Modal::EditUser {
                        id,
                        email,
                        password,
                        is_active,
                        all_roles,
                        permissions,
                        passkeys,
                        field,
                        role_selected,
                        passkey_selected,
                    };
                }
                _ => {}
            }
            true
        }
        Modal::EditRole {
            id,
            mut name,
            mut description,
            mut all_permissions,
            mut field,
            mut perm_selected,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 3;
                    app.modal = Modal::EditRole {
                        id,
                        name,
                        description,
                        all_permissions,
                        field,
                        perm_selected,
                    };
                }
                KeyCode::Up if field == 2 => {
                    perm_selected = perm_selected.saturating_sub(1);
                    app.modal = Modal::EditRole {
                        id,
                        name,
                        description,
                        all_permissions,
                        field,
                        perm_selected,
                    };
                }
                KeyCode::Down if field == 2 => {
                    if perm_selected + 1 < all_permissions.len() {
                        perm_selected += 1;
                    }
                    app.modal = Modal::EditRole {
                        id,
                        name,
                        description,
                        all_permissions,
                        field,
                        perm_selected,
                    };
                }
                KeyCode::Char(' ') if field == 2 => {
                    if let Some(entry) = all_permissions.get_mut(perm_selected) {
                        entry.2 = !entry.2;
                    }
                    app.modal = Modal::EditRole {
                        id,
                        name,
                        description,
                        all_permissions,
                        field,
                        perm_selected,
                    };
                }
                KeyCode::Enter if field != 2 && !name.is_empty() => {
                    let id2 = id.clone();
                    let n = name.clone();
                    let d = description.clone();
                    let perms = all_permissions.clone();
                    app.modal = Modal::None;
                    perform_edit_role(app, id2, n, d, perms).await;
                }
                KeyCode::Enter if field == 2 => {
                    let id2 = id.clone();
                    let n = name.clone();
                    let d = description.clone();
                    let perms = all_permissions.clone();
                    app.modal = Modal::None;
                    perform_edit_role(app, id2, n, d, perms).await;
                }
                KeyCode::Backspace if field == 0 => {
                    name.pop();
                    app.modal = Modal::EditRole {
                        id,
                        name,
                        description,
                        all_permissions,
                        field,
                        perm_selected,
                    };
                }
                KeyCode::Backspace if field == 1 => {
                    description.pop();
                    app.modal = Modal::EditRole {
                        id,
                        name,
                        description,
                        all_permissions,
                        field,
                        perm_selected,
                    };
                }
                KeyCode::Char(c) if field == 0 => {
                    name.push(c);
                    app.modal = Modal::EditRole {
                        id,
                        name,
                        description,
                        all_permissions,
                        field,
                        perm_selected,
                    };
                }
                KeyCode::Char(c) if field == 1 => {
                    description.push(c);
                    app.modal = Modal::EditRole {
                        id,
                        name,
                        description,
                        all_permissions,
                        field,
                        perm_selected,
                    };
                }
                _ => {}
            }
            true
        }
        _ => false,
    }
}
