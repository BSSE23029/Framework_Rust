use super::*;

pub(super) async fn handle_login_key(app: &mut App, code: KeyCode) {
    let AppMode::Login {
        email,
        password,
        field,
        tenant_choices,
        tenant_choice_idx,
        ..
    } = app.mode.clone()
    else {
        return;
    };

    let mk_mode = |email: String,
                   password: String,
                   field: usize,
                   error: Option<String>,
                   tenant_choices: Option<Vec<(String, String)>>,
                   tenant_choice_idx: usize| {
        AppMode::Login {
            email,
            password,
            field,
            error,
            tenant_choices,
            tenant_choice_idx,
        }
    };

    // When tenant_choices is shown, Up/Down navigates the list; Enter picks one.
    let has_choices = tenant_choices
        .as_ref()
        .map(|c| !c.is_empty())
        .unwrap_or(false);

    match code {
        KeyCode::F(2) => {
            app.mode = AppMode::Passwordless {
                email,
                slug: String::new(),
                otp: String::new(),
                requested: false,
                field: 0,
                error: None,
            };
        }
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Tab => {
            app.mode = mk_mode(
                email,
                password,
                (field + 1) % 2,
                None,
                tenant_choices,
                tenant_choice_idx,
            );
        }
        // Tenant choice navigation when multiple tenants returned
        KeyCode::Up if has_choices => {
            let choices = tenant_choices.clone().unwrap();
            let idx = if tenant_choice_idx == 0 {
                choices.len() - 1
            } else {
                tenant_choice_idx - 1
            };
            app.mode = mk_mode(email, password, field, None, Some(choices), idx);
        }
        KeyCode::Down if has_choices => {
            let choices = tenant_choices.clone().unwrap();
            let idx = (tenant_choice_idx + 1) % choices.len();
            app.mode = mk_mode(email, password, field, None, Some(choices), idx);
        }
        KeyCode::Backspace => {
            let (mut e, mut p) = (email, password);
            match field {
                0 => {
                    e.pop();
                }
                1 => {
                    p.pop();
                }
                _ => {}
            }
            app.mode = mk_mode(e, p, field, None, None, 0);
        }
        KeyCode::Char(c) => {
            let (mut e, mut p) = (email, password);
            match field {
                0 => e.push(c),
                1 => p.push(c),
                _ => {}
            }
            app.mode = mk_mode(e, p, field, None, None, 0);
        }
        KeyCode::Enter => {
            if email.is_empty() || password.is_empty() {
                return;
            }

            // If tenant choices are shown, use the selected slug and call login directly.
            if has_choices {
                if let Some(choices) = &tenant_choices {
                    let slug = choices[tenant_choice_idx].0.clone();
                    let client = app.client.clone();
                    match client.login(&email, &password, &slug).await {
                        Ok(LoginResult::Token(token)) => {
                            app.client.set_token(token.clone());
                            let mut cfg = config_file::load();
                            cfg.token = Some(token);
                            let _ = config_file::save(&cfg);
                            app.mode = AppMode::Admin;
                            load_tenants(app).await;
                            check_health(app).await;
                            let only_master = app.tenants.len() <= 1
                                && app.tenants.iter().all(|t| t.slug == "master");
                            if only_master && slug == "master" {
                                app.modal = Modal::QuickStart(QuickStartState::default());
                            }
                        }
                        Ok(LoginResult::MfaRequired { mfa_token }) => {
                            app.mode = AppMode::MfaChallenge {
                                mfa_token,
                                slug,
                                code: String::new(),
                                error: None,
                            };
                        }
                        Ok(LoginResult::TenantChoice(_)) => {
                            // Shouldn't happen when specifying a slug
                        }
                        Err(e) => {
                            app.mode = mk_mode(
                                email,
                                password,
                                field,
                                Some(format!("{e}")),
                                tenant_choices,
                                tenant_choice_idx,
                            );
                        }
                    }
                }
                return;
            }

            // Universal login — no tenant needed.
            let client = app.client.clone();
            match client.login_universal(&email, &password).await {
                Ok(LoginResult::Token(token)) => {
                    app.client.set_token(token.clone());
                    let mut cfg = config_file::load();
                    cfg.token = Some(token);
                    let _ = config_file::save(&cfg);
                    app.mode = AppMode::Admin;
                    load_tenants(app).await;
                    check_health(app).await;
                    // Auto-open wizard if only the master tenant exists
                    let only_master =
                        app.tenants.len() <= 1 && app.tenants.iter().all(|t| t.slug == "master");
                    if only_master {
                        app.modal = Modal::QuickStart(QuickStartState::default());
                    }
                }
                Ok(LoginResult::MfaRequired { mfa_token }) => {
                    app.mode = AppMode::MfaChallenge {
                        mfa_token,
                        slug: String::from("master"),
                        code: String::new(),
                        error: None,
                    };
                }
                Ok(LoginResult::TenantChoice(choices)) => {
                    app.mode = mk_mode(email, password, field, None, Some(choices), 0);
                }
                Err(e) => {
                    app.mode = mk_mode(email, password, field, Some(format!("{e}")), None, 0);
                }
            }
        }
        _ => {}
    }
}

