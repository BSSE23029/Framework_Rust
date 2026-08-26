use super::*;

pub(super) async fn handle(app: &mut App, code: KeyCode) -> bool {
    match app.modal.clone() {
        Modal::CreatePermission {
            mut name,
            mut description,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 2;
                    app.modal = Modal::CreatePermission {
                        name,
                        description,
                        field,
                    };
                }
                KeyCode::Enter if !name.is_empty() => {
                    let n = name.clone();
                    let d = description.clone();
                    app.modal = Modal::None;
                    perform_create_permission(app, n, d).await;
                }
                KeyCode::Backspace => {
                    if field == 0 {
                        name.pop();
                    } else {
                        description.pop();
                    }
                    app.modal = Modal::CreatePermission {
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
                    app.modal = Modal::CreatePermission {
                        name,
                        description,
                        field,
                    };
                }
                _ => {}
            }
            true
        }
        Modal::CreateDomain {
            mut domain_name,
            mut domain_type,
            mut is_primary,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 3;
                    app.modal = Modal::CreateDomain {
                        domain_name,
                        domain_type,
                        is_primary,
                        field,
                    };
                }
                KeyCode::Enter if !domain_name.is_empty() && !domain_type.is_empty() => {
                    app.modal = Modal::None;
                    perform_create_domain(app, domain_name, domain_type, is_primary).await;
                }
                KeyCode::Char(' ') if field == 2 => {
                    is_primary = !is_primary;
                    app.modal = Modal::CreateDomain {
                        domain_name,
                        domain_type,
                        is_primary,
                        field,
                    };
                }
                KeyCode::Backspace if field < 2 => {
                    if field == 0 {
                        domain_name.pop();
                    } else {
                        domain_type.pop();
                    }
                    app.modal = Modal::CreateDomain {
                        domain_name,
                        domain_type,
                        is_primary,
                        field,
                    };
                }
                KeyCode::Char(c) if field < 2 => {
                    if field == 0 {
                        domain_name.push(c);
                    } else {
                        domain_type.push(c);
                    }
                    app.modal = Modal::CreateDomain {
                        domain_name,
                        domain_type,
                        is_primary,
                        field,
                    };
                }
                _ => {}
            }
            true
        }
        Modal::CreatePlan {
            code: mut plan_code,
            mut name,
            mut interval,
            mut price_cents,
            mut currency,
            mut trial_days,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => field = (field + 1) % 6,
                KeyCode::Enter
                    if !plan_code.is_empty()
                        && !name.is_empty()
                        && price_cents.parse::<i64>().is_ok()
                        && trial_days.parse::<i32>().is_ok() =>
                {
                    app.modal = Modal::None;
                    perform_create_plan(
                        app,
                        plan_code,
                        name,
                        interval,
                        price_cents.parse().unwrap_or_default(),
                        currency,
                        trial_days.parse().unwrap_or_default(),
                    )
                    .await;
                    return true;
                }
                KeyCode::Backspace => {
                    let values = [
                        &mut plan_code,
                        &mut name,
                        &mut interval,
                        &mut price_cents,
                        &mut currency,
                        &mut trial_days,
                    ];
                    values[field].pop();
                }
                KeyCode::Char(character) => {
                    let values = [
                        &mut plan_code,
                        &mut name,
                        &mut interval,
                        &mut price_cents,
                        &mut currency,
                        &mut trial_days,
                    ];
                    values[field].push(character);
                }
                _ => {}
            }
            if !matches!(app.modal, Modal::None) {
                app.modal = Modal::CreatePlan {
                    code: plan_code,
                    name,
                    interval,
                    price_cents,
                    currency,
                    trial_days,
                    field,
                };
            }
            true
        }
        Modal::SetPlanQuota {
            mut resource,
            mut limit,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => field = (field + 1) % 2,
                KeyCode::Enter if !resource.is_empty() && limit.parse::<i64>().is_ok() => {
                    app.modal = Modal::None;
                    perform_set_plan_quota(app, resource, limit.parse().unwrap_or_default()).await;
                    return true;
                }
                KeyCode::Backspace => {
                    if field == 0 {
                        resource.pop();
                    } else {
                        limit.pop();
                    }
                }
                KeyCode::Char(character) => {
                    if field == 0 {
                        resource.push(character);
                    } else {
                        limit.push(character);
                    }
                }
                _ => {}
            }
            if !matches!(app.modal, Modal::None) {
                app.modal = Modal::SetPlanQuota {
                    resource,
                    limit,
                    field,
                };
            }
            true
        }
        Modal::DefineConfig {
            mut config_key,
            mut display_name,
            mut category,
            mut value_type,
            mut default_value,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => field = (field + 1) % 5,
                KeyCode::Enter if !config_key.is_empty() && !display_name.is_empty() => {
                    let parsed = if default_value.trim().is_empty() {
                        Ok(None)
                    } else {
                        serde_json::from_str(&default_value).map(Some)
                    };
                    match parsed {
                        Ok(value) => {
                            app.modal = Modal::None;
                            perform_define_config(
                                app,
                                config_key,
                                display_name,
                                category,
                                value_type,
                                value,
                            )
                            .await;
                            return true;
                        }
                        Err(error) => {
                            app.modal = Modal::Error(format!("Invalid default JSON: {error}"));
                            return true;
                        }
                    }
                }
                KeyCode::Backspace => {
                    let values = [
                        &mut config_key,
                        &mut display_name,
                        &mut category,
                        &mut value_type,
                        &mut default_value,
                    ];
                    values[field].pop();
                }
                KeyCode::Char(character) => {
                    let values = [
                        &mut config_key,
                        &mut display_name,
                        &mut category,
                        &mut value_type,
                        &mut default_value,
                    ];
                    values[field].push(character);
                }
                _ => {}
            }
            if !matches!(app.modal, Modal::None) {
                app.modal = Modal::DefineConfig {
                    config_key,
                    display_name,
                    category,
                    value_type,
                    default_value,
                    field,
                };
            }
            true
        }
        Modal::SetConfigValue { mut value } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Enter => match serde_json::from_str(&value) {
                    Ok(parsed) => {
                        app.modal = Modal::None;
                        perform_set_config_value(app, parsed).await;
                        return true;
                    }
                    Err(error) => {
                        app.modal = Modal::Error(format!("Invalid JSON: {error}"));
                        return true;
                    }
                },
                KeyCode::Backspace => {
                    value.pop();
                }
                KeyCode::Char(character) => value.push(character),
                _ => {}
            }
            if !matches!(app.modal, Modal::None) {
                app.modal = Modal::SetConfigValue { value };
            }
            true
        }
        Modal::TrustDevice {
            mut name,
            mut ttl_days,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => field = (field + 1) % 2,
                KeyCode::Enter if !name.is_empty() && ttl_days.parse::<i64>().is_ok() => {
                    app.modal = Modal::None;
                    perform_trust_device(app, name, ttl_days.parse().unwrap_or(30)).await;
                    return true;
                }
                KeyCode::Backspace => {
                    if field == 0 {
                        name.pop();
                    } else {
                        ttl_days.pop();
                    }
                }
                KeyCode::Char(character) => {
                    if field == 0 {
                        name.push(character);
                    } else {
                        ttl_days.push(character);
                    }
                }
                _ => {}
            }
            if !matches!(app.modal, Modal::None) {
                app.modal = Modal::TrustDevice {
                    name,
                    ttl_days,
                    field,
                };
            }
            true
        }
        Modal::EditPermission {
            id,
            mut name,
            mut description,
            mut field,
        } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 2;
                    app.modal = Modal::EditPermission {
                        id,
                        name,
                        description,
                        field,
                    };
                }
                KeyCode::Enter if !name.is_empty() => {
                    let id2 = id.clone();
                    let n = name.clone();
                    let d = description.clone();
                    app.modal = Modal::None;
                    perform_edit_permission(app, id2, n, d).await;
                }
                KeyCode::Backspace => {
                    if field == 0 {
                        name.pop();
                    } else {
                        description.pop();
                    }
                    app.modal = Modal::EditPermission {
                        id,
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
                    app.modal = Modal::EditPermission {
                        id,
                        name,
                        description,
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
