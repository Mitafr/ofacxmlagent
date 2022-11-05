use log::warn;
use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::document::models::feature::FeatureVersion;

use crate::db::{impl_parse_feature_version_value, impl_topmaj, OfacEntity};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "bic")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub bic: String,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::bic_sdn::Entity")]
    BicSdn,
}

impl Related<super::bic_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BicSdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl_topmaj! {
    Entity, Model, super::bic_sdn::ActiveModel, ActiveModel
}

impl_parse_feature_version_value! {
    Model, Model, bic, "Bic"
}

#[cfg(test)]
mod bic {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn parse_bic() {
        let feature: FeatureVersion = from_str(
            r#"
			  <FeatureVersion ID="33828" ReliabilityID="1">
				<Comment />
				<VersionDetail DetailTypeID="1432">HAVIGB2L</VersionDetail>
			  </FeatureVersion>"#,
        )
        .unwrap();
        let bic = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 33828,
            bic: "HAVIGB2L".to_owned(),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, bic);
    }
}
