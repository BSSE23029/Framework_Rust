use super::*;

pub(super) async fn handle_edit_settings_key(app: &mut App, code: KeyCode, section: u8) {
    let max_fields: usize = match section {
        0 => 4, // min_length + 3 toggles
        1 => 3,
        2 => 2,
        3 => 2,
        4 => 8,
        _ => 1,
    };

    // Helper: is the current field a toggle?
    let is_toggle = matches!(
        (section, app.settings.modal_field),
        (0, 1) | (0, 2) | (0, 3) | (3, 0) | (3, 1) | (4, 6) | (4, 7)
    );
    match code {
        KeyCode::Esc => {
            app.modal = Modal::None;
        }
        KeyCode::Tab => {
            app.settings.modal_field = (app.settings.modal_field + 1) % max_fields;
        }
        KeyCode::Char(' ') if is_toggle => {
            let s = &mut app.settings;
            match (section, s.modal_field) {
                (0, 1) => s.policy_require_uppercase = !s.policy_require_uppercase,
                (0, 2) => s.policy_require_digit = !s.policy_require_digit,
                (0, 3) => s.policy_require_special = !s.policy_require_special,
                (3, 0) => s.allow_public_registration = !s.allow_public_registration,
                (3, 1) => s.require_email_verified = !s.require_email_verified,
                (4, 6) => s.smtp_use_tls = !s.smtp_use_tls,
                (4, 7) => s.smtp_enabled = !s.smtp_enabled,
                _ => {}
            }
        }
        KeyCode::Backspace if !is_toggle => {
            let s = &mut app.settings;
            match (section, s.modal_field) {
                (0, 0) => {
                    s.policy_min_length.pop();
                }
                (1, 0) => {
                    s.lockout_max_attempts.pop();
                }
                (1, 1) => {
                    s.lockout_window_minutes.pop();
                }
                (1, 2) => {
                    s.lockout_duration_minutes.pop();
                }
                (2, 0) => {
                    s.access_token_ttl_minutes.pop();
                }
                (2, 1) => {
                    s.refresh_token_ttl_days.pop();
                }
                (4, 0) => {
                    s.smtp_host.pop();
                }
                (4, 1) => {
                    s.smtp_port.pop();
                }
                (4, 2) => {
                    s.smtp_username.pop();
                }
                (4, 3) => {
                    s.smtp_password.pop();
                }
                (4, 4) => {
                    s.smtp_from_name.pop();
                }
                (4, 5) => {
                    s.smtp_from_email.pop();
                }
                _ => {}
            }
        }
        KeyCode::Char(c) if !is_toggle => {
            let s = &mut app.settings;
            // SMTP text fields accept arbitrary chars; others are digits-only
            if section == 4 {
                match s.modal_field {
                    0 => s.smtp_host.push(c),
                    1 if c.is_ascii_digit() => s.smtp_port.push(c),
                    2 => s.smtp_username.push(c),
                    3 => s.smtp_password.push(c),
                    4 => s.smtp_from_name.push(c),
                    5 => s.smtp_from_email.push(c),
                    _ => {}
                }
            } else if c.is_ascii_digit() {
                match (section, s.modal_field) {
                    (0, 0) => s.policy_min_length.push(c),
                    (1, 0) => s.lockout_max_attempts.push(c),
                    (1, 1) => s.lockout_window_minutes.push(c),
                    (1, 2) => s.lockout_duration_minutes.push(c),
                    (2, 0) => s.access_token_ttl_minutes.push(c),
                    (2, 1) => s.refresh_token_ttl_days.push(c),
                    _ => {}
                }
            }
        }
        KeyCode::Enter => {
            // Save then close
            let Some(tid) = app.active_tenant_id.clone() else {
                app.modal = Modal::None;
                return;
            };
            match section {
                0 => {
                    let min_len: i32 = app.settings.policy_min_length.parse().unwrap_or(8);
                    let uu = app.settings.policy_require_uppercase;
                    let dd = app.settings.policy_require_digit;
                    let ss = app.settings.policy_require_special;
                    match app
                        .client
                        .put_password_policy(&tid, min_len, uu, dd, ss)
                        .await
                    {
                        Ok(_) => app.set_status("Password policy saved"),
                        Err(e) => {
                            app.modal = Modal::Error(format!("{e}"));
                            return;
                        }
                    }
                }
                1 => {
                    let max_att: i32 = app.settings.lockout_max_attempts.parse().unwrap_or(5);
                    let win: i32 = app.settings.lockout_window_minutes.parse().unwrap_or(15);
                    let dur: i32 = app.settings.lockout_duration_minutes.parse().unwrap_or(15);
                    match app.client.put_lockout_policy(&tid, max_att, win, dur).await {
                        Ok(_) => app.set_status("Lockout policy saved"),
                        Err(e) => {
                            app.modal = Modal::Error(format!("{e}"));
                            return;
                        }
                    }
                }
                2 => {
                    let acc: i32 = app.settings.access_token_ttl_minutes.parse().unwrap_or(15);
                    let ref_days: i32 = app.settings.refresh_token_ttl_days.parse().unwrap_or(30);
                    match app.client.put_token_ttl(&tid, acc, ref_days).await {
                        Ok(_) => app.set_status("Token TTL saved"),
                        Err(e) => {
                            app.modal = Modal::Error(format!("{e}"));
                            return;
                        }
                    }
                }
                3 => {
                    let allow = app.settings.allow_public_registration;
                    let require = app.settings.require_email_verified;
                    match app
                        .client
                        .put_registration_policy(&tid, allow, require)
                        .await
                    {
                        Ok(_) => app.set_status("Registration policy saved"),
                        Err(e) => {
                            app.modal = Modal::Error(format!("{e}"));
                            return;
                        }
                    }
                }
                4 => {
                    let port: i32 = app.settings.smtp_port.parse().unwrap_or(587);
                    let pw = if app.settings.smtp_password.is_empty() {
                        None
                    } else {
                        Some(app.settings.smtp_password.as_str())
                    };
                    let host = app.settings.smtp_host.clone();
                    let uname = app.settings.smtp_username.clone();
                    let fname = app.settings.smtp_from_name.clone();
                    let femail = app.settings.smtp_from_email.clone();
                    let use_tls = app.settings.smtp_use_tls;
                    let smtp_enabled = app.settings.smtp_enabled;
                    match app
                        .client
                        .put_smtp(
                            &tid,
                            &host,
                            port,
                            &uname,
                            pw,
                            &fname,
                            &femail,
                            use_tls,
                            smtp_enabled,
                        )
                        .await
                    {
                        Ok(_) => {
                            app.settings.smtp_password_set = !app.settings.smtp_password.is_empty()
                                || app.settings.smtp_password_set;
                            app.settings.smtp_password = String::new();
                            app.set_status("SMTP config saved");
                        }
                        Err(e) => {
                            app.modal = Modal::Error(format!("{e}"));
                            return;
                        }
                    }
                }
                _ => {}
            }
            app.modal = Modal::None;
        }
        _ => {}
    }
}

