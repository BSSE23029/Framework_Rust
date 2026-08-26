use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "plans")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: String,
    pub billing_interval: String,
    pub price_cents: i64,
    pub currency: String,
    pub trial_days: i32,
    pub features: Json,
    pub is_public: bool,
    pub is_auto_renewable: bool,
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
