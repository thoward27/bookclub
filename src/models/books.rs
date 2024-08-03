use super::_entities::{
    books::{self, ActiveModel},
    meetings,
};
use loco_rs::prelude::*;

impl super::_entities::books::Model {
    pub fn is_editable(
        &self,
        user: &super::_entities::users::Model,
        meeting: &super::_entities::meetings::Model,
    ) -> bool {
        self.user_id == user.id && meeting.date > chrono::Utc::now()
    }

    pub async fn get_with_meeting(
        db: &DatabaseConnection,
        id: i32,
    ) -> Result<(books::Model, meetings::Model)> {
        let (book, meeting) = books::Entity::find()
            .find_also_related(meetings::Entity)
            .filter(books::Column::Id.eq(id))
            .one(db)
            .await
            .unwrap()
            .unwrap();
        let meeting = meeting.unwrap();
        Ok((book, meeting))
    }
}
impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

#[derive(Debug)]
pub struct BookUpdateParams {
    pub title: String,
    pub author: String,
}

impl super::_entities::books::ActiveModel {
    pub async fn update_from_params(
        mut self,
        params: BookUpdateParams,
        db: &DatabaseConnection,
    ) -> ModelResult<books::Model> {
        self.title = ActiveValue::set(params.title);
        self.author = ActiveValue::set(params.author);
        Ok(self.update(db).await?)
    }
}
