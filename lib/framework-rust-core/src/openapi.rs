use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
mod swagger;
pub use swagger::router as swagger_router;

use crate::handlers::{
    access_context::AccessContextResponse,
    admin_access::{
        organizations::{
            AssignUserContextRequest, CreateDepartmentRequest, CreateDesignationRequest,
            CreateRddRequest,
        },
        permissions::{CreatePermissionGroupRequest, CreatePermissionScopeRequest},
    },
    admin_identity_providers::{CreateIdpRequest, IdpResponse, UpdateIdpRequest},
    admin_permissions::{
        AssignPermissionRequest, CreatePermissionRequest, PermissionResponse,
        UpdatePermissionRequest,
    },
    admin_plans::{
        CreatePlanRequest, PlanResponse, QuotaResponse, SetQuotaRequest, SubscribeRequest,
        SubscriptionResponse,
    },
    admin_roles::{AssignRoleRequest, CreateRoleRequest, RoleResponse, UpdateRoleRequest},
    admin_sessions::SessionResponse,
    admin_smtp::{SmtpConfigResponse, UpsertSmtpRequest},
    admin_tenant_domains::{CreateDomainRequest, DomainResponse},
    admin_tenant_settings::{
        LockoutResponse, PolicyResponse, RegistrationResponse, TokenTtlResponse,
        UpsertLockoutRequest, UpsertPolicyRequest, UpsertRegistrationRequest,
        UpsertTokenTtlRequest,
    },
    admin_users::{
        CreateUserRequest, PasswordResetTokenResponse, UpdateUserRequest, UserResponse,
        VerificationCodeResponse,
    },
    admin_webauthn::PasskeyInfo,
    audit_log::AuditLogEntry,
    clients::{ClientResponse, CreateClientRequest, UpdateClientRequest},
    forgot_password::ForgotPasswordRequest,
    login::{LoginRequest, MfaRequiredResponse, TokenResponse},
    logout::LogoutRequest,
    mfa::{ChallengeRequest, ConfirmRequest, SetupResponse},
    oauth_as::{
        AuthorizeParams, IntrospectRequest, TokenRequest, TokenResponse as OauthTokenResponse,
    },
    oauth_revoke::RevokeRequest,
    passwordless::{PasswordlessRequest, PasswordlessRequestResponse, PasswordlessVerifyRequest},
    refresh::RefreshRequest,
    register::{RegisterRequest, RegisterResponse},
    request_activity::ActivityResponse,
    reset_password::ResetPasswordRequest,
    tenant_config::{
        ConfigDefinitionResponse, ConfigValueResponse, DefineConfigRequest, SetConfigValueRequest,
    },
    tenant_hierarchy::{SetTenantHierarchyRequest, TenantHierarchyResponse},
    tenants::{CreateTenantRequest, TenantResponse, TenantSlugEntry},
    trusted_devices::{TrustDeviceRequest, TrustDeviceResponse, TrustedDeviceResponse},
    verify_email::VerifyOtpRequest,
    webauthn::{
        AuthFinishPayload, AuthStartPayload, RegisterFinishPayload,
        TokenResponse as WebauthnTokenResponse,
    },
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
            components.add_security_scheme(
                "admin_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new(
                    "X-FRAMEWORK-RUST-Admin-Key",
                ))),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "FRAMEWORK RUST Auth Service",
        version = "0.1.0-alpha",
        description = "Developer-first auth service — Keycloak alternative. Tenant = realm. Full production auth on free tier.",
    ),
    paths(
        // auth
        crate::handlers::login::login,
        crate::handlers::register::register,
        crate::handlers::logout::logout,
        crate::handlers::refresh::refresh,
        crate::handlers::revoke::revoke,
        crate::handlers::forgot_password::forgot_password,
        crate::handlers::reset_password::reset_password,
        crate::handlers::verify_email::verify_email,
        crate::handlers::passwordless::request,
        crate::handlers::passwordless::verify,
        // mfa
        crate::handlers::mfa::mfa_setup,
        crate::handlers::mfa::mfa_confirm,
        crate::handlers::mfa::mfa_disable,
        crate::handlers::mfa::mfa_challenge,
        crate::handlers::mfa::admin_disable_mfa,
        // webauthn
        crate::handlers::webauthn::register_start,
        crate::handlers::webauthn::register_finish,
        crate::handlers::webauthn::authenticate_start,
        crate::handlers::webauthn::authenticate_finish,
        // user
        crate::handlers::me::me,
        crate::handlers::access_context::list,
        crate::handlers::request_activity::list,
        crate::handlers::trusted_devices::trust_device,
        crate::handlers::trusted_devices::list_devices,
        crate::handlers::trusted_devices::revoke_device,
        // settings
        crate::handlers::admin_tenant_settings::get_policy,
        crate::handlers::admin_tenant_settings::put_policy,
        crate::handlers::admin_tenant_settings::get_lockout,
        crate::handlers::admin_tenant_settings::put_lockout,
        crate::handlers::admin_tenant_settings::get_token_ttl,
        crate::handlers::admin_tenant_settings::put_token_ttl,
        crate::handlers::admin_tenant_settings::get_registration,
        crate::handlers::admin_tenant_settings::put_registration,
        crate::handlers::tenant_config::list_definitions,
        crate::handlers::tenant_config::define,
        crate::handlers::tenant_config::set_value,
        // tenants
        crate::handlers::tenants::list_tenants,
        crate::handlers::tenants::create_tenant,
        crate::handlers::tenants::list_tenant_slugs,
        crate::handlers::admin_tenant_domains::list_domains,
        crate::handlers::admin_tenant_domains::create_domain,
        crate::handlers::tenant_hierarchy::set_hierarchy,
        crate::handlers::tenant_hierarchy::list_children,
        crate::handlers::admin_access::organizations::list_departments,
        crate::handlers::admin_access::organizations::create_department,
        crate::handlers::admin_access::organizations::list_designations,
        crate::handlers::admin_access::organizations::create_designation,
        crate::handlers::admin_access::organizations::list_rdds,
        crate::handlers::admin_access::organizations::create_rdd,
        crate::handlers::admin_access::organizations::list_urdds,
        crate::handlers::admin_access::organizations::assign_user_context,
        crate::handlers::admin_access::organizations::revoke_user_context,
        crate::handlers::admin_access::permissions::list_groups,
        crate::handlers::admin_access::permissions::create_group,
        crate::handlers::admin_access::permissions::add_group_permission,
        crate::handlers::admin_access::permissions::link_rdd_group,
        crate::handlers::admin_access::permissions::list_scopes,
        crate::handlers::admin_access::permissions::create_scope,
        crate::handlers::admin_access::permissions::assign_scope,
        // clients
        crate::handlers::clients::list_clients,
        crate::handlers::clients::create_client,
        crate::handlers::clients::update_client,
        crate::handlers::clients::deactivate_client,
        // admin users
        crate::handlers::admin_users::list_users,
        crate::handlers::admin_users::create_user,
        crate::handlers::admin_users::update_user,
        crate::handlers::admin_users::deactivate_user,
        crate::handlers::admin_users::get_verification_code,
        crate::handlers::admin_users::get_password_reset_token,
        // admin roles
        crate::handlers::admin_roles::list_roles,
        crate::handlers::admin_roles::create_role,
        crate::handlers::admin_roles::update_role,
        crate::handlers::admin_roles::delete_role,
        crate::handlers::admin_roles::list_user_roles,
        crate::handlers::admin_roles::assign_user_role,
        crate::handlers::admin_roles::revoke_user_role,
        crate::handlers::admin_roles::list_client_roles,
        crate::handlers::admin_roles::assign_client_role,
        crate::handlers::admin_roles::revoke_client_role,
        // admin permissions
        crate::handlers::admin_permissions::list_permissions,
        crate::handlers::admin_permissions::create_permission,
        crate::handlers::admin_permissions::update_permission,
        crate::handlers::admin_permissions::delete_permission,
        crate::handlers::admin_permissions::list_role_permissions,
        crate::handlers::admin_permissions::assign_role_permission,
        crate::handlers::admin_permissions::revoke_role_permission,
        crate::handlers::admin_plans::list,
        crate::handlers::admin_plans::create,
        crate::handlers::admin_plans::set_quota,
        crate::handlers::admin_plans::subscribe,
        // admin sessions
        crate::handlers::admin_sessions::list_sessions,
        crate::handlers::admin_sessions::delete_session,
        // admin smtp
        crate::handlers::admin_smtp::get_smtp,
        crate::handlers::admin_smtp::put_smtp,
        // admin webauthn
        crate::handlers::admin_webauthn::list_passkeys,
        crate::handlers::admin_webauthn::delete_passkey,
        // admin identity providers
        crate::handlers::admin_identity_providers::list_idps,
        crate::handlers::admin_identity_providers::create_idp,
        crate::handlers::admin_identity_providers::update_idp,
        crate::handlers::admin_identity_providers::delete_idp,
        // audit log
        crate::handlers::audit_log::list_audit_log,
        // oauth
        crate::handlers::oauth_as::authorize,
        crate::handlers::oauth_as::token,
        crate::handlers::oauth_as::introspect,
        crate::handlers::oauth_revoke::revoke,
        // well-known
        crate::handlers::well_known::discovery,
        crate::handlers::well_known::jwks,
    ),
    components(
        schemas(
            LoginRequest,
            TokenResponse,
            MfaRequiredResponse,
            RegisterRequest,
            RegisterResponse,
            LogoutRequest,
            RefreshRequest,
            ForgotPasswordRequest,
            ResetPasswordRequest,
            VerifyOtpRequest,
            SetupResponse,
            ConfirmRequest,
            ChallengeRequest,
            RegisterFinishPayload,
            AuthStartPayload,
            AuthFinishPayload,
            WebauthnTokenResponse,
            PolicyResponse,
            UpsertPolicyRequest,
            LockoutResponse,
            UpsertLockoutRequest,
            TokenTtlResponse,
            UpsertTokenTtlRequest,
            RegistrationResponse,
            UpsertRegistrationRequest,
            TenantResponse,
            TenantSlugEntry,
            CreateTenantRequest,
            DomainResponse,
            CreateDomainRequest,
            SetTenantHierarchyRequest,
            TenantHierarchyResponse,
            CreateDepartmentRequest,
            CreateDesignationRequest,
            CreateRddRequest,
            AssignUserContextRequest,
            CreatePermissionGroupRequest,
            CreatePermissionScopeRequest,
            ClientResponse,
            CreateClientRequest,
            UpdateClientRequest,
            UserResponse,
            CreateUserRequest,
            UpdateUserRequest,
            VerificationCodeResponse,
            PasswordResetTokenResponse,
            RoleResponse,
            CreateRoleRequest,
            UpdateRoleRequest,
            AssignRoleRequest,
            PermissionResponse,
            CreatePermissionRequest,
            UpdatePermissionRequest,
            AssignPermissionRequest,
            CreatePlanRequest, PlanResponse, SetQuotaRequest, QuotaResponse,
            SubscribeRequest, SubscriptionResponse,
            SessionResponse,
            SmtpConfigResponse,
            UpsertSmtpRequest,
            PasskeyInfo,
            IdpResponse,
            CreateIdpRequest,
            UpdateIdpRequest,
            AuditLogEntry,
            AuthorizeParams,
            TokenRequest,
            OauthTokenResponse,
            IntrospectRequest,
            RevokeRequest,
            PasswordlessRequest,
            PasswordlessVerifyRequest,
            PasswordlessRequestResponse,
            TrustDeviceRequest,
            TrustDeviceResponse,
            TrustedDeviceResponse,
            DefineConfigRequest, ConfigDefinitionResponse, SetConfigValueRequest, ConfigValueResponse,
            ActivityResponse,
            AccessContextResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "User-facing authentication endpoints"),
        (name = "user", description = "Authenticated user profile endpoints"),
        (name = "settings", description = "Per-tenant settings (requires JWT)"),
        (name = "oauth", description = "OAuth 2.0 / OIDC authorization server"),
        (name = "tenants", description = "Tenant management (admin)"),
        (name = "clients", description = "OAuth client management (admin)"),
        (name = "admin-users", description = "User management (admin)"),
        (name = "admin-roles", description = "Role and permission management (admin)"),
        (name = "admin-access", description = "Tenant hierarchy and contextual access management (admin)"),
        (name = "plans", description = "Plans, quotas, and tenant subscriptions (admin)"),
        (name = "admin-sessions", description = "Session management (admin)"),
        (name = "admin-smtp", description = "SMTP configuration (admin)"),
        (name = "admin-webauthn", description = "Passkey / WebAuthn management (admin)"),
        (name = "admin-idp", description = "Identity provider management (admin)"),
        (name = "audit", description = "Audit log (admin)"),
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::{ApiDoc, OpenApi};

    #[test]
    fn contextual_access_paths_are_documented() {
        let document = ApiDoc::openapi();
        for path in [
            "/admin/tenants/{tenant_id}/hierarchy",
            "/admin/tenants/{tenant_id}/children",
            "/admin/tenants/{tenant_id}/access/departments",
            "/admin/tenants/{tenant_id}/access/designations",
            "/admin/tenants/{tenant_id}/access/rdds",
            "/admin/tenants/{tenant_id}/access/urdds",
            "/admin/tenants/{tenant_id}/access/permission-groups",
            "/admin/tenants/{tenant_id}/access/permission-scopes",
        ] {
            assert!(document.paths.paths.contains_key(path), "missing {path}");
        }
    }
}
