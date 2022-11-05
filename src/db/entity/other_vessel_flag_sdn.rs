use crate::db::OfacRelEntity;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "other_vessel_flag_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub other_vessel_flag_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::other_vessel_flag::Entity",
        from = "Column::OtherVesselFlagId",
        to = "super::other_vessel_flag::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    OtherVesselFlag,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::FixedRef", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::other_vessel_flag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OtherVesselFlag.def()
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
pub struct SdnToOtherVesselFlag;

impl Linked for SdnToOtherVesselFlag {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::other_vessel_flag::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::Sdn.def().rev(),
            Entity::belongs_to(super::other_vessel_flag::Entity).from(Column::OtherVesselFlagId).to(super::other_vessel_flag::Column::Id).into(),
        ]
    }
}
