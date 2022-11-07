use std::sync::Arc;

use sea_orm::{entity::prelude::*, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, Iterable, RelationTrait, Set};

use crate::document::{models::document::Document, OfacDocumentReferences};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "document")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub doctype: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub registration_number: Option<String>,
    pub issued_by: Option<i32>,
    pub issued_date: Option<Date>,
    pub expiration_date: Option<Date>,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
    #[sea_orm(ignore)]
    pub identity: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ref_country::Entity",
        from = "Column::IssuedBy",
        to = "super::ref_country::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefCountry,
    #[sea_orm(
        belongs_to = "super::ref_document::Entity",
        from = "Column::Doctype",
        to = "super::ref_document::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    RefDocument,
    #[sea_orm(has_many = "super::document_identity::Entity")]
    DocumentIdentity,
}

impl Related<super::ref_country::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RefCountry.def()
    }
}

impl Related<super::ref_document::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RefDocument.def()
    }
}

impl Related<super::document_identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DocumentIdentity.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from_ofac_document(entity: &Document, references: &OfacDocumentReferences) -> Model {
        let mut document = Model {
            id: entity.id,
            doctype: Some(entity.type_id),
            topmaj: "N".to_owned(),
            ..Default::default()
        };
        if let Some(issued_by) = entity.registration_number.as_ref() {
            document.registration_number = Some(issued_by.to_uppercase());
        }
        if let Some(issued_by) = entity.issued_by {
            match references.area_codes.iter().find(|&area| area.id == issued_by) {
                Some(area) => document.issued_by = Some(area.id),
                None => document.issued_by = None,
            }
        }
        if let Some(dates) = entity.dates.as_ref() {
            for date in dates {
                match date.type_id {
                    1480 => {
                        let start = date.period.start.as_ref().unwrap();
                        let end = date.period.end.as_ref().unwrap();
                        if start.from == start.to && end.from == end.to {
                            document.issued_date = Some(start.from.to_sql_date());
                        }
                    }
                    1481 => {
                        let start = date.period.start.as_ref().unwrap();
                        let end = date.period.end.as_ref().unwrap();
                        if start.from == start.to && end.from == end.to {
                            document.expiration_date = Some(start.from.to_sql_date());
                        }
                    }
                    _ => {}
                }
            }
        }
        document.identity = entity.identity_id;
        document
    }
}

impl ActiveModel {
    pub async fn process_entities(entities: Vec<Model>, db: &DatabaseConnection, tx: &Arc<tokio::sync::Mutex<DatabaseTransaction>>) -> Result<(), DbErr> {
        let in_db = Arc::new(Entity::find().all(db).await?);
        let tasks: Vec<_> = entities.into_iter().map(move |e| tokio::spawn(ActiveModel::process_entity(e, Arc::clone(&in_db), Arc::clone(tx)))).collect();
        for task in tasks {
            task.await.unwrap().unwrap();
        }
        Ok(())
    }
    pub async fn process_entity(mut model: Model, in_db: Arc<Vec<Model>>, tx: Arc<tokio::sync::Mutex<DatabaseTransaction>>) -> Result<(), DbErr> {
        if insert_if_new_document(&mut model, &in_db, &tx).await? {
            return Ok(());
        }
        let id = model.id;
        if let Some(index) = in_db.iter().position(|d| d.id == id) {
            let mut in_db = in_db[index].clone();
            in_db.identity = model.identity;
            if in_db == model {
                return Ok(());
            }
            // Update document
            {
                let lock = tx.lock().await;
                set_all_active(model.clone())?.update(&*lock).await?;
            }
        }
        Ok(())
    }
}

// Return true if inserted (i.e new document)
async fn insert_if_new_document(model: &mut Model, in_db: &Arc<Vec<Model>>, tx: &Arc<tokio::sync::Mutex<DatabaseTransaction>>) -> Result<bool, DbErr> {
    let id = model.id;
    let identity = model.identity;
    if in_db.iter().any(|m| m.id == id) {
        return Ok(false);
    }
    model.topmaj = "O".to_owned();
    {
        let lock = tx.lock().await;
        ActiveModel::from(model.clone()).insert(&*lock).await?;
        super::document_identity::ActiveModel {
            identity_id: Set(identity),
            document_id: Set(id),
        }
        .insert(&*lock)
        .await?;
    }
    Ok(false)
}

fn set_all_active(document: Model) -> Result<ActiveModel, DbErr> {
    let mut active_model = ActiveModel::from(document);
    for col in Column::iter() {
        let v = active_model.get(col);
        active_model.set(col, v.into_value().unwrap());
    }
    Ok(active_model)
}
