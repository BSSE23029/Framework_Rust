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
DECLARE
    actor_id UUID := NULLIF(current_setting('app.actor_urdd_id', true), '')::uuid;
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF actor_id IS NOT NULL THEN
            NEW.created_by = actor_id;
            NEW.updated_by = actor_id;
        END IF;
        RETURN NEW;
    END IF;

    NEW.updated_at = now();
    IF actor_id IS NOT NULL THEN
        NEW.updated_by = actor_id;
    END IF;

    IF NEW.status = 'inactive' AND OLD.status IS DISTINCT FROM NEW.status THEN
        NEW.deleted_at = COALESCE(NEW.deleted_at, now());
        IF actor_id IS NOT NULL THEN
            NEW.deleted_by = actor_id;
        END IF;
    ELSIF NEW.status = 'active' AND OLD.status IS DISTINCT FROM NEW.status THEN
        NEW.deleted_at = NULL;
        NEW.deleted_by = NULL;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DO $$
DECLARE
    table_record record;
BEGIN
    FOR table_record IN
        SELECT table_name
        FROM information_schema.columns
        WHERE table_schema = 'public'
        GROUP BY table_name
        HAVING bool_or(column_name = 'status')
           AND bool_or(column_name = 'updated_at')
           AND bool_or(column_name = 'updated_by')
    LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS set_updated_at ON public.%I', table_record.table_name);
        EXECUTE format(
            'CREATE TRIGGER set_updated_at BEFORE INSERT OR UPDATE ON public.%I
             FOR EACH ROW EXECUTE FUNCTION set_updated_at()',
            table_record.table_name
        );
    END LOOP;
END $$;
"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
