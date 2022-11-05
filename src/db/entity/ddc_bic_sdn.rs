use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ddc_bic_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ddc_bic_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::ddc_bic::Entity", from = "Column::DdcBicId", to = "super::ddc_bic::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    DdcBic,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::RecordId", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::ddc_bic::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DdcBic.def()
    }
}

impl Related<super::sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug)]
pub struct SdnToDdcBic;

impl Linked for SdnToDdcBic {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::ddc_bic::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Sdn.def().rev(), Entity::belongs_to(super::ddc_bic::Entity).from(Column::DdcBicId).to(super::ddc_bic::Column::Id).into()]
    }
}
