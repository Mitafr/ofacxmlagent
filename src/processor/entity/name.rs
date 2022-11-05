use super::{extract_query_field, sdn::QuerySdnRecord};
use sea_orm::{FromQueryResult, QueryResult};

#[derive(FromQueryResult, Default, Debug)]
pub struct SdnAlias {
    pub fixed_ref: i32,
    partysubtype: i32,
    script_id: i32,
    last_name: Option<String>,
    first_name: Option<String>,
    middle_name: Option<String>,
    maiden_name: Option<String>,
    aircraft_name: Option<String>,
    entity_name: Option<String>,
    vessel_name: Option<String>,
    nickname: Option<String>,
    patronymic: Option<String>,
    matronymic: Option<String>,
    pub quality: String,
}

impl SdnAlias {
    /// Create a new SdnAlias from a QueryResult
    pub fn from_query_result(query_result: &QueryResult) -> SdnAlias {
        let mut alias = SdnAlias::default();
        extract_query_field(query_result, "fixed_ref", &mut alias.fixed_ref).unwrap();
        extract_query_field(query_result, "partysubtypeid", &mut alias.partysubtype).unwrap();
        extract_query_field(query_result, "last_name", &mut alias.last_name).unwrap();
        extract_query_field(query_result, "first_name", &mut alias.first_name).unwrap();
        extract_query_field(query_result, "middle_name", &mut alias.middle_name).unwrap();
        extract_query_field(query_result, "maiden_name", &mut alias.maiden_name).unwrap();
        extract_query_field(query_result, "aircraft_name", &mut alias.aircraft_name).unwrap();
        extract_query_field(query_result, "entity_name", &mut alias.entity_name).unwrap();
        extract_query_field(query_result, "vessel_name", &mut alias.vessel_name).unwrap();
        extract_query_field(query_result, "nickname", &mut alias.nickname).unwrap();
        extract_query_field(query_result, "patronymic", &mut alias.patronymic).unwrap();
        extract_query_field(query_result, "matronymic", &mut alias.matronymic).unwrap();
        extract_query_field(query_result, "quality", &mut alias.quality).unwrap();
        alias
    }
    /// Create a new SdnAlias from a QueryResult
    pub fn from_query_sdn_record(record: &QuerySdnRecord) -> SdnAlias {
        let alias = SdnAlias {
            fixed_ref: record.fixed_ref,
            partysubtype: record.partysubtypeid,
            script_id: record.name_script.unwrap(),
            last_name: record.name_last_name.to_owned(),
            first_name: record.name_first_name.to_owned(),
            middle_name: record.name_middle_name.to_owned(),
            maiden_name: record.name_maiden_name.to_owned(),
            aircraft_name: record.name_aircraft_name.to_owned(),
            entity_name: record.name_entity_name.to_owned(),
            vessel_name: record.name_vessel_name.to_owned(),
            nickname: record.name_nickname.to_owned(),
            patronymic: record.name_patronymic.to_owned(),
            matronymic: record.name_matronymic.to_owned(),
            quality: record.name_quality.as_ref().unwrap().to_owned(),
        };
        alias
    }

    /// Push a part_name to the provided alias if part_name is not empty
    fn push_to_alias(part_name: &Option<String>, alias: &mut String) {
        if let Some(name) = part_name {
            let formatted_name = name.replace(',', "");
            alias.push_str(&formatted_name[..]);
            alias.push(' ');
        }
    }

    /// Build the String alias with the current SdnAlias
    pub fn build_alias(&self) -> String {
        let mut alias = String::new();
        let mut last_offset = 0;
        match self.partysubtype {
            1 => alias = self.vessel_name.clone().unwrap_or_else(|| "".to_owned()),
            2 => alias = self.aircraft_name.clone().unwrap_or_else(|| "".to_owned()),
            3 => alias = self.entity_name.clone().unwrap_or_else(|| "".to_owned()),
            4 => match self.script_id {
                215 | 220 => {
                    Self::push_to_alias(&self.patronymic, &mut alias);
                    Self::push_to_alias(&self.matronymic, &mut alias);
                    Self::push_to_alias(&self.last_name, &mut alias);
                    if !alias.is_empty() {
                        alias.pop();
                        alias.push_str(", ");
                        last_offset = alias.len();
                    }
                    Self::push_to_alias(&self.first_name, &mut alias);
                    Self::push_to_alias(&self.middle_name, &mut alias);
                    Self::push_to_alias(&self.maiden_name, &mut alias);
                    Self::push_to_alias(&self.nickname, &mut alias);
                    if last_offset == alias.len() {
                        alias.pop();
                    }
                    alias.pop();
                }
                _ => {
                    Self::push_to_alias(&self.patronymic, &mut alias);
                    Self::push_to_alias(&self.matronymic, &mut alias);
                    Self::push_to_alias(&self.last_name, &mut alias);
                    Self::push_to_alias(&self.first_name, &mut alias);
                    Self::push_to_alias(&self.middle_name, &mut alias);
                    Self::push_to_alias(&self.maiden_name, &mut alias);
                    Self::push_to_alias(&self.nickname, &mut alias);
                    if !alias.is_empty() {
                        alias.pop();
                    }
                }
            },
            _ => {}
        }
        alias.to_uppercase()
    }
}

