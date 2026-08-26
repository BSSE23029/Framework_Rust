use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "login_attempts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    #[sea_orm(column_type = "Text")]
    pub email_lookup: String,
    pub attempted_at: DateTimeWithTimeZone,
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
