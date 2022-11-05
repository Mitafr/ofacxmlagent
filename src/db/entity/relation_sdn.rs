use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "relation_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub relation_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::relation::Entity", from = "Column::RelationId", to = "super::relation::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    Relation,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::FixedRef", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::relation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Relation.def()
    }
}

impl Related<super::sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug)]
pub struct SdnToRelation;

impl Linked for SdnToRelation {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::relation::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::Sdn.def().rev(),
            Entity::belongs_to(super::relation::Entity).from(Column::RelationId).to(super::relation::Column::Id).into(),
        ]
    }
}
