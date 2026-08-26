use crate::api::PasskeyInfo;

use super::QuickStartState;

#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    None,
    CreateTenant {
        name: String,
        slug: String,
        field: usize,
    },
    TenantHierarchy {
        tenant_id: String,
        tenant_type: String,
        parent_tenant_id: String,
        field: usize,
    },
    CreateClient {
        name: String,
        redirect_uri: String,
        scopes: String,
        client_type: u8,
        field: usize,
    },
    CreateUser {
        email: String,
        password: String,
        field: usize,
    },
    ConfirmDelete {
        id: String,
        label: String,
    },
    ShowSecret {
        client_id: String,
        secret: String,
    },
    Error(String),
    QuickStart(QuickStartState),
    EditClient {
        id: String,
        name: String,
        redirect_uris: String,
        scopes: String,
        access_token_ttl: String,
        refresh_token_ttl: String,
        client_type: u8,
        field: usize,
    },
    CreateIdp {
        provider: String,
        client_id: String,
        client_secret: String,
        redirect_url: String,
        scopes: String,
        field: usize,
    },
    EditIdp {
        id: String,
        provider: String,
        client_id: String,
        client_secret: String,
        redirect_url: String,
        scopes: String,
        enabled: bool,
        field: usize,
    },
    EditUser {
        id: String,
        email: String,
        password: String,
        is_active: bool,
        all_roles: Vec<(String, String, bool)>, // (id, name, assigned)
        permissions: Vec<String>,               // derived from assigned roles, read-only
        passkeys: Vec<PasskeyInfo>,             // 'd' to delete
        field: usize, // 0=email, 1=password, 2=is_active, 3=roles, 4=passkeys
        role_selected: usize,
        passkey_selected: usize,
    },
    CreateRole {
        name: String,
        description: String,
        field: usize,
    },
    EditRole {
        id: String,
        name: String,
        description: String,
        all_permissions: Vec<(String, String, bool)>, // (id, name, assigned)
        field: usize, // 0=name, 1=description, 2=permissions section
        perm_selected: usize,
    },
    CreatePermission {
        name: String,
        description: String,
        field: usize,
    },
    CreateDomain {
        domain_name: String,
        domain_type: String,
        is_primary: bool,
        field: usize,
    },
    CreatePlan {
        code: String,
        name: String,
        interval: String,
        price_cents: String,
        currency: String,
        trial_days: String,
        field: usize,
    },
    SetPlanQuota {
        resource: String,
        limit: String,
        field: usize,
    },
    DefineConfig {
        config_key: String,
        display_name: String,
        category: String,
        value_type: String,
        default_value: String,
        field: usize,
    },
    SetConfigValue {
        value: String,
    },
    TrustDevice {
        name: String,
        ttl_days: String,
        field: usize,
    },
    AccessForm {
        action: usize,
        fields: Vec<String>,
        field: usize,
    },
    EditPermission {
        id: String,
        name: String,
        description: String,
        field: usize,
    },
    ClientRoles {
        client_id: String,
        client_name: String,
        all_roles: Vec<(String, String, bool)>,
        selected: usize,
    },
    /// section: 0=PasswordPolicy, 1=Lockout, 2=Tokens, 3=Registration, 4=Smtp
    EditSettings(u8),
    PostCreateTenant {
        tenant_id: String,
        tenant_name: String,
    },
}
