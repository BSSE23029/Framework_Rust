// ── Response DTOs ─────────────────────────────────────────────────────────────

use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum LoginResult {
    Token(String),
    MfaRequired { mfa_token: String },
    TenantChoice(Vec<(String, String)>), // (slug, name)
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub plan: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessContext {
    pub urdd_id: String,
    pub rdd_id: String,
    pub role_id: String,
    pub role_name: String,
    pub designation_id: String,
    pub designation_name: String,
    pub department_id: String,
    pub department_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub code: String,
    pub description: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Designation {
    pub id: String,
    pub name: String,
    pub code: String,
    pub description: String,
    pub senior_designation_id: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleDesignationDepartment {
    pub id: String,
    pub role_id: String,
    pub designation_id: String,
    pub department_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserRoleDesignationDepartment {
    pub id: String,
    pub user_id: String,
    pub role_designation_department_id: String,
    pub attributes: serde_json::Value,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PermissionGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PermissionScope {
    pub id: String,
    pub permission_id: String,
    pub name: String,
    pub scope_type: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub conditions: serde_json::Value,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantDomain {
    pub id: String,
    pub tenant_id: String,
    pub domain_name: String,
    pub domain_type: String,
    pub is_primary: bool,
    pub is_verified: bool,
    pub verified_at: Option<String>,
    pub ssl_enabled: bool,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantPlan {
    pub id: String,
    pub code: String,
    pub name: String,
    pub billing_interval: String,
    pub price_cents: i64,
    pub currency: String,
    pub trial_days: i32,
    pub features: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlanQuota {
    pub id: String,
    pub resource: String,
    pub limit: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantSubscription {
    pub id: String,
    pub tenant_id: String,
    pub plan_id: String,
    pub state: String,
    pub starts_at: String,
    pub trial_ends_at: Option<String>,
    pub ends_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigDefinition {
    pub id: String,
    pub config_key: String,
    pub display_name: String,
    pub category: String,
    pub scope_type: String,
    pub value_type: String,
    pub is_required: bool,
    pub is_multi_value: bool,
    pub default_value: Option<serde_json::Value>,
    pub allowed_values: Option<serde_json::Value>,
    pub constraints: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigValue {
    pub id: String,
    pub definition_id: String,
    pub scope_type: String,
    pub scope_id: Option<String>,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrustedDevice {
    pub id: String,
    pub name: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub trusted_at: String,
    pub last_used_at: Option<String>,
    pub expires_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrustedDeviceIssued {
    #[serde(flatten)]
    pub device: TrustedDevice,
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestActivity {
    pub id: String,
    pub request_id: String,
    pub actor_urdd_id: String,
    pub http_method: String,
    pub request_path: String,
    pub response_status: i32,
    pub response_time_ms: i64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub platform: Option<String>,
    pub platform_version: Option<String>,
    pub created_at: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub is_active: bool,
    pub email_verified: bool,
    #[serde(default)]
    pub mfa_enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub email: String,
    pub ip: Option<String>,
    pub created_at: String,
    pub last_seen_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthClient {
    pub id: String,
    pub client_id: String,
    #[serde(default)]
    pub client_secret: Option<String>,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub grant_types: Vec<String>,
    pub is_confidential: bool,
    pub is_active: bool,
    pub access_token_ttl_minutes: Option<i32>,
    pub refresh_token_ttl_days: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdentityProvider {
    pub id: String,
    pub provider: String,
    pub client_id: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PasswordPolicyResponse {
    pub min_length: i32,
    pub require_uppercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
    pub history_size: i32,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LockoutPolicyResponse {
    pub max_attempts: i32,
    pub window_minutes: i32,
    pub duration_minutes: i32,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TokenTtlResponse {
    pub access_token_ttl_minutes: i32,
    pub refresh_token_ttl_days: i32,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RegistrationPolicyResponse {
    pub allow_public_registration: bool,
    pub require_email_verified: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SmtpConfigResponse {
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password_set: bool,
    pub from_name: String,
    pub from_email: String,
    pub use_tls: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PasskeyInfo {
    pub credential_id: String,
    pub name: String,
    pub aaguid: Option<String>,
    pub sign_count: i32,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub user_id: Option<String>,
    pub action: String,
    pub ip: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
}
