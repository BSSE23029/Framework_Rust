use std::collections::HashMap;

use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    entity::{
        departments, designations, role_designation_departments, roles,
        user_role_designation_departments,
    },
    error::AppError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct AccessContext {
    pub urdd_id: Uuid,
    pub rdd_id: Uuid,
    pub role_id: Uuid,
    pub role_name: String,
    pub designation_id: Uuid,
    pub designation_name: String,
    pub department_id: Uuid,
    pub department_name: String,
}

pub async fn list_for_user(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<AccessContext>, AppError> {
    let urdds = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(user_role_designation_departments::Column::UserId.eq(user_id))
        .filter(user_role_designation_departments::Column::Status.eq("active"))
        .filter(user_role_designation_departments::Column::DeletedAt.is_null())
        .all(txn)
        .await?;
    if urdds.is_empty() {
        return Ok(Vec::new());
    }

    let rdd_ids = urdds
        .iter()
        .map(|row| row.role_designation_department_id)
        .collect::<Vec<_>>();
    let rdds = role_designation_departments::Entity::find()
        .filter(role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(role_designation_departments::Column::Id.is_in(rdd_ids))
        .filter(role_designation_departments::Column::Status.eq("active"))
        .filter(role_designation_departments::Column::DeletedAt.is_null())
        .all(txn)
        .await?;
    let roles = active_roles(txn, tenant_id, &rdds).await?;
    let designations = active_designations(txn, tenant_id, &rdds).await?;
    let departments = active_departments(txn, tenant_id, &rdds).await?;
    let rdds = rdds
        .into_iter()
        .map(|row| (row.id, row))
        .collect::<HashMap<_, _>>();

    Ok(urdds
        .into_iter()
        .filter_map(|urdd| {
            let rdd = rdds.get(&urdd.role_designation_department_id)?;
            Some(AccessContext {
                urdd_id: urdd.id,
                rdd_id: rdd.id,
                role_id: rdd.role_id,
                role_name: roles.get(&rdd.role_id)?.clone(),
                designation_id: rdd.designation_id,
                designation_name: designations.get(&rdd.designation_id)?.clone(),
                department_id: rdd.department_id,
                department_name: departments.get(&rdd.department_id)?.clone(),
            })
        })
        .collect())
}

async fn active_roles(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    rdds: &[role_designation_departments::Model],
) -> Result<HashMap<Uuid, String>, AppError> {
    Ok(roles::Entity::find()
        .filter(roles::Column::TenantId.eq(tenant_id))
        .filter(roles::Column::Id.is_in(rdds.iter().map(|row| row.role_id)))
        .filter(roles::Column::Status.eq("active"))
        .filter(roles::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .map(|row| (row.id, row.name))
        .collect())
}

async fn active_designations(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    rdds: &[role_designation_departments::Model],
) -> Result<HashMap<Uuid, String>, AppError> {
    Ok(designations::Entity::find()
        .filter(designations::Column::TenantId.eq(tenant_id))
        .filter(designations::Column::Id.is_in(rdds.iter().map(|row| row.designation_id)))
        .filter(designations::Column::Status.eq("active"))
        .filter(designations::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .map(|row| (row.id, row.name))
        .collect())
}

async fn active_departments(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    rdds: &[role_designation_departments::Model],
) -> Result<HashMap<Uuid, String>, AppError> {
    Ok(departments::Entity::find()
        .filter(departments::Column::TenantId.eq(tenant_id))
        .filter(departments::Column::Id.is_in(rdds.iter().map(|row| row.department_id)))
        .filter(departments::Column::Status.eq("active"))
        .filter(departments::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .map(|row| (row.id, row.name))
        .collect())
}
