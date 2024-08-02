use crate::models::_entities::{books, meetings};
use askama_axum::Template;

use loco_rs::prelude::*;
use serde::Serialize;
use std::vec::Vec;

#[derive(Template, Debug, Clone)]
#[template(path = "components/meeting_detail.html")]
pub struct MeetingDetail {
    pub meeting: meetings::Model,
    pub editor: bool,
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
