use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    entity::{
        departments, designations, permission_groups, role_designation_departments, roles,
        user_role_designation_departments, users,
    },
    error::AppError,
    services::permission_group_service,
};

pub async fn list_departments(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
) -> Result<Vec<departments::Model>, AppError> {
    Ok(departments::Entity::find()
        .filter(departments::Column::TenantId.eq(tenant_id))
        .filter(departments::Column::Status.eq("active"))
        .filter(departments::Column::DeletedAt.is_null())
        .all(txn)
        .await?)
}

pub async fn create_department(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    name: &str,
    code: &str,
    description: &str,
) -> Result<departments::Model, AppError> {
    validate_name_code(name, code)?;
    Ok(departments::ActiveModel {
        tenant_id: Set(tenant_id),
        name: Set(name.trim().into()),
        code: Set(code.trim().to_ascii_uppercase()),
        description: Set(description.trim().into()),
        ..Default::default()
    }
    .insert(txn)
    .await?)
}

pub async fn list_designations(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
) -> Result<Vec<designations::Model>, AppError> {
    Ok(designations::Entity::find()
        .filter(designations::Column::TenantId.eq(tenant_id))
        .filter(designations::Column::Status.eq("active"))
        .filter(designations::Column::DeletedAt.is_null())
        .all(txn)
        .await?)
}

pub async fn create_designation(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    name: &str,
    code: &str,
    description: &str,
    senior_designation_id: Option<Uuid>,
) -> Result<designations::Model, AppError> {
    validate_name_code(name, code)?;
    if let Some(senior_id) = senior_designation_id {
        designations::Entity::find_by_id(senior_id)
            .filter(designations::Column::TenantId.eq(tenant_id))
            .filter(designations::Column::Status.eq("active"))
            .one(txn)
            .await?
            .ok_or(AppError::NotFound)?;
    }
    Ok(designations::ActiveModel {
        tenant_id: Set(tenant_id),
        name: Set(name.trim().into()),
        code: Set(code.trim().to_ascii_uppercase()),
        description: Set(description.trim().into()),
        senior_designation_id: Set(senior_designation_id),
        ..Default::default()
    }
    .insert(txn)
    .await?)
}

pub async fn list_rdds(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
) -> Result<Vec<role_designation_departments::Model>, AppError> {
    Ok(role_designation_departments::Entity::find()
        .filter(role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(role_designation_departments::Column::Status.eq("active"))
        .filter(role_designation_departments::Column::DeletedAt.is_null())
        .all(txn)
        .await?)
}

pub async fn create_rdd(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    role_id: Uuid,
    designation_id: Uuid,
    department_id: Uuid,
) -> Result<role_designation_departments::Model, AppError> {
    let role = roles::Entity::find_by_id(role_id)
        .filter(roles::Column::TenantId.eq(tenant_id))
        .filter(roles::Column::Status.eq("active"))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    require_department(txn, tenant_id, department_id).await?;
    require_designation(txn, tenant_id, designation_id).await?;
    let existing = role_designation_departments::Entity::find()
        .filter(role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(role_designation_departments::Column::RoleId.eq(role_id))
        .filter(role_designation_departments::Column::DesignationId.eq(designation_id))
        .filter(role_designation_departments::Column::DepartmentId.eq(department_id))
        .one(txn)
        .await?;
    let rdd = if let Some(row) = existing {
        let mut active = row.into_active_model();
        active.status = Set("active".into());
        active.deleted_at = Set(None);
        active.deleted_by = Set(None);
        active.update(txn).await?
    } else {
        role_designation_departments::ActiveModel {
            tenant_id: Set(tenant_id),
            role_id: Set(role_id),
            designation_id: Set(designation_id),
            department_id: Set(department_id),
            ..Default::default()
        }
        .insert(txn)
        .await?
    };
    if let Some(group) = permission_groups::Entity::find()
        .filter(permission_groups::Column::TenantId.eq(tenant_id))
        .filter(permission_groups::Column::Name.eq(role.name))
        .filter(permission_groups::Column::Status.eq("active"))
        .one(txn)
        .await?
    {
        permission_group_service::link_rdd(txn, tenant_id, rdd.id, group.id).await?;
    }
    Ok(rdd)
}

pub async fn list_urdds(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
) -> Result<Vec<user_role_designation_departments::Model>, AppError> {
    Ok(user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(user_role_designation_departments::Column::Status.eq("active"))
        .filter(user_role_designation_departments::Column::DeletedAt.is_null())
        .all(txn)
        .await?)
}

pub async fn assign_user_context(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
    rdd_id: Uuid,
    attributes: Value,
) -> Result<user_role_designation_departments::Model, AppError> {
    users::Entity::find_by_id(user_id)
        .filter(users::Column::TenantId.eq(tenant_id))
        .filter(users::Column::Status.eq("active"))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    role_designation_departments::Entity::find_by_id(rdd_id)
        .filter(role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(role_designation_departments::Column::Status.eq("active"))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let existing = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(user_role_designation_departments::Column::UserId.eq(user_id))
        .filter(user_role_designation_departments::Column::RoleDesignationDepartmentId.eq(rdd_id))
        .one(txn)
        .await?;
    let urdd = if let Some(row) = existing {
        let mut active = row.into_active_model();
        active.status = Set("active".into());
        active.ends_at = Set(None);
        active.attributes = Set(attributes);
        active.deleted_at = Set(None);
        active.deleted_by = Set(None);
        active.update(txn).await?
    } else {
        user_role_designation_departments::ActiveModel {
            tenant_id: Set(tenant_id),
            user_id: Set(user_id),
            role_designation_department_id: Set(rdd_id),
            attributes: Set(attributes),
            ..Default::default()
        }
        .insert(txn)
        .await?
    };
    permission_group_service::sync_urdd(txn, tenant_id, urdd.id, rdd_id).await?;
    Ok(urdd)
}

pub async fn revoke_user_context(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    urdd_id: Uuid,
) -> Result<(), AppError> {
    let row = user_role_designation_departments::Entity::find_by_id(urdd_id)
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut active = row.into_active_model();
    active.status = Set("inactive".into());
    active.ends_at = Set(Some(Utc::now().fixed_offset()));
    active.update(txn).await?;
    Ok(())
}

async fn require_department(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    id: Uuid,
) -> Result<(), AppError> {
    departments::Entity::find_by_id(id)
        .filter(departments::Column::TenantId.eq(tenant_id))
        .filter(departments::Column::Status.eq("active"))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(())
}

async fn require_designation(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    id: Uuid,
) -> Result<(), AppError> {
    designations::Entity::find_by_id(id)
        .filter(designations::Column::TenantId.eq(tenant_id))
        .filter(designations::Column::Status.eq("active"))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(())
}

fn validate_name_code(name: &str, code: &str) -> Result<(), AppError> {
    if name.trim().is_empty()
        || code.trim().is_empty()
        || !code
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
    {
        return Err(AppError::InvalidInput("invalid name or code".into()));
    }
    Ok(())
}
