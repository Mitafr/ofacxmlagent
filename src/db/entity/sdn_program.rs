pub use crate::db::OfacRelEntity;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sdn_program")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub program_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::program::Entity", from = "Column::ProgramId", to = "super::program::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    Program,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::FixedRef", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::program::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Program.def()
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
pub struct SdnToProgram;

impl Linked for SdnToProgram {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::program::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Sdn.def().rev(), Entity::belongs_to(super::program::Entity).from(Column::ProgramId).to(super::program::Column::Id).into()]
    }
}
