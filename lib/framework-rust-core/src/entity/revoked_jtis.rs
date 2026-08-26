use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "revoked_jtis")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub jti: String,
    pub expires_at: DateTimeWithTimeZone,
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
