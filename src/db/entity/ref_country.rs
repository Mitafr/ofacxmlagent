use async_trait::async_trait;
use sea_orm::{entity::prelude::*, DatabaseTransaction, Iterable};
use tokio::sync::MutexGuard;

use crate::{
    db::OfacRefEntity,
    document::{models::areacode::AreaCode, OfacDocumentReferences},
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ref_country")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub value: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::address::Entity")]
    Address,
    #[sea_orm(has_many = "super::document::Entity")]
    Document,
    #[sea_orm(has_many = "super::nationality::Entity")]
    Nationality,
    #[sea_orm(has_many = "super::sdn::Entity")]
    Sdn,
}

impl Related<super::address::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Address.def()
    }
}

impl Related<super::document::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Document.def()
    }
}

impl Related<super::nationality::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nationality.def()
    }
}

impl Related<super::sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[async_trait]
impl OfacRefEntity<AreaCode, ActiveModel, Model> for ActiveModel {
    async fn from_ofac_document(entity: &AreaCode, in_db: &[Model], _references: &OfacDocumentReferences, tx: &MutexGuard<DatabaseTransaction>) -> Result<Option<ActiveModel>, DbErr> {
        let id = entity.id;
        let model = Model { id, value: entity.name.to_uppercase() };
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
