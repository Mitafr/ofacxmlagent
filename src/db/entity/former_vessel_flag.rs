use log::warn;
use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::document::models::feature::FeatureVersion;

use crate::db::{impl_parse_feature_version_value, impl_topmaj, OfacEntity};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "former_vessel_flag")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub value: String,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::former_vessel_flag_sdn::Entity")]
    FormerVesselFlagSdn,
}

impl Related<super::former_vessel_flag_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FormerVesselFlagSdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl_topmaj! {
    Entity, Model, super::former_vessel_flag_sdn::ActiveModel, ActiveModel
}

impl_parse_feature_version_value! {
    Model, Model, value, "Former Vessel Flag"
}
