use std::sync::Arc;

use crate::document::models::profilerelationship::ProfileRelationship;
use sea_orm::{entity::prelude::*, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, Iterable, RelationTrait, Set};

#[derive(Clone, Debug, DeriveEntityModel, Default)]
#[sea_orm(table_name = "relation")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub linked_to: i32,
    pub relation_type_id: i32,
    #[sea_orm(ignore)]
    pub from_profile_id: i32,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.linked_to == other.linked_to && self.relation_type_id == other.relation_type_id
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::sdn::Entity", from = "Column::LinkedTo", to = "super::sdn::Column::FixedRef", on_update = "Restrict", on_delete = "Restrict")]
    Sdn,
    #[sea_orm(has_many = "super::relation_sdn::Entity")]
    RelationSdn,
}

impl Related<super::sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sdn.def()
    }
}

impl Related<super::relation_sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RelationSdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from_ofac_document(entity: &ProfileRelationship) -> Model {
        Model {
            id: entity.id,
            linked_to: entity.to_profile_id,
            relation_type_id: entity.relation_type_id,
            from_profile_id: entity.from_profile_id,
            topmaj: "N".to_owned(),
        }
    }
}

impl ActiveModel {
    pub async fn process_entities(entities: Vec<Model>, db: &DatabaseConnection, tx: &Arc<tokio::sync::Mutex<DatabaseTransaction>>) -> Result<(), DbErr> {
        let in_db = Entity::find().all(db).await?;
        let tasks: Vec<_> = entities
            .into_iter()
            .map(move |e| {
                let tx = Arc::clone(tx);
                tokio::spawn(ActiveModel::process_entity(e, in_db.clone(), tx))
            })
            .collect();
        for task in tasks {
            task.await.unwrap().unwrap();
        }
        Ok(())
    }
    pub async fn process_entity(mut model: Model, in_db: Vec<Model>, tx: Arc<tokio::sync::Mutex<DatabaseTransaction>>) -> Result<(), DbErr> {
        if insert_if_new_relation(&mut model, &in_db, &tx).await? {
            return Ok(());
        }
        let id = model.id;
        if let Some(index) = in_db.iter().position(|d| d.id == id) {
            let in_db = in_db[index].clone();
            if in_db == model {
                return Ok(());
            }
            // Update relation
            {
                let lock = tx.lock().await;
                set_all_active(model.clone())?.update(&*lock).await?;
            }
        }
        Ok(())
    }
}

// Return true if inserted (i.e new relation)
async fn insert_if_new_relation(model: &mut Model, in_db: &Vec<Model>, tx: &Arc<tokio::sync::Mutex<DatabaseTransaction>>) -> Result<bool, DbErr> {
    let id = model.id;
    let sdn = model.from_profile_id;
    if in_db.iter().any(|m| m.id == id) {
        return Ok(false);
    }
    {
        let lock = tx.lock().await;
        ActiveModel::from(model.clone()).insert(&*lock).await?;
        super::relation_sdn::ActiveModel { sdn_id: Set(sdn), relation_id: Set(id) }.insert(&*lock).await?;
    }
    Ok(false)
}

fn set_all_active(relation: Model) -> Result<ActiveModel, DbErr> {
    let mut active_model = ActiveModel::from(relation);
    for col in Column::iter() {
        let v = active_model.get(col);
        active_model.set(col, v.into_value().unwrap());
    }
    Ok(active_model)
}
