use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use rand::{rngs::OsRng, RngCore};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{entity::trusted_devices, error::AppError};

pub struct TrustDeviceInput {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub fingerprint: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub ttl_days: i64,
}

pub struct IssuedDevice {
    pub device: trusted_devices::Model,
    pub token: String,
}

fn hash(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}

pub async fn trust(
    txn: &DatabaseTransaction,
    input: TrustDeviceInput,
) -> Result<IssuedDevice, AppError> {
    if input.name.trim().is_empty() || !(1..=365).contains(&input.ttl_days) {
        return Err(AppError::InvalidInput(
            "name is required and ttl_days must be between 1 and 365".into(),
        ));
    }
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let token = URL_SAFE_NO_PAD.encode(bytes);
    let device = trusted_devices::ActiveModel {
        tenant_id: Set(input.tenant_id),
        user_id: Set(input.user_id),
        name: Set(input.name.trim().to_owned()),
        token_hash: Set(hash(&token)),
        fingerprint_hash: Set(input.fingerprint.as_deref().map(hash)),
        user_agent: Set(input.user_agent),
        ip_address: Set(input.ip_address),
        expires_at: Set((Utc::now() + Duration::days(input.ttl_days)).fixed_offset()),
        ..Default::default()
    }
    .insert(txn)
    .await?;
    Ok(IssuedDevice { device, token })
}

pub async fn list(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<trusted_devices::Model>, AppError> {
    Ok(trusted_devices::Entity::find()
        .filter(trusted_devices::Column::TenantId.eq(tenant_id))
        .filter(trusted_devices::Column::UserId.eq(user_id))
        .filter(trusted_devices::Column::Status.eq("active"))
        .filter(trusted_devices::Column::DeletedAt.is_null())
        .order_by_desc(trusted_devices::Column::TrustedAt)
        .all(txn)
        .await?)
}

pub async fn verify(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
    token: &str,
    fingerprint: Option<&str>,
) -> Result<Option<trusted_devices::Model>, AppError> {
    let Some(row) = trusted_devices::Entity::find()
        .filter(trusted_devices::Column::TenantId.eq(tenant_id))
        .filter(trusted_devices::Column::UserId.eq(user_id))
        .filter(trusted_devices::Column::TokenHash.eq(hash(token)))
        .filter(trusted_devices::Column::Status.eq("active"))
        .filter(trusted_devices::Column::DeletedAt.is_null())
        .filter(trusted_devices::Column::ExpiresAt.gt(Utc::now().fixed_offset()))
        .one(txn)
        .await?
    else {
        return Ok(None);
    };
    if row.fingerprint_hash.as_deref() != fingerprint.map(hash).as_deref() {
        return Ok(None);
    }
    let mut active = row.into_active_model();
    active.last_used_at = Set(Some(Utc::now().fixed_offset()));
    Ok(Some(active.update(txn).await?))
}

pub async fn revoke(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
    device_id: Uuid,
) -> Result<(), AppError> {
    let row = trusted_devices::Entity::find_by_id(device_id)
        .filter(trusted_devices::Column::TenantId.eq(tenant_id))
        .filter(trusted_devices::Column::UserId.eq(user_id))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut active = row.into_active_model();
    active.status = Set("inactive".into());
    active.revoked_at = Set(Some(Utc::now().fixed_offset()));
    active.update(txn).await?;
    Ok(())
}

pub async fn restore(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
    device_id: Uuid,
) -> Result<trusted_devices::Model, AppError> {
    let row = trusted_devices::Entity::find_by_id(device_id)
        .filter(trusted_devices::Column::TenantId.eq(tenant_id))
        .filter(trusted_devices::Column::UserId.eq(user_id))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    if row.expires_at <= Utc::now().fixed_offset() {
        return Err(AppError::InvalidInput(
            "expired devices cannot be restored".into(),
        ));
    }
    let mut active = row.into_active_model();
    active.status = Set("active".into());
    active.revoked_at = Set(None);
    active.deleted_at = Set(None);
    active.deleted_by = Set(None);
    Ok(active.update(txn).await?)
}
