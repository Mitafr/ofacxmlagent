use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use std::{error::Error, fmt::Display};

use super::DocumentType;
use crate::db::entity::ddc_name::Model as DdcName;
use crate::processor::entity::sdn::{SdnRecord, SdnRecordAddress, SdnRecordDocument};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum RecordType {
    Main,
    Alternative,
}

impl Default for RecordType {
    fn default() -> Self {
        Self::Main
    }
}

#[derive(Default, Debug, Eq, PartialEq)]
pub struct FofnasyRecord {
    pub doc_type: DocumentType,
    pub t_id: i32,
    pub t_alias: String,
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct AddInfoTemplate {
    title: String,
    separator: char,
    condition: bool,
    space_between: bool,
}

impl Default for AddInfoTemplate {
    fn default() -> Self {
        Self {
            title: "".to_owned(),
            separator: '/',
            condition: true,
            space_between: false,
        }
    }
}

lazy_static! {
    static ref RE_REPLACE_DOCUMENTS: Regex = Regex::new(r"(?m)[()\-/,\.\s@#]").unwrap();
    static ref SEPARATOR: &'static str = "\t";
}

type AddInfoFields<'a> = Vec<(u32, AddInfoTemplate, &'a [String])>;

type AddInfoDocuments = (String, String, String);

#[derive(Default, Debug, PartialEq, Eq)]
pub struct FofdbofRecord {
    t_add: Option<String>,
    t_bad: char,
    t_bic: Option<String>,
    t_cit: Option<String>,
    t_ctr: Option<String>,
    t_dob: String,
    t_dob_overflow: bool,
    t_dsg: String,
    t_gdr: String,
    t_inf: String,
    t_name: String,
    t_nid: Option<String>,
    t_ntl: String,
    t_oid: String,
    t_ori: String,
    t_pob: String,
    t_pob_overflow: bool,
    t_psp: Option<String>,
    t_ref: String,
    t_shk: String,
    t_sta: Option<String>,
    t_syc: String,
    t_syk: String,
    t_syn: String,
    t_sys: String,
    t_typ: char,
    t_us1: String,
    t_us2: Option<String>,
    record_type: RecordType,
    doc_type: DocumentType,
}

impl FofdbofRecord {
    pub fn from_db_record(db_record: &SdnRecord, doc_type: &DocumentType, other_names: &[DdcName]) -> Vec<FofdbofRecord> {
        let mut records = Vec::new();
        if db_record.addresses.is_empty() {
            let fake_address = SdnRecordAddress { is_primary: true, ..Default::default() };
            FofdbofRecord::construct_record(db_record, &fake_address, doc_type, &mut records, &mut 0, other_names);
        }
        let mut main_fixed_ref = db_record.fixed_ref;
        for address in db_record.addresses.iter() {
            FofdbofRecord::construct_record(db_record, address, doc_type, &mut records, &mut main_fixed_ref, other_names);
        }
        records
    }

    fn construct_record(db_record: &SdnRecord, address: &SdnRecordAddress, doc_type: &DocumentType, records: &mut Vec<FofdbofRecord>, main_fixed_ref: &mut i32, other_names: &[DdcName]) {
        let mut record = FofdbofRecord {
            t_add: address.address.clone(),
            t_cit: address.city.clone(),
            t_sta: address.state.clone(),
            t_bad: '0',
            t_dsg: if doc_type == &DocumentType::OFAC { "OFAC".to_owned() } else { "OFAC-NS".to_owned() },
            t_typ: 'V',
            t_syc: String::from(""),
            t_syk: String::from(""),
            t_sys: String::from(""),
            t_ori: if doc_type == &DocumentType::OFAC { "OFAC".to_owned() } else { "OFAC-NS".to_owned() },
            t_name: db_record.name.to_uppercase(),
            record_type: if address.is_primary { RecordType::Main } else { RecordType::Alternative },
            doc_type: *doc_type,
            ..Default::default()
        };
        if record.record_type == RecordType::Alternative {
            record.t_us2 = Some(FofdbofRecord::compute_oid_alternative(*main_fixed_ref));
        }
        if record.record_type == RecordType::Main {
            *main_fixed_ref = db_record.fixed_ref;
        }
        record.compute_oid(*main_fixed_ref, address.id);
        record.compute_typ(db_record.partysubtypeid, &db_record.programs, other_names);
        let mut all_aliases = db_record.normal_aliases.clone();
        all_aliases.append(&mut db_record.low_aliases.clone());
        all_aliases.append(&mut db_record.ddc_low_aliases.clone());
        all_aliases.append(&mut db_record.ddc_normal_aliases.clone());
        record.compute_syn(!all_aliases.is_empty(), db_record.fixed_ref);
        match record.t_typ {
            'A' | 'P' => {}
            'C' => record.compute_shk(db_record.partysubtypeid, &db_record.documents),
            _ => {}
        }
        record.compute_ref(&db_record.last_update);
        record.compute_bic(&db_record.bics, &db_record.ddc_bics);
        record.compute_psp(&db_record.documents).unwrap();
        record.compute_nid(&db_record.msn, db_record.partysubtypeid, &db_record.documents).unwrap();
        record.compute_dob(&db_record.dobs);
        record.compute_pob(&db_record.pobs);
        record.compute_us1(&db_record.ddc_programs, &db_record.programs);
        record.compute_inf(db_record);
        record.compute_ntl(&db_record.nationalities);
        record.compute_add(address);
        record.compute_ctr(address);
        record.compute_gdr(&db_record.gender);
        records.push(record);
    }

