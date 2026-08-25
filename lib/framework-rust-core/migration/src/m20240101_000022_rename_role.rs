use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // The role already has its final name. Keeping this migration as a
        // no-op preserves the migration sequence without attempting to rename
        // a PostgreSQL role to itself.
        // manager
        //     .get_connection()
        //     .execute_unprepared(
        //         "DO $$
        //         BEGIN
        //             IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'framework_rust_rls') THEN
        //                 ALTER ROLE framework_rust_rls RENAME TO framework_rust_rls;
        //             END IF;
        //         END
        //         $$;",
        //     )
        //     .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // manager
        //     .get_connection()
        //     .execute_unprepared(
        //         "DO $$
        //         BEGIN
        //             IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'framework_rust_rls') THEN
        //                 ALTER ROLE framework_rust_rls RENAME TO framework_rust_rls;
        //             END IF;
        //         END
        //         $$;",
        //     )
        //     .await?;
        Ok(())
    }
}
