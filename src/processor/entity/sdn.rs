use std::error::Error;
use std::fmt::Write;

use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;

use super::{extract_field, extract_field_as_vec, name::SdnAlias};

#[derive(FromQueryResult, Default, Debug, Clone)]
pub struct QuerySdnRecord {
    pub fixed_ref: i32,
    pub partysubtypeid: i32,
    pub last_update: Option<NaiveDate>,
    pub title: Option<String>,
    pub gender: Option<String>,
    pub comment: Option<String>,
    pub vessel_call_sign: Option<String>,
    pub other_vessel_call_sign: Option<String>,
    pub vessel_flag: Option<String>,
    pub vessel_owner: Option<String>,
    pub former_vessel_flag: Option<String>,
    pub other_vessel_flag: Option<String>,
    pub sanction_date: Option<NaiveDate>,
    pub msn: Option<String>,
    pub address_id: Option<i32>,
    pub address_address: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_city: Option<String>,
    pub address_state: Option<String>,
    pub address_country: Option<String>,
    pub address_region: Option<String>,
    pub address_is_primary: Option<bool>,
    pub dob_dob: Option<String>,
    pub pob_pob: Option<String>,
    pub citizen_location: Option<String>,
    pub website_website: Option<String>,
    pub bic_bic: Option<String>,
    pub sanction_program: Option<String>,
    pub aircraft_construction_number: Option<String>,
    pub aircraft_manufacture_date: Option<NaiveDate>,
    pub aircraft_transpondeur_code: Option<String>,
    pub aircraft_previous_tail_number: Option<String>,
    pub aircraft_tail_number: Option<String>,
    pub aircraft_model: Option<String>,
    pub nationality_nationality: Option<String>,
    pub ifca_determination: Option<String>,
    pub peesa_information: Option<String>,
    pub additional_sanctions_information: Option<String>,
    pub secondary_sanctions_risks: Option<String>,
    pub prohibited_transactions: Option<String>,
    pub organization_type: Option<String>,
    pub vessel_type: Option<String>,
    pub aircraft_operator: Option<String>,
    pub phone_number: Option<String>,
    pub organization_established_date: Option<NaiveDate>,
    pub document_registration_number: Option<String>,
    pub document_type: Option<i32>,
    pub document_type_value: Option<String>,
    pub document_issued_by: Option<String>,
    pub document_expiration_date: Option<Date>,
    pub document_issued_date: Option<Date>,
    pub document_id: Option<i32>,
    pub name_id: Option<i32>,
    pub name_name_type: Option<String>,
    pub name_script: Option<i32>,
    pub name_last_name: Option<String>,
    pub name_first_name: Option<String>,
    pub name_middle_name: Option<String>,
    pub name_maiden_name: Option<String>,
    pub name_aircraft_name: Option<String>,
    pub name_entity_name: Option<String>,
    pub name_vessel_name: Option<String>,
    pub name_nickname: Option<String>,
    pub name_patronymic: Option<String>,
    pub name_matronymic: Option<String>,
    pub name_quality: Option<String>,
    pub ddc_bic: Option<String>,
    pub ddc_alias_name: Option<String>,
    pub ddc_alias_quality: Option<String>,
    pub email_email: Option<String>,
    pub relation_linked_to: Option<i32>,
}

#[derive(Debug, Default, Clone)]
pub struct SdnRecord {
    pub fixed_ref: i32,
    pub partysubtypeid: i32,
    pub last_update: String,
    pub name: String,
    pub title: String,
    pub gender: String,
    pub ddc_bics: Vec<String>,
    pub ddc_low_aliases: Vec<String>,
    pub ddc_normal_aliases: Vec<String>,
    pub normal_aliases: Vec<String>,
    pub low_aliases: Vec<String>,
    pub programs: Vec<String>,
    pub bics: Vec<String>,
    pub addresses: Vec<SdnRecordAddress>,
    pub documents: Vec<SdnRecordDocument>,
    pub dobs: Vec<String>,
    pub pobs: Vec<String>,
    pub ddc_programs: Vec<String>,
    pub nationalities: Vec<String>,
    pub citizens: Vec<String>,
    pub websites: Vec<String>,
    pub msn: String,
    pub ifca_determination: Vec<String>,
    pub organization_established_date: Option<NaiveDate>,
    pub organization_type: String,
    pub additional_sanctions_information: Vec<String>,
    pub secondary_sanctions_risks: Vec<String>,
    pub prohibited_transactions: Vec<String>,
    pub peesa_information: Vec<String>,
    pub linked_to: Vec<i32>,
    pub comment: String,
    pub aircraft_construction_number: String,
    pub aircraft_manufacturer_serial_number: String,
    pub aircraft_manufacture_date: Option<NaiveDate>,
    pub aircraft_transpondeur_code: String,
    pub aircraft_previous_tail_number: String,
    pub aircraft_tail_number: String,
    pub aircraft_model: String,
    pub aircraft_operators: Vec<String>,
    pub emails: Vec<String>,
    pub phone_numbers: Vec<String>,
    pub vessel_call_sign: String,
    pub vessel_type: String,
    pub vessel_flag: String,
    pub vessel_owner: String,
    pub former_vessel_flag: Vec<String>,
    pub other_vessel_flag: Vec<String>,
    pub other_vessel_call_sign: String,
    pub linked_to_names: Vec<String>,
}

