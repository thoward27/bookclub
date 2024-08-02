use super::_entities::meetings::{ActiveModel, Model};
use super::_entities::users;
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NextMeetingTemplate {
    pub order: Vec<users::Model>,
}

#[derive(Debug, Deserialize, Serialize)]
struct NextMeetingTemplateRaw {
    order: Vec<i32>,
}

impl super::_entities::meetings::Model {
    pub async fn get_next_meeting_template(
        &self,
        db: &DatabaseConnection,
    ) -> Result<NextMeetingTemplate> {
        let template: NextMeetingTemplateRaw =
            serde_json::from_value(self.next_meeting_template.clone()).unwrap();
        Ok(NextMeetingTemplate {
            order: stream::iter(template.order)
                .then(|user_id| async move {
                    users::Entity::find()
                        .filter(users::Column::Id.eq(user_id))
                        .one(db)
                        .await
                        .unwrap()
                        .unwrap()
                })
                .collect::<Vec<users::Model>>()
                .await,
        })
        // Ok(serde_json::from_value(self.next_meeting_template.clone()).unwrap())
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
        if self.next_meeting_id.is_not_set() {
            self.next_meeting_template = ActiveValue::set(
                serde_json::to_value(NextMeetingTemplateRaw {
                    order: params.order,
                })
                .unwrap(),
            );
        }
        Ok(self.update(db).await?)
    }
    pub async fn set_location(
        mut self,
        location: String,
        db: &DatabaseConnection,
    ) -> ModelResult<Model> {
        self.location = ActiveValue::set(location);
        Ok(self.update(db).await?)
    }

    pub async fn set_date(
        mut self,
        date: chrono::DateTime<chrono::FixedOffset>,
        db: &DatabaseConnection,
    ) -> ModelResult<Model> {
        self.date = ActiveValue::set(date);
        Ok(self.update(db).await?)
    }
}
