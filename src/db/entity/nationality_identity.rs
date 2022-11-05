use sea_orm::entity::prelude::*;

use crate::db::OfacRelEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "nationality_identity")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub nationality_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub identity_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::nationality::Entity",
        from = "Column::NationalityId",
        to = "super::nationality::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    Nationality,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::IdentityId", to = "super::sdn::Column::Identity", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::nationality::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nationality.def()
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
pub struct SdnToNationality;

impl Linked for SdnToNationality {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::nationality::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::Sdn.def().rev(),
            Entity::belongs_to(super::nationality::Entity).from(Column::NationalityId).to(super::nationality::Column::Id).into(),
        ]
    }
}
