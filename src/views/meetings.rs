use crate::models::_entities::{books, meetings, users};
use askama_axum::Template;
use chrono::NaiveDate;
use futures::{stream, StreamExt};
use itertools::Itertools;
use loco_rs::prelude::*;
use serde::Serialize;
use std::vec::Vec;

#[derive(Template, Debug, Clone)]
#[template(path = "components/meeting_list_item.html", escape = "none")]
pub struct MeetingListItem {
    pub book: books::Model,
    pub user: users::Model,
    pub meeting: meetings::Model,
}

impl ViewRenderer for MeetingListItem {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}

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
    pub meetings: Vec<(NaiveDate, Vec<MeetingListItem>)>,
}

impl ViewRenderer for MeetingsTemplate {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}

impl MeetingsTemplate {
    pub async fn new(meetings: Vec<meetings::Model>, ctx: DatabaseConnection) -> Self {
        let meetings: Vec<MeetingListItem> = stream::iter(meetings)
            .then(|meeting| async {
                let book = meeting
                    .find_related(books::Entity)
                    .one(&ctx)
                    .await
                    .unwrap()
                    .unwrap();
                let user = book
                    .find_related(users::Entity)
                    .one(&ctx)
                    .await
                    .unwrap()
                    .unwrap();
                MeetingListItem {
                    book,
                    user,
                    meeting,
                }
            })
            .collect()
            .await;
        Self {
            meetings: meetings
                .into_iter()
                .chunk_by(|meeting| meeting.meeting.date.date_naive())
                .into_iter()
                .map(|(date, meetings)| (date, meetings.collect()))
                .collect(),
        }
    }
}
