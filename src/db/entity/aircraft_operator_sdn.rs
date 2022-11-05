use sea_orm::entity::prelude::*;

use crate::db::OfacRelEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "aircraft_operator_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub aircraft_operator_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::aircraft_operator::Entity",
        from = "Column::AircraftOperatorId",
        to = "super::aircraft_operator::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    AircraftOperator,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::FixedRef", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sdn.def()
    }
}

impl Related<super::aircraft_operator::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AircraftOperator.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl OfacRelEntity for ActiveModel {}

#[derive(Debug)]
pub struct SdnToAircraftOperator;

impl Linked for SdnToAircraftOperator {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::aircraft_operator::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::Sdn.def().rev(),
            Entity::belongs_to(super::aircraft_operator::Entity)
                .from(Column::AircraftOperatorId)
                .to(super::aircraft_operator::Column::Id)
                .into(),
        ]
    }
}
