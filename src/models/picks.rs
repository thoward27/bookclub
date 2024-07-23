use super::_entities::picks::{ActiveModel, Model};
use sea_orm::entity::prelude::*;
use std::fmt::Display;

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} by {}", self.title, self.author)
    }
}
