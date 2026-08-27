use chrono::Utc;
use sea_orm::{ActiveModelTrait, ConnectionTrait, DatabaseConnection, Set};
use uuid::Uuid;

use crate::{db, entity::audit_log, error::AppError};

pub struct AuditEvent {
    pub tenant_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub metadata: serde_json::Value,
}

impl AuditEvent {
    pub fn new(
        tenant_id: Uuid,
        actor_id: Option<Uuid>,
        action: impl Into<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            tenant_id,
            actor_id,
            action: action.into(),
            metadata,
        }
    }
}

fn build_model(event: &AuditEvent) -> audit_log::ActiveModel {
    audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(event.tenant_id),
        user_id: Set(event.actor_id),
        action: Set(event.action.clone()),
        ip: Set(event
            .metadata
            .get("ip")
            .and_then(|v| v.as_str())
            .map(|s| s.to_owned())),
        metadata: Set(Some(event.metadata.to_string())),
        created_at: Set(Utc::now().fixed_offset()),
        ..Default::default()
    }
}

/// Synchronous audit write — use inside an existing transaction.
/// The insert shares the transaction; if it fails the whole operation rolls back.
pub async fn record<C: ConnectionTrait>(db: &C, event: AuditEvent) -> Result<(), AppError> {
    build_model(&event).insert(db).await?;
    Ok(())
}

/// Fire-and-forget audit write — for events where losing the log entry is
/// acceptable (e.g. informational reads, non-critical failures).
pub fn record_best_effort(db: DatabaseConnection, event: AuditEvent) {
    tokio::spawn(async move {
        let write = async {
            let (txn, actor) =
                db::begin_user_tenant_txn(&db, event.tenant_id, event.actor_id).await?;
            if event.actor_id.is_some() && actor.is_none() {
                tracing::warn!(action = %event.action, "audit actor has no active URDD");
            }
            build_model(&event).insert(&txn).await?;
            txn.commit().await
        };
        if let Err(e) = write.await {
            tracing::warn!(action = %event.action, "audit log write failed: {e}");
        }
    });
}
