CREATE TABLE permission_scopes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    scope_type TEXT NOT NULL CHECK (scope_type IN ('tenant', 'resource_type', 'resource')),
    resource_type TEXT,
    resource_id UUID,
    conditions JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    CONSTRAINT permission_scopes_shape CHECK (
        (scope_type = 'tenant' AND resource_type IS NULL AND resource_id IS NULL) OR
        (scope_type = 'resource_type' AND resource_type IS NOT NULL AND resource_id IS NULL) OR
        (scope_type = 'resource' AND resource_type IS NOT NULL AND resource_id IS NOT NULL)
    )
);
CREATE UNIQUE INDEX permission_scopes_active_identity ON permission_scopes (
    tenant_id, permission_id, scope_type, COALESCE(resource_type, ''),
    COALESCE(resource_id, '00000000-0000-0000-0000-000000000000'::uuid)
) WHERE deleted_at IS NULL;
CREATE INDEX permission_scopes_lookup ON permission_scopes (tenant_id, permission_id, scope_type);

CREATE TABLE user_role_designation_permission_scopes (
    user_role_designation_department_id UUID NOT NULL
        REFERENCES user_role_designation_departments(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    permission_scope_id UUID NOT NULL REFERENCES permission_scopes(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    PRIMARY KEY (user_role_designation_department_id, permission_id, permission_scope_id),
    FOREIGN KEY (user_role_designation_department_id, permission_id)
        REFERENCES user_role_designation_permissions
            (user_role_designation_department_id, permission_id) ON DELETE CASCADE
);
CREATE INDEX urdp_scopes_lookup ON user_role_designation_permission_scopes
    (tenant_id, user_role_designation_department_id, permission_id);

INSERT INTO permission_scopes (tenant_id, permission_id, name, scope_type)
SELECT DISTINCT tenant_id, permission_id, 'Tenant-wide', 'tenant'
FROM user_role_designation_permissions
WHERE status = 'active' AND deleted_at IS NULL;
INSERT INTO user_role_designation_permission_scopes (
    user_role_designation_department_id, permission_id, permission_scope_id, tenant_id
)
SELECT urdp.user_role_designation_department_id, urdp.permission_id, scope.id, urdp.tenant_id
FROM user_role_designation_permissions urdp
JOIN permission_scopes scope ON scope.tenant_id = urdp.tenant_id
    AND scope.permission_id = urdp.permission_id AND scope.scope_type = 'tenant'
WHERE urdp.status = 'active' AND urdp.deleted_at IS NULL;

ALTER TABLE permission_scopes ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission_scopes FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON permission_scopes
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);
ALTER TABLE user_role_designation_permission_scopes ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_role_designation_permission_scopes FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON user_role_designation_permission_scopes
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);
CREATE TRIGGER set_updated_at BEFORE INSERT OR UPDATE ON permission_scopes
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at BEFORE INSERT OR UPDATE ON user_role_designation_permission_scopes
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
