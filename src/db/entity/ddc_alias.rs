use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ddc_alias")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub quality: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::ddc_alias_sdn::Entity")]
    DdcAliasSdn,
}

impl Related<super::ddc_alias_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DdcAliasSdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
