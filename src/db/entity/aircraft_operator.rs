use log::warn;
use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::db::{impl_parse_feature_version_value, impl_topmaj, OfacEntity};
use crate::document::models::feature::FeatureVersion;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "aircraft_operator")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub operator: String,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::aircraft_operator_sdn::Entity")]
    AircraftOperatorSdn,
}

impl Related<super::aircraft_operator_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AircraftOperatorSdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl_topmaj! {
    Entity, Model, super::aircraft_operator_sdn::ActiveModel, ActiveModel
}

impl_parse_feature_version_value! {
    Model, Model, operator, "Operator"
}
