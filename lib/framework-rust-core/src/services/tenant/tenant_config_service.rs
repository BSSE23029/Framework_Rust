use crate::{
    entity::{tenant_config_definitions, tenant_config_values},
    error::AppError,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConfigValueType {
    Boolean,
    Integer,
    Decimal,
    String,
    Datetime,
    Uuid,
    Json,
}

impl ConfigValueType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Decimal => "decimal",
            Self::String => "string",
            Self::Datetime => "datetime",
            Self::Uuid => "uuid",
            Self::Json => "json",
        }
    }
    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "boolean" => Self::Boolean,
            "integer" => Self::Integer,
            "decimal" => Self::Decimal,
            "string" => Self::String,
            "datetime" => Self::Datetime,
            "uuid" => Self::Uuid,
            "json" => Self::Json,
            _ => return None,
        })
    }
}

pub struct DefineInput {
    pub tenant_id: Uuid,
    pub config_key: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub scope_type: String,
    pub value_type: ConfigValueType,
    pub is_required: bool,
    pub is_multi_value: bool,
    pub default_value: Option<Value>,
    pub allowed_values: Option<Value>,
    pub constraints: Value,
}

pub async fn define(
    txn: &DatabaseTransaction,
    input: DefineInput,
) -> Result<tenant_config_definitions::Model, AppError> {
    validate_slug("config_key", &input.config_key)?;
    validate_slug("scope_type", &input.scope_type)?;
    if let Some(allowed) = &input.allowed_values {
        if !allowed.is_array() {
            return Err(AppError::InvalidInput(
                "allowed_values must be an array".into(),
            ));
        }
    }
    if let Some(default) = &input.default_value {
        validate_value(
            input.value_type,
            input.is_multi_value,
            default,
            input.allowed_values.as_ref(),
            &input.constraints,
        )?;
    } else if input.is_required {
        return Err(AppError::InvalidInput(
            "required definitions need a default value".into(),
        ));
    }
    Ok(tenant_config_definitions::ActiveModel {
        tenant_id: Set(input.tenant_id),
        config_key: Set(input.config_key),
        display_name: Set(input.display_name),
        description: Set(input.description),
        category: Set(input.category),
        scope_type: Set(input.scope_type),
        value_type: Set(input.value_type.as_str().into()),
        is_required: Set(input.is_required),
        is_multi_value: Set(input.is_multi_value),
        default_value: Set(input.default_value),
        allowed_values: Set(input.allowed_values),
        constraints: Set(input.constraints),
        ..Default::default()
    }
    .insert(txn)
    .await?)
}

pub async fn set_value(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    definition_id: Uuid,
    scope_id: Option<Uuid>,
    value: Value,
) -> Result<tenant_config_values::Model, AppError> {
    let definition = tenant_config_definitions::Entity::find_by_id(definition_id)
        .filter(tenant_config_definitions::Column::TenantId.eq(tenant_id))
        .filter(tenant_config_definitions::Column::Status.eq("active"))
        .filter(tenant_config_definitions::Column::DeletedAt.is_null())
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    if (definition.scope_type == "tenant") != scope_id.is_none() {
        return Err(AppError::InvalidInput(
            "tenant scope must omit scope_id; other scopes require it".into(),
        ));
    }
    let value_type = ConfigValueType::parse(&definition.value_type)
        .ok_or_else(|| AppError::InvalidInput("unknown configuration value type".into()))?;
    validate_value(
        value_type,
        definition.is_multi_value,
        &value,
        definition.allowed_values.as_ref(),
        &definition.constraints,
    )?;
    let existing = tenant_config_values::Entity::find()
        .filter(tenant_config_values::Column::TenantId.eq(tenant_id))
        .filter(tenant_config_values::Column::DefinitionId.eq(definition_id))
        .filter(tenant_config_values::Column::ScopeType.eq(&definition.scope_type))
        .filter(match scope_id {
            Some(id) => tenant_config_values::Column::ScopeId.eq(id),
            None => tenant_config_values::Column::ScopeId.is_null(),
        })
        .one(txn)
        .await?;
    if let Some(row) = existing {
        let mut active = row.into_active_model();
        active.config_value = Set(value);
        active.status = Set("active".into());
        active.deleted_at = Set(None);
        active.deleted_by = Set(None);
        return Ok(active.update(txn).await?);
    }
    Ok(tenant_config_values::ActiveModel {
        tenant_id: Set(tenant_id),
        definition_id: Set(definition_id),
        scope_type: Set(definition.scope_type),
        scope_id: Set(scope_id),
        config_value: Set(value),
        ..Default::default()
    }
    .insert(txn)
    .await?)
}

