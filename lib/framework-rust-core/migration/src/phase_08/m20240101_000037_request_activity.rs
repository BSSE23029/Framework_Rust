use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(r#"
CREATE TABLE request_activity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    actor_urdd_id UUID NOT NULL REFERENCES user_role_designation_departments(id) ON DELETE RESTRICT,
    request_id UUID NOT NULL, http_method TEXT NOT NULL, request_path TEXT NOT NULL,
    response_status INTEGER NOT NULL, response_time_ms BIGINT NOT NULL CHECK (response_time_ms >= 0),
    ip_address TEXT, user_agent TEXT, platform TEXT, platform_version TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), created_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    deleted_at TIMESTAMPTZ, deleted_by UUID REFERENCES user_role_designation_departments(id) ON DELETE SET NULL,
    UNIQUE (tenant_id, request_id)
);
CREATE INDEX request_activity_actor_time ON request_activity(tenant_id, actor_urdd_id, created_at DESC);
CREATE INDEX request_activity_user_time ON request_activity(tenant_id, user_id, created_at DESC);
ALTER TABLE request_activity ENABLE ROW LEVEL SECURITY;
ALTER TABLE request_activity FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON request_activity
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);
CREATE TRIGGER set_updated_at BEFORE INSERT OR UPDATE ON request_activity FOR EACH ROW EXECUTE FUNCTION set_updated_at();
"#).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS request_activity;")
            .await?;
        Ok(())
    }
}