pub(super) async fn handle_mfa_key(app: &mut App, key: KeyCode) {
    let (mfa_token, slug, otp_code) = match &app.mode {
        AppMode::MfaChallenge {
            mfa_token,
            slug,
            code,
            ..
        } => (mfa_token.clone(), slug.clone(), code.clone()),
        _ => return,
    };

    match key {
        KeyCode::Esc => {
            app.mode = AppMode::Login {
                email: String::new(),
                password: String::new(),
                field: 0,
                error: None,
                tenant_choices: None,
                tenant_choice_idx: 0,
            };
        }
        KeyCode::Backspace => {
            if let AppMode::MfaChallenge { code, .. } = &mut app.mode {
                code.pop();
            }
        }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            if let AppMode::MfaChallenge { code, .. } = &mut app.mode {
                if code.len() < 6 {
                    code.push(c);
                }
            }
        }
        KeyCode::Enter => {
            if otp_code.len() != 6 {
                return;
            }
            let client = app.client.clone();
            match client.mfa_challenge(&slug, &mfa_token, &otp_code).await {
                Ok(token) => {
                    app.client.set_token(token.clone());
                    let mut cfg = config_file::load();
                    cfg.token = Some(token);
                    let _ = config_file::save(&cfg);
                    app.mode = AppMode::Admin;
                    load_tenants(app).await;
                    check_health(app).await;
                    let only_master =
                        app.tenants.len() <= 1 && app.tenants.iter().all(|t| t.slug == "master");
                    if only_master && slug == "master" {
                        app.modal = Modal::QuickStart(QuickStartState::default());
                    }
                }
                Err(ApiError::Api { status: 401, .. }) => {
                    if let AppMode::MfaChallenge { error, code, .. } = &mut app.mode {
                        *error = Some("Invalid code".into());
                        code.clear();
                    }
                }
                Err(e) => {
                    if let AppMode::MfaChallenge { error, code, .. } = &mut app.mode {
                        *error = Some(format!("Error: {e}"));
                        code.clear();
                    }
                }
            }
        }
        _ => {}
    }
}

pub(super) async fn handle_passwordless_key(app: &mut App, key: KeyCode) {
    let AppMode::Passwordless {
        mut email,
        mut slug,
        mut otp,
        mut requested,
        mut field,
        mut error,
    } = app.mode.clone()
    else {
        return;
    };
    match key {
        KeyCode::F(2) | KeyCode::Esc if !requested => {
            app.mode = AppMode::Login {
                email,
                password: String::new(),
                field: 0,
                error: None,
                tenant_choices: None,
                tenant_choice_idx: 0,
            };
            return;
        }
        KeyCode::Esc if requested => {
            requested = false;
            otp.clear();
            field = 0;
        }
        KeyCode::Tab if !requested => field = (field + 1) % 2,
        KeyCode::Backspace => {
            if requested {
                otp.pop();
            } else if field == 0 {
                email.pop();
            } else {
                slug.pop();
            }
        }
        KeyCode::Char(character) => {
            if requested {
                if character.is_ascii_digit() && otp.len() < 6 {
                    otp.push(character);
                }
            } else if field == 0 {
                email.push(character);
            } else {
                slug.push(character);
            }
        }
        KeyCode::Enter if !requested && !email.is_empty() && !slug.is_empty() => {
            match app.client.request_passwordless(&email, &slug).await {
                Ok(()) => {
                    requested = true;
                    field = 0;
                    error = None;
                }
                Err(api_error) => error = Some(api_error.to_string()),
            }
        }
        KeyCode::Enter if requested && otp.len() == 6 => {
            match app.client.verify_passwordless(&email, &slug, &otp).await {
                Ok(LoginResult::Token(token)) => {
                    app.client.set_token(token.clone());
                    let mut config = config_file::load();
                    config.token = Some(token);
                    let _ = config_file::save(&config);
                    app.mode = AppMode::Admin;
                    load_tenants(app).await;
                    check_health(app).await;
                    return;
                }
                Ok(LoginResult::MfaRequired { mfa_token }) => {
                    app.mode = AppMode::MfaChallenge {
                        mfa_token,
                        slug,
                        code: String::new(),
                        error: None,
                    };
                    return;
                }
                Ok(LoginResult::TenantChoice(_)) => {
                    error = Some("Passwordless login requires a tenant slug".into());
                }
                Err(api_error) => {
                    otp.clear();
                    error = Some(api_error.to_string());
                }
            }
        }
        _ => {}
    }
    app.mode = AppMode::Passwordless {
        email,
        slug,
        otp,
        requested,
        field,
        error,
    };
}
