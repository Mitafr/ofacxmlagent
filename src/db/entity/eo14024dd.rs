use log::warn;
use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::document::models::feature::FeatureVersion;

use crate::db::{impl_parse_feature_version_id, impl_topmaj, OfacEntity};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "eo14024dd")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
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
    #[sea_orm(has_many = "super::eo14024dd_sdn::Entity")]
    Eo14024ddSdn,
}

impl Related<super::ref_reference::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RefReference.def()
    }
}

impl Related<super::eo14024dd_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Eo14024ddSdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl_topmaj! {
    Entity, Model, super::eo14024dd_sdn::ActiveModel, ActiveModel
}

impl_parse_feature_version_id! {
    Model, Model, reference_id, "EO 14024 DD"
}
