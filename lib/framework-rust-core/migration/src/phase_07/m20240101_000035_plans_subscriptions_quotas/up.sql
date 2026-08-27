CREATE TABLE plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    billing_interval TEXT NOT NULL CHECK (billing_interval IN ('monthly','yearly','one_time','lifetime')),
    price_cents BIGINT NOT NULL DEFAULT 0 CHECK (price_cents >= 0),
    currency TEXT NOT NULL DEFAULT 'USD',
    trial_days INTEGER NOT NULL DEFAULT 14 CHECK (trial_days >= 0),
    features JSONB NOT NULL DEFAULT '{}'::jsonb,
    is_public BOOLEAN NOT NULL DEFAULT true,
    is_auto_renewable BOOLEAN NOT NULL DEFAULT false,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), created_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    deleted_at TIMESTAMPTZ, deleted_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL
);
CREATE TABLE plan_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    resource TEXT NOT NULL,
    limit_value BIGINT NOT NULL CHECK (limit_value >= -1),
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), created_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    deleted_at TIMESTAMPTZ, deleted_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL
);
CREATE UNIQUE INDEX plan_quotas_active_resource ON plan_quotas(plan_id, resource) WHERE deleted_at IS NULL;
CREATE TABLE plan_permission_groups (
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    permission_group_id UUID NOT NULL REFERENCES permission_groups(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), created_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    deleted_at TIMESTAMPTZ, deleted_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    PRIMARY KEY (plan_id, permission_group_id)
);
CREATE TABLE tenant_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE RESTRICT,
    subscription_state TEXT NOT NULL CHECK (subscription_state IN ('trial','active','suspended','cancelled','expired')),
    starts_at TIMESTAMPTZ NOT NULL DEFAULT now(), trial_ends_at TIMESTAMPTZ,
    ends_at TIMESTAMPTZ, auto_renew BOOLEAN NOT NULL DEFAULT false,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), created_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    deleted_at TIMESTAMPTZ, deleted_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL
);
CREATE UNIQUE INDEX tenant_subscriptions_current ON tenant_subscriptions(tenant_id) WHERE status = 'active' AND deleted_at IS NULL;
CREATE TABLE tenant_usage (
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    resource TEXT NOT NULL,
    used_quantity BIGINT NOT NULL DEFAULT 0 CHECK (used_quantity >= 0),
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), created_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    deleted_at TIMESTAMPTZ, deleted_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    PRIMARY KEY (tenant_id, resource)
);
CREATE TABLE tenant_lifecycle_notifications (
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    stage TEXT NOT NULL, notification_date DATE NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), created_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    deleted_at TIMESTAMPTZ, deleted_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    PRIMARY KEY (tenant_id, stage, notification_date)
);
ALTER TABLE tenants ADD COLUMN current_subscription_plan_id UUID REFERENCES plans(id) ON DELETE SET NULL,
    ADD COLUMN subscription_status TEXT NOT NULL DEFAULT 'trial', ADD COLUMN trial_ends_at TIMESTAMPTZ,
    ADD COLUMN subscription_starts_at TIMESTAMPTZ, ADD COLUMN subscription_ends_at TIMESTAMPTZ,
    ADD COLUMN max_users BIGINT NOT NULL DEFAULT 10, ADD COLUMN storage_quota_mb BIGINT NOT NULL DEFAULT 1024,
    ADD COLUMN storage_used_mb BIGINT NOT NULL DEFAULT 0;
ALTER TABLE tenants ADD CONSTRAINT tenants_subscription_status_check
    CHECK (subscription_status IN ('trial','active','suspended','cancelled','expired'));

DO $$ DECLARE table_name TEXT; BEGIN
FOREACH table_name IN ARRAY ARRAY['plan_permission_groups','tenant_subscriptions','tenant_usage','tenant_lifecycle_notifications'] LOOP
    EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', table_name);
    EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', table_name);
    EXECUTE format('CREATE POLICY tenant_isolation ON %I USING (tenant_id = current_setting(''app.tenant_id'', true)::uuid) WITH CHECK (tenant_id = current_setting(''app.tenant_id'', true)::uuid)', table_name);
END LOOP; END $$;
CREATE TRIGGER set_updated_at BEFORE INSERT OR UPDATE ON plans FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at BEFORE INSERT OR UPDATE ON plan_quotas FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at BEFORE INSERT OR UPDATE ON plan_permission_groups FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at BEFORE INSERT OR UPDATE ON tenant_subscriptions FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at BEFORE INSERT OR UPDATE ON tenant_usage FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at BEFORE INSERT OR UPDATE ON tenant_lifecycle_notifications FOR EACH ROW EXECUTE FUNCTION set_updated_at();
