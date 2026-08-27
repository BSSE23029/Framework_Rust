pub use sea_orm_migration::prelude::*;

#[path = "phase_01/m20240101_000001_create_tenants.rs"]
mod m20240101_000001_create_tenants;
#[path = "phase_01/m20240101_000002_create_users.rs"]
mod m20240101_000002_create_users;
#[path = "phase_01/m20240101_000003_create_refresh_tokens.rs"]
mod m20240101_000003_create_refresh_tokens;
#[path = "phase_01/m20240101_000004_create_oauth_accounts.rs"]
mod m20240101_000004_create_oauth_accounts;
#[path = "phase_01/m20240101_000005_security_hardening.rs"]
mod m20240101_000005_security_hardening;
#[path = "phase_02/m20240101_000006_oauth_clients.rs"]
mod m20240101_000006_oauth_clients;
#[path = "phase_02/m20240101_000007_authorization_codes.rs"]
mod m20240101_000007_authorization_codes;
#[path = "phase_02/m20240101_000008_sessions.rs"]
mod m20240101_000008_sessions;
#[path = "phase_02/m20240101_000009_fix_rls_force.rs"]
mod m20240101_000009_fix_rls_force;
#[path = "phase_02/m20240101_000010_revoked_jtis.rs"]
mod m20240101_000010_revoked_jtis;
#[path = "phase_03/m20240101_000011_sessions_last_seen.rs"]
mod m20240101_000011_sessions_last_seen;
#[path = "phase_03/m20240101_000012_roles.rs"]
mod m20240101_000012_roles;
#[path = "phase_03/m20240101_000013_permissions.rs"]
mod m20240101_000013_permissions;
#[path = "phase_03/m20240101_000014_sessions_rls.rs"]
mod m20240101_000014_sessions_rls;
#[path = "phase_03/m20240101_000015_rls_app_role.rs"]
mod m20240101_000015_rls_app_role;
#[path = "phase_04/m20240101_000016_user_lifecycle.rs"]
mod m20240101_000016_user_lifecycle;
#[path = "phase_04/m20240101_000017_tenant_settings.rs"]
mod m20240101_000017_tenant_settings;
#[path = "phase_04/m20240101_000018_client_roles.rs"]
mod m20240101_000018_client_roles;
#[path = "phase_04/m20240101_000019_client_token_ttl.rs"]
mod m20240101_000019_client_token_ttl;
#[path = "phase_04/m20240101_000020_identity_providers.rs"]
mod m20240101_000020_identity_providers;
#[path = "phase_05/m20240101_000021_totp_secrets.rs"]
mod m20240101_000021_totp_secrets;
#[path = "phase_05/m20240101_000022_rename_role.rs"]
mod m20240101_000022_rename_role;
#[path = "phase_05/m20240101_000023_tenant_smtp_config.rs"]
mod m20240101_000023_tenant_smtp_config;
#[path = "phase_05/m20240101_000024_webauthn_credentials.rs"]
mod m20240101_000024_webauthn_credentials;
#[path = "phase_05/m20240101_000025_rate_limit_buckets.rs"]
mod m20240101_000025_rate_limit_buckets;
#[path = "phase_06/m20240101_000026_mfa_backup_codes.rs"]
mod m20240101_000026_mfa_backup_codes;
#[path = "phase_06/m20240101_000027_password_history.rs"]
mod m20240101_000027_password_history;
#[path = "phase_06/m20240101_000028_audit_and_contextual_rbac.rs"]
mod m20240101_000028_audit_and_contextual_rbac;
#[path = "phase_06/m20240101_000029_audit_actor_context.rs"]
mod m20240101_000029_audit_actor_context;
#[path = "phase_06/m20240101_000030_tenant_hierarchy_domains.rs"]
mod m20240101_000030_tenant_hierarchy_domains;
#[path = "phase_07/m20240101_000031_resource_permission_scopes.rs"]
mod m20240101_000031_resource_permission_scopes;
#[path = "phase_07/m20240101_000032_trusted_devices.rs"]
mod m20240101_000032_trusted_devices;
#[path = "phase_07/m20240101_000033_passwordless_email_otp.rs"]
mod m20240101_000033_passwordless_email_otp;
#[path = "phase_07/m20240101_000034_one_time_token_rls.rs"]
mod m20240101_000034_one_time_token_rls;
#[path = "phase_07/m20240101_000035_plans_subscriptions_quotas.rs"]
mod m20240101_000035_plans_subscriptions_quotas;
#[path = "phase_08/m20240101_000036_typed_tenant_config.rs"]
mod m20240101_000036_typed_tenant_config;
#[path = "phase_08/m20240101_000037_request_activity.rs"]
mod m20240101_000037_request_activity;
#[path = "phase_08/m20240101_000038_authoritative_actor_audit.rs"]
mod m20240101_000038_authoritative_actor_audit;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_tenants::Migration),
            Box::new(m20240101_000002_create_users::Migration),
            Box::new(m20240101_000003_create_refresh_tokens::Migration),
            Box::new(m20240101_000004_create_oauth_accounts::Migration),
            Box::new(m20240101_000005_security_hardening::Migration),
            Box::new(m20240101_000006_oauth_clients::Migration),
            Box::new(m20240101_000007_authorization_codes::Migration),
            Box::new(m20240101_000008_sessions::Migration),
            Box::new(m20240101_000009_fix_rls_force::Migration),
            Box::new(m20240101_000010_revoked_jtis::Migration),
            Box::new(m20240101_000011_sessions_last_seen::Migration),
            Box::new(m20240101_000012_roles::Migration),
            Box::new(m20240101_000013_permissions::Migration),
            Box::new(m20240101_000014_sessions_rls::Migration),
            Box::new(m20240101_000015_rls_app_role::Migration),
            Box::new(m20240101_000016_user_lifecycle::Migration),
            Box::new(m20240101_000017_tenant_settings::Migration),
            Box::new(m20240101_000018_client_roles::Migration),
            Box::new(m20240101_000019_client_token_ttl::Migration),
            Box::new(m20240101_000020_identity_providers::Migration),
            Box::new(m20240101_000021_totp_secrets::Migration),
            Box::new(m20240101_000022_rename_role::Migration),
            Box::new(m20240101_000023_tenant_smtp_config::Migration),
            Box::new(m20240101_000024_webauthn_credentials::Migration),
            Box::new(m20240101_000025_rate_limit_buckets::Migration),
            Box::new(m20240101_000026_mfa_backup_codes::Migration),
            Box::new(m20240101_000027_password_history::Migration),
            Box::new(m20240101_000028_audit_and_contextual_rbac::Migration),
            Box::new(m20240101_000029_audit_actor_context::Migration),
            Box::new(m20240101_000030_tenant_hierarchy_domains::Migration),
            Box::new(m20240101_000031_resource_permission_scopes::Migration),
            Box::new(m20240101_000032_trusted_devices::Migration),
            Box::new(m20240101_000033_passwordless_email_otp::Migration),
            Box::new(m20240101_000034_one_time_token_rls::Migration),
            Box::new(m20240101_000035_plans_subscriptions_quotas::Migration),
            Box::new(m20240101_000036_typed_tenant_config::Migration),
            Box::new(m20240101_000037_request_activity::Migration),
            Box::new(m20240101_000038_authoritative_actor_audit::Migration),
        ]
    }
}
