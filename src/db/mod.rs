use crate::processor::sdn::{QuerySdnRecord, SdnRecord};
use crate::{config::Config, document::OfacDocumentReferences, processor::entity::name::SdnAlias};
use async_trait::async_trait;
use chrono::NaiveDate;
use log::{error, info};
use sea_orm::entity::*;
use sea_orm::sea_query::value::FromValueTuple;
use sea_orm::sea_query::{Alias, Expr, Func, Iden, IntoCondition};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, Condition, DatabaseTransaction, DeriveColumn, EntityTrait, EnumIter, IntoActiveModel, Iterable, JoinType, ModelTrait, QueryFilter, QueryOrder, RelationTrait, Value,
};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr, QuerySelect};
use std::fmt::Write;
use std::marker::Sync;
use std::sync::Arc;
use tokio::sync::MutexGuard;

pub mod entity;
use std::collections::BTreeMap;

/// Initialize a Mysql DB Pool with the provided Config
pub async fn init_db(config: &Config) -> Result<DatabaseConnection, DbErr> {
    let mut options = ConnectOptions::new(config.get_connection_string());
    options.sqlx_logging(config.debug);
    info!("Trying to connect to db...");
    match Database::connect(options).await {
        Ok(db) => {
            info!("Successfully connected to database {}", config.get_database_name());
            Ok(db)
        }
        Err(e) => Err(e),
    }
}

/// Initialize a Mysql DB Pool to DDC Databse with the provided Config
pub async fn init_ddc_db(config: &Config) -> Result<DatabaseConnection, DbErr> {
    let mut options = ConnectOptions::new(config.get_ddc_connection_string());
    options.sqlx_logging(config.debug);
    info!("Trying to connect to db...");
    match Database::connect(options).await {
        Ok(db) => {
            info!("Successfully connected to database {}", config.get_ddc_database_name());
            Ok(db)
        }
        Err(e) => Err(e),
    }
}

/// Final operation applied to a processed OfacEntity
#[derive(PartialEq, Eq)]
pub enum OfacEntityFinalOp {
    /// OfacEntity has been inserted
    Insert,
    /// OfacEntity has been updated
    Update,
    /// OfacEntity topmaj row has been updated
    UpdateTopmajOnly,
    Nothing,
}

/// `UPPER(*)` SQL operator
struct SqlUpper;

impl Iden for SqlUpper {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(s, "UPPER").unwrap();
    }
}

/// Represents an Ofac Ref (i.e. referential) entity that can be loaded from xml document
#[async_trait]
pub trait OfacRefEntity<T: std::marker::Sync, R, M> {
    async fn from_ofac_document(entity: &T, in_db: &[M], references: &OfacDocumentReferences, tx: &MutexGuard<DatabaseTransaction>) -> Result<Option<R>, DbErr>;
}

/// Represents an Ofac Relation entity between Many to Many relation
pub trait OfacRelEntity {
    fn generate(lhs: i32, rhs: i32) -> Self
    where
        Self: ActiveModelBehavior + IntoActiveModel<Self>,
    {
        let mut columns = <<Self as sea_orm::ActiveModelTrait>::Entity as sea_orm::EntityTrait>::Column::iter();
        let mut model = <Self as ActiveModelTrait>::default();
        model.set(columns.next().unwrap(), Value::Int(Some(lhs)));
        model.set(columns.next().unwrap(), Value::Int(Some(rhs)));
        model
    }
}

macro_rules! impl_topmaj {
    ($e:ty, $m:ty, $amr:ty, $am:ty) => {
        impl OfacEntity<$e, $m, $amr, $am> for $am {
            fn set_topmaj(model: &mut $m, value: String) {
                model.topmaj = value;
            }
            fn set_topmaj_active(model: &mut $am, value: String) {
                model.topmaj = Set(value);
            }
            fn get_topmaj(model: &$m) -> String {
                model.topmaj.to_owned()
            }
        }
    };
}

pub(crate) use impl_topmaj;

