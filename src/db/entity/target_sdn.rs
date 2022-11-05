pub use crate::db::OfacRelEntity;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "target_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub target_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::target::Entity", from = "Column::TargetId", to = "super::target::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    Target,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::FixedRef", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::target::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Target.def()
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
pub struct SdnToTarget;

impl Linked for SdnToTarget {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::target::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Sdn.def().rev(), Entity::belongs_to(super::target::Entity).from(Column::TargetId).to(super::target::Column::Id).into()]
    }
}
