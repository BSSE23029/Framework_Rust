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
    ADD COLUMN IF NOT EXISTS attempt_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS max_attempts INTEGER NOT NULL DEFAULT 5;
ALTER TABLE one_time_tokens DROP CONSTRAINT IF EXISTS one_time_tokens_attempts_check;
ALTER TABLE one_time_tokens ADD CONSTRAINT one_time_tokens_attempts_check
    CHECK (attempt_count >= 0 AND max_attempts BETWEEN 1 AND 20 AND attempt_count <= max_attempts);
CREATE INDEX IF NOT EXISTS one_time_tokens_passwordless_active
    ON one_time_tokens(tenant_id, user_id, created_at DESC)
    WHERE token_type = 'passwordless_login' AND status = 'active' AND deleted_at IS NULL;
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
DROP INDEX IF EXISTS one_time_tokens_passwordless_active;
ALTER TABLE one_time_tokens DROP COLUMN IF EXISTS max_attempts;
ALTER TABLE one_time_tokens DROP COLUMN IF EXISTS attempt_count;
"#,
            )
            .await?;
        Ok(())
    }
}
