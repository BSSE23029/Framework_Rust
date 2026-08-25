use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set,
};
use uuid::Uuid;

use crate::{entity::one_time_tokens, error::AppError, services::one_time_token_service};

pub const OTP_TTL_MINUTES: i64 = 10;
pub const MAX_ATTEMPTS: i32 = 5;

fn constant_time_equal(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.as_bytes()
        .iter()
        .zip(right.as_bytes())
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

pub async fn issue(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
) -> Result<String, AppError> {
    let previous = one_time_tokens::Entity::find()
        .filter(one_time_tokens::Column::TenantId.eq(tenant_id))
        .filter(one_time_tokens::Column::UserId.eq(user_id))
        .filter(one_time_tokens::Column::TokenType.eq(one_time_tokens::TYPE_PASSWORDLESS_LOGIN))
        .filter(one_time_tokens::Column::Status.eq("active"))
        .all(txn)
        .await?;
    for row in previous {
        let mut active = row.into_active_model();
        active.status = Set("inactive".into());
        active.update(txn).await?;
    }

    let otp = one_time_token_service::generate_otp();
    one_time_tokens::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        user_id: Set(user_id),
        token_hash: Set(one_time_token_service::hash_otp(user_id, &otp)),
        token_type: Set(one_time_tokens::TYPE_PASSWORDLESS_LOGIN.into()),
        expires_at: Set((Utc::now() + Duration::minutes(OTP_TTL_MINUTES)).fixed_offset()),
        used_at: Set(None),
        attempt_count: Set(0),
        max_attempts: Set(MAX_ATTEMPTS),
        ..Default::default()
    }
    .insert(txn)
    .await?;
    Ok(otp)
}

pub async fn consume(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
    otp: &str,
) -> Result<(), AppError> {
    let row = one_time_tokens::Entity::find()
        .filter(one_time_tokens::Column::TenantId.eq(tenant_id))
        .filter(one_time_tokens::Column::UserId.eq(user_id))
        .filter(one_time_tokens::Column::TokenType.eq(one_time_tokens::TYPE_PASSWORDLESS_LOGIN))
        .filter(one_time_tokens::Column::Status.eq("active"))
        .filter(one_time_tokens::Column::DeletedAt.is_null())
        .filter(one_time_tokens::Column::UsedAt.is_null())
        .order_by_desc(one_time_tokens::Column::CreatedAt)
        .one(txn)
        .await?
        .ok_or(AppError::Unauthorized)?;
    let now = Utc::now().fixed_offset();
    if row.expires_at <= now || row.attempt_count >= row.max_attempts {
        return Err(AppError::Unauthorized);
    }
    let expected = one_time_token_service::hash_otp(user_id, otp);
    let valid = constant_time_equal(&row.token_hash, &expected);
    let mut active = row.into_active_model();
    if !valid {
        active.attempt_count = Set(active.attempt_count.as_ref() + 1);
        if active.attempt_count.as_ref() >= active.max_attempts.as_ref() {
            active.status = Set("inactive".into());
        }
        active.update(txn).await?;
        return Err(AppError::Unauthorized);
    }
    active.used_at = Set(Some(now));
    active.status = Set("inactive".into());
    active.update(txn).await?;
    Ok(())
}