// ── Settings tab helpers ──────────────────────────────────────────────────────

pub(super) async fn load_settings(app: &mut App, tid: String) {
    app.settings.loading = true;
    let client = app.client.clone();
    let (policy_r, lockout_r, tokens_r, reg_r, smtp_r) = tokio::join!(
        client.get_password_policy(&tid),
        client.get_lockout_policy(&tid),
        client.get_token_ttl(&tid),
        client.get_registration_policy(&tid),
        client.get_smtp(&tid),
    );
    if let Ok(p) = policy_r {
        app.settings.policy_min_length = p.min_length.to_string();
        app.settings.policy_require_uppercase = p.require_uppercase;
        app.settings.policy_require_digit = p.require_digit;
        app.settings.policy_require_special = p.require_special;
    }
    if let Ok(l) = lockout_r {
        app.settings.lockout_max_attempts = l.max_attempts.to_string();
        app.settings.lockout_window_minutes = l.window_minutes.to_string();
        app.settings.lockout_duration_minutes = l.duration_minutes.to_string();
    }
    if let Ok(t) = tokens_r {
        app.settings.access_token_ttl_minutes = t.access_token_ttl_minutes.to_string();
        app.settings.refresh_token_ttl_days = t.refresh_token_ttl_days.to_string();
    }
    if let Ok(r) = reg_r {
        app.settings.allow_public_registration = r.allow_public_registration;
        app.settings.require_email_verified = r.require_email_verified;
    }
    if let Ok(s) = smtp_r {
        app.settings.smtp_host = s.host;
        app.settings.smtp_port = s.port.to_string();
        app.settings.smtp_username = s.username;
        app.settings.smtp_password = String::new(); // never pre-fill password
        app.settings.smtp_from_name = s.from_name;
        app.settings.smtp_from_email = s.from_email;
        app.settings.smtp_use_tls = s.use_tls;
        app.settings.smtp_enabled = s.enabled;
        app.settings.smtp_password_set = s.password_set;
    }
    app.settings.loading = false;
}
