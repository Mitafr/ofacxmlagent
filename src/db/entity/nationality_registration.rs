use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::document::{models::feature::Feature, models::location::Locations};

use crate::db::{impl_topmaj, OfacEntity};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "nationality_registration")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub location: Option<String>,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::nationality_registration_sdn::Entity")]
    NationalityRegistrationSdn,
}

impl Related<super::nationality_registration_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NationalityRegistrationSdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub struct NationalityRegistrationEntity<'a>(pub &'a Feature, pub &'a Locations);

impl Model {
    pub fn from_ofac_document(entity: &NationalityRegistrationEntity<'_>) -> Model {
        let mut nationality_registration = Model {
            id: entity.0.id,
            location: None,
            ..Default::default()
        };
        let location_id = entity.0.version.location.unwrap().id;
        let location = entity.1.locations.iter().find(|&loc| loc.id == location_id).unwrap();
        if let Some(parts) = location.location_parts.as_ref() {
            for part in parts {
                for part_value in part.values.iter() {
                    if part_value.primary {
                        nationality_registration.location = Some(part_value.value.to_owned());
                    }
                }
            }
        }
        nationality_registration
    }
}

impl_topmaj! {
    Entity, Model, super::nationality_registration_sdn::ActiveModel, ActiveModel
}
