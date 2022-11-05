use log::warn;
use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::db::{impl_parse_feature_version_value, impl_topmaj, OfacEntity};
use crate::document::models::feature::FeatureVersion;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "pob")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub pob: String,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::pob_identity::Entity")]
    PobIdentity,
}

impl Related<super::pob_identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PobIdentity.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl_topmaj! {
    Entity, Model, super::pob_identity::ActiveModel, ActiveModel
}

impl_parse_feature_version_value! {
    Model, Model, pob, "Pob"
}

#[cfg(test)]
mod pob {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn parse_pob() {
        let feature: FeatureVersion = from_str(
            r#"
			<FeatureVersion ID="2995" ReliabilityID="1">
			  <Comment />
			  <VersionDetail DetailTypeID="1432">Culiacan, Sinaloa, Mexico</VersionDetail>
			</FeatureVersion>"#,
        )
        .unwrap();
        let pob = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 2995,
            pob: "CULIACAN, SINALOA, MEXICO".to_owned(),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, pob);
    }
}
