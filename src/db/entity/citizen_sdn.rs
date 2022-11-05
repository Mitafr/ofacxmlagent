use sea_orm::entity::prelude::*;

use crate::db::OfacRelEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "citizen_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub citizen_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::citizen::Entity", from = "Column::CitizenId", to = "super::citizen::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    Citizen,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::FixedRef", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::citizen::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Citizen.def()
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
pub struct SdnToCitizen;

impl Linked for SdnToCitizen {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::citizen::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Sdn.def().rev(), Entity::belongs_to(super::citizen::Entity).from(Column::CitizenId).to(super::citizen::Column::Id).into()]
    }
}
