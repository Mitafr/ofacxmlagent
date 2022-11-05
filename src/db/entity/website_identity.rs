pub use crate::db::OfacRelEntity;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "website_identity")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub website_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub identity_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::website::Entity", from = "Column::WebsiteId", to = "super::website::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    Website,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::IdentityId", to = "super::sdn::Column::Identity", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::website::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Website.def()
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
pub struct SdnToWebsite;

impl Linked for SdnToWebsite {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::website::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Sdn.def().rev(), Entity::belongs_to(super::website::Entity).from(Column::WebsiteId).to(super::website::Column::Id).into()]
    }
}
