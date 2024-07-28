#![allow(clippy::unused_async)]

use crate::common;
use crate::models::_entities::picks;
use crate::views::picks::PicksTemplate;
use axum::debug_handler;
use loco_rs::prelude::*;

#[debug_handler]
async fn get_picks(State(ctx): State<AppContext>) -> Result<impl IntoResponse> {
    let settings = common::settings::Settings::from_ctx(&ctx).unwrap_or_default();
    let picks = picks::Entity::find().all(&ctx.db).await.unwrap();
    Ok(PicksTemplate {
        title: settings.title,
        picks,
    })
}

pub fn routes() -> Routes {
    Routes::new().prefix("picks").add("/", get(get_picks))
}
