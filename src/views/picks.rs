use crate::models::_entities::picks;
use askama_axum::Template;
use loco_rs::prelude::*;
use serde::Serialize;
use std::fmt::Display;
use std::vec::Vec;

#[derive(Template, Debug, Clone)]
#[template(path = "picks.html")]
pub struct PicksTemplate {
    pub title: String,
    pub picks: Vec<picks::Model>,
}

impl ViewRenderer for PicksTemplate {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}

impl Display for picks::Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} by {}", self.title, self.author)
    }
}
