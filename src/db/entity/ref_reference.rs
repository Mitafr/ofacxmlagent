use async_trait::async_trait;
use sea_orm::{entity::prelude::*, DatabaseTransaction, Iterable};
use tokio::sync::MutexGuard;

use crate::{
    db::OfacRefEntity,
    document::{models::referencevaluesets::DetailReference, OfacDocumentReferences},
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ref_reference")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub value: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::caatsa235::Entity")]
    Caatsa235,
    #[sea_orm(has_many = "super::eo13662dd::Entity")]
    Eo13662dd,
    #[sea_orm(has_many = "super::eo13846inf::Entity")]
    Eo13846inf,
    #[sea_orm(has_many = "super::eo14024dd::Entity")]
    Eo14024dd,
    #[sea_orm(has_many = "super::target::Entity")]
    Target,
}

impl Related<super::caatsa235::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Caatsa235.def()
    }
}

impl Related<super::eo13662dd::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Eo13662dd.def()
    }
}

impl Related<super::eo13846inf::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Eo13846inf.def()
    }
}

impl Related<super::eo14024dd::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Eo14024dd.def()
    }
}

impl Related<super::target::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Target.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[async_trait]
impl OfacRefEntity<DetailReference, ActiveModel, Model> for ActiveModel {
    async fn from_ofac_document(entity: &DetailReference, in_db: &[Model], _references: &OfacDocumentReferences, tx: &MutexGuard<DatabaseTransaction>) -> Result<Option<ActiveModel>, DbErr> {
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
                {
                    ActiveModel::from(model).insert(&**tx).await?;
                }
                Ok(None)
            }
        }
    }
}