    fn compute_gdr(&mut self, gender: &str) {
        match gender {
            "MALE" => self.t_gdr = "M".to_owned(),
            "FEMALE" => self.t_gdr = "F".to_owned(),
            _ => self.t_gdr = "U".to_owned(),
        }
    }

    fn compute_add(&mut self, address: &SdnRecordAddress) {
        let mut add = String::new();
        if let Some(a) = &address.address {
            add.push_str(a);
        }
        if let Some(p) = &address.postal_code {
            if !add.is_empty() {
                add.push_str(", ");
            }
            add.push_str(p);
        }
        if !add.is_empty() {
            self.t_add = Some(add);
        }
    }

    fn compute_ctr(&mut self, address: &SdnRecordAddress) {
        match &address.country {
            Some(c) => {
                self.t_ctr = Some(c.to_owned());
            }
            None => {
                if let Some(region) = &address.region {
                    self.t_ctr = Some(region.to_owned());
                }
            }
        }
    }
    fn compute_ntl(&mut self, nationalities: &[String]) {
        for nationality in nationalities {
            self.t_ntl.push_str(nationality);
            self.t_ntl.push(';');
        }
        if !self.t_ntl.is_empty() {
            self.t_ntl.pop();
        }
    }

    fn compute_inf_addresses(&mut self, addresses: &[SdnRecordAddress]) -> String {
        let mut addresses_inf = String::new();
        for address in addresses {
            address.extract_inf(&mut addresses_inf);
        }
        addresses_inf
    }

    fn compute_inf_documents(&mut self, documents: &[SdnRecordDocument], partysubtypeid: i32) -> AddInfoDocuments {
        let mut passports = String::new();
        let mut cedula = String::new();
        let mut other_docs = String::new();
        for document in documents {
            if document.doc_type == 1570 {
                document.extract_inf(&mut cedula);
            }
            if document.doc_type == 1571 {
                document.extract_inf(&mut passports);
            }
            if partysubtypeid == 1 && [1626, 91264].contains(&document.doc_type) {
                document.extract_inf_with_doc_name(&mut other_docs);
            }
            if partysubtypeid == 2 && document.doc_type == 1623 {
                document.extract_inf_with_doc_name(&mut other_docs);
            }
            if partysubtypeid == 3 && document.doc_type != 1626 {
                document.extract_inf_with_doc_name(&mut other_docs);
            }
            if partysubtypeid == 4 && ![1571, 1570, 1584].contains(&document.doc_type) {
                document.extract_inf_with_doc_name(&mut other_docs);
            }
        }
        (cedula, passports, other_docs)
    }

