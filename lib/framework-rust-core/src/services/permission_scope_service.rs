use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    entity::{
        permission_scopes, permissions, user_role_designation_departments,
        user_role_designation_permission_scopes, user_role_designation_permissions,
    },
    error::AppError,
};

pub struct CreateScopeInput {
    pub tenant_id: Uuid,
    pub permission_id: Uuid,
    pub name: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub conditions: Value,
}

pub async fn ensure_tenant_scope(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    permission_id: Uuid,
) -> Result<permission_scopes::Model, AppError> {
    if let Some(scope) = permission_scopes::Entity::find()
        .filter(permission_scopes::Column::TenantId.eq(tenant_id))
        .filter(permission_scopes::Column::PermissionId.eq(permission_id))
        .filter(permission_scopes::Column::ScopeType.eq("tenant"))
        .one(txn)
        .await?
    {
        return restore_scope(txn, scope).await;
    }
    Ok(permission_scopes::ActiveModel {
        tenant_id: Set(tenant_id),
        permission_id: Set(permission_id),
        name: Set("Tenant-wide".into()),
        scope_type: Set("tenant".into()),
        conditions: Set(Value::Object(Default::default())),
        ..Default::default()
    }
    .insert(txn)
    .await?)
}

pub async fn create(
    txn: &DatabaseTransaction,
    input: CreateScopeInput,
) -> Result<permission_scopes::Model, AppError> {
    let resource_type = input.resource_type.trim();
    if resource_type.is_empty() {
        return Err(AppError::InvalidInput("resource_type is required".into()));
    }
    permissions::Entity::find_by_id(input.permission_id)
        .filter(permissions::Column::TenantId.eq(input.tenant_id))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let scope_type = if input.resource_id.is_some() {
        "resource"
    } else {
        "resource_type"
    };
    Ok(permission_scopes::ActiveModel {
        tenant_id: Set(input.tenant_id),
        permission_id: Set(input.permission_id),
        name: Set(input.name.trim().to_owned()),
        scope_type: Set(scope_type.into()),
        resource_type: Set(Some(resource_type.to_owned())),
        resource_id: Set(input.resource_id),
        conditions: Set(input.conditions),
        ..Default::default()
    }
    .insert(txn)
    .await?)
}

pub async fn assign(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    urdd_id: Uuid,
    permission_id: Uuid,
    scope_id: Uuid,
) -> Result<(), AppError> {
    let scope = permission_scopes::Entity::find_by_id(scope_id)
        .filter(permission_scopes::Column::TenantId.eq(tenant_id))
        .filter(permission_scopes::Column::PermissionId.eq(permission_id))
        .filter(permission_scopes::Column::Status.eq("active"))
        .filter(permission_scopes::Column::DeletedAt.is_null())
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let key = (urdd_id, permission_id, scope.id);
    if let Some(row) = user_role_designation_permission_scopes::Entity::find_by_id(key)
        .one(txn)
        .await?
    {
        let mut active = row.into_active_model();
        active.status = Set("active".into());
        active.deleted_at = Set(None);
        active.deleted_by = Set(None);
        active.update(txn).await?;
        return Ok(());
    }
    user_role_designation_permission_scopes::ActiveModel {
        user_role_designation_department_id: Set(urdd_id),
        permission_id: Set(permission_id),
        permission_scope_id: Set(scope.id),
        tenant_id: Set(tenant_id),
        ..Default::default()
    }
    .insert(txn)
    .await?;
    Ok(())
}

pub async fn deactivate(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    scope_id: Uuid,
) -> Result<(), AppError> {
    let row = permission_scopes::Entity::find_by_id(scope_id)
        .filter(permission_scopes::Column::TenantId.eq(tenant_id))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut active = row.into_active_model();
    active.status = Set("inactive".into());
    active.update(txn).await?;
    Ok(())
}

pub async fn restore(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    scope_id: Uuid,
) -> Result<permission_scopes::Model, AppError> {
    let row = permission_scopes::Entity::find_by_id(scope_id)
        .filter(permission_scopes::Column::TenantId.eq(tenant_id))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    restore_scope(txn, row).await
}

async fn restore_scope(
    txn: &DatabaseTransaction,
    row: permission_scopes::Model,
) -> Result<permission_scopes::Model, AppError> {
    if row.status == "active" && row.deleted_at.is_none() {
        return Ok(row);
    }
    let mut active = row.into_active_model();
    active.status = Set("active".into());
    active.deleted_at = Set(None);
    active.deleted_by = Set(None);
    Ok(active.update(txn).await?)
}

pub async fn is_allowed(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
    permission_name: &str,
    resource_type: Option<&str>,
    resource_id: Option<Uuid>,
) -> Result<bool, AppError> {
    let permission = match permissions::Entity::find()
        .filter(permissions::Column::TenantId.eq(tenant_id))
        .filter(permissions::Column::Name.eq(permission_name))
        .one(txn)
        .await?
    {
        Some(row) => row,
        None => return Ok(false),
    };
    let urdd_ids: Vec<Uuid> = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(user_role_designation_departments::Column::UserId.eq(user_id))
        .filter(user_role_designation_departments::Column::Status.eq("active"))
        .filter(user_role_designation_departments::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .map(|row| row.id)
        .collect();
    let active_urdps: Vec<Uuid> = user_role_designation_permissions::Entity::find()
        .filter(user_role_designation_permissions::Column::TenantId.eq(tenant_id))
        .filter(user_role_designation_permissions::Column::PermissionId.eq(permission.id))
        .filter(
            user_role_designation_permissions::Column::UserRoleDesignationDepartmentId
                .is_in(urdd_ids),
        )
        .filter(user_role_designation_permissions::Column::Status.eq("active"))
        .filter(user_role_designation_permissions::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .map(|row| row.user_role_designation_department_id)
        .collect();
    if active_urdps.is_empty() {
        return Ok(false);
    }
    let scope_ids: Vec<Uuid> = user_role_designation_permission_scopes::Entity::find()
        .filter(user_role_designation_permission_scopes::Column::TenantId.eq(tenant_id))
        .filter(user_role_designation_permission_scopes::Column::PermissionId.eq(permission.id))
        .filter(
            user_role_designation_permission_scopes::Column::UserRoleDesignationDepartmentId
                .is_in(active_urdps),
        )
        .filter(user_role_designation_permission_scopes::Column::Status.eq("active"))
        .filter(user_role_designation_permission_scopes::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .map(|row| row.permission_scope_id)
        .collect();
    let scopes = permission_scopes::Entity::find()
        .filter(permission_scopes::Column::Id.is_in(scope_ids))
        .filter(permission_scopes::Column::Status.eq("active"))
        .filter(permission_scopes::Column::DeletedAt.is_null())
        .all(txn)
        .await?;
    Ok(scopes
        .into_iter()
        .any(|scope| match scope.scope_type.as_str() {
            "tenant" => true,
            "resource_type" => scope.resource_type.as_deref() == resource_type,
            "resource" => {
                scope.resource_type.as_deref() == resource_type && scope.resource_id == resource_id
            }
            _ => false,
        }))
}
