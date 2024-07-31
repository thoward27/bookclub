use super::_entities::meetings::{ActiveModel, Model};
use loco_rs::prelude::*;
use sea_orm::ActiveValue;

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

impl super::_entities::meetings::ActiveModel {
    pub async fn set_location(
        mut self,
        location: String,
        db: &DatabaseConnection,
    ) -> ModelResult<Model> {
        self.location = ActiveValue::set(location);
        Ok(self.update(db).await?)
    }
}
