use crate::models::_entities::{books, meetings, users};
use crate::models::meetings::NextMeetingTemplate;
use askama_axum::Template;

use loco_rs::prelude::*;
use serde::Serialize;
use std::vec::Vec;

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

#[derive(Template, Debug, Clone)]
#[template(path = "meeting.html")]
pub struct MeetingTemplate {
    pub meeting: meetings::Model,
    pub book: books::Model,
    pub user: users::Model,

    pub form: MeetingFormTemplate,
}

impl ViewRenderer for MeetingTemplate {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}

impl MeetingTemplate {
    pub async fn new(id: i32, user: users::Model, db: &DatabaseConnection) -> Self {
        let (meeting, book) = meetings::Model::get_with_book(db, id).await.unwrap();
        let form = MeetingFormTemplate::new(meeting.clone(), book.clone(), user.clone(), db).await;
        Self {
            meeting,
            book,
            user,
            form,
        }
    }
}

#[derive(Template, Debug, Clone)]
#[template(path = "components/meeting_form.html")]
pub struct MeetingFormTemplate {
    pub meeting: meetings::Model,
    pub book: books::Model,
    pub user: users::Model,

    // Generated Attributes.
    pub editable: bool,
    pub next_meeting_template: NextMeetingTemplate,
}

impl ViewRenderer for MeetingFormTemplate {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}

impl MeetingFormTemplate {
    pub async fn new(
        meeting: meetings::Model,
        book: books::Model,
        user: users::Model,
        db: &DatabaseConnection,
    ) -> Self {
        let editable = meeting.is_editable(&user, &book);
        let next_meeting_template = meeting.get_next_meeting_template(db).await.unwrap();
        Self {
            meeting,
            book,
            user,
            editable,
            next_meeting_template,
        }
    }
}
