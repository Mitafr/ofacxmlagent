use std::sync::Arc;

use super::nationality::NationalityEntity;
use super::nationality_registration::NationalityRegistrationEntity;
use chrono::Local;
use log::info;
use sea_orm::sea_query::Expr;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{entity::prelude::*, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, Iterable, QueryFilter, RelationTrait, Set};
use sea_orm::{DatabaseTransaction, IntoActiveModel, ModelTrait};

use crate::db::{OfacEntity, OfacEntityFinalOp};
use crate::document::{
    models::{distinctparty::DistinctParty, location::Locations, sanction::SanctionsEntries},
    OfacDocumentReferences,
};

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "sdn")]
pub struct Model {
    #[sea_orm(unique)]
    pub fixed_ref: i32,
    #[sea_orm(primary_key, auto_increment)]
    pub record_id: i32,
    #[sea_orm(unique)]
    pub identity: i32,
    pub partysubtypeid: i32,
    #[sea_orm(column_type = "Text")]
    pub sdn_type: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub gender: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub title: Option<String>,
    pub additional_sanctions_information: Option<i32>,
    pub secondary_sanctions_risks: Option<i32>,
    pub organization_established_date: Option<Date>,
    pub organization_type: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub locode: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub micex_code: Option<String>,
    pub duns_number: Option<i32>,
    pub registration_country: Option<i32>,
    pub prohibited_transactions: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub vessel_call_sign: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub other_vessel_call_sign: Option<String>,
    pub vessel_type: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub vessel_flag: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub vessel_owner: Option<String>,
    pub vessel_tonnage: Option<i32>,
    pub vessel_gross_registered_tonnage: Option<i32>,
    pub other_vessel_type: Option<i32>,
    pub cmic_effective_date: Option<Date>,
    pub cmic_sales_date: Option<Date>,
    pub cmic_listing_date: Option<Date>,
    pub ifca_determination: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub dca_bch: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub dca_bsv: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub dca_btg: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub dca_dash: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub dca_etc: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub dca_eth: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub dca_ltc: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub dca_usdt: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub dca_xbt: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub dca_xmr: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub dca_xrp: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub dca_xvh: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub dca_zec: Option<String>,
    pub sanction_date: Option<Date>,
    pub sanction_status: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub construction_number: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub manufacturer_serial_number: Option<String>,
    pub manufacture_date: Option<Date>,
    #[sea_orm(column_type = "Text", nullable)]
    pub transpondeur_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub previous_tail_number: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub tail_number: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub model: Option<String>,
    pub peesa_information: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub comment: Option<String>,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())", nullable)]
    pub topmaj: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub updated_by: Option<String>,
    pub last_update: Option<Date>,
}

#[derive(Default, Clone, Debug)]
pub struct SdnInnerRelation {
    sdn: Model,
    pub is_active: bool,

    pub address: Vec<address::Model>,
    pub names: Vec<name::Model>,
    pub operators: Vec<aircraft_operator::Model>,
    pub bics: Vec<bic::Model>,
    pub biks: Vec<bik::Model>,
    pub dobs: Vec<dob::Model>,
    pub pobs: Vec<pob::Model>,
    pub caatsa235s: Vec<caatsa235::Model>,
    pub citizens: Vec<citizen::Model>,
    pub emails: Vec<email::Model>,
    pub eo13662dds: Vec<eo13662dd::Model>,
    pub eo13846infs: Vec<eo13846inf::Model>,
    pub eo14024dds: Vec<eo14024dd::Model>,
    pub equity_tickers: Vec<equity_ticker::Model>,
    pub former_vessel_flags: Vec<former_vessel_flag::Model>,
    pub isins: Vec<isin::Model>,
    pub issuer_names: Vec<issuer_name::Model>,
    pub nationalities: Vec<nationality::Model>,
    pub nationality_registrations: Vec<nationality_registration::Model>,
    pub other_vessel_flags: Vec<other_vessel_flag::Model>,
    pub programs: Vec<program::Model>,
    pub phone_numbers: Vec<phone_number::Model>,
    pub targets: Vec<target::Model>,
    pub websites: Vec<website::Model>,
}

