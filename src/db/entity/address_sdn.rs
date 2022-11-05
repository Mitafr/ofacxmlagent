use sea_orm::entity::prelude::*;

use crate::db::OfacRelEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "address_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub address_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub identity_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::address::Entity", from = "Column::AddressId", to = "super::address::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    Address,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::IdentityId", to = "super::sdn::Column::Identity", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::address::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Address.def()
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
pub struct SdnToAddress;

impl Linked for SdnToAddress {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::address::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Sdn.def().rev(), Entity::belongs_to(super::address::Entity).from(Column::AddressId).to(super::address::Column::Id).into()]
    }
}
