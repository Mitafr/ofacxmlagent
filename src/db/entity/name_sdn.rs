use sea_orm::entity::prelude::*;

use crate::db::OfacRelEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "name_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub name_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::name::Entity", from = "Column::NameId", to = "super::name::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    Name,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::FixedRef", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::name::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Name.def()
    }
}

impl Related<super::sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl OfacRelEntity for ActiveModel {}

#[derive(Debug)]
pub struct SdnToName;

impl Linked for SdnToName {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::name::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Sdn.def().rev(), Entity::belongs_to(super::name::Entity).from(Column::NameId).to(super::name::Column::Id).into()]
    }
}
