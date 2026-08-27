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
ALTER TABLE one_time_tokens
    ADD CONSTRAINT one_time_tokens_tenant_fk FOREIGN KEY (tenant_id)
        REFERENCES tenants(id) ON DELETE CASCADE NOT VALID,
    ADD CONSTRAINT one_time_tokens_user_fk FOREIGN KEY (user_id)
        REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE one_time_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE one_time_tokens FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON one_time_tokens;
CREATE POLICY tenant_isolation ON one_time_tokens
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);
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
DROP POLICY IF EXISTS tenant_isolation ON one_time_tokens;
ALTER TABLE one_time_tokens DISABLE ROW LEVEL SECURITY;
ALTER TABLE one_time_tokens DROP CONSTRAINT IF EXISTS one_time_tokens_user_fk;
ALTER TABLE one_time_tokens DROP CONSTRAINT IF EXISTS one_time_tokens_tenant_fk;
"#,
            )
            .await?;
        Ok(())
    }
}