macro_rules! impl_parse_feature_version_value {
    ($name:ident, $m:ty, $featureattr:ident, $entityname:tt) => {
        impl $m {
            pub fn from_ofac_document(entity: &FeatureVersion) -> $m {
                let mut model = $name {
                    id: entity.id,
                    topmaj: "N".to_owned(),
                    ..Default::default()
                };

                if entity.detail.is_none() {
                    warn!("{} Version detail is empty for FeatureVersionId={}", $entityname, model.id);
                    return model;
                }
                let detail = entity.detail.as_ref().unwrap();
                if detail.value.is_none() {
                    warn!("{} Version detail has no value for FeatureVersionId={}", $entityname, model.id);
                    return model;
                }
                model.$featureattr = detail.value.as_ref().unwrap().to_uppercase();
                model
            }
        }
    };
}

macro_rules! impl_parse_feature_version_id {
    ($name:ident, $m:ty, $featureattr:ident, $entityname:tt) => {
        impl $m {
            pub fn from_ofac_document(entity: &FeatureVersion) -> $m {
                let mut model = $name {
                    id: entity.id,
                    topmaj: "N".to_owned(),
                    ..Default::default()
                };

                if entity.detail.is_none() {
                    warn!("{} Version detail is empty for FeatureVersionId={}", $entityname, model.id);
                    return model;
                }
                let detail = entity.detail.as_ref().unwrap();
                model.$featureattr = detail.detail_reference_id;
                model
            }
        }
    };
}

pub(crate) use impl_parse_feature_version_id;
pub(crate) use impl_parse_feature_version_value;

