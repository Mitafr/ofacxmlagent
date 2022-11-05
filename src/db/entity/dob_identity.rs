use sea_orm::entity::prelude::*;

use crate::db::OfacRelEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "dob_identity")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub dob_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub identity_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::dob::Entity", from = "Column::DobId", to = "super::dob::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    Dob,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::IdentityId", to = "super::sdn::Column::Identity", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::dob::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Dob.def()
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
pub struct SdnToDob;

impl Linked for SdnToDob {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::dob::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Sdn.def().rev(), Entity::belongs_to(super::dob::Entity).from(Column::DobId).to(super::dob::Column::Id).into()]
    }
}
