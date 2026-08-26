use std::collections::HashMap;

use crate::api::{
    AccessContext, AuditLogEntry, Client, ConfigDefinition, IdentityProvider, OAuthClient,
    Permission, PlanQuota, RequestActivity, Role, Session, Tenant, TenantDomain, TenantPlan,
    TrustedDevice, User,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Login {
        email: String,
        password: String,
        field: usize, // 0=email, 1=password
        error: Option<String>,
        /// When the server returns multiple tenant matches, store them here
        /// so the user can pick one before we do the final login.
        tenant_choices: Option<Vec<(String, String)>>, // (slug, name)
        tenant_choice_idx: usize,
    },
    MfaChallenge {
        mfa_token: String,
        slug: String,
        code: String,
        error: Option<String>,
    },
    Passwordless {
        email: String,
        slug: String,
        otp: String,
        requested: bool,
        field: usize,
        error: Option<String>,
    },
    Admin,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Clients,
    Users,
    Roles,
    Permissions,
    Sessions,
    Settings,
    IdentityProviders,
    Domains,
    Plans,
    Config,
    Security,
    AuditLog,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsState {
    // Section 0 — Password Policy
    pub policy_min_length: String,
    pub policy_require_uppercase: bool,
    pub policy_require_digit: bool,
    pub policy_require_special: bool,
    // Section 1 — Lockout
    pub lockout_max_attempts: String,
    pub lockout_window_minutes: String,
    pub lockout_duration_minutes: String,
    // Section 2 — Token TTL
    pub access_token_ttl_minutes: String,
    pub refresh_token_ttl_days: String,
    // Section 3 — Registration
    pub allow_public_registration: bool,
    pub require_email_verified: bool,
    // Section 4 — SMTP
    pub smtp_host: String,
    pub smtp_port: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from_name: String,
    pub smtp_from_email: String,
    pub smtp_use_tls: bool,
    pub smtp_enabled: bool,
    pub smtp_password_set: bool,
    // UI
    pub section_selected: usize, // 0-4, which section is highlighted in section list
    pub modal_field: usize,      // active field index inside EditSettings modal
    pub loading: bool,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            policy_min_length: String::from("8"),
            policy_require_uppercase: false,
            policy_require_digit: false,
            policy_require_special: false,
            lockout_max_attempts: String::from("5"),
            lockout_window_minutes: String::from("15"),
            lockout_duration_minutes: String::from("15"),
            access_token_ttl_minutes: String::from("15"),
            refresh_token_ttl_days: String::from("30"),
            allow_public_registration: true,
            require_email_verified: false,
            smtp_host: String::new(),
            smtp_port: String::from("587"),
            smtp_username: String::new(),
            smtp_password: String::new(),
            smtp_from_name: String::new(),
            smtp_from_email: String::new(),
            smtp_use_tls: true,
            smtp_enabled: false,
            smtp_password_set: false,
            section_selected: 0,
            modal_field: 0,
            loading: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Sidebar,
    Content,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuickStartState {
    pub step: u8,
    // Step 1 — tenant
    pub tenant_name: String,
    pub tenant_slug: String,
    // Step 2 — client
    pub client_name: String,
    pub redirect_uri: String,
    pub scopes: String,
    pub client_type: u8, // 0=Confidential, 1=SPA/Mobile, 2=Machine
    // Step 3 — user
    pub user_email: String,
    pub user_password: String,
    // Results stored after each API call
    pub created_tenant_id: Option<String>,
    pub created_tenant_name: Option<String>,
    pub created_client_id: Option<String>,
    pub created_secret: Option<String>,
    pub show_secret: bool,
    // Active input field index within the current step
    pub field: usize,
    pub error: Option<String>,
}

impl Default for QuickStartState {
    fn default() -> Self {
        Self {
            step: 1,
            tenant_name: String::new(),
            tenant_slug: String::new(),
            client_name: String::new(),
            redirect_uri: String::from("http://localhost:8080/callback"),
            scopes: String::from("openid email profile"),
            client_type: 0,
            user_email: String::new(),
            user_password: String::new(),
            created_tenant_id: None,
            created_tenant_name: None,
            created_client_id: None,
            created_secret: None,
            show_secret: false,
            field: 0,
            error: None,
        }
    }
}

mod modal;
pub use modal::Modal;

pub struct App {
    pub client: Client,
    pub mode: AppMode,
    pub focus: Focus,
    pub tab: Tab,
    pub modal: Modal,

    pub tenants: Vec<Tenant>,
    pub tenant_selected: usize,
    pub tenants_loading: bool,

    pub clients: Vec<OAuthClient>,
    pub client_selected: usize,
    pub clients_loading: bool,

    pub users: Vec<User>,
    pub user_selected: usize,
    pub users_loading: bool,

    pub sessions: Vec<Session>,
    pub session_selected: usize,
    pub sessions_loading: bool,

    pub roles: Vec<Role>,
    pub role_selected: usize,
    pub roles_loading: bool,

    pub permissions: Vec<Permission>,
    pub permission_selected: usize,
    pub permissions_loading: bool,

    pub identity_providers: Vec<IdentityProvider>,
    pub idp_selected: usize,
    pub idps_loading: bool,

    pub audit_log: Vec<AuditLogEntry>,
    pub audit_log_selected: usize,
    pub audit_log_loading: bool,

    pub domains: Vec<TenantDomain>,
    pub domain_selected: usize,
    pub domains_loading: bool,

    pub plans: Vec<TenantPlan>,
    pub plan_selected: usize,
    pub plans_loading: bool,
    pub plan_quotas: Vec<PlanQuota>,

    pub config_definitions: Vec<ConfigDefinition>,
    pub config_selected: usize,
    pub config_loading: bool,
    pub config_values: HashMap<String, String>,

    pub trusted_devices: Vec<TrustedDevice>,
    pub trusted_device_selected: usize,
    pub request_activity: Vec<RequestActivity>,

    pub active_tenant_id: Option<String>,
    pub access_contexts: Vec<AccessContext>,
    pub access_context_selected: usize,

    pub settings: SettingsState,

    pub health_status: Option<String>,
    pub health_error: Option<String>,

    pub status_msg: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            mode: AppMode::Login {
                email: String::new(),
                password: String::new(),
                field: 0,
                error: None,
                tenant_choices: None,
                tenant_choice_idx: 0,
            },
            focus: Focus::Sidebar,
            tab: Tab::Clients,
            modal: Modal::None,

            tenants: vec![],
            tenant_selected: 0,
            tenants_loading: false,

            clients: vec![],
            client_selected: 0,
            clients_loading: false,

            users: vec![],
            user_selected: 0,
            users_loading: false,

            sessions: vec![],
            session_selected: 0,
            sessions_loading: false,

            roles: vec![],
            role_selected: 0,
            roles_loading: false,

            permissions: vec![],
            permission_selected: 0,
            permissions_loading: false,

            identity_providers: vec![],
            idp_selected: 0,
            idps_loading: false,

            audit_log: vec![],
            audit_log_selected: 0,
            audit_log_loading: false,

            domains: vec![],
            domain_selected: 0,
            domains_loading: false,

            plans: vec![],
            plan_selected: 0,
            plans_loading: false,
            plan_quotas: vec![],

            config_definitions: vec![],
            config_selected: 0,
            config_loading: false,
            config_values: HashMap::new(),

            trusted_devices: vec![],
            trusted_device_selected: 0,
            request_activity: vec![],

            active_tenant_id: None,
            access_contexts: vec![],
            access_context_selected: 0,

            settings: SettingsState::default(),

            health_status: None,
            health_error: None,

            status_msg: None,
            should_quit: false,
        }
    }

    pub fn selected_tenant(&self) -> Option<&Tenant> {
        self.tenants.get(self.tenant_selected)
    }

    pub fn selected_client(&self) -> Option<&OAuthClient> {
        self.clients.get(self.client_selected)
    }

    pub fn selected_user(&self) -> Option<&User> {
        self.users.get(self.user_selected)
    }

    pub fn selected_session(&self) -> Option<&Session> {
        self.sessions.get(self.session_selected)
    }

    pub fn selected_role(&self) -> Option<&Role> {
        self.roles.get(self.role_selected)
    }

    pub fn selected_permission(&self) -> Option<&Permission> {
        self.permissions.get(self.permission_selected)
    }

    pub fn selected_idp(&self) -> Option<&IdentityProvider> {
        self.identity_providers.get(self.idp_selected)
    }

    pub fn selected_domain(&self) -> Option<&TenantDomain> {
        self.domains.get(self.domain_selected)
    }

    pub fn selected_plan(&self) -> Option<&TenantPlan> {
        self.plans.get(self.plan_selected)
    }

    pub fn selected_config(&self) -> Option<&ConfigDefinition> {
        self.config_definitions.get(self.config_selected)
    }

    pub fn selected_trusted_device(&self) -> Option<&TrustedDevice> {
        self.trusted_devices.get(self.trusted_device_selected)
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some(msg.into());
    }

    pub fn clear_status(&mut self) {
        self.status_msg = None;
    }

    pub fn active_tenant_name(&self) -> Option<&str> {
        self.tenants
            .get(self.tenant_selected)
            .map(|t| t.name.as_str())
    }

    pub fn active_access_context(&self) -> Option<&AccessContext> {
        self.access_contexts.get(self.access_context_selected)
    }
}
