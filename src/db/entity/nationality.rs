use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::document::{models::feature::Feature, models::location::Locations, OfacDocumentReferences};

use crate::db::{impl_topmaj, OfacEntity};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "nationality")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub nationality: Option<i32>,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ref_country::Entity",
        from = "Column::Nationality",
        to = "super::ref_country::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefCountry,
    #[sea_orm(has_many = "super::nationality_identity::Entity")]
    NationalityIdentity,
}

impl Related<super::ref_country::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RefCountry.def()
    }
}

impl Related<super::nationality_identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NationalityIdentity.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub struct NationalityEntity<'a>(pub &'a Feature, pub &'a Locations);

impl Model {
    pub fn from_ofac_document(entity: &NationalityEntity<'_>, references: &OfacDocumentReferences) -> Model {
        let mut nationality = Model { id: entity.0.id, ..Default::default() };
        let location_id = entity.0.version.location.unwrap().id;
        let location = entity.1.locations.iter().find(|&loc| loc.id == location_id).unwrap();
        if let Some(parts) = location.location_parts.as_ref() {
            for part in parts {
                for part_value in part.values.iter() {
                    if part_value.primary {
                        let value = part_value.value.to_owned();
                        match references.area_codes.iter().find(|&area| area.name == value) {
                            Some(area) => nationality.nationality = Some(area.id),
                            None => nationality.nationality = None,
                        }
                    }
                }
            }
        }
        nationality
    }
}

impl_topmaj! {
    Entity, Model, super::nationality_identity::ActiveModel, ActiveModel
}
