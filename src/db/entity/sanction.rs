use sea_orm::{entity::prelude::*, DbErr};

use crate::document::{models::sanction::SanctionsEntry, OfacDocumentReferences};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub id: i32,
    pub date: Date,
    pub status: String,
    pub topmaj: String,
    pub programs: Vec<super::program::Model>,
    pub sdn_id: i32,
}

impl Model {
    pub fn from_ofac_document(entity: &SanctionsEntry, _references: &OfacDocumentReferences) -> Result<Model, DbErr> {
        let id = entity.id;
        let entry = entity.events.get(0).unwrap();
        let mut sanction = Model {
            id,
            status: "ACTIVE".to_owned(),
            topmaj: "N".to_owned(),
            sdn_id: entity.profile_id,
            date: entry.date.to_sql_date(),
            programs: Vec::new(),
        };
        for measure in entity.measures.iter() {
            if measure.program.is_none() {
                continue;
            }
            sanction.programs.push(super::program::Model::from_ofac_document(measure));
        }
        Ok(sanction)
    }
}