impl SdnInnerRelation {
    async fn process_relations(&mut self, db: &DatabaseConnection, tx: &Arc<tokio::sync::Mutex<DatabaseTransaction>>) -> Result<OfacEntityFinalOp, DbErr> {
        let mut op = OfacEntityFinalOp::Nothing;
        if !self.is_active {
            info!("SDN with fixed_ref {} is INACTIVE and skipped", self.sdn.fixed_ref);
            return Ok(op);
        }
        let id = self.sdn.fixed_ref;
        let identity = self.sdn.identity;
        address::ActiveModel::process_entity(&mut self.address, &mut self.sdn.find_linked(address_sdn::SdnToAddress).all(db).await?, db, tx, identity, &mut op).await?;
        aircraft_operator::ActiveModel::process_entity(&mut self.operators, &mut self.sdn.find_linked(aircraft_operator_sdn::SdnToAircraftOperator).all(db).await?, db, tx, id, &mut op).await?;
        name::ActiveModel::process_entity(&mut self.names, &mut self.sdn.find_linked(name_sdn::SdnToName).all(db).await?, db, tx, id, &mut op).await?;
        bic::ActiveModel::process_entity(&mut self.bics, &mut self.sdn.find_linked(bic_sdn::SdnToBic).all(db).await?, db, tx, id, &mut op).await?;
        bik::ActiveModel::process_entity(&mut self.biks, &mut self.sdn.find_linked(bik_sdn::SdnToBik).all(db).await?, db, tx, id, &mut op).await?;
        dob::ActiveModel::process_entity(&mut self.dobs, &mut self.sdn.find_linked(dob_identity::SdnToDob).all(db).await?, db, tx, identity, &mut op).await?;
        caatsa235::ActiveModel::process_entity(&mut self.caatsa235s, &mut self.sdn.find_linked(caatsa235_sdn::SdnToCaatsa235).all(db).await?, db, tx, id, &mut op).await?;
        citizen::ActiveModel::process_entity(&mut self.citizens, &mut self.sdn.find_linked(citizen_sdn::SdnToCitizen).all(db).await?, db, tx, id, &mut op).await?;
        email::ActiveModel::process_entity(&mut self.emails, &mut self.sdn.find_linked(email_sdn::SdnToEmail).all(db).await?, db, tx, id, &mut op).await?;
        eo13662dd::ActiveModel::process_entity(&mut self.eo13662dds, &mut self.sdn.find_linked(eo13662dd_sdn::SdnToEo13662dd).all(db).await?, db, tx, id, &mut op).await?;
        eo13846inf::ActiveModel::process_entity(&mut self.eo13846infs, &mut self.sdn.find_linked(eo13846inf_sdn::SdnToEo13846inf).all(db).await?, db, tx, id, &mut op).await?;
        eo14024dd::ActiveModel::process_entity(&mut self.eo14024dds, &mut self.sdn.find_linked(eo14024dd_sdn::SdnToEo14024dd).all(db).await?, db, tx, id, &mut op).await?;
        equity_ticker::ActiveModel::process_entity(&mut self.equity_tickers, &mut self.sdn.find_linked(equity_ticker_sdn::SdnToEquityTicker).all(db).await?, db, tx, id, &mut op).await?;
        former_vessel_flag::ActiveModel::process_entity(
            &mut self.former_vessel_flags,
            &mut self.sdn.find_linked(former_vessel_flag_sdn::SdnToFormerVesselFlag).all(db).await?,
            db,
            tx,
            id,
            &mut op,
        )
        .await?;
        isin::ActiveModel::process_entity(&mut self.isins, &mut self.sdn.find_linked(isin_sdn::SdnToIsin).all(db).await?, db, tx, id, &mut op).await?;
        issuer_name::ActiveModel::process_entity(&mut self.issuer_names, &mut self.sdn.find_linked(issuer_name_sdn::SdnToIssuerName).all(db).await?, db, tx, id, &mut op).await?;
        nationality::ActiveModel::process_entity(&mut self.nationalities, &mut self.sdn.find_linked(nationality_identity::SdnToNationality).all(db).await?, db, tx, identity, &mut op).await?;
        nationality_registration::ActiveModel::process_entity(
            &mut self.nationality_registrations,
            &mut self.sdn.find_linked(nationality_registration_sdn::SdnToNationalityRegistration).all(db).await?,
            db,
            tx,
            id,
            &mut op,
        )
        .await?;
        other_vessel_flag::ActiveModel::process_entity(
            &mut self.other_vessel_flags,
            &mut self.sdn.find_linked(other_vessel_flag_sdn::SdnToOtherVesselFlag).all(db).await?,
            db,
            tx,
            id,
            &mut op,
        )
        .await?;
        phone_number::ActiveModel::process_entity(&mut self.phone_numbers, &mut self.sdn.find_linked(phone_number_sdn::SdnToPhoneNumber).all(db).await?, db, tx, id, &mut op).await?;
        program::ActiveModel::process_entity(&mut self.programs, &mut self.sdn.find_linked(sdn_program::SdnToProgram).all(db).await?, db, tx, id, &mut op).await?;
        pob::ActiveModel::process_entity(&mut self.pobs, &mut self.sdn.find_linked(pob_identity::SdnToPob).all(db).await?, db, tx, identity, &mut op).await?;
        target::ActiveModel::process_entity(&mut self.targets, &mut self.sdn.find_linked(target_sdn::SdnToTarget).all(db).await?, db, tx, id, &mut op).await?;
        website::ActiveModel::process_entity(&mut self.websites, &mut self.sdn.find_linked(website_identity::SdnToWebsite).all(db).await?, db, tx, identity, &mut op).await?;
        Ok(op)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ref_reference::Entity",
        from = "Column::AdditionalSanctionsInformation",
        to = "super::ref_reference::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefReference7,
    #[sea_orm(
        belongs_to = "super::ref_type::Entity",
        from = "Column::Partysubtypeid",
        to = "super::ref_type::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefType,
    #[sea_orm(
        belongs_to = "super::ref_reference::Entity",
        from = "Column::SecondarySanctionsRisks",
        to = "super::ref_reference::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefReference8,
    #[sea_orm(
        belongs_to = "super::ref_reference::Entity",
        from = "Column::OrganizationType",
        to = "super::ref_reference::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefReference6,
    #[sea_orm(
        belongs_to = "super::ref_reference::Entity",
        from = "Column::ProhibitedTransactions",
        to = "super::ref_reference::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefReference5,
    #[sea_orm(
        belongs_to = "super::ref_reference::Entity",
        from = "Column::VesselType",
        to = "super::ref_reference::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefReference4,
    #[sea_orm(
        belongs_to = "super::ref_reference::Entity",
        from = "Column::PeesaInformation",
        to = "super::ref_reference::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefReference3,
    #[sea_orm(
        belongs_to = "super::ref_reference::Entity",
        from = "Column::OtherVesselType",
        to = "super::ref_reference::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefReference2,
    #[sea_orm(
        belongs_to = "super::ref_country::Entity",
        from = "Column::RegistrationCountry",
        to = "super::ref_country::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefCountry,
    #[sea_orm(
        belongs_to = "super::ref_reference::Entity",
        from = "Column::IfcaDetermination",
        to = "super::ref_reference::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefReference1,
    #[sea_orm(has_many = "super::address_sdn::Entity")]
    AddressSdn,
    #[sea_orm(has_many = "super::aircraft_operator_sdn::Entity")]
    AircraftOperatorSdn,
    #[sea_orm(has_many = "super::name_sdn::Entity")]
    NameSdn,
    #[sea_orm(has_many = "super::bic_sdn::Entity")]
    BicSdn,
    #[sea_orm(has_many = "super::bik_sdn::Entity")]
    BikSdn,
    #[sea_orm(has_many = "super::caatsa235_sdn::Entity")]
    Caatsa235Sdn,
    #[sea_orm(has_many = "super::citizen_sdn::Entity")]
    CitizenSdn,
    #[sea_orm(has_many = "super::ddc_alias_sdn::Entity")]
    DdcAliasSdn,
    #[sea_orm(has_many = "super::dob_identity::Entity")]
    DobIdentity,
    #[sea_orm(has_many = "super::document_identity::Entity")]
    DocumentIdentity,
    #[sea_orm(has_many = "super::email_sdn::Entity")]
    EmailSdn,
    #[sea_orm(has_many = "super::eo13662dd_sdn::Entity")]
    Eo13662ddSdn,
    #[sea_orm(has_many = "super::eo13846inf_sdn::Entity")]
    Eo13846infSdn,
    #[sea_orm(has_many = "super::eo14024dd_sdn::Entity")]
    Eo14024ddSdn,
    #[sea_orm(has_many = "super::equity_ticker_sdn::Entity")]
    EquityTickerSdn,
    #[sea_orm(has_many = "super::former_vessel_flag_sdn::Entity")]
    FormerVesselFlagSdn,
    #[sea_orm(has_many = "super::isin_sdn::Entity")]
    IsinSdn,
    #[sea_orm(has_many = "super::issuer_name_sdn::Entity")]
    IssuerNameSdn,
    #[sea_orm(has_many = "super::nationality_identity::Entity")]
    NationalityIdentity,
    #[sea_orm(has_many = "super::nationality_registration_sdn::Entity")]
    NationalityRegistrationSdn,
    #[sea_orm(has_many = "super::other_vessel_flag_sdn::Entity")]
    OtherVesselFlagSdn,
    #[sea_orm(has_many = "super::phone_number_sdn::Entity")]
    PhoneNumberSdn,
    #[sea_orm(has_many = "super::pob_identity::Entity")]
    PobIdentity,
    #[sea_orm(has_many = "super::relation::Entity")]
    Relation,
    #[sea_orm(has_many = "super::relation_sdn::Entity")]
    RelationSdn,
    #[sea_orm(has_many = "super::sdn_program::Entity")]
    ProgramSdn,
    #[sea_orm(has_many = "super::target_sdn::Entity")]
    TargetSdn,
    #[sea_orm(has_many = "super::website_identity::Entity")]
    WebsiteIdentity,
}

impl Related<super::ref_type::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RefType.def()
    }
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

