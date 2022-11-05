use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::document::models::{feature::Feature, location::Locations};

use crate::db::{impl_topmaj, OfacEntity};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "citizen")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub location: Option<String>,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::citizen_sdn::Entity")]
    CitizenSdn,
}

impl Related<super::citizen_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CitizenSdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from_ofac_document(entity: (&Feature, &Locations)) -> Model {
        let mut citizen = Model { id: entity.0.id, ..Default::default() };
        let location_id = entity.0.version.location.unwrap().id;
        let location = entity.1.locations.iter().find(|&loc| loc.id == location_id).unwrap();
        if let Some(parts) = location.location_parts.as_ref() {
            for part in parts {
                for part_value in part.values.iter() {
                    if part_value.primary {
                        citizen.location = Some(part_value.value.to_uppercase());
                    }
                }
            }
        }
        citizen
    }
}

impl_topmaj! {
    Entity, Model, super::citizen_sdn::ActiveModel, ActiveModel
}
