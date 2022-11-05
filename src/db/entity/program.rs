use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::db::{impl_topmaj, OfacEntity};
use crate::document::models::sanction::SanctionsMeasure;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "program")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub program: String,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::sdn_program::Entity")]
    SanctionProgram,
}

impl Related<super::sdn_program::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SanctionProgram.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from_ofac_document(entity: &SanctionsMeasure) -> Model {
        let mut program = Model {
            id: entity.id,
            program: String::new(),
            topmaj: "N".to_owned(),
        };
        program.program = entity.program.as_ref().unwrap().to_uppercase();
        program
    }
}

impl_topmaj! {
    Entity, Model, super::sdn_program::ActiveModel, ActiveModel
}

#[cfg(test)]
mod program {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn parse_program() {
        let sanction: SanctionsMeasure = from_str(
            r#"
			<SanctionsMeasure ID="126053" SanctionsTypeID="1">
				<Comment>CUBA</Comment>
				<DatePeriod CalendarTypeID="1" YearFixed="true" MonthFixed="true" DayFixed="true" />
			</SanctionsMeasure>"#,
        )
        .unwrap();
        let program = Model::from_ofac_document(&sanction);
        let excepted = Model {
            id: 126053,
            program: "CUBA".to_owned(),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, program);
    }
}
