use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use uuid::Uuid;

use crate::{
    entity::{
        departments, designations, role_designation_departments, role_permissions,
        user_role_designation_departments, user_role_designation_permissions,
    },
    error::AppError,
};

#[path = "contextual_rbac_service/permissions.rs"]
mod permissions;
pub(crate) use permissions::ensure_urdp;
pub use permissions::{grant_role_permission, revoke_role_permission};

async fn default_department(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
) -> Result<departments::Model, AppError> {
    if let Some(row) = departments::Entity::find()
        .filter(departments::Column::TenantId.eq(tenant_id))
        .filter(departments::Column::Code.eq("GENERAL"))
        .one(txn)
        .await?
    {
        return Ok(row);
    }

    Ok(departments::ActiveModel {
        tenant_id: Set(tenant_id),
        name: Set("General".into()),
        code: Set("GENERAL".into()),
        description: Set("Default department".into()),
        ..Default::default()
    }
    .insert(txn)
    .await?)
}

async fn default_designation(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
) -> Result<designations::Model, AppError> {
    if let Some(row) = designations::Entity::find()
        .filter(designations::Column::TenantId.eq(tenant_id))
        .filter(designations::Column::Code.eq("MEMBER"))
        .one(txn)
        .await?
    {
        return Ok(row);
    }

    Ok(designations::ActiveModel {
        tenant_id: Set(tenant_id),
        name: Set("Member".into()),
        code: Set("MEMBER".into()),
        description: Set("Default designation".into()),
        ..Default::default()
    }
    .insert(txn)
    .await?)
}

pub async fn ensure_default_rdd(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    role_id: Uuid,
) -> Result<role_designation_departments::Model, AppError> {
    let department = default_department(txn, tenant_id).await?;
    let designation = default_designation(txn, tenant_id).await?;

    if let Some(row) = role_designation_departments::Entity::find()
        .filter(role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(role_designation_departments::Column::RoleId.eq(role_id))
        .filter(role_designation_departments::Column::DesignationId.eq(designation.id))
        .filter(role_designation_departments::Column::DepartmentId.eq(department.id))
        .one(txn)
        .await?
    {
        return Ok(row);
    }

    Ok(role_designation_departments::ActiveModel {
        tenant_id: Set(tenant_id),
        role_id: Set(role_id),
        designation_id: Set(designation.id),
        department_id: Set(department.id),
        ..Default::default()
    }
    .insert(txn)
    .await?)
}

pub async fn assign_role(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
    role_id: Uuid,
) -> Result<Uuid, AppError> {
    let rdd = ensure_default_rdd(txn, tenant_id, role_id).await?;
    let existing = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(user_role_designation_departments::Column::UserId.eq(user_id))
        .filter(user_role_designation_departments::Column::RoleDesignationDepartmentId.eq(rdd.id))
        .one(txn)
        .await?;

    let urdd = match existing {
        Some(row) => {
            let mut active = row.into_active_model();
            active.status = Set("active".into());
            active.ends_at = Set(None);
            active.deleted_at = Set(None);
            active.deleted_by = Set(None);
            active.update(txn).await?
        }
        None => {
            user_role_designation_departments::ActiveModel {
                tenant_id: Set(tenant_id),
                user_id: Set(user_id),
                role_designation_department_id: Set(rdd.id),
                ..Default::default()
            }
            .insert(txn)
            .await?
        }
    };

    let permission_ids: Vec<Uuid> = role_permissions::Entity::find()
        .filter(role_permissions::Column::TenantId.eq(tenant_id))
        .filter(role_permissions::Column::RoleId.eq(role_id))
        .filter(role_permissions::Column::Status.eq("active"))
        .filter(role_permissions::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .map(|row| row.permission_id)
        .collect();

    for permission_id in permission_ids {
        ensure_urdp(txn, urdd.id, permission_id, tenant_id).await?;
    }
    Ok(urdd.id)
}

pub async fn revoke_role(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
    role_id: Uuid,
) -> Result<(), AppError> {
    let rdd_ids: Vec<Uuid> = role_designation_departments::Entity::find()
        .filter(role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(role_designation_departments::Column::RoleId.eq(role_id))
        .all(txn)
        .await?
        .into_iter()
        .map(|row| row.id)
        .collect();

    let rows = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(user_role_designation_departments::Column::UserId.eq(user_id))
        .filter(
            user_role_designation_departments::Column::RoleDesignationDepartmentId.is_in(rdd_ids),
        )
        .all(txn)
        .await?;

    for row in rows {
        let urdp_rows = user_role_designation_permissions::Entity::find()
            .filter(
                user_role_designation_permissions::Column::UserRoleDesignationDepartmentId
                    .eq(row.id),
            )
            .all(txn)
            .await?;
        for urdp in urdp_rows {
            let mut active = urdp.into_active_model();
            active.status = Set("inactive".into());
            active.update(txn).await?;
        }

        let mut active = row.into_active_model();
        active.status = Set("inactive".into());
        active.ends_at = Set(Some(Utc::now().fixed_offset()));
        active.update(txn).await?;
    }
    Ok(())
}
