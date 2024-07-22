use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

// put this in src/common/settings.rs
#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub title: String,
}

impl Settings {
    pub fn from_json(value: &serde_json::Value) -> Result<Self, Error> {
        Ok(serde_json::from_value(value.clone())?)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            title: "Bookclub".to_string(),
        }
    }
}