    fn compute_inf(&mut self, db_record: &SdnRecord) {
        let title = Vec::from([db_record.title.to_owned()]);
        let comment = Vec::from([db_record.comment.to_owned()]);
        let aircraft_construction_number = Vec::from([db_record.aircraft_construction_number.to_owned()]);
        let aircraft_manufacture_date = if let Some(aircraft_manufacture_date) = db_record.aircraft_manufacture_date {
            Vec::from([aircraft_manufacture_date.to_string()])
        } else {
            Vec::new()
        };
        let aircraft_transpondeur_code = Vec::from([db_record.aircraft_transpondeur_code.to_owned()]);
        let aircraft_model = Vec::from([db_record.aircraft_model.to_owned()]);
        let aircraft_tail_number = Vec::from([db_record.aircraft_tail_number.to_owned()]);
        let aircraft_previous_tail_number = Vec::from([db_record.aircraft_previous_tail_number.to_owned()]);
        let gender = Vec::from([db_record.gender.to_owned()]);
        let organization_type = Vec::from([db_record.organization_type.to_owned()]);
        let organization_established_date = if let Some(organization_established_date) = db_record.organization_established_date {
            Vec::from([organization_established_date.to_string()])
        } else {
            Vec::new()
        };
        let vessel_call_sign = Vec::from([db_record.vessel_call_sign.to_owned()]);
        let vessel_type = Vec::from([db_record.vessel_type.to_owned()]);
        let vessel_flag = Vec::from([db_record.vessel_flag.to_owned()]);
        let vessel_owner = Vec::from([db_record.vessel_owner.to_owned()]);
        let other_vessel_call_sign = Vec::from([db_record.other_vessel_call_sign.to_owned()]);
        let manufacturer_serial_number = Vec::from([db_record.msn.to_owned()]);

        let mut fields: AddInfoFields = Vec::from([
            (
                1,
                AddInfoTemplate {
                    title: "PROGRAM".to_owned(),
                    space_between: true,
                    ..Default::default()
                },
                &db_record.programs[..],
            ),
            (
                2,
                AddInfoTemplate {
                    title: "DOB".to_owned(),
                    condition: self.t_dob_overflow,
                    ..Default::default()
                },
                &db_record.dobs[..],
            ),
            (
                3,
                AddInfoTemplate {
                    title: "POB".to_owned(),
                    condition: self.t_pob_overflow,
                    ..Default::default()
                },
                &db_record.pobs[..],
            ),
            (
                4,
                AddInfoTemplate {
                    title: "NATIONALITY".to_owned(),
                    ..Default::default()
                },
                &db_record.nationalities[..],
            ),
            (
                5,
                AddInfoTemplate {
                    title: "CITIZEN".to_owned(),
                    space_between: true,
                    ..Default::default()
                },
                &db_record.citizens[..],
            ),
            (
                6,
                AddInfoTemplate {
                    title: "AIRCRAFT CONSTRUCTION NUMBER".to_owned(),
                    ..Default::default()
                },
                &aircraft_construction_number[..],
            ),
            (
                7,
                AddInfoTemplate {
                    title: "AIRCRAFT MANUFACTURE DATE".to_owned(),
                    ..Default::default()
                },
                &aircraft_manufacture_date[..],
            ),
            (
                8,
                AddInfoTemplate {
                    title: "AIRCRAFT MODE S TRANSPONDEUR CODE".to_owned(),
                    ..Default::default()
                },
                &aircraft_transpondeur_code[..],
            ),
            (
                9,
                AddInfoTemplate {
                    title: "AIRCRAFT MODEL".to_owned(),
                    ..Default::default()
                },
                &aircraft_model[..],
            ),
            (
                10,
                AddInfoTemplate {
                    title: "AIRCRAFT OPERATOR".to_owned(),
                    ..Default::default()
                },
                &db_record.aircraft_operators[..],
            ),
            (
                11,
                AddInfoTemplate {
                    title: "AIRCRAFT TAIL NUMBER".to_owned(),
                    ..Default::default()
                },
                &aircraft_tail_number[..],
            ),
            (
                12,
                AddInfoTemplate {
                    title: "PREVIOUS AIRCRAFT TAIL NUMBER".to_owned(),
                    ..Default::default()
                },
                &aircraft_previous_tail_number[..],
            ),
            (
                13,
                AddInfoTemplate {
                    title: "AIRCRAFT MANUFACTURER'S SERIAL NUMBER (MSN)".to_owned(),
                    ..Default::default()
                },
                &manufacturer_serial_number[..],
            ),
            (
                14,
                AddInfoTemplate {
                    title: "WEBSITE".to_owned(),
                    space_between: true,
                    ..Default::default()
                },
                &db_record.websites[..],
            ),
            (
                15,
                AddInfoTemplate {
                    title: "IFCA Determination".to_owned(),
                    ..Default::default()
                },
                &db_record.ifca_determination[..],
            ),
            (
                16,
                AddInfoTemplate {
                    title: "ADDITIONAL SANCTIONS INFORMATION -".to_owned(),
                    ..Default::default()
                },
                &db_record.additional_sanctions_information[..],
            ),
            (
                17,
                AddInfoTemplate {
                    title: "PEESA INFORMATION -".to_owned(),
                    ..Default::default()
                },
                &db_record.peesa_information[..],
            ),
            (
                18,
                AddInfoTemplate {
                    title: "SECONDARY SANCTIONS RISKS -".to_owned(),
                    ..Default::default()
                },
                &db_record.secondary_sanctions_risks[..],
            ),
            (
                19,
                AddInfoTemplate {
                    title: "TRANSACTIONS PROHIBITED FOR PERSONS OWNED OR CONTROLLED BY U.S. FINANCIAL Institutions:".to_owned(),
                    ..Default::default()
                },
                &db_record.prohibited_transactions[..],
            ),
            (
                20,
                AddInfoTemplate {
                    title: "EMAIL".to_owned(),
                    ..Default::default()
                },
                &db_record.emails[..],
            ),
            (
                21,
                AddInfoTemplate {
                    title: "PHONE NUMBER".to_owned(),
                    ..Default::default()
                },
                &db_record.phone_numbers[..],
            ),
            (
                22,
                AddInfoTemplate {
                    title: "GENDER".to_owned(),
                    ..Default::default()
                },
                &gender[..],
            ),
            (
                25,
                AddInfoTemplate {
                    title: "ORGANIZATION ESTABLISHED DATE".to_owned(),
                    ..Default::default()
                },
                &organization_established_date[..],
            ),
            (
                26,
                AddInfoTemplate {
                    title: "ORGANIZATION TYPE".to_owned(),
                    ..Default::default()
                },
                &organization_type[..],
            ),
            (
                28,
                AddInfoTemplate {
                    title: "VESSEL CALL SIGN".to_owned(),
                    ..Default::default()
                },
                &vessel_call_sign[..],
            ),
            (
                29,
                AddInfoTemplate {
                    title: "VESSEL TYPE".to_owned(),
                    ..Default::default()
                },
                &vessel_type[..],
            ),
            (
                30,
                AddInfoTemplate {
                    title: "VESSEL FLAG".to_owned(),
                    ..Default::default()
                },
                &vessel_flag[..],
            ),
            (
                31,
                AddInfoTemplate {
                    title: "VESSEL OWNER".to_owned(),
                    ..Default::default()
                },
                &vessel_owner[..],
            ),
            (
                32,
                AddInfoTemplate {
                    title: "FORMER VESSEL FLAG".to_owned(),
                    ..Default::default()
                },
                &db_record.former_vessel_flag[..],
            ),
            (
                33,
                AddInfoTemplate {
                    title: "OTHER VESSEL FLAG".to_owned(),
                    ..Default::default()
                },
                &db_record.other_vessel_flag[..],
            ),
            (
                34,
                AddInfoTemplate {
                    title: "OTHER VESSEL CALL SIGN".to_owned(),
                    ..Default::default()
                },
                &other_vessel_call_sign[..],
            ),
            (35, AddInfoTemplate { ..Default::default() }, &comment[..]),
            (36, AddInfoTemplate { ..Default::default() }, &title[..]),
            (
                37,
                AddInfoTemplate {
                    title: "LOW A.K.A".to_owned(),
                    space_between: true,
                    ..Default::default()
                },
                &db_record.low_aliases[..],
            ),
            (
                38,
                AddInfoTemplate {
                    title: "LOW A.K.A".to_owned(),
                    space_between: true,
                    ..Default::default()
                },
                &db_record.ddc_low_aliases[..],
            ),
            (
                39,
                AddInfoTemplate {
                    title: "LINKED TO".to_owned(),
                    space_between: true,
                    ..Default::default()
                },
                &db_record.linked_to_names[..],
            ),
        ]);

        let addresses = [self.compute_inf_addresses(&db_record.addresses)];
        fields.push((
            39,
            AddInfoTemplate {
                title: "ADDRESS".to_owned(),
                ..Default::default()
            },
            &addresses,
        ));

        let docs = self.compute_inf_documents(&db_record.documents, db_record.partysubtypeid);
        let cedula = [docs.0];
        let passports = [docs.1];
        let other = [docs.2];
        fields.push((
            23,
            AddInfoTemplate {
                title: "CEDULA NO.".to_owned(),
                separator: ' ',
                ..Default::default()
            },
            &cedula,
        ));
        fields.push((
            24,
            AddInfoTemplate {
                title: "PASSPORT".to_owned(),
                separator: ' ',
                ..Default::default()
            },
            &passports,
        ));
        fields.push((
            27,
            AddInfoTemplate {
                title: "".to_owned(),
                separator: ' ',
                ..Default::default()
            },
            &other,
        ));
        fields.sort_by(|f1, f2| f1.0.cmp(&f2.0));
        for field in fields {
            self.extract_inf(&field);
        }
    }

