use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::document::models::distinctparty::{Identity, MasterNamePartGroup, NamePartGroups};

use crate::db::{impl_topmaj, OfacEntity};
use crate::document::models::referencevaluesets::ScriptValues;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "name")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text", column_name = "type")]
    pub name_type: String,
    pub script: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub last_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub first_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub middle_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub maiden_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub aircraft_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub entity_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub vessel_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub nickname: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub patronymic: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub matronymic: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub quality: Option<String>,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
    #[sea_orm(ignore)]
    pub is_primary_215: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::name_sdn::Entity")]
    NameSdn,
}

impl Related<super::name_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NameSdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from_ofac_document(entity: &Identity, name_part_groups: &NamePartGroups, script_values: &ScriptValues) -> Vec<Model> {
        let mut name = Vec::new();
        let mut full_name = String::new();
        for sdn_name in entity.alias.iter() {
            let is_primary = sdn_name.primary;
            let quality = if sdn_name.quality { "Low".to_string() } else { "Normal".to_string() };
            for documented_name in sdn_name.documented_name.iter() {
                let mut model_name = Model {
                    id: documented_name.id,
                    name_type: "ALIAS".to_owned(),
                    quality: Some(quality.to_uppercase()),
                    ..Default::default()
                };
                let name_status = documented_name.doc_name_status;
                full_name.clear();
                for document_part in documented_name.parts.iter() {
                    let name_part_group_id = document_part.name_part.name_part_group_id;
                    let name_part_type_id = name_part_groups.master_name_part_group.iter().find(|n| n.name_part_group.id == name_part_group_id).unwrap();
                    let name = document_part.name_part.name.clone();
                    model_name.split_name(script_values, name_part_type_id, name, document_part.name_part.script_id);
                    model_name.script = document_part.name_part.script_id;
                    if is_primary && document_part.name_part.script_id == 215 && name_status == 1 {
                        model_name.is_primary_215 = true;
                        model_name.name_type = "NAME".to_owned();
                    }
                }
                name.push(model_name);
            }
        }
        name
    }

    fn split_name(&mut self, script_values: &ScriptValues, name_part_type_id: &MasterNamePartGroup, name: Option<String>, script_id: i32) {
        let mut is_arabic = false;
        if let Some(arabic_script) = script_values.scripts.iter().find(|v| v.code == "Arab") {
            is_arabic = arabic_script.id == script_id;
        }
        match name_part_type_id.name_part_group.name_part_type_id {
            1520 => {
                if !is_arabic {
                    self.last_name = name
                } else {
                    self.first_name = name
                }
            }
            1521 => {
                if !is_arabic {
                    self.first_name = name
                } else {
                    self.last_name = name
                }
            }
            1522 => self.middle_name = name,
            1523 => self.maiden_name = name,
            1524 => self.aircraft_name = name,
            1525 => self.entity_name = name,
            1526 => self.vessel_name = name,
            1528 => self.nickname = name,
            91708 => self.patronymic = name,
            91709 => self.matronymic = name,
            _ => {}
        }
    }
}

impl_topmaj! {
    Entity, Model, super::name_sdn::ActiveModel, ActiveModel
}
