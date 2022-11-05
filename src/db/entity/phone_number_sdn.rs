use sea_orm::entity::prelude::*;

use crate::db::OfacRelEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "phone_number_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub phone_number_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::phone_number::Entity",
        from = "Column::PhoneNumberId",
        to = "super::phone_number::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    PhoneNumber,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::FixedRef", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::phone_number::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PhoneNumber.def()
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
pub struct SdnToPhoneNumber;

impl Linked for SdnToPhoneNumber {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::phone_number::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::Sdn.def().rev(),
            Entity::belongs_to(super::phone_number::Entity).from(Column::PhoneNumberId).to(super::phone_number::Column::Id).into(),
        ]
    }
}