impl SdnRecord {
    pub fn from_query_sdn_record(query_record: &QuerySdnRecord, record: &mut SdnRecord) -> Result<(), Box<dyn Error>> {
        extract_field_as_vec(query_record.relation_linked_to, &mut record.linked_to)?;
        record.fixed_ref = query_record.fixed_ref;
        record.partysubtypeid = query_record.partysubtypeid;
        if let Some(last_update) = query_record.last_update {
            record.last_update = last_update.format("%Y/%m/%d").to_string();
        }
        if let Some(id) = query_record.document_id {
            let document = SdnRecordDocument {
                id,
                doc_type: query_record.document_type.unwrap(),
                doc_type_value: query_record.document_type_value.as_ref().unwrap().to_owned(),
                expiration_date: query_record.document_expiration_date,
                issued_date: query_record.document_issued_date,
                issued_by: query_record.document_issued_by.clone(),
                registration_number: query_record.document_registration_number.as_ref().unwrap().to_owned(),
            };
            if !record.documents.contains(&document) {
                record.documents.push(document);
            }
        }
        if let Some(id) = query_record.address_id {
            let mut address = SdnRecordAddress {
                is_primary: query_record.address_is_primary.unwrap(),
                ..Default::default()
            };
            address.id = id;
            address.city = query_record.address_city.clone();
            address.address = query_record.address_address.clone();
            address.postal_code = query_record.address_postal_code.clone();
            address.region = query_record.address_region.clone();
            address.country = query_record.address_country.clone();
            address.state = query_record.address_state.clone();
            if !record.addresses.contains(&address) {
                record.addresses.push(address);
            }
        }

        if query_record.name_id.is_some() {
            let name = SdnAlias::from_query_sdn_record(query_record);
            let mut alias_type = String::new();
            extract_field(query_record.name_name_type.clone(), &mut alias_type).unwrap();
            match &alias_type[..] {
                "NAME" => record.name = name.build_alias(),
                "ALIAS" => {
                    let alias = name.build_alias();
                    if name.quality == "LOW" && !record.low_aliases.contains(&alias) {
                        record.low_aliases.push(alias);
                    } else if name.quality == "NORMAL" && !record.normal_aliases.contains(&alias) {
                        record.normal_aliases.push(alias);
                    }
                }
                _ => {}
            }
        }
        if query_record.ddc_alias_name.is_some() {
            if query_record.ddc_alias_quality.as_ref().unwrap() == "Low" && !record.ddc_low_aliases.contains(query_record.ddc_alias_name.as_ref().unwrap()) {
                record.ddc_low_aliases.push(query_record.ddc_alias_name.as_ref().unwrap().to_owned());
            } else if query_record.ddc_alias_quality.as_ref().unwrap() == "Normal" && !record.ddc_normal_aliases.contains(query_record.ddc_alias_name.as_ref().unwrap()) {
                record.ddc_normal_aliases.push(query_record.ddc_alias_name.as_ref().unwrap().to_owned());
            }
        }
        extract_field_as_vec(query_record.ddc_bic.clone(), &mut record.ddc_bics)?;
        extract_field_as_vec(query_record.dob_dob.clone(), &mut record.dobs)?;
        extract_field_as_vec(query_record.sanction_program.clone(), &mut record.programs)?;
        extract_field_as_vec(query_record.bic_bic.clone(), &mut record.bics)?;
        extract_field_as_vec(query_record.pob_pob.clone(), &mut record.pobs)?;
        extract_field_as_vec(query_record.nationality_nationality.clone(), &mut record.nationalities)?;
        extract_field_as_vec(query_record.citizen_location.clone(), &mut record.citizens)?;
        extract_field_as_vec(query_record.website_website.clone(), &mut record.websites)?;
        extract_field_as_vec(query_record.ifca_determination.clone(), &mut record.ifca_determination)?;
        extract_field_as_vec(query_record.additional_sanctions_information.clone(), &mut record.additional_sanctions_information)?;
        extract_field_as_vec(query_record.peesa_information.clone(), &mut record.peesa_information)?;
        extract_field_as_vec(query_record.secondary_sanctions_risks.clone(), &mut record.secondary_sanctions_risks)?;
        extract_field_as_vec(query_record.prohibited_transactions.clone(), &mut record.prohibited_transactions)?;
        extract_field_as_vec(query_record.aircraft_operator.clone(), &mut record.aircraft_operators)?;
        extract_field_as_vec(query_record.email_email.clone(), &mut record.emails)?;
        extract_field_as_vec(query_record.phone_number.clone(), &mut record.phone_numbers)?;
        extract_field_as_vec(query_record.former_vessel_flag.clone(), &mut record.former_vessel_flag)?;
        extract_field_as_vec(query_record.other_vessel_flag.clone(), &mut record.other_vessel_flag)?;
        // TODO extract_field_as_vec(query_record.linked_to, &mut record.linked_to)?;
        if let Some(title) = &query_record.title {
            record.title = title.to_owned();
        };
        if let Some(comment) = &query_record.comment {
            record.comment = comment.to_owned();
        };
        if let Some(aircraft_construction_number) = &query_record.aircraft_construction_number {
            record.aircraft_construction_number = aircraft_construction_number.to_owned();
        };
        if let Some(aircraft_transpondeur_code) = &query_record.aircraft_transpondeur_code {
            record.aircraft_transpondeur_code = aircraft_transpondeur_code.to_owned();
        };
        if let Some(aircraft_previous_tail_number) = &query_record.aircraft_previous_tail_number {
            record.aircraft_previous_tail_number = aircraft_previous_tail_number.to_owned();
        };
        if let Some(aircraft_tail_number) = &query_record.aircraft_tail_number {
            record.aircraft_tail_number = aircraft_tail_number.to_owned();
        };
        if let Some(aircraft_model) = &query_record.aircraft_model {
            record.aircraft_model = aircraft_model.to_owned();
        };
        if let Some(gender) = &query_record.gender {
            record.gender = gender.to_owned();
        };
        if let Some(organization_type) = &query_record.organization_type {
            record.organization_type = organization_type.to_owned();
        };
        if let Some(vessel_call_sign) = &query_record.vessel_call_sign {
            record.vessel_call_sign = vessel_call_sign.to_owned();
        };
        if let Some(other_vessel_call_sign) = &query_record.other_vessel_call_sign {
            record.other_vessel_call_sign = other_vessel_call_sign.to_owned();
        };
        if let Some(vessel_type) = &query_record.vessel_type {
            record.vessel_type = vessel_type.to_owned();
        };
        if let Some(vessel_flag) = &query_record.vessel_flag {
            record.vessel_flag = vessel_flag.to_owned();
        };
        if let Some(vessel_owner) = &query_record.vessel_owner {
            record.vessel_owner = vessel_owner.to_owned();
        };
        if let Some(msn) = &query_record.msn {
            record.msn = msn.to_owned();
        };
        if let Some(aircraft_manufacture_date) = query_record.aircraft_manufacture_date {
            record.aircraft_manufacture_date = Some(aircraft_manufacture_date);
        }
        if let Some(organization_established_date) = query_record.organization_established_date {
            record.organization_established_date = Some(organization_established_date);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SdnRecordAddress {
    pub id: i32,
    pub country: Option<String>,
    pub address: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub region: Option<String>,
    pub is_primary: bool,
}

impl SdnRecordAddress {
    pub fn extract_inf(&self, inf: &mut String) {
        if !inf.is_empty() {
            inf.push_str(" / ");
        }
        if let Some(address) = &self.address {
            write!(inf, "{}", address).unwrap();
        }
        if let Some(postal_code) = &self.postal_code {
            write!(inf, " {}", postal_code).unwrap();
        }
        if let Some(city) = &self.city {
            write!(inf, " {}", city).unwrap();
        }
        if let Some(state) = &self.state {
            write!(inf, " {}", state).unwrap();
        }
        if let Some(region) = &self.region {
            write!(inf, " {}", region).unwrap();
        }
        if let Some(country) = &self.country {
            write!(inf, " {}", country).unwrap();
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SdnRecordDocument {
    pub id: i32,
    pub doc_type: i32,
    pub doc_type_value: String,
    pub expiration_date: Option<Date>,
    pub issued_date: Option<Date>,
    pub issued_by: Option<String>,
    pub registration_number: String,
}

impl SdnRecordDocument {
    pub fn extract_inf(&self, inf: &mut String) {
        if !inf.is_empty() {
            inf.push_str(" / ");
        }
        inf.push_str(&self.registration_number);
        if let Some(issued_by) = &self.issued_by {
            write!(inf, " ({})", issued_by).unwrap();
        }
        if let Some(issued_date) = &self.issued_date {
            write!(inf, " ISSUED {}", to_firco_date(issued_date)).unwrap();
        }
        if let Some(expiration_date) = &self.expiration_date {
            write!(inf, " EXPIRES {}", to_firco_date(expiration_date)).unwrap();
        }
    }
    pub fn extract_inf_with_doc_name(&self, inf: &mut String) {
        if !inf.is_empty() {
            inf.push_str(" / ");
        }
        inf.push_str(&self.doc_type_value.to_uppercase());
        inf.push(' ');
        inf.push_str(&self.registration_number);
        if let Some(issued_by) = &self.issued_by {
            write!(inf, " ({})", issued_by).unwrap();
        }
        if let Some(issued_date) = &self.issued_date {
            write!(inf, " ISSUED {}", to_firco_date(issued_date)).unwrap();
        }
        if let Some(expiration_date) = &self.expiration_date {
            write!(inf, " EXPIRES {}", to_firco_date(expiration_date)).unwrap();
        }
    }
}

fn to_firco_date(date: &NaiveDate) -> String {
    date.format("%d %b %Y").to_string().to_uppercase()
}

#[cfg(test)]
mod sdn_document {
    use super::*;

    fn init_one_document() -> SdnRecordDocument {
        SdnRecordDocument {
            id: 1,
            doc_type: 1570,
            doc_type_value: String::from("Cedula No."),
            expiration_date: Some(NaiveDate::from_ymd(1997, 10, 26)),
            issued_date: Some(NaiveDate::from_ymd(1996, 10, 26)),
            issued_by: Some(String::from("France")),
            registration_number: String::from("12345"),
        }
    }

    fn init_multiple_documents() -> Vec<SdnRecordDocument> {
        let mut documents = Vec::new();
        for i in 0..5 {
            documents.push(SdnRecordDocument {
                id: i,
                doc_type: if i % 2 == 0 { 1570 } else { 1571 },
                doc_type_value: if i % 2 == 0 { String::from("Cedula No.") } else { String::from("Passport") },
                expiration_date: Some(NaiveDate::from_ymd(1997 + i, 10, 26)),
                issued_date: Some(NaiveDate::from_ymd(1996 + i, 10, 26)),
                issued_by: if i % 2 == 0 {
                    Some(String::from("France"))
                } else if i == 0 {
                    Some(String::from("Italia"))
                } else {
                    None
                },
                registration_number: String::from(((i + 1) * 12345).to_string()),
            });
        }
        documents
    }

    #[test]
    fn extract_inf_from_one_document() {
        let mut inf = String::new();
        let expected = String::from("12345 (France) ISSUED 26 OCT 1996 EXPIRES 26 OCT 1997");

        init_one_document().extract_inf(&mut inf);

        assert_eq!(expected, inf);
    }

    #[test]
    fn extract_inf_from_one_document_with_doc_name() {
        let mut inf = String::new();
        let expected = String::from("CEDULA NO. 12345 (France) ISSUED 26 OCT 1996 EXPIRES 26 OCT 1997");

        init_one_document().extract_inf_with_doc_name(&mut inf);

        assert_eq!(expected, inf);
    }

    #[test]
    fn extract_inf_from_multiple_document() {
        let mut inf = String::new();
        let expected = String::from("12345 (France) ISSUED 26 OCT 1996 EXPIRES 26 OCT 1997 / 24690 ISSUED 26 OCT 1997 EXPIRES 26 OCT 1998 / 37035 (France) ISSUED 26 OCT 1998 EXPIRES 26 OCT 1999 / 49380 ISSUED 26 OCT 1999 EXPIRES 26 OCT 2000 / 61725 (France) ISSUED 26 OCT 2000 EXPIRES 26 OCT 2001");

        for d in init_multiple_documents() {
            d.extract_inf(&mut inf);
        }

        assert_eq!(expected, inf);
    }

    #[test]
    fn extract_inf_from_multiple_document_with_doc_name() {
        let mut inf = String::new();
        let expected = String::from("CEDULA NO. 12345 (France) ISSUED 26 OCT 1996 EXPIRES 26 OCT 1997 / PASSPORT 24690 ISSUED 26 OCT 1997 EXPIRES 26 OCT 1998 / CEDULA NO. 37035 (France) ISSUED 26 OCT 1998 EXPIRES 26 OCT 1999 / PASSPORT 49380 ISSUED 26 OCT 1999 EXPIRES 26 OCT 2000 / CEDULA NO. 61725 (France) ISSUED 26 OCT 2000 EXPIRES 26 OCT 2001");

        for d in init_multiple_documents() {
            d.extract_inf_with_doc_name(&mut inf);
        }

        assert_eq!(expected, inf);
    }
}
