use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use uuid::Uuid;

use crate::{
    entity::{role_designation_departments, roles, user_role_designation_departments, user_roles},
    error::AppError,
    services::contextual_rbac_service,
};

mod client;
pub use client::{
    assign_client_role, list_client_role_names_for_user, list_for_client, revoke_client_role,
};

pub struct CreateRoleInput {
    pub tenant_id: Uuid,
    pub name: String,
    pub description: String,
}

pub async fn create(
    txn: &DatabaseTransaction,
    input: CreateRoleInput,
) -> Result<roles::Model, AppError> {
    Ok(roles::ActiveModel {
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
) -> Result<Vec<roles::Model>, AppError> {
    Ok(roles::Entity::find()
        .filter(roles::Column::TenantId.eq(tenant_id))
        .filter(roles::Column::Status.eq("active"))
        .filter(roles::Column::DeletedAt.is_null())
        .all(txn)
        .await?)
}

pub async fn find_by_id(
    txn: &DatabaseTransaction,
    id: Uuid,
) -> Result<Option<roles::Model>, AppError> {
    Ok(roles::Entity::find_by_id(id)
        .filter(roles::Column::Status.eq("active"))
        .filter(roles::Column::DeletedAt.is_null())
        .one(txn)
        .await?)
}

pub async fn update(
    txn: &DatabaseTransaction,
    id: Uuid,
    name: String,
    description: String,
) -> Result<(), AppError> {
    let role = find_by_id(txn, id).await?.ok_or(AppError::NotFound)?;
    let mut active = role.into_active_model();
    active.name = Set(name);
    active.description = Set(description);
    active.update(txn).await?;
    Ok(())
}

pub async fn delete(txn: &DatabaseTransaction, id: Uuid) -> Result<(), AppError> {
    let role = roles::Entity::find_by_id(id)
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut active = role.into_active_model();
    active.status = Set("inactive".into());
    active.update(txn).await?;
    Ok(())
}

pub async fn list_for_user(
    txn: &DatabaseTransaction,
    user_id: Uuid,
    tenant_id: Uuid,
) -> Result<Vec<roles::Model>, AppError> {
    let rdd_ids: Vec<Uuid> = user_role_designation_departments::Entity::find()
        .filter(user_role_designation_departments::Column::UserId.eq(user_id))
        .filter(user_role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(user_role_designation_departments::Column::Status.eq("active"))
        .filter(user_role_designation_departments::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .map(|urdd| urdd.role_designation_department_id)
        .collect();

    if rdd_ids.is_empty() {
        return Ok(vec![]);
    }

    let role_ids: Vec<Uuid> = role_designation_departments::Entity::find()
        .filter(role_designation_departments::Column::Id.is_in(rdd_ids))
        .filter(role_designation_departments::Column::TenantId.eq(tenant_id))
        .filter(role_designation_departments::Column::Status.eq("active"))
        .filter(role_designation_departments::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .map(|rdd| rdd.role_id)
        .collect();

    Ok(roles::Entity::find()
        .filter(roles::Column::Id.is_in(role_ids))
        .filter(roles::Column::TenantId.eq(tenant_id))
        .filter(roles::Column::Status.eq("active"))
        .filter(roles::Column::DeletedAt.is_null())
        .all(txn)
        .await?)
}

pub async fn list_names_for_user(
    txn: &DatabaseTransaction,
    user_id: Uuid,
    tenant_id: Uuid,
) -> Result<Vec<String>, AppError> {
    Ok(list_for_user(txn, user_id, tenant_id)
        .await?
        .into_iter()
        .map(|r| r.name)
        .collect())
}

pub async fn assign(
    txn: &DatabaseTransaction,
    user_id: Uuid,
    role_id: Uuid,
    tenant_id: Uuid,
) -> Result<(), AppError> {
    let now = Utc::now().fixed_offset();
    if let Some(row) = user_roles::Entity::find_by_id((user_id, role_id))
        .one(txn)
        .await?
    {
        let mut active = row.into_active_model();
        active.status = Set("active".into());
        active.deleted_at = Set(None);
        active.deleted_by = Set(None);
        active.update(txn).await?;
    } else {
        user_roles::ActiveModel {
            user_id: Set(user_id),
            role_id: Set(role_id),
            tenant_id: Set(tenant_id),
            assigned_at: Set(now),
            ..Default::default()
        }
        .insert(txn)
        .await?;
    }

    contextual_rbac_service::assign_role(txn, tenant_id, user_id, role_id).await?;

    Ok(())
}

pub async fn revoke(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
    role_id: Uuid,
) -> Result<(), AppError> {
    if let Some(row) = user_roles::Entity::find_by_id((user_id, role_id))
        .one(txn)
        .await?
    {
        let mut active = row.into_active_model();
        active.status = Set("inactive".into());
        active.update(txn).await?;
    }
    contextual_rbac_service::revoke_role(txn, tenant_id, user_id, role_id).await?;
    Ok(())
}
