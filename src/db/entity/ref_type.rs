use async_trait::async_trait;
use sea_orm::{entity::prelude::*, DatabaseTransaction, Iterable};
use tokio::sync::MutexGuard;

use crate::{
    db::OfacRefEntity,
    document::{models::referencevaluesets::PartySubType, OfacDocumentReferences},
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ref_type")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub value: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub program: Option<String>,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())", nullable)]
    pub type_fmm: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::sdn::Entity")]
    Sdn,
}

impl Related<super::sdn::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sdn.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[async_trait]
impl OfacRefEntity<PartySubType, ActiveModel, Model> for ActiveModel {
    async fn from_ofac_document(entity: &PartySubType, in_db: &[Model], _references: &OfacDocumentReferences, tx: &MutexGuard<DatabaseTransaction>) -> Result<Option<ActiveModel>, DbErr> {
        let id = entity.id;
        let model = Model {
            id,
            value: entity.value.to_uppercase(),
            program: None,
            type_fmm: None,
        };
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
