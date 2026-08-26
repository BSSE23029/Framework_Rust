use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tenant_subscriptions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub plan_id: Uuid,
    pub subscription_state: String,
    pub starts_at: DateTimeWithTimeZone,
    pub trial_ends_at: Option<DateTimeWithTimeZone>,
    pub ends_at: Option<DateTimeWithTimeZone>,
    pub auto_renew: bool,
    pub status: String,
    pub created_at: DateTimeWithTimeZone,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTimeWithTimeZone,
    pub updated_by: Option<Uuid>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub deleted_by: Option<Uuid>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
