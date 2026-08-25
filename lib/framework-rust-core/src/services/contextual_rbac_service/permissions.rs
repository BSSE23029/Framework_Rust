use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use uuid::Uuid;

use crate::{
    entity::{
        permission_group_permissions, permission_groups, rdd_permission_groups,
        role_designation_departments, roles, user_role_designation_departments,
        user_role_designation_permissions,
    },
    error::AppError,
    services::permission_scope_service,
};

use super::ensure_default_rdd;

pub(super) async fn ensure_urdp(
    txn: &DatabaseTransaction,
    urdd_id: Uuid,
    permission_id: Uuid,
    tenant_id: Uuid,
) -> Result<(), AppError> {
    if let Some(row) =
        user_role_designation_permissions::Entity::find_by_id((urdd_id, permission_id))
            .one(txn)
            .await?
    {
        if row.status != "active" || row.deleted_at.is_some() {
            let mut active = row.into_active_model();
            active.status = Set("active".into());
            active.deleted_at = Set(None);
            active.deleted_by = Set(None);
            active.update(txn).await?;
        }
        let scope =
            permission_scope_service::ensure_tenant_scope(txn, tenant_id, permission_id).await?;
        permission_scope_service::assign(txn, tenant_id, urdd_id, permission_id, scope.id).await?;
        return Ok(());
    }

    user_role_designation_permissions::ActiveModel {
        user_role_designation_department_id: Set(urdd_id),
        permission_id: Set(permission_id),
        tenant_id: Set(tenant_id),
        source: Set("group".into()),
        ..Default::default()
    }
    .insert(txn)
    .await?;
    let scope =
        permission_scope_service::ensure_tenant_scope(txn, tenant_id, permission_id).await?;
    permission_scope_service::assign(txn, tenant_id, urdd_id, permission_id, scope.id).await?;
    Ok(())
}

pub async fn grant_role_permission(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    role_id: Uuid,
    permission_id: Uuid,
) -> Result<(), AppError> {
    let role = roles::Entity::find_by_id(role_id)
        .filter(roles::Column::TenantId.eq(tenant_id))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let rdd = ensure_default_rdd(txn, tenant_id, role_id).await?;

    let group = match permission_groups::Entity::find()
        .filter(permission_groups::Column::TenantId.eq(tenant_id))
        .filter(permission_groups::Column::Name.eq(&role.name))
        .one(txn)
        .await?
    {
        Some(row) => row,
        None => {
            permission_groups::ActiveModel {
                tenant_id: Set(tenant_id),
                name: Set(role.name),
                description: Set("Role permission group".into()),
                ..Default::default()
            }
            .insert(txn)
            .await?
        }
    };

    if permission_group_permissions::Entity::find_by_id((group.id, permission_id))
        .one(txn)
        .await?
        .is_none()
    {
        permission_group_permissions::ActiveModel {
            permission_group_id: Set(group.id),
            permission_id: Set(permission_id),
            tenant_id: Set(tenant_id),
            ..Default::default()
        }
        .insert(txn)
        .await?;
    }

    if rdd_permission_groups::Entity::find_by_id((rdd.id, group.id))
        .one(txn)
        .await?
        .is_none()
    {
        rdd_permission_groups::ActiveModel {
            role_designation_department_id: Set(rdd.id),
            permission_group_id: Set(group.id),
            tenant_id: Set(tenant_id),
            ..Default::default()
        }
        .insert(txn)
        .await?;
    }

    let urdds = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(user_role_designation_departments::Column::RoleDesignationDepartmentId.eq(rdd.id))
        .filter(user_role_designation_departments::Column::Status.eq("active"))
        .all(txn)
        .await?;
    for urdd in urdds {
        ensure_urdp(txn, urdd.id, permission_id, tenant_id).await?;
    }
    Ok(())
}

pub async fn revoke_role_permission(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    role_id: Uuid,
    permission_id: Uuid,
) -> Result<(), AppError> {
    let rdd_ids: Vec<Uuid> = role_designation_departments::Entity::find()
        .filter(role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(role_designation_departments::Column::RoleId.eq(role_id))
        .all(txn)
        .await?
        .into_iter()
        .map(|row| row.id)
        .collect();
    let urdd_ids: Vec<Uuid> = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(
            user_role_designation_departments::Column::RoleDesignationDepartmentId.is_in(rdd_ids),
        )
        .all(txn)
        .await?
        .into_iter()
        .map(|row| row.id)
        .collect();
    let rows = user_role_designation_permissions::Entity::find()
        .filter(
            user_role_designation_permissions::Column::UserRoleDesignationDepartmentId
                .is_in(urdd_ids),
        )
        .filter(user_role_designation_permissions::Column::PermissionId.eq(permission_id))
        .all(txn)
        .await?;
    for row in rows {
        let mut active = row.into_active_model();
        active.status = Set("inactive".into());
        active.update(txn).await?;
    }
    Ok(())
}
