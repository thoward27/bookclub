use super::_entities::meetings::{ActiveModel, Model};
use super::_entities::{books, meetings, users};
use futures::{stream, StreamExt};
use loco_rs::prelude::*;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MeetingUpdateParams {
    pub location: String,
    pub date: chrono::DateTime<chrono::FixedOffset>,
    pub order: Vec<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct NextMeetingTemplateRaw {
    order: Vec<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NextMeetingTemplate {
    pub order: Vec<users::Model>,
    // Users who are not in Order.
    pub bench: Vec<users::Model>,
}

impl super::_entities::meetings::Model {
    /// Create a new Meeting.
    ///
    /// Takes a previous meeting ID, a book ID, and database connection.
    ///
    /// TODO: We are not accounting for when there are no meetings here.
    pub async fn create_next(
        meeting: Model,
        book: books::Model,
        db: &DatabaseConnection,
    ) -> Result<(Model, books::Model)> {
        let mut template = meeting.get_next_meeting_template(db).await.unwrap();
        template.order.rotate_left(1);
        let book_placeholder = books::Model::get_or_create_placeholder(
            book.circuit_id,
            template.order.first().unwrap().id,
            db,
        )
        .await?;
        let next_meeting = meetings::ActiveModel {
            book_id: ActiveValue::set(book_placeholder.id),
            date: ActiveValue::set(meeting.date + chrono::Duration::days(30)),
            location: ActiveValue::set("TBD".to_string()),
            next_meeting_id: ActiveValue::set(None),
            next_meeting_template: ActiveValue::set(
                serde_json::to_value(NextMeetingTemplateRaw {
                    order: template.order.iter().map(|u| u.id).collect(),
                })
                .unwrap(),
            ),
            ..Default::default()
        }
        .insert(db)
        .await?;
        meeting
            .into_active_model()
            .set_next_meeting_id(next_meeting.id, db)
            .await?;
        Ok((next_meeting, book_placeholder))
    }

    /// Whether the next meeting has *not* been set, the book belongs to the user, and the meeting is in the future.
    pub fn is_editable(&self, user: &users::Model, book: &books::Model) -> bool {
        self.next_meeting_id.is_none() && book.user_id == user.id && self.date > chrono::Utc::now()
    }

    /// Get the next meeting template.
    pub async fn get_next_meeting_template(
        &self,
        db: &DatabaseConnection,
    ) -> Result<NextMeetingTemplate> {
        let template: NextMeetingTemplateRaw =
            serde_json::from_value(self.next_meeting_template.clone()).unwrap();
        Ok(NextMeetingTemplate {
            order: stream::iter(&template.order)
                .then(|user_id| async move {
                    users::Entity::find()
                        .filter(users::Column::Id.eq(*user_id))
                        .one(db)
                        .await
                        .unwrap()
                        .unwrap()
                })
                .collect::<Vec<users::Model>>()
                .await,
            bench: users::Entity::find()
                .filter(users::Column::Id.is_not_in(template.order))
                .all(db)
                .await
                .unwrap(),
        })
    }

    /// Get the meeting with the book.
    pub async fn get_with_book(db: &DatabaseConnection, id: i32) -> Result<(Model, books::Model)> {
        let (meeting, book) = meetings::Entity::find()
            .find_also_related(books::Entity)
            .filter(meetings::Column::Id.eq(id))
            .one(db)
            .await
            .unwrap()
            .unwrap();
        let book = book.unwrap();
        Ok((meeting, book))
    }
}

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

impl super::_entities::meetings::ActiveModel {
    pub async fn update_from_params(
        mut self,
        params: MeetingUpdateParams,
        db: &DatabaseConnection,
    ) -> ModelResult<Model> {
        self.location = ActiveValue::set(params.location);
        self.date = ActiveValue::set(params.date);
        self.next_meeting_template = ActiveValue::set(
            serde_json::to_value(NextMeetingTemplateRaw {
                order: params.order,
            })
            .unwrap(),
        );
        Ok(self.update(db).await?)
    }
    pub async fn set_next_meeting_id(
        mut self,
        next_meeting_id: i32,
        db: &DatabaseConnection,
    ) -> ModelResult<Model> {
        self.next_meeting_id = ActiveValue::set(Some(next_meeting_id));
        Ok(self.update(db).await?)
    }
}
