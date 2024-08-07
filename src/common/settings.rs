use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

// put this in src/common/settings.rs
#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub title: String,
    pub auth: Option<Auth>,
}

impl Settings {
    pub fn from_json(value: &serde_json::Value) -> Result<Self, Error> {
        Ok(serde_json::from_value(value.clone())?)
    }

    pub fn from_ctx(value: &AppContext) -> Result<Self, Error> {
        Self::from_json(&value.config.settings.clone().unwrap_or(json!["{}"]))
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            title: "Bookclub".to_string(),
            auth: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Auth {
    pub anonymous: Option<bool>,
    pub proxy: Option<Proxy>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Proxy {
    pub enabled: bool,
    pub auto_sign_up: bool,
    pub headers: ProxyHeaders,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxyHeaders {
    pub email: String,
    pub name: String,
}
