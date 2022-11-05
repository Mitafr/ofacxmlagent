use log::warn;
use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::db::{impl_parse_feature_version_id, impl_topmaj, OfacEntity};
use crate::document::models::feature::FeatureVersion;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "target")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub target: Option<i32>,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ref_reference::Entity",
        from = "Column::Target",
        to = "super::ref_reference::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefReference,
    #[sea_orm(has_many = "super::target_sdn::Entity")]
    TargetSdn,
}

impl Related<super::ref_reference::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RefReference.def()
    }
}

impl Related<super::target_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TargetSdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl_topmaj! {
    Entity, Model, super::target_sdn::ActiveModel, ActiveModel
}

impl_parse_feature_version_id! {
    Model, Model, target, "Target"
}

#[cfg(test)]
mod target {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn parse_target() {
        let feature: FeatureVersion = from_str(
            r#"
			<FeatureVersion ID="47587" ReliabilityID="1">
				<Comment />
				<VersionDetail DetailTypeID="1431" DetailReferenceID="92062" />
			</FeatureVersion>"#,
        )
        .unwrap();
        let target = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 47587,
            target: Some(92062),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, target);
    }

    #[test]
    fn parse_target_empty_detail() {
        let feature: FeatureVersion = from_str(
            r#"
			<FeatureVersion ID="47587" ReliabilityID="1">
				<Comment />
			</FeatureVersion>"#,
        )
        .unwrap();
        let target = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 47587,
            target: None,
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, target);
    }
}
