use crate::models::_entities::circuits;
use askama_axum::Template;
use loco_rs::prelude::*;
use serde::Serialize;
use std::vec::Vec;

#[derive(Template, Debug, Clone)]
#[template(path = "components/circuits_nav.html", escape = "none")]
pub struct CircuitsNav {
    pub circuits: Vec<circuits::Model>,
}

impl ViewRenderer for CircuitsNav {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}
