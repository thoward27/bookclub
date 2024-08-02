use crate::models::_entities::{books, meetings, users};
use crate::models::meetings::NextMeetingTemplate;
use askama_axum::Template;

use loco_rs::prelude::*;
use serde::Serialize;
use std::vec::Vec;

#[derive(Template, Debug, Clone)]
#[template(path = "components/meeting_detail.html")]
pub struct MeetingDetail {
    pub meeting: meetings::Model,
    pub book: books::Model,
    pub user: users::Model,

    pub next_meeting_template: NextMeetingTemplate,
    pub editor: bool,
}

impl MeetingDetail {
    pub async fn new(
        meeting: meetings::Model,
        book: books::Model,
        user: users::Model,
        db: &DatabaseConnection,
    ) -> Self {
        let next_meeting_template = meeting.get_next_meeting_template(db).await.unwrap();
        let editor = user.id == book.user_id;
        Self {
            meeting,
            book,
            user,
            next_meeting_template,
            editor,
        }
    }
}

impl ViewRenderer for MeetingDetail {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}

#[derive(Template, Debug, Clone)]
#[template(path = "meetings.html", escape = "none")]
pub struct MeetingsTemplate {
    pub meetings: Vec<(meetings::Model, books::Model)>,
}

impl ViewRenderer for MeetingsTemplate {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}
