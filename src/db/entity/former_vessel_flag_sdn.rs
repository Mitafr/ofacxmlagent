use sea_orm::entity::prelude::*;

use crate::db::OfacRelEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "former_vessel_flag_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub former_vessel_flag_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::former_vessel_flag::Entity",
        from = "Column::FormerVesselFlagId",
        to = "super::former_vessel_flag::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    FormerVesselFlag,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::FixedRef", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::former_vessel_flag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FormerVesselFlag.def()
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
pub struct SdnToFormerVesselFlag;

impl Linked for SdnToFormerVesselFlag {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::former_vessel_flag::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::Sdn.def().rev(),
            Entity::belongs_to(super::former_vessel_flag::Entity)
                .from(Column::FormerVesselFlagId)
                .to(super::former_vessel_flag::Column::Id)
                .into(),
        ]
    }
}
