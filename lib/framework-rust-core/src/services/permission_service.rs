use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::{
    entity::{
        permissions, role_permissions, user_role_designation_departments,
        user_role_designation_permissions,
    },
    error::AppError,
    services::contextual_rbac_service,
};

pub struct CreatePermissionInput {
    pub tenant_id: Uuid,
    pub name: String,
    pub description: String,
}

pub async fn create(
    txn: &DatabaseTransaction,
    input: CreatePermissionInput,
) -> Result<permissions::Model, AppError> {
    Ok(permissions::ActiveModel {
        tenant_id: Set(input.tenant_id),
        name: Set(input.name),
        description: Set(input.description),
        ..Default::default()
    }
    .insert(txn)
    .await?)
}

pub async fn list_all(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
) -> Result<Vec<permissions::Model>, AppError> {
    Ok(permissions::Entity::find()
        .filter(permissions::Column::TenantId.eq(tenant_id))
        .all(txn)
        .await?)
}

pub async fn find_by_id(
    txn: &DatabaseTransaction,
    id: Uuid,
) -> Result<Option<permissions::Model>, AppError> {
    Ok(permissions::Entity::find_by_id(id).one(txn).await?)
}

pub async fn update(
    txn: &DatabaseTransaction,
    id: Uuid,
    name: String,
    description: String,
) -> Result<(), AppError> {
    let perm = find_by_id(txn, id).await?.ok_or(AppError::NotFound)?;
    let mut active: permissions::ActiveModel = perm.into();
    active.name = Set(name);
    active.description = Set(description);
    active.update(txn).await?;
    Ok(())
}

pub async fn delete(txn: &DatabaseTransaction, id: Uuid) -> Result<(), AppError> {
    permissions::Entity::delete_by_id(id).exec(txn).await?;
    Ok(())
}

// ── Role ↔ Permission ─────────────────────────────────────────────────────────

pub async fn list_for_role(
    txn: &DatabaseTransaction,
    role_id: Uuid,
    tenant_id: Uuid,
) -> Result<Vec<permissions::Model>, AppError> {
    let perm_ids: Vec<Uuid> = role_permissions::Entity::find()
        .filter(role_permissions::Column::RoleId.eq(role_id))
        .filter(role_permissions::Column::TenantId.eq(tenant_id))
        .all(txn)
        .await?
        .into_iter()
        .map(|rp| rp.permission_id)
        .collect();

    if perm_ids.is_empty() {
        return Ok(vec![]);
    }

    Ok(permissions::Entity::find()
        .filter(permissions::Column::Id.is_in(perm_ids))
        .filter(permissions::Column::TenantId.eq(tenant_id))
        .all(txn)
        .await?)
}

pub async fn assign_to_role(
    txn: &DatabaseTransaction,
    role_id: Uuid,
    permission_id: Uuid,
    tenant_id: Uuid,
) -> Result<(), AppError> {
    let now = Utc::now().fixed_offset();
    if let Some(row) = role_permissions::Entity::find_by_id((role_id, permission_id))
        .one(txn)
        .await?
    {
        let mut active: role_permissions::ActiveModel = row.into();
        active.status = Set("active".into());
        active.deleted_at = Set(None);
        active.deleted_by = Set(None);
        active.update(txn).await?;
    } else {
        role_permissions::ActiveModel {
            role_id: Set(role_id),
            permission_id: Set(permission_id),
            tenant_id: Set(tenant_id),
            assigned_at: Set(now),
            ..Default::default()
        }
        .insert(txn)
        .await?;
    }

    contextual_rbac_service::grant_role_permission(txn, tenant_id, role_id, permission_id).await?;

    Ok(())
}

pub async fn revoke_from_role(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    role_id: Uuid,
    permission_id: Uuid,
) -> Result<(), AppError> {
    if let Some(row) = role_permissions::Entity::find_by_id((role_id, permission_id))
        .one(txn)
        .await?
    {
        let mut active: role_permissions::ActiveModel = row.into();
        active.status = Set("inactive".into());
        active.update(txn).await?;
    }
    contextual_rbac_service::revoke_role_permission(txn, tenant_id, role_id, permission_id).await?;
    Ok(())
}

// ── User permission names (via contextual URDD assignments) ───────────────────

pub async fn list_names_for_user(
    txn: &DatabaseTransaction,
    user_id: Uuid,
    tenant_id: Uuid,
) -> Result<Vec<String>, AppError> {
    let urdd_ids: Vec<Uuid> = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::UserId.eq(user_id))
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(user_role_designation_departments::Column::Status.eq("active"))
        .filter(user_role_designation_departments::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .map(|urdd| urdd.id)
        .collect();

    if urdd_ids.is_empty() {
        return Ok(vec![]);
    }

    let perm_ids: Vec<Uuid> = user_role_designation_permissions::Entity::find()
        .filter(
            user_role_designation_permissions::Column::UserRoleDesignationDepartmentId
                .is_in(urdd_ids),
        )
        .filter(user_role_designation_permissions::Column::Status.eq("active"))
        .filter(user_role_designation_permissions::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .map(|urdp| urdp.permission_id)
        .collect();

    if perm_ids.is_empty() {
        return Ok(vec![]);
    }

    let perms = permissions::Entity::find()
        .filter(permissions::Column::Id.is_in(perm_ids))
        .filter(permissions::Column::TenantId.eq(tenant_id))
        .all(txn)
        .await?;

    let mut names: Vec<String> = perms.into_iter().map(|p| p.name).collect();
    names.sort();
    names.dedup();
    Ok(names)
}
