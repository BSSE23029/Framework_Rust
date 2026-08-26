use super::*;
use ratatui::Frame;

pub(super) fn render_modal(frame: &mut Frame, app: &App) {
    match &app.modal.clone() {
        Modal::None => {}
        Modal::ConfirmDelete { id: _, label } => {
            modal::render_confirm(frame, label);
        }
        Modal::ShowSecret { client_id, secret } => {
            modal::render_secret(frame, client_id, secret);
        }
        Modal::Error(msg) => {
            modal::render_error(frame, msg);
        }
        Modal::CreateTenant { name, slug, field } => {
            modal::render_form(
                frame,
                "New Tenant",
                &[
                    ("Name", name, "e.g. Acme Corp"),
                    ("Slug", slug, "e.g. acme (URL-safe, unique)"),
                ],
                *field,
            );
        }
        Modal::TenantHierarchy {
            tenant_type,
            parent_tenant_id,
            field,
            ..
        } => modal::render_form(
            frame,
            "Tenant Hierarchy",
            &[
                ("Type", tenant_type, "organization, hotel, or branch"),
                ("Parent tenant ID", parent_tenant_id, "blank for root"),
            ],
            *field,
        ),
        Modal::CreateClient {
            name,
            redirect_uri,
            scopes,
            client_type,
            field,
        } => {
            modal::render_create_client(frame, name, redirect_uri, scopes, *client_type, *field);
        }
        Modal::CreateUser {
            email,
            password,
            field,
        } => {
            modal::render_form(
                frame,
                "New User",
                &[
                    ("Email", email, "user@example.com"),
                    ("Password", password, "min 8 characters"),
                ],
                *field,
            );
        }
        Modal::QuickStart(_) => {
            ui::quickstart::render(frame, app);
        }
        Modal::EditClient {
            name,
            redirect_uris,
            scopes,
            access_token_ttl,
            refresh_token_ttl,
            client_type,
            field,
            ..
        } => {
            modal::render_edit_client(
                frame,
                name,
                redirect_uris,
                scopes,
                access_token_ttl,
                refresh_token_ttl,
                *client_type,
                *field,
            );
        }
        Modal::CreateIdp {
            provider,
            client_id,
            client_secret,
            redirect_url,
            scopes,
            field,
        } => {
            modal::render_form(
                frame,
                "New Identity Provider",
                &[
                    ("Provider", provider, "google  or  github"),
                    (
                        "Client ID",
                        client_id,
                        "OAuth app Client ID from provider console",
                    ),
                    (
                        "Client Secret",
                        client_secret,
                        "OAuth app Client Secret from provider console",
                    ),
                    (
                        "Redirect URL",
                        redirect_url,
                        "https://your-domain/auth/callback/google",
                    ),
                    ("Scopes", scopes, "email profile  (space-separated)"),
                ],
                *field,
            );
        }
        Modal::EditIdp {
            provider,
            client_id,
            client_secret,
            redirect_url,
            scopes,
            enabled,
            field,
            ..
        } => {
            let enabled_str = if *enabled { "true" } else { "false" };
            modal::render_form(
                frame,
                &format!("Edit Identity Provider — {provider}"),
                &[
                    (
                        "Client ID",
                        client_id,
                        "OAuth app Client ID from provider console",
                    ),
                    (
                        "Client Secret",
                        client_secret,
                        "blank = keep existing secret",
                    ),
                    (
                        "Redirect URL",
                        redirect_url,
                        "https://your-domain/auth/callback/google",
                    ),
                    ("Scopes", scopes, "email profile  (space-separated)"),
                    ("Enabled", enabled_str, "Space to toggle"),
                ],
                *field,
            );
        }
        Modal::EditUser {
            email,
            password,
            is_active,
            all_roles,
            permissions,
            passkeys,
            field,
            role_selected,
            passkey_selected,
            ..
        } => {
            modal::render_edit_user(
                frame,
                email,
                password,
                *is_active,
                all_roles,
                permissions,
                passkeys,
                *field,
                *role_selected,
                *passkey_selected,
            );
        }
        Modal::CreateRole {
            name,
            description,
            field,
        } => {
            modal::render_form(
                frame,
                "New Role",
                &[
                    ("Name", name, "e.g. admin, editor, viewer"),
                    ("Description", description, "optional description"),
                ],
                *field,
            );
        }
        Modal::EditRole {
            name,
            description,
            all_permissions,
            field,
            perm_selected,
            ..
        } => {
            modal::render_edit_role(
                frame,
                name,
                description,
                all_permissions,
                *field,
                *perm_selected,
            );
        }
        Modal::CreatePermission {
            name,
            description,
            field,
        } => {
            modal::render_form(
                frame,
                "New Permission",
                &[
                    ("Name", name, "e.g. documents:read"),
                    ("Description", description, "optional description"),
                ],
                *field,
            );
        }
        Modal::CreateDomain {
            domain_name,
            domain_type,
            is_primary,
            field,
        } => {
            let primary = if *is_primary { "yes" } else { "no" };
            modal::render_form(
                frame,
                "New Tenant Domain",
                &[
                    ("Domain", domain_name, "login.example.com"),
                    ("Type", domain_type, "custom or platform"),
                    ("Primary", primary, "Space toggles"),
                ],
                *field,
            );
        }
        Modal::CreatePlan {
            code,
            name,
            interval,
            price_cents,
            currency,
            trial_days,
            field,
        } => modal::render_form(
            frame,
            "New Plan",
            &[
                ("Code", code, "pro"),
                ("Name", name, "Professional"),
                ("Interval", interval, "monthly/yearly/one_time/lifetime"),
                ("Price cents", price_cents, "2500"),
                ("Currency", currency, "USD"),
                ("Trial days", trial_days, "14"),
            ],
            *field,
        ),
        Modal::SetPlanQuota {
            resource,
            limit,
            field,
        } => modal::render_form(
            frame,
            "Set Plan Quota",
            &[
                ("Resource", resource, "users/storage_mb/api_requests"),
                ("Limit", limit, "-1 means unlimited"),
            ],
            *field,
        ),
        Modal::DefineConfig {
            config_key,
            display_name,
            category,
            value_type,
            default_value,
            field,
        } => modal::render_form(
            frame,
            "Define Tenant Configuration",
            &[
                ("Key", config_key, "check_in_time"),
                ("Display name", display_name, "Check-in time"),
                ("Category", category, "operations"),
                ("Type", value_type, "string/integer/boolean/json/..."),
                ("Default JSON", default_value, "\"15:00\" or 15 or true"),
            ],
            *field,
        ),
        Modal::SetConfigValue { value } => modal::render_form(
            frame,
            "Set Tenant Configuration Value",
            &[("Value JSON", value, "\"text\", 42, true, or {...}")],
            0,
        ),
        Modal::TrustDevice {
            name,
            ttl_days,
            field,
        } => modal::render_form(
            frame,
            "Trust This Device",
            &[
                ("Name", name, "Raza's laptop"),
                ("TTL days", ttl_days, "30"),
            ],
            *field,
        ),
        Modal::AccessForm {
            action,
            fields,
            field,
        } => modal::render_access_form(frame, *action, fields, *field),
        Modal::EditPermission {
            name,
            description,
            field,
            ..
        } => {
            modal::render_form(
                frame,
                "Edit Permission",
                &[
                    ("Name", name, "e.g. documents:read"),
                    ("Description", description, "optional description"),
                ],
                *field,
            );
        }
        Modal::ClientRoles {
            client_name,
            all_roles,
            selected,
            ..
        } => {
            modal::render_user_roles(frame, client_name, all_roles, *selected);
        }
        Modal::EditSettings(section) => {
            let s = &app.settings;
            let f = s.modal_field;
            match section {
                0 => modal::render_settings_password_policy(
                    frame,
                    &s.policy_min_length,
                    s.policy_require_uppercase,
                    s.policy_require_digit,
                    s.policy_require_special,
                    f,
                ),
                1 => modal::render_settings_lockout(
                    frame,
                    &s.lockout_max_attempts,
                    &s.lockout_window_minutes,
                    &s.lockout_duration_minutes,
                    f,
                ),
                2 => modal::render_settings_tokens(
                    frame,
                    &s.access_token_ttl_minutes,
                    &s.refresh_token_ttl_days,
                    f,
                ),
                3 => modal::render_settings_registration(
                    frame,
                    s.allow_public_registration,
                    s.require_email_verified,
                    f,
                ),
                4 => modal::render_settings_smtp(
                    frame,
                    &s.smtp_host,
                    &s.smtp_port,
                    &s.smtp_username,
                    &s.smtp_password,
                    &s.smtp_from_name,
                    &s.smtp_from_email,
                    s.smtp_use_tls,
                    s.smtp_enabled,
                    f,
                ),
                _ => {}
            }
        }
        Modal::PostCreateTenant { tenant_name, .. } => {
            modal::render_post_create_tenant(frame, tenant_name);
        }
    }
}
