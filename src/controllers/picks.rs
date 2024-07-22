#![allow(clippy::unused_async)]

use crate::common;
use crate::models::_entities::picks;

use axum::debug_handler;
use loco_rs::prelude::*;
use serde_json::json;

use crate::views::picks::PicksTemplate;

#[debug_handler]
async fn get_picks(State(ctx): State<AppContext>) -> Result<impl IntoResponse> {
    let settings =
        common::settings::Settings::from_json(&ctx.config.settings.unwrap_or(json!["{}"]))
            .unwrap_or_default();
    let picks = picks::Entity::find().all(&ctx.db).await.unwrap();
    Ok(PicksTemplate {
        title: settings.title,
        picks,
    })
}

pub fn routes() -> Routes {
    Routes::new().prefix("picks").add("/", get(get_picks))
}
