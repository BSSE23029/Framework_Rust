use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS trigger AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Retrofit every application table with the same lifecycle/audit contract.
-- Actor foreign keys are installed after URDD exists below.
DO $$
DECLARE
    table_record record;
BEGIN
    FOR table_record IN
        SELECT tablename
        FROM pg_tables
        WHERE schemaname = 'public'
          AND tablename <> 'seaql_migrations'
    LOOP
        EXECUTE format(
            'ALTER TABLE public.%I
                ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT ''active'',
                ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                ADD COLUMN IF NOT EXISTS created_by UUID,
                ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                ADD COLUMN IF NOT EXISTS updated_by UUID,
                ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ,
                ADD COLUMN IF NOT EXISTS deleted_by UUID',
            table_record.tablename
        );

        EXECUTE format('DROP TRIGGER IF EXISTS set_updated_at ON public.%I', table_record.tablename);
        EXECUTE format(
            'CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.%I
             FOR EACH ROW EXECUTE FUNCTION set_updated_at()',
            table_record.tablename
        );
    END LOOP;
END $$;

CREATE TABLE departments (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    code        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status      TEXT NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by  UUID,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by  UUID,
    deleted_at  TIMESTAMPTZ,
    deleted_by  UUID,
    UNIQUE (tenant_id, code)
);

CREATE TABLE designations (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name                  TEXT NOT NULL,
    code                  TEXT NOT NULL,
    description           TEXT NOT NULL DEFAULT '',
    senior_designation_id UUID REFERENCES designations(id) ON DELETE SET NULL,
    status                TEXT NOT NULL DEFAULT 'active',
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by            UUID,
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by            UUID,
    deleted_at            TIMESTAMPTZ,
    deleted_by            UUID,
    UNIQUE (tenant_id, code)
);

ALTER TABLE roles ADD COLUMN IF NOT EXISTS senior_role_id UUID REFERENCES roles(id) ON DELETE SET NULL;

CREATE TABLE role_designation_departments (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id      UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    role_id        UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    designation_id UUID NOT NULL REFERENCES designations(id) ON DELETE RESTRICT,
    department_id  UUID NOT NULL REFERENCES departments(id) ON DELETE RESTRICT,
    status         TEXT NOT NULL DEFAULT 'active',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by     UUID,
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by     UUID,
    deleted_at     TIMESTAMPTZ,
    deleted_by     UUID,
    UNIQUE (tenant_id, role_id, designation_id, department_id)
);

CREATE TABLE user_role_designation_departments (
    id                             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id                      UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id                        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_designation_department_id UUID NOT NULL REFERENCES role_designation_departments(id) ON DELETE RESTRICT,
    starts_at                      TIMESTAMPTZ NOT NULL DEFAULT now(),
    ends_at                        TIMESTAMPTZ,
    attributes                     JSONB NOT NULL DEFAULT '{}'::jsonb,
    status                         TEXT NOT NULL DEFAULT 'active',
    created_at                     TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by                     UUID,
    updated_at                     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by                     UUID,
    deleted_at                     TIMESTAMPTZ,
    deleted_by                     UUID,
    UNIQUE (tenant_id, user_id, role_designation_department_id)
);

CREATE TABLE permission_groups (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status      TEXT NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by  UUID,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by  UUID,
    deleted_at  TIMESTAMPTZ,
    deleted_by  UUID,
    UNIQUE (tenant_id, name)
);

CREATE TABLE permission_group_permissions (
    permission_group_id UUID NOT NULL REFERENCES permission_groups(id) ON DELETE CASCADE,
    permission_id       UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    tenant_id           UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    status              TEXT NOT NULL DEFAULT 'active',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by          UUID,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by          UUID,
    deleted_at          TIMESTAMPTZ,
    deleted_by          UUID,
    PRIMARY KEY (permission_group_id, permission_id)
);

CREATE TABLE rdd_permission_groups (
    role_designation_department_id UUID NOT NULL REFERENCES role_designation_departments(id) ON DELETE CASCADE,
    permission_group_id            UUID NOT NULL REFERENCES permission_groups(id) ON DELETE CASCADE,
    tenant_id                      UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    status                         TEXT NOT NULL DEFAULT 'active',
    created_at                     TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by                     UUID,
    updated_at                     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by                     UUID,
    deleted_at                     TIMESTAMPTZ,
    deleted_by                     UUID,
    PRIMARY KEY (role_designation_department_id, permission_group_id)
);

-- URDP is a synchronized effective-permission snapshot, not the assignment source.
CREATE TABLE user_role_designation_permissions (
    user_role_designation_department_id UUID NOT NULL REFERENCES user_role_designation_departments(id) ON DELETE CASCADE,
    permission_id                       UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    tenant_id                           UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    source                              TEXT NOT NULL DEFAULT 'group',
    status                              TEXT NOT NULL DEFAULT 'active',
    created_at                          TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by                          UUID,
    updated_at                          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by                          UUID,
    deleted_at                          TIMESTAMPTZ,
    deleted_by                          UUID,
    PRIMARY KEY (user_role_designation_department_id, permission_id)
);

-- Preserve current role assignments by creating a neutral org scaffold per tenant.
INSERT INTO departments (tenant_id, name, code, description)
SELECT id, 'General', 'GENERAL', 'Default department for migrated role assignments'
FROM tenants
ON CONFLICT (tenant_id, code) DO NOTHING;

INSERT INTO designations (tenant_id, name, code, description)
SELECT id, 'Member', 'MEMBER', 'Default designation for migrated role assignments'
FROM tenants
ON CONFLICT (tenant_id, code) DO NOTHING;

