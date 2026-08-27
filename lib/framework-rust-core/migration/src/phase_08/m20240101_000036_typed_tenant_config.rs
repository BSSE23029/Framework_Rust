use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(r#"
CREATE TABLE tenant_config_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    config_key TEXT NOT NULL, display_name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'general', scope_type TEXT NOT NULL DEFAULT 'tenant',
    value_type TEXT NOT NULL CHECK (value_type IN ('boolean','integer','decimal','string','datetime','uuid','json')),
    is_required BOOLEAN NOT NULL DEFAULT false, is_multi_value BOOLEAN NOT NULL DEFAULT false,
    default_value JSONB, allowed_values JSONB, constraints JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), created_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    deleted_at TIMESTAMPTZ, deleted_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL
);
CREATE UNIQUE INDEX tenant_config_definitions_active_key
    ON tenant_config_definitions(tenant_id, scope_type, config_key) WHERE deleted_at IS NULL;
CREATE TABLE tenant_config_values (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    definition_id UUID NOT NULL REFERENCES tenant_config_definitions(id) ON DELETE CASCADE,
    scope_type TEXT NOT NULL, scope_id UUID, config_value JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), created_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    deleted_at TIMESTAMPTZ, deleted_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL
);
CREATE UNIQUE INDEX tenant_config_values_active_scope ON tenant_config_values(
    tenant_id, definition_id, scope_type, COALESCE(scope_id, '00000000-0000-0000-0000-000000000000'::uuid)
) WHERE deleted_at IS NULL;
ALTER TABLE tenant_config_definitions ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_config_definitions FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON tenant_config_definitions
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);
ALTER TABLE tenant_config_values ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_config_values FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON tenant_config_values
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);
CREATE TRIGGER set_updated_at BEFORE INSERT OR UPDATE ON tenant_config_definitions FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at BEFORE INSERT OR UPDATE ON tenant_config_values FOR EACH ROW EXECUTE FUNCTION set_updated_at();
"#).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "DROP TABLE IF EXISTS tenant_config_values; DROP TABLE IF EXISTS tenant_config_definitions;"
        ).await?;
        Ok(())
    }
}