#[cfg(test)]
mod name {
    use super::SdnAlias;

    #[test]
    fn format_simple_name_individual() {
        let name = SdnAlias {
            fixed_ref: 10853,
            partysubtype: 4,
            script_id: 215,
            last_name: Some(String::from("DELOS REYES")),
            first_name: Some(String::from("Feliciano Semborio, Jr.")),
            ..Default::default()
        };
        assert_eq!("DELOS REYES, FELICIANO SEMBORIO JR.", name.build_alias());
    }

    #[test]
    fn format_patronymic_name_individual() {
        let name = SdnAlias {
            fixed_ref: 10853,
            partysubtype: 4,
            script_id: 215,
            last_name: Some(String::from("DELOS REYES")),
            first_name: Some(String::from("Feliciano Semborio, Jr.")),
            patronymic: Some(String::from("PREYES")),
            matronymic: Some(String::from("MREYES")),
            ..Default::default()
        };
        assert_eq!("PREYES MREYES DELOS REYES, FELICIANO SEMBORIO JR.", name.build_alias());
    }

    #[test]
    fn format_vessel_name() {
        let name = SdnAlias {
            fixed_ref: 10853,
            partysubtype: 1,
            script_id: 215,
            vessel_name: Some(String::from("VESSEL QDFZ' NAME HE")),
            ..Default::default()
        };
        assert_eq!("VESSEL QDFZ' NAME HE", name.build_alias());
    }

    #[test]
    fn format_aircraft_name() {
        let name = SdnAlias {
            fixed_ref: 10853,
            partysubtype: 2,
            script_id: 215,
            aircraft_name: Some(String::from("AIRCRAFT QDFZ' NAME")),
            ..Default::default()
        };
        assert_eq!("AIRCRAFT QDFZ' NAME", name.build_alias());
    }

    #[test]
    fn format_entity_name() {
        let name = SdnAlias {
            fixed_ref: 10853,
            partysubtype: 3,
            script_id: 215,
            entity_name: Some(String::from("ENTITY QDFZ' NAME HE")),
            ..Default::default()
        };
        assert_eq!("ENTITY QDFZ' NAME HE", name.build_alias());
    }

    #[test]
    fn format_complete_name() {
        let mut name = SdnAlias {
            fixed_ref: 10853,
            partysubtype: 4,
            script_id: 215,
            last_name: Some(String::from("LAST NAME")),
            first_name: Some(String::from("FIRST NAME")),
            middle_name: Some(String::from("MIDDLE NAME")),
            maiden_name: Some(String::from("MAIDEN NAME")),
            aircraft_name: Some(String::from("AIRCRAFT NAME")),
            entity_name: Some(String::from("ENTITY NAME")),
            vessel_name: Some(String::from("VESSEL NAME")),
            nickname: Some(String::from("NICKNAME")),
            patronymic: Some(String::from("PATRONYMIC")),
            matronymic: Some(String::from("MATRONYMIC")),
            quality: String::from("NORMAL"),
        };
        assert_eq!("PATRONYMIC MATRONYMIC LAST NAME, FIRST NAME MIDDLE NAME MAIDEN NAME NICKNAME", name.build_alias());
        name.partysubtype = 1;
        assert_eq!("VESSEL NAME", name.build_alias());
        name.partysubtype = 2;
        assert_eq!("AIRCRAFT NAME", name.build_alias());
        name.partysubtype = 3;
        assert_eq!("ENTITY NAME", name.build_alias());
    }
}