impl Related<super::aircraft_operator_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AircraftOperatorSdn.def()
    }
}

impl Related<super::name_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NameSdn.def()
    }
}

impl Related<super::bic_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BicSdn.def()
    }
}

impl Related<super::bik_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BikSdn.def()
    }
}

impl Related<super::caatsa235_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Caatsa235Sdn.def()
    }
}

impl Related<super::citizen_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CitizenSdn.def()
    }
}

impl Related<super::ddc_alias_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DdcAliasSdn.def()
    }
}

impl Related<super::dob_identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DobIdentity.def()
    }
}

impl Related<super::document_identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DocumentIdentity.def()
    }
}

impl Related<super::email_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EmailSdn.def()
    }
}

impl Related<super::eo13662dd_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Eo13662ddSdn.def()
    }
}

impl Related<super::eo13846inf_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Eo13846infSdn.def()
    }
}

impl Related<super::eo14024dd_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Eo14024ddSdn.def()
    }
}

impl Related<super::equity_ticker_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EquityTickerSdn.def()
    }
}

impl Related<super::former_vessel_flag_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FormerVesselFlagSdn.def()
    }
}

impl Related<super::isin_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IsinSdn.def()
    }
}

impl Related<super::issuer_name_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IssuerNameSdn.def()
    }
}

