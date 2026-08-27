ALTER TABLE tenants DROP CONSTRAINT IF EXISTS tenants_subscription_status_check;
ALTER TABLE tenants DROP COLUMN IF EXISTS storage_used_mb, DROP COLUMN IF EXISTS storage_quota_mb,
    DROP COLUMN IF EXISTS max_users, DROP COLUMN IF EXISTS subscription_ends_at,
    DROP COLUMN IF EXISTS subscription_starts_at, DROP COLUMN IF EXISTS trial_ends_at,
    DROP COLUMN IF EXISTS subscription_status, DROP COLUMN IF EXISTS current_subscription_plan_id;
DROP TABLE IF EXISTS tenant_lifecycle_notifications;
DROP TABLE IF EXISTS tenant_usage;
DROP TABLE IF EXISTS tenant_subscriptions;
DROP TABLE IF EXISTS plan_permission_groups;
DROP TABLE IF EXISTS plan_quotas;
DROP TABLE IF EXISTS plans;