/// Represents an Ofac Entity
#[async_trait]
pub trait OfacEntity<E, M, R, AM>
where
    E: EntityTrait<Model = M>,
    M: Sync + Send + ModelTrait<Entity = E> + PartialEq + IntoActiveModel<AM>,
    R: Send + Sync + OfacRelEntity + ActiveModelBehavior + ActiveModelTrait + IntoActiveModel<R>,
    AM: Send + Sync + PartialEq + ActiveModelBehavior + ActiveModelTrait<Entity = E>,
{
    async fn process_entity(models: &mut [M], related: &mut Vec<M>, db: &DatabaseConnection, tx: &Arc<tokio::sync::Mutex<DatabaseTransaction>>, identity: i32, op: &mut OfacEntityFinalOp) -> Result<(), DbErr> {
        for model in models.iter_mut() {
            if Self::insert_if_new(model, tx, related, identity).await? {
                if *op == OfacEntityFinalOp::Nothing {
                    *op = OfacEntityFinalOp::Insert;
                }
                continue;
            }
            if let Some(index) = related.iter().position(|m| Self::get_primary_key(m).unwrap() == Self::get_primary_key(model).unwrap()) {
                let mut in_db = related.remove(index);
                Self::set_topmaj(model, "N".to_owned());
                let in_db_topmaj = Self::get_topmaj(&in_db);
                Self::set_topmaj(&mut in_db, "N".to_owned());
                if &in_db == model {
                    if in_db_topmaj == *"O" {
                        Self::update_only_topmaj(&mut in_db, db).await?;
                        if *op == OfacEntityFinalOp::Nothing {
                            *op = OfacEntityFinalOp::UpdateTopmajOnly;
                        }
                    }
                    continue;
                }
                Self::update_entity(model, db).await?;
                if *op == OfacEntityFinalOp::Nothing {
                    *op = OfacEntityFinalOp::Update;
                }
            }
        }
        Self::process_related(related, db, identity).await?;
        Ok(())
    }

    /// Set all active Column for the given ActiveModel
    fn set_all_active(entity: &mut AM) -> Result<(), DbErr> {
        for col in <<AM as sea_orm::ActiveModelTrait>::Entity as sea_orm::EntityTrait>::Column::iter() {
            let v = entity.get(col);
            entity.set(col, v.into_value().unwrap());
        }
        Ok(())
    }

    /// If entity is same as DB except for topmaj
    /// We have to update topmaj only
    async fn update_only_topmaj(model: &mut M, db: &DatabaseConnection) -> Result<(), DbErr> {
        let mut am: AM = model.clone().into_active_model();
        Self::set_topmaj_active(&mut am, "N".to_owned());
        am.update(db).await?;
        Ok(())
    }

    /// If entity is different from in DB
    /// We have to update all field
    async fn update_entity(model: &mut M, db: &DatabaseConnection) -> Result<(), DbErr> {
        Self::set_topmaj(model, "O".to_owned());
        let mut am = model.clone().into_active_model();
        Self::set_all_active(&mut am)?;
        am.update(db).await?;
        Ok(())
    }

    /// If related (i.e entity is in DB but not present in xml doc) remains after entity processed
    /// We have to delete these entities in DB
    async fn process_related(related: &[M], db: &DatabaseConnection, rhs: i32) -> Result<(), DbErr> {
        for model_rel in related {
            let am = model_rel.clone().into_active_model();
            let primary_key_value = match am.get_primary_key_value() {
                Some(val) => FromValueTuple::from_value_tuple(val),
                None => return Err(DbErr::Exec(sea_orm::RuntimeErr::Internal("Fail to get primary key from model".to_owned()))),
            };
            R::generate(primary_key_value, rhs).delete(db).await?;
            am.delete(db).await?;
        }
        Ok(())
    }

    /// Insert the entity if it is not present in DB
    /// Return true if inserted (i.e new entity in db)
    async fn insert_if_new(model: &mut M, tx: &Arc<tokio::sync::Mutex<DatabaseTransaction>>, related: &[M], rhs: i32) -> Result<bool, DbErr> {
        let id = Self::get_primary_key(model).unwrap();
        if related.iter().any(|m| Self::get_primary_key(m).unwrap() == id) {
            return Ok(false);
        }
        Self::set_topmaj(model, "O".to_owned());
        {
            let am: AM = model.clone().into_active_model();
            let lock = tx.lock().await;
            match am.insert(&*lock).await {
                Ok(_) => <R as sea_orm::ActiveModelTrait>::Entity::insert(R::generate(id, rhs)).exec(&*lock).await.unwrap(),
                Err(e) => {
                    error!("{:?} {}", model, e);
                    return Err(e);
                }
            };
        }
        Ok(true)
    }

    /// Get the primary of a Model
    fn get_primary_key(model: &M) -> Result<i32, DbErr> {
        let am = model.clone().into_active_model();
        let pk = match am.get_primary_key_value() {
            Some(val) => FromValueTuple::from_value_tuple(val),
            None => return Err(DbErr::Exec(sea_orm::RuntimeErr::Internal("Fail to get primary key from model".to_owned()))),
        };
        Ok(pk)
    }

    fn set_topmaj(model: &mut M, value: String);
    fn set_topmaj_active(model: &mut AM, value: String);
    fn get_topmaj(model: &M) -> String;
}

pub async fn get_last_issued_date(db: &DatabaseConnection) -> NaiveDate {
    match entity::dateofissue::Entity::find_by_id(0).one(db).await.unwrap() {
        Some(date_of_issue) => date_of_issue.last_document,
        None => NaiveDate::from_ymd(1970, 1, 1),
    }
}

