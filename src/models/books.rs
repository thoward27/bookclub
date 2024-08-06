use super::_entities::{
    books::{self, ActiveModel},
    meetings,
};
use loco_rs::prelude::*;
use migration::extension::postgres::PgExpr;
use sea_orm::sea_query::Expr;

impl super::_entities::books::Model {
    /// Return a placeholder book for a user in a circuit or create it.
    pub async fn get_or_create_placeholder(
        circuit_title: String,
        user_id: i32,
        db: &DatabaseConnection,
    ) -> ModelResult<books::Model> {
        if let Some(book) = books::Entity::find()
            .filter(Expr::col(books::Column::CircuitTitle).ilike(&circuit_title))
            .filter(books::Column::UserId.eq(user_id))
            .filter(books::Column::Title.eq("TBD"))
            .one(db)
            .await?
        {
            Ok(book)
        } else {
            let book = books::ActiveModel {
                title: ActiveValue::set("TBD".to_string()),
                author: ActiveValue::set("TBD".to_string()),
                circuit_title: ActiveValue::set(circuit_title),
                user_id: ActiveValue::set(user_id),
                calibre_link: ActiveValue::set("".to_string()),
                isbn10: ActiveValue::set("TBD".to_string()),
                isbn13: ActiveValue::set("TBD".to_string()),
                ..Default::default()
            }
            .insert(db)
            .await?;
            Ok(book)
        }
    }

    pub fn is_editable(
        &self,
        user: &super::_entities::users::Model,
        meeting: &super::_entities::meetings::Model,
    ) -> bool {
        user.is_superuser || (self.user_id == user.id && meeting.date > chrono::Utc::now())
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