pub async fn resolve(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    definition_id: Uuid,
    scope_id: Option<Uuid>,
) -> Result<Option<Value>, AppError> {
    let definition = tenant_config_definitions::Entity::find_by_id(definition_id)
        .filter(tenant_config_definitions::Column::TenantId.eq(tenant_id))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let value = tenant_config_values::Entity::find()
        .filter(tenant_config_values::Column::TenantId.eq(tenant_id))
        .filter(tenant_config_values::Column::DefinitionId.eq(definition_id))
        .filter(tenant_config_values::Column::Status.eq("active"))
        .filter(tenant_config_values::Column::DeletedAt.is_null())
        .filter(match scope_id {
            Some(id) => tenant_config_values::Column::ScopeId.eq(id),
            None => tenant_config_values::Column::ScopeId.is_null(),
        })
        .one(txn)
        .await?;
    Ok(value
        .map(|row| row.config_value)
        .or(definition.default_value))
}

pub async fn deactivate_value(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    value_id: Uuid,
) -> Result<(), AppError> {
    let row = tenant_config_values::Entity::find_by_id(value_id)
        .filter(tenant_config_values::Column::TenantId.eq(tenant_id))
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut active = row.into_active_model();
    active.status = Set("inactive".into());
    active.update(txn).await?;
    Ok(())
}

fn validate_slug(field: &str, value: &str) -> Result<(), AppError> {
    if value.is_empty()
        || !value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
    {
        return Err(AppError::InvalidInput(format!(
            "{field} must be a lowercase slug"
        )));
    }
    Ok(())
}

fn validate_value(
    kind: ConfigValueType,
    multi: bool,
    value: &Value,
    allowed: Option<&Value>,
    constraints: &Value,
) -> Result<(), AppError> {
    let values: Vec<&Value> = if multi {
        value
            .as_array()
            .ok_or_else(|| {
                AppError::InvalidInput("multi-value configuration requires an array".into())
            })?
            .iter()
            .collect()
    } else {
        vec![value]
    };
    for item in values {
        let valid = match kind {
            ConfigValueType::Boolean => item.is_boolean(),
            ConfigValueType::Integer => item.as_i64().is_some(),
            ConfigValueType::Decimal => item.as_f64().is_some(),
            ConfigValueType::String => item.is_string(),
            ConfigValueType::Datetime => item
                .as_str()
                .is_some_and(|v| chrono::DateTime::parse_from_rfc3339(v).is_ok()),
            ConfigValueType::Uuid => item.as_str().is_some_and(|v| Uuid::parse_str(v).is_ok()),
            ConfigValueType::Json => !item.is_null(),
        };
        if !valid {
            return Err(AppError::InvalidInput(format!(
                "value does not match {}",
                kind.as_str()
            )));
        }
        if allowed
            .and_then(Value::as_array)
            .is_some_and(|items| !items.contains(item))
        {
            return Err(AppError::InvalidInput("value is not allowed".into()));
        }
        if let Some(number) = item.as_f64() {
            if constraints
                .get("min")
                .and_then(Value::as_f64)
                .is_some_and(|min| number < min)
                || constraints
                    .get("max")
                    .and_then(Value::as_f64)
                    .is_some_and(|max| number > max)
            {
                return Err(AppError::InvalidInput(
                    "numeric value is outside constraints".into(),
                ));
            }
        }
        if let Some(text) = item.as_str() {
            let len = text.chars().count() as u64;
            if constraints
                .get("min_length")
                .and_then(Value::as_u64)
                .is_some_and(|min| len < min)
                || constraints
                    .get("max_length")
                    .and_then(Value::as_u64)
                    .is_some_and(|max| len > max)
            {
                return Err(AppError::InvalidInput(
                    "string length is outside constraints".into(),
                ));
            }
        }
    }
    Ok(())
}