pub async fn find_fixed_ref_with_names(db: &DatabaseConnection) -> Result<BTreeMap<i32, Vec<String>>, DbErr> {
    let mut records: BTreeMap<i32, Vec<String>> = BTreeMap::new();
    let in_db_records: Vec<SdnAlias> = entity::sdn::Entity::find()
        .order_by_asc(entity::sdn::Column::FixedRef)
        .select_only()
        .column(entity::sdn::Column::FixedRef)
        .column_as(entity::name::Column::Script, "script_id")
        .column_as(entity::sdn::Column::Partysubtypeid, "partysubtype")
        .column_as(entity::name::Column::LastName, "last_name")
        .column_as(entity::name::Column::FirstName, "first_name")
        .column_as(entity::name::Column::MiddleName, "middle_name")
        .column_as(entity::name::Column::MaidenName, "maiden_name")
        .column_as(entity::name::Column::AircraftName, "aircraft_name")
        .column_as(entity::name::Column::EntityName, "entity_name")
        .column_as(entity::name::Column::VesselName, "vessel_name")
        .column_as(entity::name::Column::Nickname, "nickname")
        .column_as(entity::name::Column::Patronymic, "patronymic")
        .column_as(entity::name::Column::Matronymic, "matronymic")
        .column_as(entity::name::Column::Quality, "quality")
        .join_rev(
            JoinType::InnerJoin,
            entity::name_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::name_sdn::Column::SdnId)
                .to(entity::sdn::Column::FixedRef)
                .into(),
        )
        .join(JoinType::InnerJoin, entity::name_sdn::Relation::Name.def())
        .filter(
            Condition::all()
                .add(entity::name::Column::Quality.eq("NORMAL".to_owned()))
                .add(entity::name::Column::NameType.eq("ALIAS".to_owned())),
        )
        .into_model::<SdnAlias>()
        .all(db)
        .await?;
    for record in in_db_records {
        records.entry(record.fixed_ref).or_default().push(record.build_alias());
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
    enum DdcAlias {
        FixedRef,
        Name,
    }
    let ddc_aliases: Vec<(i32, String)> = entity::sdn::Entity::find()
        .order_by_asc(entity::sdn::Column::FixedRef)
        .select_only()
        .column_as(entity::sdn::Column::FixedRef, DdcAlias::FixedRef)
        .column_as(entity::ddc_alias::Column::Name, DdcAlias::Name)
        .join_rev(
            JoinType::InnerJoin,
            entity::ddc_alias_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::ddc_alias_sdn::Column::SdnId)
                .to(entity::sdn::Column::RecordId)
                .into(),
        )
        .join(JoinType::InnerJoin, entity::ddc_alias_sdn::Relation::DdcAlias.def())
        .filter(Condition::all().add(entity::ddc_alias::Column::Quality.eq("NORMAL".to_owned())))
        .into_values::<_, DdcAlias>()
        .all(db)
        .await?;

    for record in ddc_aliases {
        records.entry(record.0).or_default().push(record.1);
    }

    Ok(records)
}

