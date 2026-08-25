use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    Set, SqlErr,
};
use uuid::Uuid;

use crate::{
    entity::{tenant_domains, tenants},
    error::AppError,
};

fn valid_domain(value: &str) -> bool {
    value.len() <= 253
        && value.contains('.')
        && !value.starts_with('.')
        && !value.ends_with('.')
        && value.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
}

pub async fn set_hierarchy(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    tenant_type: &str,
    parent_tenant_id: Option<Uuid>,
) -> Result<(), AppError> {
    if !matches!(tenant_type, "organization" | "hotel" | "branch") {
        return Err(AppError::InvalidInput("invalid tenant type".into()));
    }
    if parent_tenant_id == Some(tenant_id) {
        return Err(AppError::InvalidInput(
            "a tenant cannot parent itself".into(),
        ));
    }

    let tenant = tenants::Entity::find_by_id(tenant_id)
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let parent = match parent_tenant_id {
        Some(id) => Some(
            tenants::Entity::find_by_id(id)
                .one(txn)
                .await?
                .ok_or(AppError::NotFound)?,
        ),
        None => None,
    };

    let valid_parent = match (
        tenant_type,
        parent.as_ref().map(|row| row.tenant_type.as_str()),
    ) {
        ("organization", None) | ("hotel", None) => true,
        ("hotel", Some("organization")) | ("branch", Some("hotel")) => true,
        _ => false,
    };
    if !valid_parent {
        return Err(AppError::InvalidInput(
            "hierarchy must be organization → hotel → branch".into(),
        ));
    }

    let mut ancestor = parent.and_then(|row| row.parent_tenant_id);
    while let Some(id) = ancestor {
        if id == tenant_id {
            return Err(AppError::InvalidInput("tenant hierarchy cycle".into()));
        }
        ancestor = tenants::Entity::find_by_id(id)
            .one(txn)
            .await?
            .and_then(|row| row.parent_tenant_id);
    }

    let mut active = tenant.into_active_model();
    active.tenant_type = Set(tenant_type.into());
    active.parent_tenant_id = Set(parent_tenant_id);
    active.update(txn).await?;
    Ok(())
}

pub async fn list_children(
    txn: &DatabaseTransaction,
    parent_tenant_id: Uuid,
) -> Result<Vec<tenants::Model>, AppError> {
    Ok(tenants::Entity::find()
        .filter(tenants::Column::ParentTenantId.eq(parent_tenant_id))
        .filter(tenants::Column::Status.eq("active"))
        .filter(tenants::Column::DeletedAt.is_null())
        .all(txn)
        .await?)
}

pub async fn create_domain(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    domain_name: &str,
    domain_type: &str,
    is_primary: bool,
) -> Result<tenant_domains::Model, AppError> {
    let normalized = domain_name.trim().to_ascii_lowercase();
    if !valid_domain(&normalized) {
        return Err(AppError::InvalidInput("invalid domain name".into()));
    }
    if !matches!(domain_type, "subdomain" | "custom") {
        return Err(AppError::InvalidInput("invalid domain type".into()));
    }
    if tenant_domains::Entity::find()
        .filter(tenant_domains::Column::DomainName.eq(&normalized))
        .filter(tenant_domains::Column::DeletedAt.is_null())
        .one(txn)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict);
    }
    if is_primary {
        clear_primary(txn, tenant_id).await?;
    }

    let insert = tenant_domains::ActiveModel {
        tenant_id: Set(tenant_id),
        domain_name: Set(normalized),
        domain_type: Set(domain_type.into()),
        is_primary: Set(is_primary),
        ..Default::default()
    }
    .insert(txn)
    .await;
    match insert {
        Ok(domain) => Ok(domain),
        Err(error) if matches!(error.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) => {
            Err(AppError::Conflict)
        }
        Err(error) => Err(error.into()),
    }
}

pub async fn list_domains(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
) -> Result<Vec<tenant_domains::Model>, AppError> {
    Ok(tenant_domains::Entity::find()
        .filter(tenant_domains::Column::TenantId.eq(tenant_id))
        .filter(tenant_domains::Column::Status.eq("active"))
        .filter(tenant_domains::Column::DeletedAt.is_null())
        .all(txn)
        .await?)
}

async fn clear_primary(txn: &DatabaseTransaction, tenant_id: Uuid) -> Result<(), AppError> {
    tenant_domains::Entity::update_many()
        .col_expr(tenant_domains::Column::IsPrimary, false.into())
        .filter(tenant_domains::Column::TenantId.eq(tenant_id))
        .filter(tenant_domains::Column::IsPrimary.eq(true))
        .exec(txn)
        .await?;
    Ok(())
}

pub async fn set_primary(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    domain_id: Uuid,
) -> Result<(), AppError> {
    let domain = tenant_domains::Entity::find_by_id(domain_id)
        .filter(tenant_domains::Column::TenantId.eq(tenant_id))
        .filter(tenant_domains::Column::Status.eq("active"))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    clear_primary(txn, tenant_id).await?;
    let mut active = domain.into_active_model();
    active.is_primary = Set(true);
    active.update(txn).await?;
    Ok(())
}

pub async fn verify_domain(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    domain_id: Uuid,
) -> Result<(), AppError> {
    let domain = tenant_domains::Entity::find_by_id(domain_id)
        .filter(tenant_domains::Column::TenantId.eq(tenant_id))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut active = domain.into_active_model();
    active.is_verified = Set(true);
    active.verified_at = Set(Some(Utc::now().fixed_offset()));
    active.update(txn).await?;
    Ok(())
}

pub async fn set_domain_active(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    domain_id: Uuid,
    enabled: bool,
) -> Result<(), AppError> {
    let domain = tenant_domains::Entity::find_by_id(domain_id)
        .filter(tenant_domains::Column::TenantId.eq(tenant_id))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut active = domain.into_active_model();
    active.status = Set(if enabled { "active" } else { "inactive" }.into());
    if enabled {
        active.deleted_at = Set(None);
        active.deleted_by = Set(None);
    }
    active.update(txn).await?;
    Ok(())
}
