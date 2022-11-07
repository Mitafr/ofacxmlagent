use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;

use log::{info, warn};
use sea_orm::error::DbErr;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, Set, TransactionTrait};
use tokio::sync::Mutex;

use crate::db::entity::sdn::DocumentEntity;
use crate::{
    db::{entity::*, get_last_issued_date, OfacRefEntity},
    document::inputs::{OfacDocument, OfacDocumentReferences},
};

#[derive(Debug)]
pub enum ImporterErr {
    AlreadyImported(String),
}

impl Display for ImporterErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImporterErr::AlreadyImported(message) => write!(f, "{}", message),
        }
    }
}
impl Error for ImporterErr {}

pub struct Importer {
    transaction_manager: Arc<Mutex<DatabaseTransaction>>,
}

impl Importer {
    pub async fn init(db: &DatabaseConnection) -> Importer {
        Importer {
            transaction_manager: Arc::new(Mutex::new(db.begin().await.unwrap())),
        }
    }

    async fn save_date_of_issue(&self, db: &DatabaseConnection, references: &OfacDocumentReferences) -> Result<(), DbErr> {
        let lock = self.transaction_manager.lock().await;
        if dateofissue::Entity::find_by_id(0).one(db).await?.is_some() {
            dateofissue::ActiveModel {
                id: Set(0),
                last_document: Set(references.date_of_issue.to_sql_date()),
            }
            .save(&*lock)
            .await?;
            return Ok(());
        }
        dateofissue::ActiveModel {
            id: NotSet,
            last_document: Set(references.date_of_issue.to_sql_date()),
        }
        .save(db)
        .await?;
        Ok(())
    }

    async fn save_references(&self, db: &DatabaseConnection, references: &OfacDocumentReferences) -> Result<(), DbErr> {
        info!("Saving References");
        let ref_reference = ref_reference::Entity::find().all(db).await?;
        {
            let lock = self.transaction_manager.lock().await;
            for reference in references.detail_references.detail_references.iter() {
                if let Some(e) = ref_reference::ActiveModel::from_ofac_document(reference, &ref_reference, references, &lock).await? {
                    e.update(&*lock).await?;
                }
            }
            let ref_type = ref_type::Entity::find().all(db).await?;
            for reftype in references.party_sub_type_values.values.iter() {
                if let Some(e) = ref_type::ActiveModel::from_ofac_document(reftype, &ref_type, references, &lock).await? {
                    e.update(&*lock).await?;
                }
            }
            let ref_country = ref_country::Entity::find().all(db).await?;
            for refcountry in references.area_codes.iter() {
                if let Some(e) = ref_country::ActiveModel::from_ofac_document(refcountry, &ref_country, references, &lock).await? {
                    e.update(&*lock).await?;
                }
            }
            let ref_document = ref_document::Entity::find().all(db).await?;
            for refdocument in references.reg_doc_types.reg_doc_types.iter() {
                if let Some(e) = ref_document::ActiveModel::from_ofac_document(refdocument, &ref_document, references, &lock).await? {
                    e.update(&*lock).await?;
                }
            }
            let ref_feature = ref_feature::Entity::find().all(db).await?;
            for reffeature in references.feature_types.types.iter() {
                if let Some(e) = ref_feature::ActiveModel::from_ofac_document(reffeature, &ref_feature, references, &lock).await? {
                    e.update(&*lock).await?;
                }
            }
        }
        info!("References saved");
        Ok(())
    }

    async fn save_sdns(&self, db: &DatabaseConnection, references: &OfacDocumentReferences, document: &OfacDocument) -> Result<(), DbErr> {
        info!("Saving DistinctParties");
        let mut sdns = Vec::new();
        for distinct_party in document.distinct_parties.parties.iter() {
            sdns.push(sdn::Model::from_ofac_document(&DocumentEntity(distinct_party, &document.locations, &document.sanction_entries), references)?);
        }
        info!("DistinctParties parsed, found {} entities", sdns.len());
        sdn::ActiveModel::process_entities(&sdns, db.clone(), &self.transaction_manager).await?;
        info!("DistinctParties saved");
        Ok(())
    }

    async fn save_documents(&self, db: &DatabaseConnection, references: &OfacDocumentReferences, document: &OfacDocument) -> Result<(), DbErr> {
        info!("Saving Documents");
        let mut documents = Vec::new();
        for document in document.documents.documents.iter() {
            documents.push(document::Model::from_ofac_document(document, references));
        }
        info!("Documents parsed, found {} entities", documents.len());
        document::ActiveModel::process_entities(documents.clone(), db, &self.transaction_manager).await?;
        info!("Documents saved");
        Ok(())
    }

    async fn save_relationships(&self, db: &DatabaseConnection, document: &OfacDocument) -> Result<(), DbErr> {
        info!("Saving Relationships");
        let mut relations = Vec::new();
        for relationshipdoc in document.profile_relationships.profile_relationships.iter() {
            relations.push(relation::Model::from_ofac_document(relationshipdoc));
        }
        info!("Relationships parsed, found {} entities", relations.len());
        relation::ActiveModel::process_entities(relations.clone(), db, &self.transaction_manager).await?;
        info!("Relationships saved");
        Ok(())
    }

    /// Will process the Document provided within the db pool
    /// # Arguments
    ///
    /// * `db` - A DB pool to process the document
    /// * `document` - A loaded OfacDocument
    pub async fn process_document(&mut self, db: &DatabaseConnection, document: &OfacDocument, force: bool) -> Result<(), ImporterErr> {
        if !document.is_loaded {
            warn!("This document has not be loaded correctly");
            return Ok(());
        }
        let last_date_of_issue = get_last_issued_date(db).await;
        if last_date_of_issue == document.references.date_of_issue.to_sql_date() && !force {
            warn!("This document has already been imported in the database ({:?}) to force import use -f flag to true", last_date_of_issue);
            return Err(ImporterErr::AlreadyImported(
                "This document has already been imported in the database to force import use -f flag to true".to_string(),
            ));
        }
        self.save_date_of_issue(db, &document.references).await.unwrap();
        self.save_references(db, &document.references).await.unwrap();
        self.save_sdns(db, &document.references, document).await.unwrap();
        self.save_documents(db, &document.references, document).await.unwrap();
        self.save_relationships(db, document).await.unwrap();
        info!("Ofac Document Successfully saved to database");
        Ok(())
    }

    pub async fn commit(self) -> Result<(), DbErr> {
        Arc::try_unwrap(self.transaction_manager).unwrap().into_inner().commit().await?;
        info!("Main Transaction succesfully commited");
        Ok(())
    }
}
