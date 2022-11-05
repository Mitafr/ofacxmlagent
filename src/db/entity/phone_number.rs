use log::warn;
use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::db::{impl_parse_feature_version_value, impl_topmaj, OfacEntity};
use crate::document::models::feature::FeatureVersion;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "phone_number")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub phone_number: String,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::phone_number_sdn::Entity")]
    PhoneNumberSdn,
}

impl Related<super::phone_number_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PhoneNumberSdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl_topmaj! {
    Entity, Model, super::phone_number_sdn::ActiveModel, ActiveModel
}

impl_parse_feature_version_value! {
    Model, Model, phone_number, "Phone Number"
}
