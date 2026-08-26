use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use uuid::Uuid;

use crate::{
    entity::{
        permission_group_permissions, permission_groups, permission_scopes, permissions,
        rdd_permission_groups, role_designation_departments, user_role_designation_departments,
    },
    error::AppError,
    services::{contextual_rbac_service::ensure_urdp, permission_scope_service},
};

pub async fn list_groups(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
) -> Result<Vec<permission_groups::Model>, AppError> {
    Ok(permission_groups::Entity::find()
        .filter(permission_groups::Column::TenantId.eq(tenant_id))
        .filter(permission_groups::Column::Status.eq("active"))
        .filter(permission_groups::Column::DeletedAt.is_null())
        .all(txn)
        .await?)
}

pub async fn create_group(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    name: &str,
    description: &str,
) -> Result<permission_groups::Model, AppError> {
    if name.trim().is_empty() {
        return Err(AppError::InvalidInput("group name is required".into()));
    }
    Ok(permission_groups::ActiveModel {
        tenant_id: Set(tenant_id),
        name: Set(name.trim().into()),
        description: Set(description.trim().into()),
        ..Default::default()
    }
    .insert(txn)
    .await?)
}

pub async fn add_permission(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    group_id: Uuid,
    permission_id: Uuid,
) -> Result<(), AppError> {
    require_group(txn, tenant_id, group_id).await?;
    permissions::Entity::find_by_id(permission_id)
        .filter(permissions::Column::TenantId.eq(tenant_id))
        .filter(permissions::Column::Status.eq("active"))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let key = (group_id, permission_id);
    if let Some(row) = permission_group_permissions::Entity::find_by_id(key)
        .one(txn)
        .await?
    {
        let mut active = row.into_active_model();
        active.status = Set("active".into());
        active.deleted_at = Set(None);
        active.deleted_by = Set(None);
        active.update(txn).await?;
    } else {
        permission_group_permissions::ActiveModel {
            permission_group_id: Set(group_id),
            permission_id: Set(permission_id),
            tenant_id: Set(tenant_id),
            ..Default::default()
        }
        .insert(txn)
        .await?;
    }
    sync_group(txn, tenant_id, group_id).await
}

pub async fn link_rdd(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    rdd_id: Uuid,
    group_id: Uuid,
) -> Result<(), AppError> {
    require_group(txn, tenant_id, group_id).await?;
    role_designation_departments::Entity::find_by_id(rdd_id)
        .filter(role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(role_designation_departments::Column::Status.eq("active"))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let key = (rdd_id, group_id);
    if let Some(row) = rdd_permission_groups::Entity::find_by_id(key)
        .one(txn)
        .await?
    {
        let mut active = row.into_active_model();
        active.status = Set("active".into());
        active.deleted_at = Set(None);
        active.deleted_by = Set(None);
        active.update(txn).await?;
    } else {
        rdd_permission_groups::ActiveModel {
            role_designation_department_id: Set(rdd_id),
            permission_group_id: Set(group_id),
            tenant_id: Set(tenant_id),
            ..Default::default()
        }
        .insert(txn)
        .await?;
    }
    let urdds = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(user_role_designation_departments::Column::RoleDesignationDepartmentId.eq(rdd_id))
        .filter(user_role_designation_departments::Column::Status.eq("active"))
        .all(txn)
        .await?;
    for urdd in urdds {
        sync_urdd(txn, tenant_id, urdd.id, rdd_id).await?;
    }
    Ok(())
}

pub async fn sync_urdd(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    urdd_id: Uuid,
    rdd_id: Uuid,
) -> Result<(), AppError> {
    let group_ids = rdd_permission_groups::Entity::find()
        .filter(rdd_permission_groups::Column::TenantId.eq(tenant_id))
        .filter(rdd_permission_groups::Column::RoleDesignationDepartmentId.eq(rdd_id))
        .filter(rdd_permission_groups::Column::Status.eq("active"))
        .filter(rdd_permission_groups::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .map(|row| row.permission_group_id)
        .collect::<Vec<_>>();
    let permission_ids = permission_group_permissions::Entity::find()
        .filter(permission_group_permissions::Column::TenantId.eq(tenant_id))
        .filter(permission_group_permissions::Column::PermissionGroupId.is_in(group_ids))
        .filter(permission_group_permissions::Column::Status.eq("active"))
        .filter(permission_group_permissions::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .map(|row| row.permission_id)
        .collect::<Vec<_>>();
    for permission_id in permission_ids {
        ensure_urdp(txn, urdd_id, permission_id, tenant_id).await?;
    }
    Ok(())
}

pub async fn list_scopes(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
) -> Result<Vec<permission_scopes::Model>, AppError> {
    Ok(permission_scopes::Entity::find()
        .filter(permission_scopes::Column::TenantId.eq(tenant_id))
        .filter(permission_scopes::Column::Status.eq("active"))
        .filter(permission_scopes::Column::DeletedAt.is_null())
        .all(txn)
        .await?)
}

pub async fn assign_scope(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    urdd_id: Uuid,
    permission_id: Uuid,
    scope_id: Uuid,
) -> Result<(), AppError> {
    user_role_designation_departments::Entity::find_by_id(urdd_id)
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(user_role_designation_departments::Column::Status.eq("active"))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    ensure_urdp(txn, urdd_id, permission_id, tenant_id).await?;
    permission_scope_service::assign(txn, tenant_id, urdd_id, permission_id, scope_id).await
}

async fn sync_group(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    group_id: Uuid,
) -> Result<(), AppError> {
    let rdd_ids = rdd_permission_groups::Entity::find()
        .filter(rdd_permission_groups::Column::TenantId.eq(tenant_id))
        .filter(rdd_permission_groups::Column::PermissionGroupId.eq(group_id))
        .filter(rdd_permission_groups::Column::Status.eq("active"))
        .all(txn)
        .await?
        .into_iter()
        .map(|row| row.role_designation_department_id)
        .collect::<Vec<_>>();
    for rdd_id in rdd_ids {
        let urdds = user_role_designation_departments::Entity::find()
            .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
            .filter(
                user_role_designation_departments::Column::RoleDesignationDepartmentId.eq(rdd_id),
            )
            .filter(user_role_designation_departments::Column::Status.eq("active"))
            .all(txn)
            .await?;
        for urdd in urdds {
            sync_urdd(txn, tenant_id, urdd.id, rdd_id).await?;
        }
    }
    Ok(())
}

async fn require_group(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    group_id: Uuid,
) -> Result<(), AppError> {
    permission_groups::Entity::find_by_id(group_id)
        .filter(permission_groups::Column::TenantId.eq(tenant_id))
        .filter(permission_groups::Column::Status.eq("active"))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(())
}
