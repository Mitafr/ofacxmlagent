use sea_orm::entity::prelude::*;

use crate::db::OfacRelEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "caatsa235_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub caatsa235_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::caatsa235::Entity", from = "Column::Caatsa235Id", to = "super::caatsa235::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    Caatsa235,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::FixedRef", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::caatsa235::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Caatsa235.def()
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
pub struct SdnToCaatsa235;

impl Linked for SdnToCaatsa235 {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::caatsa235::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::Sdn.def().rev(),
            Entity::belongs_to(super::caatsa235::Entity).from(Column::Caatsa235Id).to(super::caatsa235::Column::Id).into(),
        ]
    }
}
