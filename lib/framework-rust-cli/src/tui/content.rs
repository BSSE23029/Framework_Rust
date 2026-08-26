use super::*;

pub(super) async fn handle_content_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.focus = Focus::Sidebar;
        }
        KeyCode::Char('c') if !app.access_contexts.is_empty() => {
            app.access_context_selected =
                (app.access_context_selected + 1) % app.access_contexts.len();
            let selected = app.active_access_context().cloned();
            app.client
                .set_actor_context(selected.as_ref().map(|context| context.urdd_id.clone()));
            if let Some(context) = selected {
                app.set_status(format!(
                    "Acting as {} / {} / {}",
                    context.role_name, context.designation_name, context.department_name
                ));
            }
        }
        // ── Main tab navigation ──
        KeyCode::Left => {
            app.tab = navigation::previous_tab(&app.tab);
            load_current_tab(app).await;
        }
        KeyCode::Right => {
            app.tab = navigation::next_tab(&app.tab);
            load_current_tab(app).await;
        }
        // ── Settings section list navigation ──
        KeyCode::Up if app.tab == Tab::Settings => {
            app.settings.section_selected = app.settings.section_selected.saturating_sub(1);
        }
        KeyCode::Down if app.tab == Tab::Settings && app.settings.section_selected < 4 => {
            app.settings.section_selected += 1;
        }
        KeyCode::Enter if app.tab == Tab::Settings => {
            app.settings.modal_field = 0;
            app.modal = Modal::EditSettings(app.settings.section_selected as u8);
        }
        KeyCode::Up => navigation::move_up(app).await,
        KeyCode::Down => navigation::move_down(app).await,
        KeyCode::Char('n') => match app.tab {
            Tab::Clients => {
                if app.active_tenant_id.is_some() {
                    app.modal = Modal::CreateClient {
                        name: String::new(),
                        redirect_uri: String::new(),
                        scopes: String::from("openid email profile"),
                        client_type: 0,
                        field: 0,
                    };
                }
            }
            Tab::Users => {
                if app.active_tenant_id.is_some() {
                    app.modal = Modal::CreateUser {
                        email: String::new(),
                        password: String::new(),
                        field: 0,
                    };
                }
            }
            Tab::Roles => {
                if app.active_tenant_id.is_some() {
                    app.modal = Modal::CreateRole {
                        name: String::new(),
                        description: String::new(),
                        field: 0,
                    };
                }
            }
            Tab::Permissions => {
                if app.active_tenant_id.is_some() {
                    app.modal = Modal::CreatePermission {
                        name: String::new(),
                        description: String::new(),
                        field: 0,
                    };
                }
            }
            Tab::Access => {
                let action = app.access_section;
                app.modal = Modal::AccessForm {
                    action,
                    fields: vec![String::new(); modal::access_field_count(action)],
                    field: 0,
                };
            }
            Tab::Sessions => {}
            Tab::Settings => {}
            Tab::AuditLog => {}
            Tab::Domains => {
                if app.active_tenant_id.is_some() {
                    app.modal = Modal::CreateDomain {
                        domain_name: String::new(),
                        domain_type: String::from("custom"),
                        is_primary: false,
                        field: 0,
                    };
                }
            }
            Tab::Plans => {
                app.modal = Modal::CreatePlan {
                    code: String::new(),
                    name: String::new(),
                    interval: "monthly".into(),
                    price_cents: "0".into(),
                    currency: "USD".into(),
                    trial_days: "14".into(),
                    field: 0,
                };
            }
            Tab::Config => {
                app.modal = Modal::DefineConfig {
                    config_key: String::new(),
                    display_name: String::new(),
                    category: "general".into(),
                    value_type: "string".into(),
                    default_value: String::new(),
                    field: 0,
                };
            }
            Tab::Security => {
                app.modal = Modal::TrustDevice {
                    name: String::new(),
                    ttl_days: "30".into(),
                    field: 0,
                };
            }
            Tab::IdentityProviders => {
                if app.active_tenant_id.is_some() {
                    app.modal = Modal::CreateIdp {
                        provider: String::from("google"),
                        client_id: String::new(),
                        client_secret: String::new(),
                        redirect_url: String::new(),
                        scopes: String::from("email profile"),
                        field: 0,
                    };
                }
            }
        },
        KeyCode::Char('e') => match app.tab {
            Tab::Clients => {
                if let Some(c) = app.selected_client() {
                    let client_type = if c.grant_types.iter().any(|g| g == "client_credentials") {
                        2 // Machine/M2M
                    } else if !c.is_confidential {
                        1 // SPA/Mobile
                    } else {
                        0 // Confidential/Web
                    };
                    app.modal = Modal::EditClient {
                        id: c.id.clone(),
                        name: c.name.clone(),
                        redirect_uris: c.redirect_uris.join(", "),
                        scopes: c.scopes.join(" "),
                        access_token_ttl: c
                            .access_token_ttl_minutes
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                        refresh_token_ttl: c
                            .refresh_token_ttl_days
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                        client_type,
                        field: 0,
                    };
                }
            }
            Tab::Users => {
                if let (Some(u), Some(tid)) =
                    (app.selected_user().cloned(), app.active_tenant_id.clone())
                {
                    open_edit_user(app, u.id, u.email, u.is_active, tid).await;
                }
            }
            Tab::Roles => {
                if let (Some(r), Some(tid)) =
                    (app.selected_role().cloned(), app.active_tenant_id.clone())
                {
                    open_edit_role(app, r.id, r.name, r.description, tid).await;
                }
            }
            Tab::Permissions => {
                if let Some(p) = app.selected_permission() {
                    app.modal = Modal::EditPermission {
                        id: p.id.clone(),
                        name: p.name.clone(),
                        description: p.description.clone(),
                        field: 0,
                    };
                }
            }
            Tab::Access => {}
            Tab::Sessions => {}
            Tab::Settings => {}
            Tab::AuditLog => {}
            Tab::Domains => {}
            Tab::Plans => {}
            Tab::Config => {
                if let Some(definition) = app.selected_config() {
                    let value = app
                        .config_values
                        .get(&definition.id)
                        .cloned()
                        .or_else(|| definition.default_value.as_ref().map(ToString::to_string))
                        .unwrap_or_else(|| "null".into());
                    app.modal = Modal::SetConfigValue { value };
                }
            }
            Tab::Security => {}
            Tab::IdentityProviders => {
                if let Some(idp) = app.selected_idp() {
                    app.modal = Modal::EditIdp {
                        id: idp.id.clone(),
                        provider: idp.provider.clone(),
                        client_id: idp.client_id.clone(),
                        client_secret: String::new(),
                        redirect_url: idp.redirect_url.clone(),
                        scopes: idp.scopes.join(" "),
                        enabled: idp.enabled,
                        field: 0,
                    };
                }
            }
        },
        KeyCode::Char('l') if app.tab == Tab::Clients => {
            if let (Some(c), Some(tid)) =
                (app.selected_client().cloned(), app.active_tenant_id.clone())
            {
                let is_m2m = c.grant_types.iter().any(|g| g == "client_credentials")
                    && !c.grant_types.iter().any(|g| g == "authorization_code");
                if is_m2m {
                    open_client_roles(app, c.id, c.name, tid).await;
                } else {
                    app.set_status("Roles only available for M2M (Machine) clients");
                }
            }
        }
        KeyCode::Char('r') => {
            load_current_tab(app).await;
        }
        KeyCode::Char('v') if app.tab == Tab::Domains => {
            perform_domain_action(app, "verify").await;
        }
        KeyCode::Char('p') if app.tab == Tab::Domains => {
            perform_domain_action(app, "primary").await;
        }
        KeyCode::Char('u') if app.tab == Tab::Plans && app.selected_plan().is_some() => {
            app.modal = Modal::SetPlanQuota {
                resource: String::new(),
                limit: String::new(),
                field: 0,
            };
        }
        KeyCode::Char('s') if app.tab == Tab::Plans => {
            perform_subscribe_plan(app).await;
        }
        KeyCode::Char('m') if app.tab == Tab::Users => {
            if let (Some(u), Some(tid)) =
                (app.selected_user().cloned(), app.active_tenant_id.clone())
            {
                if u.mfa_enabled {
                    match app.client.admin_disable_user_mfa(&tid, &u.id).await {
                        Ok(_) => {
                            app.set_status(format!("MFA disabled for {}", u.email));
                            load_users(app, tid).await;
                        }
                        Err(e) => app.set_status(format!("Error: {e}")),
                    }
                } else {
                    app.set_status("MFA is not enabled for this user");
                }
            }
        }
        KeyCode::Char('a') if app.tab == Tab::Access => {
            let action = match app.access_section {
                2 => Some(7),
                4 => Some(6),
                5 => Some(8),
                _ => None,
            };
            if let Some(action) = action {
                app.modal = Modal::AccessForm {
                    action,
                    fields: vec![String::new(); modal::access_field_count(action)],
                    field: 0,
                };
            } else {
                app.set_status("Link/assign is available in RDDs, Groups, and Scopes");
            }
        }
        KeyCode::Char('d') => match app.tab {
            Tab::Clients => {
                if let Some(c) = app.selected_client() {
                    app.modal = Modal::ConfirmDelete {
                        id: c.id.clone(),
                        label: c.name.clone(),
                    };
                }
            }
            Tab::Users => {
                if let Some(u) = app.selected_user() {
                    app.modal = Modal::ConfirmDelete {
                        id: u.id.clone(),
                        label: u.email.clone(),
                    };
                }
            }
            Tab::Roles => {
                if let Some(r) = app.selected_role() {
                    app.modal = Modal::ConfirmDelete {
                        id: r.id.clone(),
                        label: r.name.clone(),
                    };
                }
            }
            Tab::Permissions => {
                if let Some(p) = app.selected_permission() {
                    app.modal = Modal::ConfirmDelete {
                        id: p.id.clone(),
                        label: p.name.clone(),
                    };
                }
            }
            Tab::Access => {
                if app.access_section == 3 {
                    if let Some(context) = app.urdds.get(app.access_selected) {
                        app.modal = Modal::ConfirmDelete {
                            id: context.id.clone(),
                            label: format!("URDD {}", context.id),
                        };
                    }
                }
            }
            Tab::Sessions => {
                if let Some(s) = app.selected_session() {
                    app.modal = Modal::ConfirmDelete {
                        id: s.id.clone(),
                        label: s.email.clone(),
                    };
                }
            }
            Tab::Settings => {}
            Tab::AuditLog => {}
            Tab::Domains => {
                if let Some(domain) = app.selected_domain() {
                    app.modal = Modal::ConfirmDelete {
                        id: domain.id.clone(),
                        label: domain.domain_name.clone(),
                    };
                }
            }
            Tab::Plans => {}
            Tab::Config => {}
            Tab::Security => {
                if let Some(device) = app.selected_trusted_device() {
                    app.modal = Modal::ConfirmDelete {
                        id: device.id.clone(),
                        label: device.name.clone(),
                    };
                }
            }
            Tab::IdentityProviders => {
                if let Some(idp) = app.selected_idp() {
                    app.modal = Modal::ConfirmDelete {
                        id: idp.id.clone(),
                        label: idp.provider.clone(),
                    };
                }
            }
        },
        KeyCode::Char('x') if app.tab == Tab::AuditLog => {
            if let Some(tid) = app.active_tenant_id.clone() {
                export_audit_log_csv(app, tid).await;
            }
        }
        KeyCode::Char('[') if app.tab == Tab::Access => {
            app.access_section = app.access_section.saturating_sub(1);
            app.access_selected = 0;
        }
        KeyCode::Char(']') if app.tab == Tab::Access => {
            app.access_section = (app.access_section + 1).min(5);
            app.access_selected = 0;
        }
        _ => {}
    }
}
