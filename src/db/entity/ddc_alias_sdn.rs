use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ddc_alias_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ddc_alias_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::ddc_alias::Entity", from = "Column::DdcAliasId", to = "super::ddc_alias::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    DdcAlias,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::RecordId", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::ddc_alias::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DdcAlias.def()
    }
}

impl Related<super::sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug)]
pub struct SdnToDdcAlias;

impl Linked for SdnToDdcAlias {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::ddc_alias::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::Sdn.def().rev(),
            Entity::belongs_to(super::ddc_alias::Entity).from(Column::DdcAliasId).to(super::ddc_alias::Column::Id).into(),
        ]
    }
}
