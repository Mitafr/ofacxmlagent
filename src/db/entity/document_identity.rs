use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "document_identity")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub document_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub identity_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::document::Entity", from = "Column::DocumentId", to = "super::document::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    Document,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::IdentityId", to = "super::sdn::Column::Identity", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::document::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Document.def()
    }
}

impl Related<super::sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug)]
pub struct SdnToDocument;

impl Linked for SdnToDocument {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::document::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::Sdn.def().rev(),
            Entity::belongs_to(super::document::Entity).from(Column::DocumentId).to(super::document::Column::Id).into(),
        ]
    }
}
