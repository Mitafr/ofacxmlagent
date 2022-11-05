use log::warn;
use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::document::models::feature::FeatureVersion;

use crate::db::{impl_parse_feature_version_id, impl_topmaj, OfacEntity};

#[derive(Clone, Debug, Default, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "caatsa235")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub reference_id: Option<i32>,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ref_reference::Entity",
        from = "Column::ReferenceId",
        to = "super::ref_reference::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefReference,
    #[sea_orm(has_many = "super::caatsa235_sdn::Entity")]
    Caatsa235Sdn,
}

impl Related<super::ref_reference::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RefReference.def()
    }
}

impl Related<super::caatsa235_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Caatsa235Sdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl_topmaj! {
    Entity, Model, super::caatsa235_sdn::ActiveModel, ActiveModel
}

impl_parse_feature_version_id! {
    Model, Model, reference_id, "CAATSA235"
}
