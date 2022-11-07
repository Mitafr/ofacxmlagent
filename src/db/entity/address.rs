use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::{
    db::impl_topmaj,
    document::{
        models::{feature::Feature, location::Locations},
        OfacDocumentReferences,
    },
};

use crate::db::OfacEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "address")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub address: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub city: Option<String>,
    pub country: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub postal_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub region: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub state: Option<String>,
    pub is_primary: bool,
    #[sea_orm(column_type = "Text")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::ref_country::Entity", from = "Column::Country", to = "super::ref_country::Column::Id", on_update = "Restrict", on_delete = "Restrict")]
    RefCountry,
    #[sea_orm(has_many = "super::address_sdn::Entity")]
    AddressSdn,
}

impl Related<super::ref_country::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RefCountry.def()
    }
}

impl Related<super::address_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AddressSdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from_ofac_document(entity: (&Feature, &Locations), references: &OfacDocumentReferences, is_primary: bool) -> Model {
        let mut address = Model {
            id: entity.0.version.location.unwrap().id,
            is_primary,
            topmaj: "N".to_owned(),
            ..Default::default()
        };
        let location = entity.1.locations.iter().find(|&loc| loc.id == address.id).unwrap();
        if let Some(country) = location.location_country.as_ref() {
            match references.area_codes.iter().find(|&area| area.id == country.id) {
                Some(area) => address.country = Some(area.id),
                None => address.country = None,
            }
        }
        if let Some(parts) = location.location_parts.as_ref() {
            for part in parts {
                let mut value = None;
                for part_value in part.values.iter() {
                    if part_value.primary {
                        value = Some(part_value.value.to_owned());
                    }
                }
                if let Some(mut value) = value {
                    value = value.to_uppercase();
                    match part.id {
                        1450 => address.region = Some(value),
                        1451 => address.address = Some(value),
                        1452 => {
                            if address.address.is_some() {
                                address.address.as_mut().unwrap().push(',');
                                address.address.as_mut().unwrap().push(' ');
                                address.address.as_mut().unwrap().push_str(&value);
                            }
                        }
                        1453 => {
                            if address.address.is_some() {
                                address.address.as_mut().unwrap().push(',');
                                address.address.as_mut().unwrap().push(' ');
                                address.address.as_mut().unwrap().push_str(&value);
                            }
                        }
                        1454 => address.city = Some(value),
                        1455 => address.state = Some(value),
                        1456 => address.postal_code = Some(value),
                        _ => {}
                    }
                }
            }
        }
        address
    }
}

impl_topmaj! {
    Entity, Model, super::address_sdn::ActiveModel, ActiveModel
}