    fn extract_inf(&mut self, inf: &(u32, AddInfoTemplate, &[String])) {
        if !inf.1.condition {
            return;
        }
        if !inf.2.is_empty() && self.t_inf.len() < 2048 {
            let mut inf_tmp = String::new();
            let field = if inf.1.space_between {
                let mut separator = String::new();
                separator.push(' ');
                separator.push(inf.1.separator);
                separator.push(' ');
                inf.2.join(&separator)
            } else {
                inf.2.join(&inf.1.separator.to_string())
            };
            if field.len() <= 1 {
                return;
            }
            inf_tmp.push_str(&inf.1.title);
            inf_tmp.push(' ');
            inf_tmp.push_str(&field);
            inf_tmp.push(';');
            inf_tmp.push(' ');
            if self.t_inf.len() + inf_tmp.len() >= 2048 {
                return;
            }
            self.t_inf.push_str(&inf_tmp);
        }
        if self.t_inf.len() > 2048 {
            if let Some((idx, _)) = self.t_inf.char_indices().nth(2048) {
                self.t_inf = self.t_inf[..idx].to_owned()
            }
            warn!("Add info has been truncated at 2048 char (oid {})", self.t_oid);
        }
    }

    fn compute_nid(&mut self, msn: &String, partysubtypeid: i32, documents: &[SdnRecordDocument]) -> Result<(), Box<dyn Error>> {
        match self.t_typ {
            'A' => {}
            'P' => {
                self.t_nid = filter_documents(documents, &[1570, 1572, 1584], ' ', Some(&RE_REPLACE_DOCUMENTS));
            }
            'V' => {
                if partysubtypeid == 1 {
                    self.t_nid = filter_documents(documents, &[1626], ' ', Some(&RE_REPLACE_DOCUMENTS));
                } else if partysubtypeid == 2 {
                    if msn.is_empty() {
                        self.t_nid = Some(format!("MSN{}", self.t_name));
                    } else if msn.len() > 5 {
                        self.t_nid = Some(msn.clone());
                    } else {
                        self.t_nid = Some(format!("MSN{}", msn));
                    }
                    self.t_nid = Some(RE_REPLACE_DOCUMENTS.replace_all(self.t_nid.as_ref().unwrap(), "").to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn compute_shk(&mut self, partysubtypeid: i32, documents: &[SdnRecordDocument]) {
        match partysubtypeid {
            1 => {
                let documents: Vec<&SdnRecordDocument> = documents.iter().filter(|d| d.doc_type == 1626).collect();
                for document in documents {
                    self.t_shk.push_str(&document.registration_number);
                }
            }
            2 => self.t_shk = self.t_name.clone(),
            _ => {}
        }
    }
    fn compute_dob(&mut self, dobs: &[String]) {
        // TODO rajouter le troncage à plus 128
        for dob in dobs {
            if self.t_dob.len() + dob.len() < 128 {
                self.t_dob.push_str(dob);
                self.t_dob.push('/');
            } else {
                self.t_dob_overflow = true;
            }
        }
        if !self.t_dob.is_empty() {
            self.t_dob.pop();
        }
    }
    fn compute_pob(&mut self, pobs: &[String]) {
        for pob in pobs {
            if self.t_pob.len() + pob.len() < 128 {
                self.t_pob.push_str(pob);
                self.t_pob.push('/');
            } else {
                self.t_pob_overflow = true;
            }
        }
        if !self.t_pob.is_empty() {
            self.t_pob.pop();
        }
    }
    fn compute_syn(&mut self, is_alias: bool, fixed_ref: i32) {
        match self.record_type {
            RecordType::Main => {
                if is_alias {
                    self.t_syn = self.t_oid.to_owned()
                }
            }
            RecordType::Alternative => {
                if is_alias {
                    self.t_syn = FofdbofRecord::compute_oid_alternative(fixed_ref)
                }
            }
        }
    }
    fn compute_bic(&mut self, bics: &[String], ddc_bics: &[String]) {
        match self.t_typ {
            'A' | 'P' | 'V' => {}
            _ => {
                let mut bics_str = String::new();
                for bic in bics.iter() {
                    bics_str.push_str(bic);
                    bics_str.push(' ');
                    if bic.len() > 6 {
                        bics_str.push_str(bic.to_string().drain(0..6).as_str());
                        bics_str.push(' ');
                    }
                }
                for bic in ddc_bics.iter() {
                    bics_str.push_str(bic);
                    bics_str.push(' ');
                }
                self.t_bic = Some(bics_str.to_owned());
            }
        }
    }
    fn compute_psp(&mut self, documents: &[SdnRecordDocument]) -> Result<(), Box<dyn Error>> {
        self.t_psp = filter_documents(documents, &[1571], ' ', Some(&RE_REPLACE_DOCUMENTS));
        Ok(())
    }
    fn compute_oid(&mut self, fixed_ref: i32, address_id: i32) {
        let mut loid = String::new();
        match self.doc_type {
            DocumentType::OFAC => loid.push_str("OFAC"),
            DocumentType::OFACNS => loid.push_str("OFNS"),
        }
        let mut roid = String::new();
        match self.record_type {
            RecordType::Main => {
                roid.push_str(&fixed_ref.to_string());
            }
            RecordType::Alternative => {
                loid.push('Z');
                roid.push_str(&address_id.to_string());
            }
        }
        let zeros = 10 - (loid.len() + roid.len());
        self.t_oid = loid + &format!("{:0>zeros$}", "", zeros = zeros) + &roid;
    }
    fn compute_oid_alternative(fixed_ref: i32) -> String {
        let loid = String::from("OFAC");
        let roid = String::from(&fixed_ref.to_string());
        let zeros = 10 - (loid.len() + roid.len());
        loid + &format!("{:0>zeros$}", "", zeros = zeros) + &roid
    }
    fn compute_ref(&mut self, last_update: &str) {
        match self.doc_type {
            DocumentType::OFAC => self.t_ref = format!("OFAC_{}", last_update),
            DocumentType::OFACNS => self.t_ref = format!("OFAC-NS_{}", last_update),
        }
    }
    fn compute_typ(&mut self, partysubtypeid: i32, programs: &[String], other_names: &[DdcName]) {
        match partysubtypeid {
            1 | 2 => self.t_typ = 'V',
            3 => {
                if programs.iter().any(|e| e == "FTO") || other_names.iter().any(|e| e.name == self.t_name) {
                    self.t_typ = 'A'
                } else {
                    self.t_typ = 'C'
                }
            }
            4 => self.t_typ = 'P',
            _ => warn!("Unrecognized TYP value for oid {}", self.t_oid),
        }
    }
    fn compute_us1(&mut self, ddc_programs: &[String], record_pgms: &[String]) {
        let mut intersect = Vec::new();
        let mut v_other: Vec<_> = record_pgms.iter().collect();
        for e1 in ddc_programs.iter() {
            if let Some(pos) = v_other.iter().position(|e2| e1 == *e2) {
                intersect.push(e1);
                v_other.remove(pos);
            }
        }
        self.t_us1 = if !intersect.is_empty() { "Yes".to_owned() } else { "No".to_owned() }
    }
}

fn filter_documents(documents: &[SdnRecordDocument], doctypes: &[i32], separator: char, re: Option<&Regex>) -> Option<String> {
    let documents: Vec<&SdnRecordDocument> = documents.iter().filter(|d| doctypes.contains(&d.doc_type)).collect();
    if documents.is_empty() {
        return None;
    }
    let mut document_str = String::new();
    for document in documents {
        let mut registration_number = document.registration_number.clone();
        if let Some(re) = re {
            registration_number = re.replace_all(&registration_number, "").as_ref().to_owned();
        }
        if document.doc_type == 1626 {
            if registration_number.starts_with("IMO") {
                registration_number = registration_number[3..].to_owned();
            } else {
                return None;
            }
        }
        document_str.push_str(&registration_number);
        document_str.push(separator);
    }
    document_str.pop();
    Some(document_str)
}

impl Display for FofdbofRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut inf = &self.t_inf[..];
        if inf.is_empty() {
            inf = &SEPARATOR;
        } else {
            inf = &inf[0..inf.len() - 1];
        }
        writeln!(
            f,
            "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
            self.t_oid.to_uppercase(),
            *SEPARATOR,
            self.t_name.to_uppercase(),
            *SEPARATOR,
            self.t_add.as_deref().unwrap_or("").to_uppercase(),
            *SEPARATOR,
            self.t_cit.as_deref().unwrap_or("").to_uppercase(),
            *SEPARATOR,
            self.t_ctr.as_deref().unwrap_or("").to_uppercase(),
            *SEPARATOR,
            self.t_sta.as_deref().unwrap_or("").to_uppercase(),
            *SEPARATOR,
            self.t_typ.to_uppercase(),
            *SEPARATOR,
            self.t_bad.to_uppercase(),
            *SEPARATOR,
            self.t_shk.to_uppercase(),
            *SEPARATOR,
            self.t_syn.to_uppercase(),
            *SEPARATOR,
            self.t_syc,
            *SEPARATOR,
            self.t_syk,
            *SEPARATOR,
            self.t_sys,
            *SEPARATOR,
            self.t_ori.to_uppercase(),
            *SEPARATOR,
            self.t_dsg.to_uppercase(),
            *SEPARATOR,
            self.t_us1.to_uppercase(),
            *SEPARATOR,
            self.t_us2.as_deref().unwrap_or("").to_uppercase(),
            *SEPARATOR,
            self.t_ref.to_uppercase(),
            *SEPARATOR,
            self.t_bic.as_deref().unwrap_or("").to_uppercase(),
            *SEPARATOR,
            self.t_psp.as_deref().unwrap_or("").to_uppercase(),
            *SEPARATOR,
            self.t_nid.as_deref().unwrap_or("").to_uppercase(),
            *SEPARATOR,
            if self.t_pob.is_empty() { ("").to_owned() } else { self.t_pob.to_uppercase() },
            *SEPARATOR,
            if self.t_dob.is_empty() { ("").to_owned() } else { self.t_dob.to_uppercase() },
            *SEPARATOR,
            "", // BGH
            *SEPARATOR,
            inf,
            *SEPARATOR,
            "", // ORH
            *SEPARATOR,
            "", // TGH
            *SEPARATOR,
            "", // IDH
            *SEPARATOR,
            "", // UNH
            *SEPARATOR,
            "", // SRH
            *SEPARATOR,
            '0', // PEP
            *SEPARATOR,
            '0', // FEP
            *SEPARATOR,
            "", // KWS
            *SEPARATOR,
            "", // HLK
            *SEPARATOR,
            self.t_ntl.to_uppercase(),
            *SEPARATOR,
            self.t_gdr,
            *SEPARATOR,
        )
    }
}
#[cfg(test)]
mod record {
    use crate::processor::entity::sdn::SdnRecordAddress;

    use super::*;

    #[test]
    fn format_alternative_record() {
        let mut db_record = SdnRecord::default();
        db_record.addresses = vec![SdnRecordAddress::default()];
        db_record.last_update = "1970/01/01".to_owned();
        let record = FofdbofRecord::from_db_record(&db_record, &DocumentType::OFAC, &[]);

        let mut excepted_records = Vec::new();
        excepted_records.push(FofdbofRecord {
            t_typ: 'V',
            t_bad: '0',
            t_dsg: "OFAC".to_owned(),
            t_ori: "OFAC".to_owned(),
            t_oid: "OFACZ00000".to_owned(),
            t_ref: "OFAC_1970/01/01".to_owned(),
            t_us1: "No".to_owned(),
            t_us2: Some("OFAC000000".to_owned()),
            t_ntl: "".to_owned(),
            t_syc: "".to_owned(),
            t_syk: "".to_owned(),
            t_sys: "".to_owned(),
            t_gdr: "U".to_owned(),
            record_type: RecordType::Alternative,
            ..Default::default()
        });

        assert_eq!(excepted_records, *record);
    }
}