INSERT INTO role_designation_departments (tenant_id, role_id, designation_id, department_id)
SELECT r.tenant_id, r.id, d.id, dep.id
FROM roles r
JOIN designations d ON d.tenant_id = r.tenant_id AND d.code = 'MEMBER'
JOIN departments dep ON dep.tenant_id = r.tenant_id AND dep.code = 'GENERAL'
ON CONFLICT (tenant_id, role_id, designation_id, department_id) DO NOTHING;

INSERT INTO user_role_designation_departments
    (tenant_id, user_id, role_designation_department_id, starts_at)
SELECT ur.tenant_id, ur.user_id, rdd.id, ur.assigned_at
FROM user_roles ur
JOIN role_designation_departments rdd
  ON rdd.tenant_id = ur.tenant_id AND rdd.role_id = ur.role_id
ON CONFLICT (tenant_id, user_id, role_designation_department_id) DO NOTHING;

-- Create one permission group per existing role and preserve role permissions.
INSERT INTO permission_groups (tenant_id, name, description)
SELECT tenant_id, name, 'Migrated role permission group'
FROM roles
ON CONFLICT (tenant_id, name) DO NOTHING;

INSERT INTO permission_group_permissions (permission_group_id, permission_id, tenant_id)
SELECT pg.id, rp.permission_id, rp.tenant_id
FROM role_permissions rp
JOIN roles r ON r.id = rp.role_id
JOIN permission_groups pg ON pg.tenant_id = r.tenant_id AND pg.name = r.name
ON CONFLICT (permission_group_id, permission_id) DO NOTHING;

INSERT INTO rdd_permission_groups (role_designation_department_id, permission_group_id, tenant_id)
SELECT rdd.id, pg.id, rdd.tenant_id
FROM role_designation_departments rdd
JOIN roles r ON r.id = rdd.role_id
JOIN permission_groups pg ON pg.tenant_id = r.tenant_id AND pg.name = r.name
ON CONFLICT (role_designation_department_id, permission_group_id) DO NOTHING;

INSERT INTO user_role_designation_permissions
    (user_role_designation_department_id, permission_id, tenant_id)
SELECT urdd.id, pgp.permission_id, urdd.tenant_id
FROM user_role_designation_departments urdd
JOIN rdd_permission_groups rpg
  ON rpg.role_designation_department_id = urdd.role_designation_department_id
JOIN permission_group_permissions pgp
  ON pgp.permission_group_id = rpg.permission_group_id
ON CONFLICT (user_role_designation_department_id, permission_id) DO NOTHING;

-- Apply tenant RLS and update timestamps to every newly introduced tenant table.
DO $$
DECLARE
    table_name TEXT;
BEGIN
    FOREACH table_name IN ARRAY ARRAY[
        'departments', 'designations', 'role_designation_departments',
        'user_role_designation_departments', 'permission_groups',
        'permission_group_permissions', 'rdd_permission_groups',
        'user_role_designation_permissions'
    ]
    LOOP
        EXECUTE format('ALTER TABLE public.%I ENABLE ROW LEVEL SECURITY', table_name);
        EXECUTE format('ALTER TABLE public.%I FORCE ROW LEVEL SECURITY', table_name);
        EXECUTE format(
            'CREATE POLICY tenant_isolation ON public.%I
             USING (tenant_id = current_setting(''app.tenant_id'', true)::uuid)
             WITH CHECK (tenant_id = current_setting(''app.tenant_id'', true)::uuid)',
            table_name
        );
        EXECUTE format(
            'CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.%I
             FOR EACH ROW EXECUTE FUNCTION set_updated_at()',
            table_name
        );
    END LOOP;
END $$;

CREATE INDEX idx_departments_tenant ON departments(tenant_id);
CREATE INDEX idx_designations_tenant ON designations(tenant_id);
CREATE INDEX idx_rdd_tenant ON role_designation_departments(tenant_id);
CREATE INDEX idx_urdd_tenant_user ON user_role_designation_departments(tenant_id, user_id);
CREATE INDEX idx_urdp_tenant_urdd ON user_role_designation_permissions(tenant_id, user_role_designation_department_id);

-- Actor references are nullable for system/bootstrap work and point to contextual identities.
DO $$
DECLARE
    table_record record;
    actor_column TEXT;
    constraint_name TEXT;
BEGIN
    FOR table_record IN
        SELECT tablename
        FROM pg_tables
        WHERE schemaname = 'public'
          AND tablename <> 'seaql_migrations'
    LOOP
        FOREACH actor_column IN ARRAY ARRAY['created_by', 'updated_by', 'deleted_by']
        LOOP
            constraint_name := left('fk_' || table_record.tablename || '_' || actor_column || '_urdd', 63);
            IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = constraint_name) THEN
                EXECUTE format(
                    'ALTER TABLE public.%I ADD CONSTRAINT %I FOREIGN KEY (%I)
                     REFERENCES user_role_designation_departments(id) ON DELETE SET NULL',
                    table_record.tablename, constraint_name, actor_column
                );
            END IF;
        END LOOP;
    END LOOP;
END $$;
"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
DROP TABLE IF EXISTS user_role_designation_permissions;
DROP TABLE IF EXISTS rdd_permission_groups;
DROP TABLE IF EXISTS permission_group_permissions;
DROP TABLE IF EXISTS permission_groups;
DROP TABLE IF EXISTS user_role_designation_departments;
DROP TABLE IF EXISTS role_designation_departments;
DROP TABLE IF EXISTS designations;
DROP TABLE IF EXISTS departments;
ALTER TABLE roles DROP COLUMN IF EXISTS senior_role_id;
"#,
            )
            .await?;
        Ok(())
    }
}
