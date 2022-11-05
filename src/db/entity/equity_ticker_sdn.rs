use sea_orm::entity::prelude::*;

use crate::db::OfacRelEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "equity_ticker_sdn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub equity_ticker_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sdn_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::equity_ticker::Entity",
        from = "Column::EquityTickerId",
        to = "super::equity_ticker::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    EquityTicker,
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::SdnId", to = "super::sdn::Column::FixedRef", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
}

impl Related<super::equity_ticker::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EquityTicker.def()
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
pub struct SdnToEquityTicker;

impl Linked for SdnToEquityTicker {
    type FromEntity = super::sdn::Entity;

    type ToEntity = super::equity_ticker::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::Sdn.def().rev(),
            Entity::belongs_to(super::equity_ticker::Entity).from(Column::EquityTickerId).to(super::equity_ticker::Column::Id).into(),
        ]
    }
}
