use async_trait::async_trait;
use sea_orm::{entity::prelude::*, DatabaseTransaction, Iterable};
use tokio::sync::MutexGuard;

use crate::{
    db::OfacRefEntity,
    document::{models::referencevaluesets::IDRegDocType, OfacDocumentReferences},
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ref_document")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub value: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::document::Entity")]
    Document,
}

impl Related<super::document::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Document.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[async_trait]
impl OfacRefEntity<IDRegDocType, ActiveModel, Model> for ActiveModel {
    async fn from_ofac_document(entity: &IDRegDocType, in_db: &[Model], _references: &OfacDocumentReferences, tx: &MutexGuard<DatabaseTransaction>) -> Result<Option<ActiveModel>, DbErr> {
        let id = entity.id;
        let model = Model { id, value: entity.value.to_uppercase() };
        match in_db.iter().find(|reference| reference.id == id) {
            Some(e) => {
                if e == &model {
                    return Ok(None);
                }
                let mut am: ActiveModel = model.into();
                for col in <<ActiveModel as sea_orm::ActiveModelTrait>::Entity as sea_orm::EntityTrait>::Column::iter() {
                    let v = am.get(col);
                    am.set(col, v.into_value().unwrap());
                }
                Ok(Some(am))
            }
            None => {
                ActiveModel::from(model).insert(&**tx).await?;
                Ok(None)
            }
        }
    }
}