impl Related<super::nationality_identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NationalityIdentity.def()
    }
}

impl Related<super::nationality_registration_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NationalityRegistrationSdn.def()
    }
}

impl Related<super::other_vessel_flag_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OtherVesselFlagSdn.def()
    }
}

impl Related<super::phone_number_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PhoneNumberSdn.def()
    }
}

impl Related<super::pob_identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PobIdentity.def()
    }
}

impl Related<super::relation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Relation.def()
    }
}

impl Related<super::relation_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RelationSdn.def()
    }
}

impl Related<super::sdn_program::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProgramSdn.def()
    }
}

impl Related<super::target_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TargetSdn.def()
    }
}

impl Related<super::website_identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WebsiteIdentity.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn before_save(mut self, _insert: bool) -> Result<Self, DbErr> {
        self.last_update = Set(Some(Local::today().naive_local()));
        self.updated_by = Set(Some("BATCH".to_owned()));
        Ok(self)
    }
}

pub struct DocumentEntity<'a>(pub &'a DistinctParty, pub &'a Locations, pub &'a SanctionsEntries);

impl Model {
    pub fn from_ofac_document(entity: &DocumentEntity<'_>, references: &OfacDocumentReferences) -> Result<(Model, SdnInnerRelation), DbErr> {
        let fixed_ref = entity.0.fixed_ref;
        let identity = entity.0.profile.identity.id;
        let sanction = sanction::Model::from_ofac_document(entity.2.entries.iter().find(|s| s.profile_id == fixed_ref).unwrap(), references)?;
        let mut sdn_db = Model {
            identity,
            fixed_ref,
            sanction_date: Some(sanction.date),
            sanction_status: sanction.status,
            partysubtypeid: entity.0.profile.party_sub_id,
            topmaj: "N".to_owned(),
            ..Default::default()
        };
        sdn_db.sdn_type = match sdn_db.partysubtypeid {
            1 => "VESSEL".to_owned(),
            2 => "AIRCRAFT".to_owned(),
            3 => "ENTITY".to_owned(),
            4 => "INDIVIDUAL".to_owned(),
            _ => "VESSEL".to_owned(),
        };
        if let Some(comment) = &entity.0.comment {
            if !comment.is_empty() {
                sdn_db.comment = Some(comment.to_uppercase());
            }
        }
        let mut inner_relations = SdnInnerRelation {
            programs: sanction.programs,
            names: name::Model::from_ofac_document(&entity.0.profile.identity, &entity.0.profile.identity.name_part_groups, &references.script_values),
            ..Default::default()
        };
        if let Some(features) = &entity.0.profile.feature {
            let mut is_primary_address = true;
            for feature in features {
                if let Some(date_period) = &feature.version.date_period.clone() {
                    match feature.feature_type {
                        8 => inner_relations.dobs.push(dob::Model::from_ofac_document(&feature.version)),
                        867 => {
                            let start = date_period.start.as_ref().unwrap();
                            let end = date_period.end.as_ref().unwrap();
                            if start.from == start.to && end.from == end.to {
                                sdn_db.cmic_effective_date = Some(start.from.to_sql_date());
                            }
                        }
                        868 => {
                            let start = date_period.start.as_ref().unwrap();
                            let end = date_period.end.as_ref().unwrap();
                            if start.from == start.to && end.from == end.to {
                                sdn_db.cmic_sales_date = Some(start.from.to_sql_date());
                            }
                        }
                        869 => {
                            let start = date_period.start.as_ref().unwrap();
                            let end = date_period.end.as_ref().unwrap();
                            if start.from == start.to && end.from == end.to {
                                sdn_db.cmic_listing_date = Some(start.from.to_sql_date());
                            }
                        }
                        _ => {}
                    }
                }
                if let Some(location) = feature.version.location {
                    match feature.feature_type {
                        10 => inner_relations.nationalities.push(nationality::Model::from_ofac_document(&NationalityEntity(feature, entity.1), references)),
                        11 => inner_relations.citizens.push(citizen::Model::from_ofac_document((feature, entity.1))),
                        25 => {
                            inner_relations.address.push(address::Model::from_ofac_document((feature, entity.1), references, is_primary_address));
                            is_primary_address = false;
                        }
                        365 => inner_relations
                            .nationality_registrations
                            .push(nationality_registration::Model::from_ofac_document(&NationalityRegistrationEntity(feature, entity.1))),
                        404 => {
                            let location = entity.1.locations.iter().find(|&loc| loc.id == location.id).unwrap();
                            if let Some(parts) = location.location_parts.as_ref() {
                                for part in parts {
                                    for part_value in part.values.iter() {
                                        if part_value.primary {
                                            let value = part_value.value.to_owned();
                                            match references.area_codes.iter().find(|&area| area.name == value) {
                                                Some(area) => sdn_db.registration_country = Some(area.id),
                                                None => sdn_db.registration_country = None,
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if let Some(detail) = feature.version.detail.clone() {
                    match feature.feature_type {
                        1 => sdn_db.vessel_call_sign = if detail.value.is_some() { Some(detail.value.unwrap().to_uppercase()) } else { None },
                        2 => sdn_db.vessel_type = detail.detail_reference_id,
                        3 => sdn_db.vessel_flag = if detail.value.is_some() { Some(detail.value.unwrap().to_uppercase()) } else { None },
                        4 => sdn_db.vessel_owner = if detail.value.is_some() { Some(detail.value.unwrap().to_uppercase()) } else { None },
                        5 => sdn_db.vessel_tonnage = if let Ok(tonnage) = detail.value.unwrap().parse::<i32>() { Some(tonnage) } else { None },
                        6 => sdn_db.vessel_gross_registered_tonnage = if let Ok(gross_tonnage) = detail.value.unwrap().parse::<i32>() { Some(gross_tonnage) } else { None },
                        9 => inner_relations.pobs.push(pob::Model::from_ofac_document(&feature.version)),
                        13 => inner_relations.bics.push(bic::Model::from_ofac_document(&feature.version)),
                        14 => inner_relations.websites.push(website::Model::from_ofac_document(&feature.version)),
                        21 => inner_relations.emails.push(email::Model::from_ofac_document(&feature.version)),
                        24 => {
                            if sdn_db.partysubtypeid == 1 {
                                inner_relations.former_vessel_flags.push(former_vessel_flag::Model::from_ofac_document(&feature.version));
                            }
                        }
                        26 => sdn_db.title = if detail.value.is_some() { Some(detail.value.unwrap().to_uppercase()) } else { None },
                        44 => sdn_db.construction_number = Some(feature.version.detail.as_ref().unwrap().value.as_ref().unwrap().to_uppercase()),
                        45 => {
                            let date_of_period = feature.version.date_period.as_ref().unwrap();
                            let start = date_of_period.start.as_ref().unwrap();
                            let end = date_of_period.end.as_ref().unwrap();
                            if start.from == start.to && end.from == end.to {
                                sdn_db.manufacture_date = Some(start.from.to_sql_date());
                            }
                        }
                        46 => {
                            sdn_db.transpondeur_code = Some(feature.version.detail.as_ref().unwrap().value.as_ref().unwrap().to_uppercase());
                        }
                        47 => {
                            sdn_db.model = Some(feature.version.detail.as_ref().unwrap().value.as_ref().unwrap().to_uppercase());
                        }
                        48 => inner_relations.operators.push(aircraft_operator::Model::from_ofac_document(&feature.version)),
                        49 => {
                            sdn_db.previous_tail_number = Some(feature.version.detail.as_ref().unwrap().value.as_ref().unwrap().to_uppercase());
                        }
                        50 => {
                            sdn_db.manufacturer_serial_number = Some(feature.version.detail.as_ref().unwrap().value.as_ref().unwrap().to_uppercase());
                        }
                        64 => {
                            sdn_db.tail_number = Some(feature.version.detail.as_ref().unwrap().value.as_ref().unwrap().to_uppercase());
                        }
                        104 => sdn_db.ifca_determination = detail.detail_reference_id,
                        125 => {
                            let reference_id = detail.detail_reference_id.unwrap();
                            sdn_db.additional_sanctions_information = Some(reference_id);
                        }
                        164 => inner_relations.biks.push(bik::Model::from_ofac_document(&feature.version)),
                        204 => inner_relations.eo13662dds.push(eo13662dd::Model::from_ofac_document(&feature.version)),
                        224 => {
                            sdn_db.gender = if detail.detail_type_id.unwrap() == 1431 && detail.detail_reference_id.unwrap() == 91526 {
                                Some("MALE".to_owned())
                            } else if detail.detail_type_id.unwrap() == 1431 && detail.detail_reference_id.unwrap() == 91527 {
                                Some("FEMALE".to_owned())
                            } else {
                                None
                            }
                        }
                        264 => {
                            if let Some(locode) = detail.value {
                                sdn_db.locode = Some(locode.to_uppercase());
                            }
                        }
                        304 => {
                            if let Some(micex_code) = detail.value {
                                sdn_db.micex_code = Some(micex_code.to_uppercase());
                            }
                        }
                        344 => match &mut sdn_db.dca_xbt {
                            Some(address) => {
                                address.push('/');
                                address.push_str(&detail.value.unwrap_or_else(|| "".to_owned()));
                            }
                            None => sdn_db.dca_xbt = detail.value,
                        },
                        345 => match &mut sdn_db.dca_eth {
                            Some(address) => {
                                address.push('/');
                                address.push_str(&detail.value.unwrap_or_else(|| "".to_owned()));
                            }
                            None => sdn_db.dca_eth = detail.value,
                        },
                        364 => sdn_db.duns_number = if let Ok(duns_number) = detail.value.unwrap().parse::<i32>() { Some(duns_number) } else { None },
                        424 => inner_relations.other_vessel_flags.push(other_vessel_flag::Model::from_ofac_document(&feature.version)),
                        425 => sdn_db.other_vessel_call_sign = detail.value.clone(),
                        444 => match &mut sdn_db.dca_xmr {
                            Some(address) => {
                                address.push('/');
                                address.push_str(&detail.value.unwrap_or_else(|| "".to_owned()));
                            }
                            None => sdn_db.dca_xmr = detail.value,
                        },
                        504 => sdn_db.secondary_sanctions_risks = detail.detail_reference_id,
                        524 => inner_relations.phone_numbers.push(phone_number::Model::from_ofac_document(&feature.version)),
                        525 => inner_relations.caatsa235s.push(caatsa235::Model::from_ofac_document(&feature.version)),
                        526 => sdn_db.other_vessel_type = detail.detail_reference_id,
                        566 => match &mut sdn_db.dca_ltc {
                            Some(address) => {
                                address.push('/');
                                address.push_str(&detail.value.unwrap_or_else(|| "".to_owned()));
                            }
                            None => sdn_db.dca_ltc = detail.value,
                        },
                        586 => inner_relations.eo13846infs.push(eo13846inf::Model::from_ofac_document(&feature.version)),
                        626 => sdn_db.prohibited_transactions = detail.detail_reference_id,
                        646 => {
                            if let Some(date_period) = feature.version.date_period.clone() {
                                sdn_db.organization_established_date = Some(date_period.start.as_ref().unwrap().from.to_sql_date());
                            }
                        }
                        647 => sdn_db.organization_type = feature.version.detail.as_ref().unwrap().detail_reference_id,
                        686 => match &mut sdn_db.dca_zec {
                            Some(address) => {
                                address.push('/');
                                address.push_str(&detail.value.unwrap_or_else(|| "".to_owned()));
                            }
                            None => sdn_db.dca_zec = detail.value,
                        },
                        687 => match &mut sdn_db.dca_dash {
                            Some(address) => {
                                address.push('/');
                                address.push_str(&detail.value.unwrap_or_else(|| "".to_owned()));
                            }
                            None => sdn_db.dca_dash = detail.value,
                        },
                        688 => match &mut sdn_db.dca_btg {
                            Some(address) => {
                                address.push('/');
                                address.push_str(&detail.value.unwrap_or_else(|| "".to_owned()));
                            }
                            None => sdn_db.dca_btg = detail.value,
                        },
                        689 => match &mut sdn_db.dca_etc {
                            Some(address) => {
                                address.push('/');
                                address.push_str(&detail.value.unwrap_or_else(|| "".to_owned()));
                            }
                            None => sdn_db.dca_etc = detail.value,
                        },
                        706 => match &mut sdn_db.dca_bsv {
                            Some(address) => {
                                address.push('/');
                                address.push_str(&detail.value.unwrap_or_else(|| "".to_owned()));
                            }
                            None => sdn_db.dca_bsv = detail.value,
                        },
                        726 => match &mut sdn_db.dca_bch {
                            Some(address) => {
                                address.push('/');
                                address.push_str(&detail.value.unwrap_or_else(|| "".to_owned()));
                            }
                            None => sdn_db.dca_bch = detail.value,
                        },
                        746 => match &mut sdn_db.dca_xvh {
                            Some(address) => {
                                address.push('/');
                                address.push_str(&detail.value.unwrap_or_else(|| "".to_owned()));
                            }
                            None => sdn_db.dca_xvh = detail.value,
                        },
                        766 => inner_relations.equity_tickers.push(equity_ticker::Model::from_ofac_document(&feature.version)),
                        767 => inner_relations.issuer_names.push(issuer_name::Model::from_ofac_document(&feature.version)),
                        806 => inner_relations.isins.push(isin::Model::from_ofac_document(&feature.version)),
                        826 => inner_relations.targets.push(target::Model::from_ofac_document(&feature.version)),
                        827 => sdn_db.peesa_information = detail.detail_reference_id,
                        887 => match &mut sdn_db.dca_usdt {
                            Some(address) => {
                                address.push('/');
                                address.push_str(&detail.value.unwrap_or_else(|| "".to_owned()));
                            }
                            None => sdn_db.dca_usdt = detail.value,
                        },
                        907 => match &mut sdn_db.dca_xrp {
                            Some(address) => {
                                address.push('/');
                                address.push_str(&detail.value.unwrap_or_else(|| "".to_owned()));
                            }
                            None => sdn_db.dca_xrp = detail.value,
                        },
                        947 | 948 => inner_relations.eo14024dds.push(eo14024dd::Model::from_ofac_document(&feature.version)),
                        _ => {}
                    }
                }
            }
        }
        Ok((sdn_db, inner_relations))
    }
}

impl ActiveModel {
    /// Process each entities in paralell
    /// * `db` - is used for SELECT
    /// * `tx` - is used for INSERT/UPDATE/DELETE
    pub async fn process_entities(entities: &[(Model, SdnInnerRelation)], db: DatabaseConnection, tx: &Arc<tokio::sync::Mutex<DatabaseTransaction>>) -> Result<(), DbErr> {
        let tasks: Vec<_> = entities
            .iter()
            .map(|e| {
                let tx = Arc::clone(tx);
                tokio::spawn(ActiveModel::process_entity(e.0.clone(), e.1.clone(), db.clone(), tx))
            })
            .collect();
        let mut saved_sdns = Vec::new();
        for task in tasks {
            saved_sdns.push(task.await.unwrap().unwrap());
        }
        set_sanction_inactive(Arc::clone(tx), &saved_sdns).await?;
        Ok(())
    }

    /// Process an entity to save it in DB
    async fn process_entity(mut sdn: Model, mut relations: SdnInnerRelation, db: DatabaseConnection, tx: Arc<tokio::sync::Mutex<DatabaseTransaction>>) -> Result<i32, DbErr> {
        let fixed_ref = sdn.fixed_ref;
        if let Some(in_db) = Entity::find().filter(Column::FixedRef.eq(sdn.fixed_ref)).one(&db).await? {
            relations.sdn = in_db.clone();
            let mut sdn_db = in_db;
            let in_db_topmaj = sdn_db.topmaj;
            sdn_db.topmaj = sdn.topmaj.to_owned();
            sdn.record_id = sdn_db.record_id;
            sdn_db.last_update = sdn.last_update;
            sdn_db.updated_by = sdn.updated_by.clone();
            relations.is_active = sdn_db.sanction_status != *"INACTIVE";
            if sdn == sdn_db {
                if in_db_topmaj == *"O" {
                    {
                        let lock = tx.lock().await;
                        let mut model = sdn.clone().into_active_model();
                        model.topmaj = Set("N".to_owned());
                        model.update(&*lock).await?;
                    }
                }
                if relations.process_relations(&db, &tx).await? != OfacEntityFinalOp::Nothing {
                    let lock = tx.lock().await;
                    let mut model = sdn.into_active_model();
                    model.topmaj = Set("O".to_owned());
                    model.update(&*lock).await?;
                }
                return Ok(fixed_ref);
            }
            let mut model = ActiveModel::from(sdn);
            model.topmaj = Set("O".to_owned());
            for col in Column::iter() {
                let v = model.get(col);
                model.set(col, v.into_value().unwrap());
            }
            {
                let lock = tx.lock().await;
                model.update(&*lock).await?;
            }
            relations.process_relations(&db, &tx).await?;
            return Ok(fixed_ref);
        }
        relations.sdn = sdn.clone();
        relations.is_active = true;
        sdn.topmaj = "O".to_owned();
        {
            let lock = tx.lock().await;
            let mut am = sdn.clone().into_active_model();
            am.record_id = NotSet;
            am.insert(&*lock).await?;
        }
        relations.process_relations(&db, &tx).await?;
        Ok(fixed_ref)
    }
}

/// Given fixed_refs are ACTIVE (i.e. presents in current xml document), other will be updated to INACTIVE
pub async fn set_sanction_inactive(tx: Arc<tokio::sync::Mutex<DatabaseTransaction>>, fixed_refs: &[i32]) -> Result<(), DbErr> {
    if fixed_refs.is_empty() {
        return Ok(());
    }
    let tx = tx.lock().await;
    Entity::update_many()
        .col_expr(Column::SanctionStatus, Expr::value("INACTIVE"))
        .col_expr(Column::Topmaj, Expr::value("N"))
        .filter(Column::FixedRef.is_not_in(fixed_refs.to_vec()))
        .filter(Column::SanctionStatus.eq("ACTIVE"))
        .exec(&*tx)
        .await?;
    Ok(())
}
