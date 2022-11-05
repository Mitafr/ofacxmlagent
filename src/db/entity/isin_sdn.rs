use sea_orm::entity::prelude::*;

use crate::db::OfacRelEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "isin_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub isin_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::isin::Entity", from = "Column::IsinId", to = "super::isin::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    Isin,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::FixedRef", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::isin::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Isin.def()
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
pub struct SdnToIsin;

impl Linked for SdnToIsin {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::isin::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Sdn.def().rev(), Entity::belongs_to(super::isin::Entity).from(Column::IsinId).to(super::isin::Column::Id).into()]
    }
}
