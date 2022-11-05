use log::warn;
use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::db::{impl_parse_feature_version_value, impl_topmaj, OfacEntity};
use crate::document::models::feature::FeatureVersion;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "website")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub website: String,
    #[sea_orm(column_type = "Text")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::website_identity::Entity")]
    WebsiteIdentity,
}

impl Related<super::website_identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WebsiteIdentity.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl_topmaj! {
    Entity, Model, super::website_identity::ActiveModel, ActiveModel
}

impl_parse_feature_version_value! {
    Model, Model, website, "Website"
}

#[cfg(test)]
mod website {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn parse_website() {
        let feature: FeatureVersion = from_str(
            r#"
			<FeatureVersion ID="6855" ReliabilityID="1">
				<Comment />
				<VersionDetail DetailTypeID="1432">www.arrai.tv</VersionDetail>
			</FeatureVersion>"#,
        )
        .unwrap();
        let website = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 6855,
            website: "WWW.ARRAI.TV".to_owned(),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, website);
    }

    #[test]
    fn parse_website_empty_detail() {
        let feature: FeatureVersion = from_str(
            r#"
			  <FeatureVersion ID="6855" ReliabilityID="1">
				<Comment />
				<VersionDetail DetailTypeID="1432"></VersionDetail>
			  </FeatureVersion>"#,
        )
        .unwrap();
        let website = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 6855,
            website: "".to_owned(),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, website);
    }
}