pub async fn find_records(db: &DatabaseConnection, ddc_db: &DatabaseConnection) -> Result<(Vec<SdnRecord>, Vec<entity::ddc_name::Model>), DbErr> {
    let sdn_record: Vec<QuerySdnRecord> = entity::sdn::Entity::find()
        .select_only()
        .column(entity::sdn::Column::FixedRef)
        .column(entity::sdn::Column::Partysubtypeid)
        .column(entity::sdn::Column::LastUpdate)
        .column(entity::sdn::Column::Title)
        .column(entity::sdn::Column::Comment)
        .column(entity::sdn::Column::SanctionDate)
        .column(entity::sdn::Column::Gender)
        .column(entity::sdn::Column::VesselFlag)
        .column(entity::sdn::Column::VesselCallSign)
        .column(entity::sdn::Column::OtherVesselCallSign)
        .column(entity::sdn::Column::VesselOwner)
        .column(entity::sdn::Column::OrganizationEstablishedDate)
        .column(entity::sdn::Column::OtherVesselCallSign)
        .column_as(entity::sdn::Column::ConstructionNumber, "aircraft_construction_number")
        .column_as(entity::sdn::Column::ManufactureDate, "aircraft_manufacture_date")
        .column_as(entity::sdn::Column::TranspondeurCode, "aircraft_transpondeur_code")
        .column_as(entity::sdn::Column::PreviousTailNumber, "aircraft_previous_tail_number")
        .column_as(entity::sdn::Column::TailNumber, "aircraft_tail_number")
        .column_as(entity::sdn::Column::Model, "aircraft_model")
        .column_as(entity::sdn::Column::ManufacturerSerialNumber, "msn")
        .column_as(entity::address::Column::Id, "address_id")
        .column_as(entity::address::Column::Address, "address_address")
        .column_as(entity::address::Column::City, "address_city")
        .column_as(entity::address::Column::PostalCode, "address_postal_code")
        .column_as(Expr::tbl(Alias::new("ref_address_country"), entity::ref_reference::Column::Value).into_simple_expr(), "address_country")
        .column_as(entity::address::Column::State, "address_state")
        .column_as(entity::address::Column::Region, "address_region")
        .column_as(entity::address::Column::IsPrimary, "address_is_primary")
        .column_as(entity::dob::Column::Dob, "dob_dob")
        .column_as(entity::pob::Column::Pob, "pob_pob")
        .column_as(entity::citizen::Column::Location, "citizen_location")
        .column_as(entity::website::Column::Website, "website_website")
        .column_as(entity::bic::Column::Bic, "bic_bic")
        .column_as(entity::program::Column::Program, "sanction_program")
        .column_as(entity::email::Column::Email, "email_email")
        .column_as(entity::former_vessel_flag::Column::Value, "former_vessel_flag")
        .column_as(entity::other_vessel_flag::Column::Value, "other_vessel_flag")
        .column_as(entity::ref_country::Column::Value, "nationality_nationality")
        .column_as(entity::aircraft_operator::Column::Operator, "aircraft_operator")
        .column_as(entity::phone_number::Column::PhoneNumber, "phone_number")
        .column_as(entity::document::Column::RegistrationNumber, "document_registration_number")
        .column_as(entity::document::Column::Doctype, "document_type")
        .column_as(entity::document::Column::ExpirationDate, "document_expiration_date")
        .column_as(entity::document::Column::IssuedDate, "document_issued_date")
        .column_as(entity::ref_document::Column::Value, "document_type_value")
        .column_as(Expr::tbl(Alias::new("ref_country_document"), entity::ref_country::Column::Value).into_simple_expr(), "document_issued_by")
        .column_as(entity::document::Column::Id, "document_id")
        .column_as(entity::name::Column::Id, "name_id")
        .column_as(entity::name::Column::NameType, "name_name_type")
        .column_as(entity::name::Column::Script, "name_script")
        .column_as(entity::name::Column::LastName, "name_last_name")
        .column_as(entity::name::Column::FirstName, "name_first_name")
        .column_as(entity::name::Column::MiddleName, "name_middle_name")
        .column_as(entity::name::Column::MaidenName, "name_maiden_name")
        .column_as(entity::name::Column::AircraftName, "name_aircraft_name")
        .column_as(entity::name::Column::EntityName, "name_entity_name")
        .column_as(entity::name::Column::VesselName, "name_vessel_name")
        .column_as(entity::name::Column::Nickname, "name_nickname")
        .column_as(entity::name::Column::Patronymic, "name_patronymic")
        .column_as(entity::name::Column::Matronymic, "name_matronymic")
        .column_as(entity::name::Column::Quality, "name_quality")
        .column_as(entity::relation::Column::LinkedTo, "relation_linked_to")
        .column_as(entity::ddc_bic::Column::Bic, "ddc_bic")
        .column_as(entity::ddc_alias::Column::Name, "ddc_alias_name")
        .column_as(entity::ddc_alias::Column::Quality, "ddc_alias_quality")
        .column_as(Expr::tbl(Alias::new("ref_ifca"), entity::ref_reference::Column::Value).into_simple_expr(), "ifca_determination")
        .column_as(Expr::tbl(Alias::new("ref_peesa_information"), entity::ref_reference::Column::Value).into_simple_expr(), "peesa_information")
        .column_as(
            Expr::tbl(Alias::new("ref_additional_sanctions_information"), entity::ref_reference::Column::Value).into_simple_expr(),
            "additional_sanctions_information",
        )
        .column_as(
            Expr::tbl(Alias::new("ref_secondary_sanctions_risks"), entity::ref_reference::Column::Value).into_simple_expr(),
            "secondary_sanctions_risks",
        )
        .column_as(
            Expr::tbl(Alias::new("ref_prohibited_transactions"), entity::ref_reference::Column::Value).into_simple_expr(),
            "prohibited_transactions",
        )
        .column_as(Expr::tbl(Alias::new("ref_organization_type"), entity::ref_reference::Column::Value).into_simple_expr(), "organization_type")
        .column_as(Expr::tbl(Alias::new("ref_vessel_type"), entity::ref_reference::Column::Value).into_simple_expr(), "vessel_type")
        .join_rev(
            JoinType::LeftJoin,
            entity::address_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::address_sdn::Column::IdentityId)
                .to(entity::sdn::Column::Identity)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::address_sdn::Relation::Address.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::dob_identity::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::dob_identity::Column::IdentityId)
                .to(entity::sdn::Column::Identity)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::dob_identity::Relation::Dob.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::pob_identity::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::pob_identity::Column::IdentityId)
                .to(entity::sdn::Column::Identity)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::pob_identity::Relation::Pob.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::citizen_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::citizen_sdn::Column::SdnId)
                .to(entity::sdn::Column::FixedRef)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::citizen_sdn::Relation::Citizen.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::website_identity::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::website_identity::Column::IdentityId)
                .to(entity::sdn::Column::Identity)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::website_identity::Relation::Website.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::bic_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::bic_sdn::Column::SdnId)
                .to(entity::sdn::Column::FixedRef)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::bic_sdn::Relation::Bic.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::sdn_program::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::sdn_program::Column::SdnId)
                .to(entity::sdn::Column::FixedRef)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::sdn_program::Relation::Program.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::email_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::email_sdn::Column::SdnId)
                .to(entity::sdn::Column::FixedRef)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::email_sdn::Relation::Email.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::former_vessel_flag_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::former_vessel_flag_sdn::Column::SdnId)
                .to(entity::sdn::Column::FixedRef)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::former_vessel_flag_sdn::Relation::FormerVesselFlag.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::other_vessel_flag_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::other_vessel_flag_sdn::Column::SdnId)
                .to(entity::sdn::Column::FixedRef)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::other_vessel_flag_sdn::Relation::OtherVesselFlag.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::nationality_identity::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::nationality_identity::Column::IdentityId)
                .to(entity::sdn::Column::Identity)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::nationality_identity::Relation::Nationality.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::aircraft_operator_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::aircraft_operator_sdn::Column::SdnId)
                .to(entity::sdn::Column::FixedRef)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::aircraft_operator_sdn::Relation::AircraftOperator.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::phone_number_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::phone_number_sdn::Column::SdnId)
                .to(entity::sdn::Column::FixedRef)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::phone_number_sdn::Relation::PhoneNumber.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::document_identity::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::document_identity::Column::IdentityId)
                .to(entity::sdn::Column::Identity)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::document_identity::Relation::Document.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::name_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::name_sdn::Column::SdnId)
                .to(entity::sdn::Column::FixedRef)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::document::Relation::RefDocument.def())
        // LINKED TO
        .join_rev(
            JoinType::LeftJoin,
            entity::relation_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::relation_sdn::Column::SdnId)
                .to(entity::sdn::Column::FixedRef)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::relation_sdn::Relation::Relation.def())
        // LINKED TO
        .join(JoinType::LeftJoin, entity::name_sdn::Relation::Name.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::ddc_alias_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::ddc_alias_sdn::Column::SdnId)
                .to(entity::sdn::Column::RecordId)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::ddc_alias_sdn::Relation::DdcAlias.def())
        .join_rev(
            JoinType::LeftJoin,
            entity::ddc_bic_sdn::Entity::belongs_to(entity::sdn::Entity)
                .from(entity::ddc_bic_sdn::Column::SdnId)
                .to(entity::sdn::Column::RecordId)
                .into(),
        )
        .join(JoinType::LeftJoin, entity::ddc_bic_sdn::Relation::DdcBic.def())
        .join(JoinType::LeftJoin, entity::nationality::Relation::RefCountry.def())
        .join_as(JoinType::LeftJoin, entity::address::Relation::RefCountry.def(), Alias::new("ref_address_country"))
        .join_as(JoinType::LeftJoin, entity::sdn::Relation::RefReference1.def(), Alias::new("ref_ifca"))
        .join_as(JoinType::LeftJoin, entity::document::Relation::RefCountry.def(), Alias::new("ref_country_document"))
        .join_as(JoinType::LeftJoin, entity::sdn::Relation::RefReference3.def(), Alias::new("ref_peesa_information"))
        .join_as(JoinType::LeftJoin, entity::sdn::Relation::RefReference7.def(), Alias::new("ref_additional_sanctions_information"))
        .join_as(JoinType::LeftJoin, entity::sdn::Relation::RefReference8.def(), Alias::new("ref_secondary_sanctions_risks"))
        .join_as(JoinType::LeftJoin, entity::sdn::Relation::RefReference5.def(), Alias::new("ref_prohibited_transactions"))
        .join_as(JoinType::LeftJoin, entity::sdn::Relation::RefReference6.def(), Alias::new("ref_organization_type"))
        .join_as(JoinType::LeftJoin, entity::sdn::Relation::RefReference4.def(), Alias::new("ref_vessel_type"))
        .filter(Condition::all().add(entity::sdn::Column::SanctionStatus.eq("ACTIVE".to_owned())))
        .order_by_asc(entity::sdn::Column::FixedRef)
        .into_model::<QuerySdnRecord>()
        .all(db)
        .await
        .unwrap();
    let mut records = Vec::new();
    let mut current_fixed_ref = 0;
    let mut current_record = SdnRecord::default();
    let ddc_programs: Vec<String> = entity::ddc_pgm::Entity::find()
        .select_only()
        .column_as(Func::cust(SqlUpper).args(vec![Expr::col(entity::ddc_pgm::Column::Program)]), Pgm::Program)
        .filter(Condition::all().add(entity::ddc_pgm::Column::Sanctioned.eq(true)))
        .into_values::<_, Pgm>()
        .all(ddc_db)
        .await
        .unwrap();
    let other_names = entity::ddc_name::Entity::find().all(ddc_db).await?;
    let mut related_names = Vec::new();
    for sdn in sdn_record {
        if sdn.fixed_ref != current_fixed_ref && current_fixed_ref != 0 {
            related_names.clear();
            records.push(current_record);
            current_record = SdnRecord::default();
            current_record.ddc_programs = ddc_programs.clone();
        }
        current_fixed_ref = sdn.fixed_ref;
        SdnRecord::from_query_sdn_record(&sdn, &mut current_record).unwrap();
        if let Some(linked_to) = sdn.relation_linked_to {
            if !related_names.contains(&linked_to) {
                related_names.push(linked_to);
                let name = entity::name::Entity::find()
                    .select_only()
                    .column_as(entity::sdn::Column::FixedRef, "fixed_ref")
                    .column_as(entity::sdn::Column::Partysubtypeid, "partysubtype")
                    .column_as(entity::name::Column::Script, "script_id")
                    .column_as(entity::name::Column::LastName, "last_name")
                    .column_as(entity::name::Column::FirstName, "first_name")
                    .column_as(entity::name::Column::MiddleName, "middle_name")
                    .column_as(entity::name::Column::MaidenName, "maiden_name")
                    .column_as(entity::name::Column::AircraftName, "aircraft_name")
                    .column_as(entity::name::Column::EntityName, "entity_name")
                    .column_as(entity::name::Column::VesselName, "vessel_name")
                    .column_as(entity::name::Column::Nickname, "nickname")
                    .column_as(entity::name::Column::Patronymic, "patronymic")
                    .column_as(entity::name::Column::Matronymic, "matronymic")
                    .column_as(entity::name::Column::Quality, "quality")
                    .join_rev(
                        JoinType::LeftJoin,
                        entity::name_sdn::Relation::Name
                            .def()
                            .on_condition(move |left, _right| Expr::tbl(left, entity::name_sdn::Column::SdnId).eq(linked_to).into_condition()),
                    )
                    .join_rev(JoinType::InnerJoin, entity::sdn::Relation::NameSdn.def())
                    .filter(Condition::all().add(entity::name::Column::NameType.eq("NAME")))
                    .into_model::<SdnAlias>()
                    .one(db)
                    .await
                    .unwrap();
                current_record.linked_to_names.push(name.unwrap().build_alias());
            }
        }
    }
    Ok((records, other_names))
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum Pgm {
    Program,
}
