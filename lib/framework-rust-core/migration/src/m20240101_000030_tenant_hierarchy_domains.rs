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
ALTER TABLE tenants
    ADD COLUMN IF NOT EXISTS tenant_type TEXT NOT NULL DEFAULT 'organization',
    ADD COLUMN IF NOT EXISTS parent_tenant_id UUID REFERENCES tenants(id) ON DELETE RESTRICT,
    ADD COLUMN IF NOT EXISTS code TEXT,
    ADD COLUMN IF NOT EXISTS timezone TEXT NOT NULL DEFAULT 'UTC',
    ADD COLUMN IF NOT EXISTS locale TEXT NOT NULL DEFAULT 'en';

ALTER TABLE tenants DROP CONSTRAINT IF EXISTS tenants_type_check;
ALTER TABLE tenants ADD CONSTRAINT tenants_type_check
    CHECK (tenant_type IN ('organization', 'hotel', 'branch'));
ALTER TABLE tenants DROP CONSTRAINT IF EXISTS tenants_parent_not_self;
ALTER TABLE tenants ADD CONSTRAINT tenants_parent_not_self
    CHECK (parent_tenant_id IS NULL OR parent_tenant_id <> id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_tenants_code_unique
    ON tenants(code) WHERE code IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_tenants_parent ON tenants(parent_tenant_id);

CREATE TABLE tenant_domains (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    domain_name TEXT NOT NULL,
    domain_type TEXT NOT NULL DEFAULT 'custom'
        CHECK (domain_type IN ('subdomain', 'custom')),
    is_primary  BOOLEAN NOT NULL DEFAULT false,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verified_at TIMESTAMPTZ,
    ssl_enabled BOOLEAN NOT NULL DEFAULT false,
    status      TEXT NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by  UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by  UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    deleted_at  TIMESTAMPTZ,
    deleted_by  UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL
);
CREATE UNIQUE INDEX idx_tenant_domains_active_name
    ON tenant_domains(lower(domain_name)) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_tenant_domains_one_primary
    ON tenant_domains(tenant_id) WHERE is_primary AND deleted_at IS NULL;
CREATE INDEX idx_tenant_domains_tenant ON tenant_domains(tenant_id);

ALTER TABLE tenant_domains ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_domains FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON tenant_domains
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);
CREATE TRIGGER set_updated_at BEFORE INSERT OR UPDATE ON tenant_domains
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
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
DROP TABLE IF EXISTS tenant_domains;
DROP INDEX IF EXISTS idx_tenants_code_unique;
DROP INDEX IF EXISTS idx_tenants_parent;
ALTER TABLE tenants
    DROP COLUMN IF EXISTS locale,
    DROP COLUMN IF EXISTS timezone,
    DROP COLUMN IF EXISTS code,
    DROP COLUMN IF EXISTS parent_tenant_id,
    DROP COLUMN IF EXISTS tenant_type;
"#,
            )
            .await?;
        Ok(())
    }
}
